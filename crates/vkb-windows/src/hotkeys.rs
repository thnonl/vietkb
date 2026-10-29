//! Compile the configured [`HotkeySet`] into the hook-side [`Matcher`].
//!
//! Runs only when the config changes — never per keystroke, and it allocates
//! nothing into the matcher (fixed table).

use vkb_config::{Hotkey as ConfigHotkey, HotkeyKey, HotkeySet};
use vkb_core::Modifiers;
use vkb_platform::hotkeys::{Combo, Matcher};
use vkb_platform::Hotkey as Action;

use crate::keys;

/// One row per configurable action; the order is irrelevant to behaviour.
pub fn compile(set: &HotkeySet) -> Matcher {
    let mut matcher = Matcher::new();
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
    Some(Combo::new(action, modifiers, vk))
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
        let mut matcher = compile(&HotkeySet::default());
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
    fn cycle_hotkey_compiles_and_fires() {
        // The `Đổi kiểu gõ` row: unbound by default, bound here.
        let set = HotkeySet {
            cycle_method: vkb_config::Hotkey::parse("Ctrl+Alt+M").ok(),
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
            Some(Action::Toggle)
        );
    }

    #[test]
    fn ctrl_alt_letter_combo_maps_to_the_letter_key() {
        let set = HotkeySet {
            open_settings: vkb_config::Hotkey::parse("Ctrl+Alt+V").ok(),
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
