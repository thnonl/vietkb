//! Hotkey matching inside the input hook — no `RegisterHotKey`.
//!
//! Why not `RegisterHotKey`: a registered hotkey is consumed by the system before
//! any low-level hook sees it, cannot express a modifier-only combo (`Ctrl+Shift`),
//! and cannot be reinstalled atomically when the hook is rebuilt. Matching in the
//! hook also lets the backend decide, per combo, whether the trigger key is
//! swallowed before the app receives it.
//!
//! The matcher is a fixed-size, allocation-free table: [`Matcher::on_key`] runs on
//! the hook thread for every key transition.

use crate::Hotkey;
use vkb_core::Modifiers;

/// One configured combo: modifiers + optional virtual key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combo {
    pub hotkey: Hotkey,
    /// Exact modifier set required; extra modifiers block the match.
    pub modifiers: Modifiers,
    /// `None` = modifier-only combo (`Ctrl+Shift`).
    pub vk: Option<u16>,
    /// Wait for every key of the chord to come up before firing. `restore_word`
    /// needs it: text injected while Ctrl/Shift are still down reaches the app
    /// as a shortcut (`Ctrl+Shift+B`), not as the character.
    pub fire_on_full_release: bool,
}

impl Combo {
    pub const fn new(hotkey: Hotkey, modifiers: Modifiers, vk: Option<u16>) -> Self {
        Self {
            hotkey,
            modifiers,
            vk,
            fire_on_full_release: false,
        }
    }

    /// `true` when this key transition completes the combo.
    fn completes(&self, vk: u16, modifiers: Modifiers, is_modifier_key: bool) -> bool {
        if modifiers != self.modifiers {
            return false;
        }
        match self.vk {
            Some(key) => key == vk,
            None => is_modifier_key,
        }
    }

    /// `true` while the combo can still be considered held after this transition.
    fn still_held(&self, released_vk: u16, modifiers: Modifiers) -> bool {
        match self.vk {
            Some(key) => key != released_vk && modifiers == self.modifiers,
            // Modifier-only combos stay held until the modifier set changes.
            None => modifiers == self.modifiers,
        }
    }
}

/// When a configured combo fires.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FireMode {
    /// As soon as the chord is complete (key-down).
    Press,
    /// When a key of the chord is released, once per press.
    #[default]
    Release,
}

/// One slot per configurable action (`enable` … `cycle_method`).
pub const MAX_COMBOS: usize = 9;

/// Fixed-capacity table of combos with a per-combo "already fired" latch.
#[derive(Clone, Copy, Debug, Default)]
pub struct Matcher {
    combos: [Option<Combo>; MAX_COMBOS],
    latched: u16,
    /// Keyed combo index and action waiting for its release (release mode only).
    pending: Option<(u8, Hotkey)>,
    /// Modifier-only combo index and action held back until the chord is released.
    held: Option<(u8, Hotkey)>,
    /// A non-modifier key went down while a modifier-only chord was held, so the
    /// modifier-only action is cancelled for this hold.
    key_in_chord: bool,
    mode: FireMode,
    len: u8,
}

impl Matcher {
    pub const fn new() -> Self {
        Self {
            combos: [None; MAX_COMBOS],
            latched: 0,
            pending: None,
            held: None,
            key_in_chord: false,
            mode: FireMode::Press,
            len: 0,
        }
    }

    /// Choose whether combos fire on key-down or on release.
    pub fn set_mode(&mut self, mode: FireMode) {
        self.mode = mode;
        self.pending = None;
        self.held = None;
        self.key_in_chord = false;
    }

    pub fn mode(&self) -> FireMode {
        self.mode
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Add a combo. Returns `false` when the table is full.
    pub fn push(&mut self, combo: Combo) -> bool {
        if (self.len as usize) >= MAX_COMBOS {
            return false;
        }
        self.combos[self.len as usize] = Some(combo);
        self.len += 1;
        true
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn discard_latches(&mut self) {
        self.latched = 0;
        self.pending = None;
        self.held = None;
        self.key_in_chord = false;
    }

    /// `true` when a keyed combo has completed and is waiting for its release.
    ///
    /// The hook keeps the current word alive until then: `restore_word` rewrites
    /// the word it is holding, so a shortcut chord must not clear it first.
    pub fn awaiting_release(&self) -> bool {
        self.pending.is_some()
    }

    /// `true` when another combo owns these exact modifiers plus a key: the
    /// modifier-only chord is then an ambiguous prefix of that longer chord.
    fn has_keyed_sibling(&self, index: usize) -> bool {
        let Some(combo) = self.combos[index] else {
            return false;
        };
        (0..self.len as usize).any(|j| {
            j != index
                && self.combos[j]
                    .is_some_and(|other| other.vk.is_some() && other.modifiers == combo.modifiers)
        })
    }

    /// `true` when `modifiers` is exactly the modifier set of a modifier-only combo.
    fn modifier_only_set_matches(&self, modifiers: Modifiers) -> bool {
        (0..self.len as usize).any(|j| {
            self.combos[j].is_some_and(|combo| combo.vk.is_none() && combo.modifiers == modifiers)
        })
    }

    /// `true` while a combo that is bound to the non-modifier key `vk` is held
    /// (its latch is set), so the key-down must not reach the application.
    ///
    /// In [`FireMode::Release`] the combo only fires when the key comes back up,
    /// but the application must not see the trigger on the way down either: `F1`
    /// is VS Code's command palette and `F2` its rename box, and a trigger that
    /// leaks opens them and steals the following keystrokes (D-E2E-03).
    ///
    /// `RestoreWord` is deliberately excluded: `20-spec.md` ("Manual hotkey
    /// `restore_word`") says the chord is passed to the application when there
    /// is nothing to restore (`Ctrl+Shift+Z` = redo), which is only known to the
    /// application layer after the action fires.
    pub fn swallows_trigger(&self, vk: u16) -> bool {
        let len = self.len as usize;
        for i in 0..len {
            let Some(combo) = self.combos[i] else { continue };
            if self.latched & (1u16 << i) != 0
                && combo.vk == Some(vk)
                && combo.hotkey != Hotkey::RestoreWord
            {
                return true;
            }
        }
        false
    }

    /// Feed one key transition.
    ///
    /// * `vk` — the virtual key that changed.
    /// * `down` — key-down vs key-up.
    /// * `modifiers` — modifier state *after* applying this transition.
    /// * `is_modifier_key` — `true` for Ctrl/Alt/Shift/Win themselves.
    ///
    /// Returns the hotkey to fire, at most once per press of the combo. Releasing
    /// (any part of) the combo re-arms it, so pressing the toggle twice always
    /// returns to the previous state.
    ///
    /// A modifier-only combo (`Ctrl+Shift`) never fires while a longer chord that
    /// shares its modifiers (`Ctrl+Shift+Z`) can still complete: it fires on the
    /// release of the chord, and only when no other key went down during the hold.
    /// A key that completes such a longer chord therefore wins — the toggle is
    /// dropped, never doubled.
    ///
    /// A keyed combo fires on the completing key-down in [`FireMode::Press`] and
    /// on the key-up in [`FireMode::Release`], except a `fire_on_full_release`
    /// combo (`restore_word`), which fires only once every key of the chord is up
    /// so the action can type plain text into the target.
    pub fn on_key(
        &mut self,
        vk: u16,
        down: bool,
        modifiers: Modifiers,
        is_modifier_key: bool,
    ) -> Option<Hotkey> {
        let mut fired = None;
        let len = self.len as usize;

        // A typed key inside a modifier-only chord means typing, not a toggle.
        if down && !is_modifier_key && self.modifier_only_set_matches(modifiers) {
            self.key_in_chord = true;
            self.held = None;
        }

        // A key that is not part of a waiting chord cancels the wait.
        if down && !is_modifier_key {
            if let Some((index, _)) = self.pending {
                let ours = self.combos[index as usize].is_some_and(|combo| combo.vk == Some(vk));
                if !ours {
                    self.pending = None;
                    self.latched &= !(1u16 << index);
                }
            }
        }

        for i in 0..len {
            let Some(combo) = self.combos[i] else { continue };
            let bit = 1u16 << i;
            if down {
                if self.latched & bit == 0 && combo.completes(vk, modifiers, is_modifier_key) {
                    self.latched |= bit;
                    match combo.vk {
                        Some(_) => {
                            let on_key_down = self.mode == FireMode::Press
                                && !combo.fire_on_full_release;
                            if on_key_down {
                                if fired.is_none() {
                                    fired = Some(combo.hotkey);
                                }
                            } else if self.pending.is_none() {
                                self.pending = Some((i as u8, combo.hotkey));
                            }
                        }
                        None => {
                            let unambiguous =
                                self.mode == FireMode::Press && !self.has_keyed_sibling(i);
                            if unambiguous {
                                if fired.is_none() {
                                    fired = Some(combo.hotkey);
                                }
                            } else if self.held.is_none() {
                                self.held = Some((i as u8, combo.hotkey));
                            }
                        }
                    }
                }
            } else if self.latched & bit != 0 && !combo.still_held(vk, modifiers) {
                let waiting_for_quiet = combo.fire_on_full_release && modifiers != Modifiers::NONE;
                if waiting_for_quiet {
                    // `Ctrl`/`Shift` still down: typing now would arrive as a
                    // shortcut, so the chord is not released yet.
                    continue;
                }
                self.latched &= !bit;
                match combo.vk {
                    Some(_) => {
                        if let Some((index, action)) = self.pending {
                            if index as usize == i {
                                self.pending = None;
                                if fired.is_none() {
                                    fired = Some(action);
                                }
                            }
                        }
                    }
                    None => {
                        if let Some((index, action)) = self.held {
                            if index as usize == i {
                                self.held = None;
                                if !self.key_in_chord && fired.is_none() {
                                    fired = Some(action);
                                }
                            }
                        }
                    }
                }
            }
        }

        // The chord is over: a new hold starts clean.
        if !self.modifier_only_set_matches(modifiers) {
            self.key_in_chord = false;
        }

        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mods(ctrl: bool, alt: bool, shift: bool, win: bool) -> Modifiers {
        Modifiers {
            shift,
            ctrl,
            alt,
            win,
        }
    }

    const VK_SHIFT: u16 = 0x10;
    const VK_CONTROL: u16 = 0x11;
    const VK_MENU: u16 = 0x12;
    const VK_F1: u16 = 0x70;
    const VK_F24: u16 = 0x87;

    fn matcher_with(combos: &[Combo]) -> Matcher {
        let mut m = Matcher::new();
        for c in combos {
            assert!(m.push(*c));
        }
        m
    }

    fn toggle_combo() -> Matcher {
        matcher_with(&[Combo::new(
            Hotkey::Toggle,
            mods(true, false, true, false),
            None,
        )])
    }

    fn f1_combo() -> Matcher {
        matcher_with(&[Combo::new(Hotkey::Enable, Modifiers::NONE, Some(VK_F1))])
    }

    #[test]
    fn modifier_only_combo_fires_once_on_completion() {
        let mut m = toggle_combo();
        assert_eq!(
            m.on_key(VK_CONTROL, true, mods(true, false, false, false), true),
            None
        );
        assert_eq!(
            m.on_key(VK_SHIFT, true, mods(true, false, true, false), true),
            Some(Hotkey::Toggle)
        );
    }

    #[test]
    fn autorepeat_does_not_retrigger() {
        let mut m = toggle_combo();
        m.on_key(VK_CONTROL, true, mods(true, false, false, false), true);
        assert_eq!(
            m.on_key(VK_SHIFT, true, mods(true, false, true, false), true),
            Some(Hotkey::Toggle)
        );
        for _ in 0..5 {
            assert_eq!(
                m.on_key(VK_SHIFT, true, mods(true, false, true, false), true),
                None,
                "held combo must not fire again"
            );
        }
    }

    #[test]
    fn releasing_and_repressing_toggles_back() {
        let mut m = toggle_combo();
        m.on_key(VK_CONTROL, true, mods(true, false, false, false), true);
        assert_eq!(
            m.on_key(VK_SHIFT, true, mods(true, false, true, false), true),
            Some(Hotkey::Toggle)
        );
        assert_eq!(
            m.on_key(VK_SHIFT, false, mods(true, false, false, false), true),
            None
        );
        assert_eq!(
            m.on_key(VK_SHIFT, true, mods(true, false, true, false), true),
            Some(Hotkey::Toggle),
            "second press must produce the second toggle"
        );
    }

    #[test]
    fn extra_modifier_blocks_modifier_only_combo() {
        let mut m = toggle_combo();
        m.on_key(VK_CONTROL, true, mods(true, false, false, false), true);
        assert_eq!(
            m.on_key(VK_MENU, true, mods(true, true, false, false), true),
            None
        );
        assert_eq!(
            m.on_key(VK_SHIFT, true, mods(true, true, true, false), true),
            None,
            "Ctrl+Alt+Shift is not Ctrl+Shift"
        );
    }

    #[test]
    fn plain_letter_without_modifier_never_matches_modifier_combo() {
        let mut m = toggle_combo();
        assert_eq!(
            m.on_key(0x41, true, mods(false, false, false, false), false),
            None
        );
    }

    #[test]
    fn key_combo_fires_once_per_press() {
        let mut m = f1_combo();
        assert_eq!(
            m.on_key(VK_F1, true, Modifiers::NONE, false),
            Some(Hotkey::Enable)
        );
        assert_eq!(
            m.on_key(VK_F1, true, Modifiers::NONE, false),
            None,
            "autorepeat"
        );
        m.on_key(VK_F1, false, Modifiers::NONE, false);
        assert_eq!(
            m.on_key(VK_F1, true, Modifiers::NONE, false),
            Some(Hotkey::Enable)
        );
    }

    #[test]
    fn release_mode_trigger_is_swallowed_on_the_way_down() {
        let mut m = f1_combo();
        m.set_mode(FireMode::Release);
        assert!(!m.swallows_trigger(VK_F1), "nothing held yet");
        assert_eq!(m.on_key(VK_F1, true, Modifiers::NONE, false), None);
        assert!(m.swallows_trigger(VK_F1), "F1 down must not reach the app");
        assert_eq!(m.on_key(VK_F1, true, Modifiers::NONE, false), None);
        assert!(m.swallows_trigger(VK_F1), "autorepeat too");
        assert_eq!(
            m.on_key(VK_F1, false, Modifiers::NONE, false),
            Some(Hotkey::Enable)
        );
        assert!(!m.swallows_trigger(VK_F1), "released: latch cleared");
    }

    #[test]
    fn restore_word_and_modifier_only_combos_are_never_swallowed() {
        let mut m = matcher_with(&[
            Combo::new(Hotkey::RestoreWord, mods(true, false, true, false), Some(0x5A)),
            Combo::new(Hotkey::Toggle, mods(true, false, true, false), None),
        ]);
        m.on_key(VK_CONTROL, true, mods(true, false, false, false), true);
        m.on_key(VK_SHIFT, true, mods(true, false, true, false), true);
        m.on_key(0x5A, true, mods(true, false, true, false), false);
        assert!(!m.swallows_trigger(0x5A), "redo must reach the app when nothing is restored");
        assert!(!m.swallows_trigger(VK_SHIFT));
    }

    #[test]
    fn key_combo_requires_exact_modifier_set() {
        let mut m = f1_combo();
        assert_eq!(
            m.on_key(VK_F1, true, mods(true, false, false, false), false),
            None,
            "Ctrl+F1 is not F1"
        );
    }

    #[test]
    fn ctrl_f1_combo_does_not_match_plain_f1() {
        let mut m = matcher_with(&[Combo::new(
            Hotkey::MacroPanel,
            mods(true, false, false, false),
            Some(VK_F1),
        )]);
        assert_eq!(m.on_key(VK_F1, true, Modifiers::NONE, false), None);
        assert_eq!(
            m.on_key(VK_F1, true, mods(true, false, false, false), false),
            Some(Hotkey::MacroPanel)
        );
    }

    #[test]
    fn latch_clears_when_modifier_released_before_key() {
        let mut m = matcher_with(&[Combo::new(
            Hotkey::AppExclude,
            mods(false, true, false, false),
            Some(VK_F24),
        )]);
        assert_eq!(
            m.on_key(VK_F24, true, mods(false, true, false, false), false),
            Some(Hotkey::AppExclude)
        );
        // Alt released while F24 still held: the combo can no longer complete.
        m.on_key(VK_MENU, false, Modifiers::NONE, true);
        assert_eq!(
            m.on_key(VK_F24, true, mods(false, true, false, false), false),
            Some(Hotkey::AppExclude),
            "a fresh Alt+F24 must fire again"
        );
    }

    #[test]
    fn press_mode_fires_on_key_down() {
        let mut m = f1_combo();
        assert_eq!(m.mode(), FireMode::Press);
        assert_eq!(
            m.on_key(VK_F1, true, Modifiers::NONE, false),
            Some(Hotkey::Enable)
        );
    }

    #[test]
    fn release_mode_does_not_fire_on_key_down() {
        let mut m = f1_combo();
        m.set_mode(FireMode::Release);
        assert_eq!(m.on_key(VK_F1, true, Modifiers::NONE, false), None);
    }

    #[test]
    fn release_mode_fires_on_key_up() {
        let mut m = f1_combo();
        m.set_mode(FireMode::Release);
        assert_eq!(m.on_key(VK_F1, true, Modifiers::NONE, false), None);
        assert_eq!(
            m.on_key(VK_F1, false, Modifiers::NONE, false),
            Some(Hotkey::Enable)
        );
        assert_eq!(
            m.on_key(VK_F1, false, Modifiers::NONE, false),
            None,
            "one fire per press"
        );
    }

    #[test]
    fn modifier_only_chord_respects_the_mode() {
        let mut m = toggle_combo();
        m.set_mode(FireMode::Release);
        assert_eq!(
            m.on_key(VK_CONTROL, true, mods(true, false, false, false), true),
            None
        );
        assert_eq!(
            m.on_key(VK_SHIFT, true, mods(true, false, true, false), true),
            None,
            "release mode must not fire on completion"
        );
        assert_eq!(
            m.on_key(VK_SHIFT, false, mods(true, false, false, false), true),
            Some(Hotkey::Toggle)
        );
    }

    #[test]
    fn autorepeat_does_not_fire_twice_in_release_mode() {
        let mut m = f1_combo();
        m.set_mode(FireMode::Release);
        m.on_key(VK_F1, true, Modifiers::NONE, false);
        for _ in 0..5 {
            assert_eq!(m.on_key(VK_F1, true, Modifiers::NONE, false), None);
        }
        assert_eq!(
            m.on_key(VK_F1, false, Modifiers::NONE, false),
            Some(Hotkey::Enable)
        );
        assert_eq!(m.on_key(VK_F1, false, Modifiers::NONE, false), None);
    }

    #[test]
    fn table_is_bounded() {
        let mut m = Matcher::new();
        for _ in 0..MAX_COMBOS {
            assert!(m.push(Combo::new(Hotkey::Enable, Modifiers::NONE, Some(VK_F1))));
        }
        assert!(!m.push(Combo::new(Hotkey::Enable, Modifiers::NONE, Some(VK_F1))));
        assert_eq!(m.len(), MAX_COMBOS);
        m.clear();
        assert!(m.is_empty());
        assert!(m.push(Combo::new(Hotkey::Enable, Modifiers::NONE, Some(VK_F1))));
    }

    #[test]
    fn discard_latches_rearms_without_key_up() {
        let mut m = f1_combo();
        m.on_key(VK_F1, true, Modifiers::NONE, false);
        m.discard_latches();
        assert_eq!(
            m.on_key(VK_F1, true, Modifiers::NONE, false),
            Some(Hotkey::Enable)
        );
    }

    // -- D-E2E-02: `Ctrl+Shift` toggle vs `Ctrl+Shift+Z` restore --------------

    const VK_Z: u16 = 0x5A;
    const VK_S: u16 = 0x53;

    fn ctrl() -> Modifiers {
        mods(true, false, false, false)
    }

    fn ctrl_shift() -> Modifiers {
        mods(true, false, true, false)
    }

    fn shift() -> Modifiers {
        mods(false, false, true, false)
    }

    /// `Ctrl+Shift` toggle plus `Ctrl+Shift+Z` restore; `mode` is the toggle mode.
    fn toggle_and_restore(mode: FireMode) -> Matcher {
        let mut m = matcher_with(&[
            Combo::new(Hotkey::Toggle, ctrl_shift(), None),
            Combo::new(Hotkey::RestoreWord, ctrl_shift(), Some(VK_Z)),
        ]);
        m.set_mode(mode);
        m
    }

    #[test]
    fn longer_chord_wins_over_modifier_only_in_release_mode() {
        let mut m = toggle_and_restore(FireMode::Release);
        assert_eq!(m.on_key(VK_CONTROL, true, ctrl(), true), None);
        assert_eq!(m.on_key(VK_SHIFT, true, ctrl_shift(), true), None);
        assert_eq!(m.on_key(VK_Z, true, ctrl_shift(), false), None);
        assert_eq!(
            m.on_key(VK_Z, false, ctrl_shift(), false),
            Some(Hotkey::RestoreWord),
            "the keyed chord fires on release"
        );
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            None,
            "the toggle must not fire as well"
        );
        assert_eq!(m.on_key(VK_CONTROL, false, Modifiers::NONE, true), None);
    }

    #[test]
    fn longer_chord_wins_over_modifier_only_in_press_mode() {
        let mut m = toggle_and_restore(FireMode::Press);
        assert_eq!(m.on_key(VK_CONTROL, true, ctrl(), true), None);
        assert_eq!(
            m.on_key(VK_SHIFT, true, ctrl_shift(), true),
            None,
            "an ambiguous modifier-only chord waits for the release"
        );
        assert_eq!(
            m.on_key(VK_Z, true, ctrl_shift(), false),
            Some(Hotkey::RestoreWord)
        );
        assert_eq!(m.on_key(VK_Z, false, ctrl_shift(), false), None);
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            None,
            "the toggle must not fire as well"
        );
        assert_eq!(m.on_key(VK_CONTROL, false, Modifiers::NONE, true), None);
    }

    #[test]
    fn ambiguous_toggle_still_fires_on_release_in_press_mode() {
        let mut m = toggle_and_restore(FireMode::Press);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        assert_eq!(m.on_key(VK_SHIFT, true, ctrl_shift(), true), None);
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            Some(Hotkey::Toggle)
        );
        // Second press: the latch was cleared, so it toggles back exactly once.
        assert_eq!(m.on_key(VK_SHIFT, true, ctrl_shift(), true), None);
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            Some(Hotkey::Toggle)
        );
    }

    #[test]
    fn full_release_restore_fires_after_every_key_is_up() {
        // `restore_word` is compiled `fire_on_full_release`: typing while Ctrl or
        // Shift is still down would reach the app as a shortcut.
        let mut restore = Combo::new(Hotkey::RestoreWord, ctrl_shift(), Some(VK_Z));
        restore.fire_on_full_release = true;
        let mut m = matcher_with(&[Combo::new(Hotkey::Toggle, ctrl_shift(), None), restore]);
        m.set_mode(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        assert_eq!(m.on_key(VK_SHIFT, true, ctrl_shift(), true), None);
        assert_eq!(m.on_key(VK_Z, true, ctrl_shift(), false), None);
        assert_eq!(
            m.on_key(VK_Z, false, ctrl_shift(), false),
            None,
            "not while the modifiers are still held"
        );
        assert_eq!(m.on_key(VK_SHIFT, false, ctrl(), true), None);
        assert_eq!(
            m.on_key(VK_CONTROL, false, Modifiers::NONE, true),
            Some(Hotkey::RestoreWord),
            "fires once the chord is completely released"
        );
        assert_eq!(m.on_key(VK_CONTROL, false, Modifiers::NONE, true), None);
    }

    #[test]
    fn full_release_restore_also_waits_in_press_mode() {
        let mut restore = Combo::new(Hotkey::RestoreWord, ctrl_shift(), Some(VK_Z));
        restore.fire_on_full_release = true;
        let mut m = matcher_with(&[Combo::new(Hotkey::Toggle, ctrl_shift(), None), restore]);
        m.set_mode(FireMode::Press);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        assert_eq!(m.on_key(VK_SHIFT, true, ctrl_shift(), true), None);
        assert_eq!(m.on_key(VK_Z, true, ctrl_shift(), false), None);
        assert_eq!(m.on_key(VK_Z, false, ctrl_shift(), false), None);
        assert_eq!(m.on_key(VK_SHIFT, false, ctrl(), true), None);
        assert_eq!(
            m.on_key(VK_CONTROL, false, Modifiers::NONE, true),
            Some(Hotkey::RestoreWord)
        );
    }

    #[test]
    fn unrelated_key_cancels_a_full_release_wait() {
        let mut restore = Combo::new(Hotkey::RestoreWord, ctrl_shift(), Some(VK_Z));
        restore.fire_on_full_release = true;
        let mut m = matcher_with(&[restore]);
        m.set_mode(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        m.on_key(VK_Z, true, ctrl_shift(), false);
        m.on_key(VK_Z, false, ctrl_shift(), false);
        // 0x41 = `A`: the user moved on to another chord.
        assert_eq!(m.on_key(0x41, true, ctrl_shift(), false), None);
        assert_eq!(m.on_key(0x41, false, ctrl(), false), None);
        assert_eq!(
            m.on_key(VK_CONTROL, false, Modifiers::NONE, true),
            None,
            "the cancelled restore must not fire later"
        );
    }

    #[test]
    fn key_released_before_modifiers_in_release_mode() {
        let mut m = toggle_and_restore(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        m.on_key(VK_Z, true, ctrl_shift(), false);
        assert_eq!(
            m.on_key(VK_Z, false, ctrl_shift(), false),
            Some(Hotkey::RestoreWord)
        );
        assert_eq!(m.on_key(VK_SHIFT, false, ctrl(), true), None);
        assert_eq!(m.on_key(VK_CONTROL, false, Modifiers::NONE, true), None);
    }

    #[test]
    fn shift_released_before_key_does_not_double_action() {
        let mut m = toggle_and_restore(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        m.on_key(VK_Z, true, ctrl_shift(), false);
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            Some(Hotkey::RestoreWord),
            "breaking the modifier set releases the pending restore"
        );
        assert_eq!(
            m.on_key(VK_Z, false, ctrl(), false),
            None,
            "the toggle never fires, and the restore fires only once"
        );
        assert_eq!(m.on_key(VK_CONTROL, false, Modifiers::NONE, true), None);
    }

    #[test]
    fn control_released_before_key_fires_only_once() {
        let mut m = toggle_and_restore(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        m.on_key(VK_Z, true, ctrl_shift(), false);
        assert_eq!(
            m.on_key(VK_CONTROL, false, shift(), true),
            Some(Hotkey::RestoreWord),
            "breaking the modifier set releases the pending restore"
        );
        assert_eq!(m.on_key(VK_Z, false, shift(), false), None);
        assert_eq!(m.on_key(VK_SHIFT, false, Modifiers::NONE, true), None);
    }

    #[test]
    fn autorepeat_of_the_longer_chord_fires_once() {
        let mut m = toggle_and_restore(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        m.on_key(VK_Z, true, ctrl_shift(), false);
        for _ in 0..5 {
            assert_eq!(
                m.on_key(VK_Z, true, ctrl_shift(), false),
                None,
                "held chord must not fire again"
            );
        }
        assert_eq!(
            m.on_key(VK_Z, false, ctrl_shift(), false),
            Some(Hotkey::RestoreWord)
        );
        assert_eq!(m.on_key(VK_Z, false, ctrl_shift(), false), None);
        assert_eq!(m.on_key(VK_SHIFT, false, ctrl(), true), None);
    }

    #[test]
    fn extra_key_cancels_the_modifier_only_toggle() {
        let mut m = matcher_with(&[Combo::new(Hotkey::Toggle, ctrl_shift(), None)]);
        m.set_mode(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        // 0x41 = `A`, no combo owns it: still typing, not toggling.
        assert_eq!(m.on_key(0x41, true, ctrl_shift(), false), None);
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            None,
            "the extra key cancels the modifier-only action"
        );
        assert_eq!(m.on_key(0x41, false, ctrl(), false), None);
        // The next hold is clean: no stuck latch.
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            Some(Hotkey::Toggle)
        );
    }

    #[test]
    fn two_modifier_only_combos_fire_independently() {
        let mut m = matcher_with(&[
            Combo::new(Hotkey::Toggle, ctrl_shift(), None),
            Combo::new(Hotkey::CycleMethod, mods(true, true, false, false), None),
        ]);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        assert_eq!(
            m.on_key(VK_SHIFT, true, ctrl_shift(), true),
            Some(Hotkey::Toggle)
        );
        assert_eq!(m.on_key(VK_SHIFT, false, ctrl(), true), None);
        assert_eq!(
            m.on_key(VK_MENU, true, mods(true, true, false, false), true),
            Some(Hotkey::CycleMethod)
        );
        assert_eq!(m.on_key(VK_MENU, false, ctrl(), true), None);
        assert_eq!(m.on_key(VK_CONTROL, false, Modifiers::NONE, true), None);
    }

    // -- CH1: the measured E2E sequence, replayed keystroke by keystroke -------

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ReleaseOrder {
        /// `release shift`: Shift up, then Ctrl up.
        ShiftFirst,
        /// `release ctrl`: Ctrl up, then Shift up.
        CtrlFirst,
        /// `release both`: the pair comes up back to back, key order unspecified.
        Both,
    }

    /// `Ctrl` down, `Shift` down (with autorepeat), then the chord's release.
    /// Returns every action the matcher fired along the way.
    fn press_ctrl_shift(m: &mut Matcher, order: ReleaseOrder) -> Vec<Option<Hotkey>> {
        let mut fired = vec![
            m.on_key(VK_CONTROL, true, ctrl(), true),
            m.on_key(VK_SHIFT, true, ctrl_shift(), true),
            // Autorepeat of a held modifier must never re-fire.
            m.on_key(VK_SHIFT, true, ctrl_shift(), true),
            m.on_key(VK_CONTROL, true, ctrl_shift(), true),
        ];
        match order {
            ReleaseOrder::ShiftFirst | ReleaseOrder::Both => {
                fired.push(m.on_key(VK_SHIFT, false, ctrl(), true));
                fired.push(m.on_key(VK_CONTROL, false, Modifiers::NONE, true));
            }
            ReleaseOrder::CtrlFirst => {
                fired.push(m.on_key(VK_CONTROL, false, shift(), true));
                fired.push(m.on_key(VK_SHIFT, false, Modifiers::NONE, true));
            }
        }
        fired
    }

    #[test]
    fn modifier_only_toggle_fires_exactly_once_in_every_release_order() {
        for mode in [FireMode::Press, FireMode::Release] {
            for order in [
                ReleaseOrder::ShiftFirst,
                ReleaseOrder::CtrlFirst,
                ReleaseOrder::Both,
            ] {
                let mut m = toggle_and_restore(mode);
                let fired = press_ctrl_shift(&mut m, order);
                let toggles = fired.iter().filter(|h| **h == Some(Hotkey::Toggle)).count();
                assert_eq!(
                    toggles, 1,
                    "{mode:?} / {order:?}: `Ctrl+Shift` must toggle exactly once, fired {fired:?}"
                );
                assert!(
                    !fired.contains(&Some(Hotkey::RestoreWord)),
                    "{mode:?} / {order:?}: no key was pressed, restore must not fire"
                );
            }
        }
    }

    #[test]
    fn a_beaten_chord_does_not_latch_the_next_toggle() {
        // `Ctrl+Shift+X`: X is unbound, so the hold is typing, not a toggle.
        let mut m = toggle_and_restore(FireMode::Release);
        let mut fired = vec![
            m.on_key(VK_CONTROL, true, ctrl(), true),
            m.on_key(VK_SHIFT, true, ctrl_shift(), true),
        ];
        const VK_X: u16 = 0x58;
        fired.push(m.on_key(VK_X, true, ctrl_shift(), false));
        fired.push(m.on_key(VK_X, false, ctrl_shift(), false));
        fired.push(m.on_key(VK_SHIFT, false, ctrl(), true));
        fired.push(m.on_key(VK_CONTROL, false, Modifiers::NONE, true));
        assert!(
            fired.iter().all(Option::is_none),
            "a beaten chord must fire nothing, fired {fired:?}"
        );

        // The next clean `Ctrl+Shift` still toggles exactly once.
        let next = press_ctrl_shift(&mut m, ReleaseOrder::ShiftFirst);
        assert_eq!(
            next.iter().filter(|h| **h == Some(Hotkey::Toggle)).count(),
            1,
            "no latch may survive the beaten chord, fired {next:?}"
        );
    }

    #[test]
    fn third_combo_sharing_the_modifiers_keeps_the_toggle_on_release() {
        let mut m = matcher_with(&[
            Combo::new(Hotkey::Toggle, ctrl_shift(), None),
            Combo::new(Hotkey::RestoreWord, ctrl_shift(), Some(VK_Z)),
            Combo::new(Hotkey::OpenSettings, ctrl_shift(), Some(VK_S)),
        ]);
        m.set_mode(FireMode::Release);
        m.on_key(VK_CONTROL, true, ctrl(), true);
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        m.on_key(VK_S, true, ctrl_shift(), false);
        assert_eq!(
            m.on_key(VK_S, false, ctrl_shift(), false),
            Some(Hotkey::OpenSettings)
        );
        assert_eq!(m.on_key(VK_SHIFT, false, ctrl(), true), None);
        // `Ctrl+Shift` alone still toggles.
        m.on_key(VK_SHIFT, true, ctrl_shift(), true);
        assert_eq!(
            m.on_key(VK_SHIFT, false, ctrl(), true),
            Some(Hotkey::Toggle)
        );
    }
}
