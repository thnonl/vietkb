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
}

impl Combo {
    pub const fn new(hotkey: Hotkey, modifiers: Modifiers, vk: Option<u16>) -> Self {
        Self {
            hotkey,
            modifiers,
            vk,
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

/// One slot per configurable action (`enable` … `cycle_method`), including the
/// two cycling actions.
pub const MAX_COMBOS: usize = 11;

/// Fixed-capacity table of combos with a per-combo "already fired" latch.
#[derive(Clone, Copy, Debug, Default)]
pub struct Matcher {
    combos: [Option<Combo>; MAX_COMBOS],
    latched: u16,
    len: u8,
}

impl Matcher {
    pub const fn new() -> Self {
        Self {
            combos: [None; MAX_COMBOS],
            latched: 0,
            len: 0,
        }
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
    pub fn on_key(
        &mut self,
        vk: u16,
        down: bool,
        modifiers: Modifiers,
        is_modifier_key: bool,
    ) -> Option<Hotkey> {
        let mut fired = None;
        let len = self.len as usize;
        for i in 0..len {
            let Some(combo) = self.combos[i] else { continue };
            let bit = 1u16 << i;
            if down {
                if self.latched & bit == 0 && combo.completes(vk, modifiers, is_modifier_key) {
                    self.latched |= bit;
                    if fired.is_none() {
                        fired = Some(combo.hotkey);
                    }
                }
            } else if self.latched & bit != 0 && !combo.still_held(vk, modifiers) {
                self.latched &= !bit;
            }
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
            Hotkey::ConvertTool,
            mods(true, false, false, false),
            Some(VK_F1),
        )]);
        assert_eq!(m.on_key(VK_F1, true, Modifiers::NONE, false), None);
        assert_eq!(
            m.on_key(VK_F1, true, mods(true, false, false, false), false),
            Some(Hotkey::ConvertTool)
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
}
