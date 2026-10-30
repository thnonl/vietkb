//! Compile the configured [`HotkeySet`] into the hook-side [`Matcher`].
//!
//! Runs only when the config changes — never per keystroke, and it allocates
//! nothing into the matcher (fixed table).

use vkb_config::{Hotkey as ConfigHotkey, HotkeyKey, HotkeySet};
use vkb_core::Modifiers;
use vkb_platform::hotkeys::{Combo, FireMode, Matcher};
use vkb_platform::Hotkey as Action;

use crate::keys;

/// One row per configurable action; the order is irrelevant to behaviour.
pub fn compile(set: &HotkeySet) -> Matcher {
    let mut matcher = Matcher::new();
    matcher.set_mode(match set.toggle_mode {
        vkb_config::ToggleMode::Press => FireMode::Press,
        vkb_config::ToggleMode::Release => FireMode::Release,
    });
    let rows: [(Action, Option<&ConfigHotkey>); 9] = [
        (Action::Enable, set.enable.as_ref()),
        (Action::Disable, set.disable.as_ref()),
        (Action::Toggle, set.toggle.as_ref()),
        (Action::RestoreWord, set.restore_word.as_ref()),
        (Action::AppExclude, set.app_exclude.as_ref()),
        (Action::MacroPanel, set.macro_panel.as_ref()),
        (Action::MacroToggle, set.macro_toggle.as_ref()),
        (Action::OpenSettings, set.open_settings.as_ref()),
        (Action::CycleMethod, set.cycle_method.as_ref()),
    ];
    for (action, hotkey) in rows {
        if let Some(hotkey) = hotkey {
            if let Some(combo) = to_combo(action, hotkey) {
                matcher.push(combo);
            } else {
                tracing::warn!(hotkey = %hotkey.as_str(), "hotkey has no virtual key; ignored");
            }
        }
    }
    matcher
}

fn to_combo(action: Action, hotkey: &ConfigHotkey) -> Option<Combo> {
    let mut modifiers = Modifiers {
        shift: hotkey.modifiers.shift,
        ctrl: hotkey.modifiers.ctrl,
        alt: hotkey.modifiers.alt,
        win: hotkey.modifiers.win,
    };
    let vk = match &hotkey.key {
        None => None,
        Some(key) => {
            let (vk, needs_shift) = vk_for_key(key)?;
            if needs_shift {
                modifiers.shift = true;
            }
            Some(vk)
        }
    };
    let mut combo = Combo::new(action, modifiers, vk);
    // `restore_word` types into the target, so it must run only once the chord is
    // fully released: with Ctrl or Shift still down the injection arrives as a
    // shortcut (`Ctrl+Shift+B`), not as a character.
    combo.fire_on_full_release = action == Action::RestoreWord;
    Some(combo)
}

/// Virtual key + whether the key needs Shift to produce the character.
pub fn vk_for_key(key: &HotkeyKey) -> Option<(u16, bool)> {
    match key {
        HotkeyKey::Char(ch) => char_vk(*ch),
        HotkeyKey::Named(name) => named_vk(name),
    }
}

fn char_vk(ch: char) -> Option<(u16, bool)> {
    match ch {
        // A letter hotkey means the key on the keyboard, whatever its case.
        'a'..='z' => Some(((ch as u8 - b'a') as u16 + 0x41, false)),
        'A'..='Z' => Some(((ch as u8 - b'A') as u16 + 0x41, false)),
        '0'..='9' => Some(((ch as u8 - b'0') as u16 + 0x30, false)),
        other => keys::us_vk_for_char(other).map(|(vk, shift)| (vk, shift && other.is_ascii_punctuation())),
    }
}

fn named_vk(name: &str) -> Option<(u16, bool)> {
    let vk = match name {
        "Space" => keys::VK_SPACE,
        "Enter" => keys::VK_RETURN,
        "Tab" => keys::VK_TAB,
        "Escape" => keys::VK_ESCAPE,
        "Backspace" => keys::VK_BACK,
        "Delete" => keys::VK_DELETE,
        "Insert" => 0x2D,
        "Home" => keys::VK_HOME,
        "End" => keys::VK_END,
        "PageUp" => keys::VK_PRIOR,
        "PageDown" => keys::VK_NEXT,
        "Up" => keys::VK_UP,
        "Down" => keys::VK_DOWN,
        "Left" => keys::VK_LEFT,
        "Right" => keys::VK_RIGHT,
        other => {
            if let Some(rest) = other.strip_prefix('F') {
                if let Ok(n) = rest.parse::<u16>() {
                    if (1..=24).contains(&n) {
                        return Some((keys::VK_F1 + n - 1, false));
                    }
                }
            }
            return keys::us_vk_for_char(other.chars().next()?);
        }
    };
    Some((vk, false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vkb_platform::hotkeys::Combo;
    use vkb_platform::Hotkey as Action;
    use vkb_core::Modifiers;

    fn key(name: &str) -> HotkeyKey {
        vkb_config::Hotkey::parse(name).unwrap().key.unwrap()
    }

    #[test]
    fn default_hotkeys_compile_and_fire() {
        // Press mode keeps the historical key-down behaviour for these assertions.
        let set = HotkeySet {
            toggle_mode: vkb_config::ToggleMode::Press,
            ..HotkeySet::default()
        };
        let mut matcher = compile(&set);

        // F1 F2 F11 F8 F9 plus the design defaults `Ctrl+Shift` (toggle),
        // `Ctrl+Shift+Z` (restore_word), `Ctrl+Alt+S` (open_settings) — see
        // `design/HANDOVER.md` and `70-impl-notes-ui.md`.
        assert_eq!(
            matcher.len(),
            8,
            "F1 F2 F11 F8 F9 + Ctrl+Shift + Ctrl+Shift+Z + Ctrl+Alt+S"
        );

        // F1 with no modifiers enables Vietnamese.
        assert_eq!(
            matcher.on_key(keys::VK_F1, true, Modifiers::NONE, false),
            Some(Action::Enable)
        );
        matcher.on_key(keys::VK_F1, false, Modifiers::NONE, false);
        assert_eq!(
            matcher.on_key(keys::VK_F2, true, Modifiers::NONE, false),
            Some(Action::Disable)
        );
    }

    #[test]
    fn default_config_fires_on_release() {
        let set = HotkeySet::default();
        assert_eq!(set.toggle_mode, vkb_config::ToggleMode::Release);
        let mut matcher = compile(&set);
        assert_eq!(matcher.mode(), FireMode::Release);
        assert_eq!(
            matcher.on_key(keys::VK_F1, true, Modifiers::NONE, false),
            None
        );
        assert_eq!(
            matcher.on_key(keys::VK_F1, false, Modifiers::NONE, false),
            Some(Action::Enable),
            "release mode fires when the key comes up"
        );
    }

    #[test]
    fn cycle_hotkey_compiles_and_fires() {
        // The `Đổi kiểu gõ` row: unbound by default, bound here.
        let set = HotkeySet {
            cycle_method: vkb_config::Hotkey::parse("Ctrl+Alt+M").ok(),
            toggle_mode: vkb_config::ToggleMode::Press,
            ..HotkeySet::default()
        };
        let mut matcher = compile(&set);
        assert_eq!(matcher.len(), 9, "8 defaults + the cycling action");

        let mods = Modifiers {
            shift: false,
            ctrl: true,
            alt: true,
            win: false,
        };
        assert_eq!(
            matcher.on_key(0x4D, true, mods, false),
            Some(Action::CycleMethod)
        );
    }

    #[test]
    fn unbound_hotkeys_are_skipped() {
        // The design defaults bind these three, so unbind them here to exercise the
        // skip path itself.
        let set = HotkeySet {
            toggle: None,
            restore_word: None,
            open_settings: None,
            ..HotkeySet::default()
        };
        let matcher = compile(&set);
        assert_eq!(matcher.len(), 5);
    }

    #[test]
    fn modifier_only_toggle_compiles_without_a_key() {
        let set = HotkeySet {
            toggle: vkb_config::Hotkey::parse("Ctrl+Shift").ok(),
            toggle_mode: vkb_config::ToggleMode::Press,
            ..HotkeySet::default()
        };
        let mut matcher = compile(&set);
        let ctrl = Modifiers {
            shift: false,
            ctrl: true,
            alt: false,
            win: false,
        };
        let both = Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
            win: false,
        };
        assert_eq!(matcher.on_key(keys::VK_CONTROL, true, ctrl, true), None);
        assert_eq!(
            matcher.on_key(keys::VK_SHIFT, true, both, true),
            None,
            "`Ctrl+Shift+Z` is also bound, so the toggle waits for the release"
        );
        assert_eq!(
            matcher.on_key(keys::VK_SHIFT, false, ctrl, true),
            Some(Action::Toggle)
        );
    }

    #[test]
    fn default_restore_chord_beats_the_toggle_in_press_mode() {
        let set = HotkeySet {
            toggle_mode: vkb_config::ToggleMode::Press,
            ..HotkeySet::default()
        };
        let mut matcher = compile(&set);
        let none = Modifiers::NONE;
        let ctrl = Modifiers {
            shift: false,
            ctrl: true,
            alt: false,
            win: false,
        };
        let both = Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
            win: false,
        };
        matcher.on_key(keys::VK_CONTROL, true, ctrl, true);
        assert_eq!(matcher.on_key(keys::VK_SHIFT, true, both, true), None);
        assert_eq!(matcher.on_key(0x5A, true, both, false), None);
        assert_eq!(matcher.on_key(0x5A, false, both, false), None);
        assert_eq!(matcher.on_key(keys::VK_SHIFT, false, ctrl, true), None);
        assert_eq!(
            matcher.on_key(keys::VK_CONTROL, false, none, true),
            Some(Action::RestoreWord),
            "`Ctrl+Shift+Z` restores in press mode, once the chord is up"
        );
    }

    #[test]
    fn owner_config_restore_chord_fires_when_the_chord_is_up() {
        // The owner's live config: `toggle = "Alt+Shift"`, toggle_mode = release,
        // `restore_word = "Ctrl+Shift+Z"` — D-E2E-02. The restore waits for the
        // whole chord: typing while Ctrl/Shift are down would be a shortcut.
        let set = HotkeySet {
            toggle: vkb_config::Hotkey::parse("Alt+Shift").ok(),
            restore_word: vkb_config::Hotkey::parse("Ctrl+Shift+Z").ok(),
            toggle_mode: vkb_config::ToggleMode::Release,
            ..HotkeySet::default()
        };
        let mut matcher = compile(&set);
        let none = Modifiers::NONE;
        let ctrl = Modifiers {
            shift: false,
            ctrl: true,
            alt: false,
            win: false,
        };
        let both = Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
            win: false,
        };
        assert_eq!(matcher.on_key(keys::VK_CONTROL, true, ctrl, true), None);
        assert_eq!(matcher.on_key(keys::VK_SHIFT, true, both, true), None);
        assert_eq!(matcher.on_key(0x5A, true, both, false), None);
        for _ in 0..3 {
            assert_eq!(
                matcher.on_key(0x5A, true, both, false),
                None,
                "autorepeat must not arm twice"
            );
        }
        assert_eq!(matcher.on_key(0x5A, false, both, false), None);
        assert_eq!(matcher.on_key(keys::VK_SHIFT, false, ctrl, true), None);
        assert_eq!(
            matcher.on_key(keys::VK_CONTROL, false, none, true),
            Some(Action::RestoreWord)
        );
        assert_eq!(matcher.on_key(0x5A, false, none, false), None);
    }

    #[test]
    fn default_config_toggles_on_ctrl_shift_in_every_release_order() {
        // The shipped default binds `toggle = "Ctrl+Shift"` next to
        // `restore_word = "Ctrl+Shift+Z"`: the modifier-only chord is an
        // ambiguous prefix, so it must still fire once per press.
        let set = HotkeySet::default();
        assert_eq!(set.toggle_mode, vkb_config::ToggleMode::Release);
        for order in 0..3 {
            let mut matcher = compile(&set);
            assert_eq!(matcher.on_key(keys::VK_CONTROL, true, ctrl(), true), None);
            assert_eq!(
                matcher.on_key(keys::VK_SHIFT, true, both(), true),
                None,
                "waits for the release: `Ctrl+Shift+Z` could still complete"
            );
            let fired = match order {
                0 => matcher.on_key(keys::VK_SHIFT, false, ctrl(), true),
                1 => matcher.on_key(keys::VK_CONTROL, false, shift_only(), true),
                _ => matcher.on_key(keys::VK_SHIFT, false, ctrl(), true),
            };
            assert_eq!(fired, Some(Action::Toggle), "release order {order}");
            assert_eq!(
                matcher.on_key(keys::VK_CONTROL, false, Modifiers::NONE, true),
                None,
                "one fire per press, order {order}"
            );
        }
    }

    /// CH1's measured keystrokes against the config that was actually in force.
    ///
    /// The run's `config.toml` binds `toggle = "Alt+Shift"`
    /// (`qa/e2e/config.backup.toml`, `91-round-plan-sound-cases.md` §B), while
    /// `run-host-e2e.ps1` presses `Ctrl+Shift`; an unbound chord toggles
    /// nothing, which is the `ON -> ON` the gate recorded. `RT2` (Alt+Shift ×2)
    /// passing in the same run is the same fact from the other side.
    #[test]
    fn the_run_config_binds_alt_shift_and_not_ctrl_shift() {
        let set = HotkeySet {
            toggle: vkb_config::Hotkey::parse("Alt+Shift").ok(),
            restore_word: vkb_config::Hotkey::parse("Ctrl+Shift+Z").ok(),
            open_settings: vkb_config::Hotkey::parse("Ctrl+Alt+S").ok(),
            toggle_mode: vkb_config::ToggleMode::Release,
            ..HotkeySet::default()
        };
        let mut matcher = compile(&set);
        matcher.on_key(keys::VK_CONTROL, true, ctrl(), true);
        assert_eq!(matcher.on_key(keys::VK_SHIFT, true, both(), true), None);
        assert_eq!(matcher.on_key(keys::VK_SHIFT, false, ctrl(), true), None);
        assert_eq!(matcher.on_key(keys::VK_CONTROL, false, Modifiers::NONE, true), None);

        // The configured chord does toggle, exactly once.
        assert_eq!(matcher.on_key(keys::VK_MENU, true, alt(), true), None);
        assert_eq!(matcher.on_key(keys::VK_SHIFT, true, alt_shift(), true), None);
        assert_eq!(
            matcher.on_key(keys::VK_SHIFT, false, alt(), true),
            Some(Action::Toggle)
        );
        assert_eq!(matcher.on_key(keys::VK_MENU, false, Modifiers::NONE, true), None);
    }

    fn ctrl() -> Modifiers {
        Modifiers {
            ctrl: true,
            alt: false,
            shift: false,
            win: false,
        }
    }

    fn alt() -> Modifiers {
        Modifiers {
            ctrl: false,
            alt: true,
            shift: false,
            win: false,
        }
    }

    fn shift_only() -> Modifiers {
        Modifiers {
            ctrl: false,
            alt: false,
            shift: true,
            win: false,
        }
    }

    fn both() -> Modifiers {
        Modifiers {
            ctrl: true,
            alt: false,
            shift: true,
            win: false,
        }
    }

    fn alt_shift() -> Modifiers {
        Modifiers {
            ctrl: false,
            alt: true,
            shift: true,
            win: false,
        }
    }

    #[test]
    fn ctrl_alt_letter_combo_maps_to_the_letter_key() {
        let set = HotkeySet {
            open_settings: vkb_config::Hotkey::parse("Ctrl+Alt+V").ok(),
            toggle_mode: vkb_config::ToggleMode::Press,
            ..HotkeySet::default()
        };
        let mut matcher = compile(&set);
        let mods = Modifiers {
            shift: false,
            ctrl: true,
            alt: true,
            win: false,
        };
        assert_eq!(matcher.on_key(0x56, true, mods, false), Some(Action::OpenSettings));
    }

    #[test]
    fn function_and_named_keys_have_virtual_codes() {
        assert_eq!(vk_for_key(&key("F1")), Some((0x70, false)));
        assert_eq!(vk_for_key(&key("F24")), Some((0x87, false)));
        assert_eq!(vk_for_key(&key("Alt+Space")), Some((0x20, false)));
        assert_eq!(vk_for_key(&key("Ctrl+PageDown")), Some((0x22, false)));
        assert_eq!(vk_for_key(&key("Win+Up")), Some((0x26, false)));
    }

    #[test]
    fn shifted_punctuation_hotkey_requires_shift() {
        let parsed = vkb_config::Hotkey::parse("Ctrl+?").unwrap();
        let combo = to_combo(Action::MacroPanel, &parsed).unwrap();
        assert_eq!(combo.vk, Some(keys::VK_OEM_2));
        assert!(combo.modifiers.shift, "'?' needs Shift");
        assert!(combo.modifiers.ctrl);
    }

    #[test]
    fn letters_do_not_gain_shift() {
        let parsed = vkb_config::Hotkey::parse("Ctrl+V").unwrap();
        let combo = to_combo(Action::OpenSettings, &parsed).unwrap();
        assert_eq!(combo.vk, Some(0x56));
        assert!(!combo.modifiers.shift);
    }

    #[test]
    fn combo_is_a_copy_type() {
        let combo = Combo::new(Action::Enable, Modifiers::NONE, Some(0x70));
        let copy = combo;
        assert_eq!(combo, copy);
    }
}
