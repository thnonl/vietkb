//! Compile the configured [`HotkeySet`] into the tap-side [`Matcher`].
//!
//! Runs only when the config changes, never per keystroke. Modifier naming on
//! macOS (decision M2): `Ctrl` = Control, `Alt` = Option, `Win` = Command.

use vkb_config::{Hotkey as ConfigHotkey, HotkeyKey, HotkeySet};
use vkb_core::Modifiers;
use vkb_platform::hotkeys::{Combo, FireMode, Matcher};
use vkb_platform::Hotkey as Action;

use crate::keys;

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
            match to_combo(action, hotkey) {
                Some(combo) => {
                    matcher.push(combo);
                }
                None => tracing::warn!(hotkey = %hotkey.as_str(), "hotkey has no key; ignored"),
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
    // `restore_word` types into the target: wait until the chord is fully up.
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
        'a'..='z' => Some(((ch as u8 - b'a') as u16 + 0x41, false)),
        'A'..='Z' => Some(((ch as u8 - b'A') as u16 + 0x41, false)),
        '0'..='9' => Some(((ch as u8 - b'0') as u16 + 0x30, false)),
        other => keys::us_vk_for_char(other)
            .map(|(vk, shift)| (vk, shift && other.is_ascii_punctuation())),
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
        "Insert" => keys::VK_INSERT,
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
    use vkb_config::{Hotkey as Cfg, ToggleMode};

    const CTRL_SHIFT: Modifiers = Modifiers { shift: true, ctrl: true, alt: false, win: false };
    const CTRL_ALT: Modifiers = Modifiers { shift: false, ctrl: true, alt: true, win: false };
    const NO_MODS: Modifiers = Modifiers::NONE;

    fn only(hotkey: &str, mode: ToggleMode, field: fn(&mut HotkeySet, Option<Cfg>)) -> Matcher {
        let mut set = HotkeySet {
            enable: None,
            disable: None,
            toggle: None,
            restore_word: None,
            app_exclude: None,
            macro_panel: None,
            macro_toggle: None,
            open_settings: None,
            cycle_method: None,
            toggle_mode: mode,
        };
        field(&mut set, Cfg::parse(hotkey).ok());
        compile(&set)
    }

    #[test]
    fn default_toggle_is_ctrl_shift_released_on_release() {
        let set = HotkeySet::default();
        assert_eq!(set.toggle_mode, ToggleMode::Release);
        let mut matcher = compile(&set);
        assert_eq!(matcher.mode(), FireMode::Release);
        // Control down, Shift down (flags now say both), Shift up: fires on release.
        assert_eq!(matcher.on_key(keys::VK_LCONTROL, true, Modifiers { ctrl: true, ..NO_MODS }, true), None);
        assert_eq!(matcher.on_key(keys::VK_LSHIFT, true, CTRL_SHIFT, true), None);
        assert_eq!(
            matcher.on_key(keys::VK_LSHIFT, false, Modifiers { ctrl: true, ..NO_MODS }, true),
            Some(Action::Toggle)
        );
        // Re-armed: pressing again fires again.
        matcher.on_key(keys::VK_LCONTROL, false, NO_MODS, true);
        matcher.on_key(keys::VK_RSHIFT, true, Modifiers { shift: true, ..NO_MODS }, true);
        matcher.on_key(keys::VK_LCONTROL, true, CTRL_SHIFT, true);
        assert_eq!(
            matcher.on_key(keys::VK_LCONTROL, false, Modifiers { shift: true, ..NO_MODS }, true),
            Some(Action::Toggle)
        );
    }

    #[test]
    fn typing_inside_the_toggle_chord_cancels_it() {
        let mut matcher = compile(&HotkeySet::default());
        matcher.on_key(keys::VK_LCONTROL, true, Modifiers { ctrl: true, ..NO_MODS }, true);
        matcher.on_key(keys::VK_LSHIFT, true, CTRL_SHIFT, true);
        // Ctrl+Shift+Q (not a configured combo): the toggle must not fire afterwards.
        assert_eq!(matcher.on_key(0x51, true, CTRL_SHIFT, false), None);
        matcher.on_key(0x51, false, CTRL_SHIFT, false);
        assert_eq!(
            matcher.on_key(keys::VK_LSHIFT, false, Modifiers { ctrl: true, ..NO_MODS }, true),
            None
        );
    }

    #[test]
    fn open_settings_is_control_option_s() {
        let mut matcher = only("Ctrl+Alt+S", ToggleMode::Press, |s, h| s.open_settings = h);
        assert_eq!(matcher.len(), 1);
        assert_eq!(matcher.on_key(0x53, true, CTRL_ALT, false), Some(Action::OpenSettings));
        // Command+S (win slot) is not it.
        let mut matcher = only("Ctrl+Alt+S", ToggleMode::Press, |s, h| s.open_settings = h);
        assert_eq!(
            matcher.on_key(0x53, true, Modifiers { win: true, ..NO_MODS }, false),
            None
        );
    }

    #[test]
    fn command_maps_to_the_win_modifier() {
        let mut matcher = only("Win+Shift+M", ToggleMode::Press, |s, h| s.cycle_method = h);
        let mods = Modifiers { win: true, shift: true, ..NO_MODS };
        assert_eq!(matcher.on_key(0x4D, true, mods, false), Some(Action::CycleMethod));
    }

    #[test]
    fn restore_word_fires_only_after_the_chord_is_released() {
        let mut matcher = only("Ctrl+Shift+Z", ToggleMode::Press, |s, h| s.restore_word = h);
        assert_eq!(matcher.on_key(0x5A, true, CTRL_SHIFT, false), None);
        assert!(matcher.awaiting_release());
        assert_eq!(matcher.on_key(0x5A, false, CTRL_SHIFT, false), None, "Ctrl/Shift still down");
        assert_eq!(
            matcher.on_key(keys::VK_LSHIFT, false, Modifiers { ctrl: true, ..NO_MODS }, true),
            None
        );
        assert_eq!(
            matcher.on_key(keys::VK_LCONTROL, false, NO_MODS, true),
            Some(Action::RestoreWord)
        );
    }

    #[test]
    fn function_key_hotkey_swallows_its_trigger_in_release_mode() {
        let mut matcher = only("F5", ToggleMode::Release, |s, h| s.macro_panel = h);
        assert_eq!(matcher.on_key(keys::VK_F1 + 4, true, NO_MODS, false), None);
        assert!(matcher.swallows_trigger(keys::VK_F1 + 4));
        assert_eq!(
            matcher.on_key(keys::VK_F1 + 4, false, NO_MODS, false),
            Some(Action::MacroPanel)
        );
    }

    #[test]
    fn unbound_slots_are_skipped_and_named_keys_resolve() {
        let matcher = only("Ctrl+Alt+Space", ToggleMode::Press, |s, h| s.macro_toggle = h);
        assert_eq!(matcher.len(), 1);
        assert_eq!(named_vk("Space"), Some((keys::VK_SPACE, false)));
        assert_eq!(named_vk("F12"), Some((keys::VK_F1 + 11, false)));
        assert_eq!(named_vk("PageDown"), Some((keys::VK_NEXT, false)));
        assert_eq!(char_vk('z'), Some((0x5A, false)));
        assert_eq!(char_vk('Z'), Some((0x5A, false)));
        assert_eq!(char_vk('1'), Some((0x31, false)));
        let none = HotkeySet {
            enable: None,
            disable: None,
            toggle: None,
            restore_word: None,
            app_exclude: None,
            macro_panel: None,
            macro_toggle: None,
            open_settings: None,
            cycle_method: None,
            toggle_mode: ToggleMode::Release,
        };
        assert!(compile(&none).is_empty());
    }

    /// Every key token the Mac capture field can produce (`macKeyFromCode` in
    /// ui/src/lib/hotkey.ts) with the macOS keycode of that physical key.
    fn ui_tokens() -> Vec<(String, u16)> {
        let mut out: Vec<(String, u16)> = Vec::new();
        let letters: [(char, u16); 26] = [
            ('A', 0), ('B', 11), ('C', 8), ('D', 2), ('E', 14), ('F', 3), ('G', 5), ('H', 4),
            ('I', 34), ('J', 38), ('K', 40), ('L', 37), ('M', 46), ('N', 45), ('O', 31),
            ('P', 35), ('Q', 12), ('R', 15), ('S', 1), ('T', 17), ('U', 32), ('V', 9),
            ('W', 13), ('X', 7), ('Y', 16), ('Z', 6),
        ];
        for (ch, code) in letters {
            out.push((ch.to_string(), code));
        }
        let digits: [(char, u16); 10] = [
            ('0', 29), ('1', 18), ('2', 19), ('3', 20), ('4', 21), ('5', 23), ('6', 22),
            ('7', 26), ('8', 28), ('9', 25),
        ];
        for (ch, code) in digits {
            out.push((ch.to_string(), code));
        }
        for (token, code) in [
            ("-", 27), ("=", 24), ("[", 33), ("]", 30), (";", 41), ("'", 39), (",", 43),
            (".", 47), ("/", 44), ("\\", 42), ("`", 50), ("Space", 49), ("Up", 126),
            ("Down", 125), ("Left", 123), ("Right", 124), ("Home", 115), ("End", 119),
            ("PageUp", 116), ("PageDown", 121), ("Insert", 114),
        ] {
            out.push((token.to_string(), code));
        }
        let fkeys: [u16; 20] = [
            122, 120, 99, 118, 96, 97, 98, 100, 101, 109, 103, 111, 105, 107, 113, 106, 64, 79,
            80, 90,
        ];
        for (i, code) in fkeys.into_iter().enumerate() {
            out.push((format!("F{}", i + 1), code));
        }
        out
    }

    #[test]
    fn every_token_the_mac_ui_records_parses_and_matches_its_physical_key() {
        let mods = Modifiers { ctrl: true, alt: true, ..NO_MODS };
        for (token, code) in ui_tokens() {
            let text = format!("Ctrl+Alt+{token}");
            let hotkey = Cfg::parse(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(hotkey.as_str(), text, "canonical round trip of {text}");
            let combo = to_combo(Action::Toggle, &hotkey)
                .unwrap_or_else(|| panic!("{text}: no virtual key"));
            let vk = keys::vk_for_keycode(code);
            assert_ne!(vk, keys::VK_NONE, "{token}: keycode {code} has no vk");
            assert_eq!(combo.vk, Some(vk), "{text}: vk");
            assert_eq!(combo.modifiers, mods, "{text}: the key must not force Shift");
            // End to end through the matcher, as the tap drives it.
            let mut matcher = Matcher::new();
            assert!(matcher.push(combo));
            assert_eq!(matcher.on_key(vk, true, mods, false), Some(Action::Toggle), "{text}");
            assert!(matcher.swallows_trigger(vk), "{text}: trigger is swallowed");
            matcher.on_key(vk, false, mods, false);
            // Another modifier set does not fire it.
            assert_eq!(
                matcher.on_key(vk, true, Modifiers { shift: true, ..mods }, false),
                None,
                "{text}: extra Shift"
            );
        }
    }

    #[test]
    fn each_modifier_and_a_token_round_trips_for_all_four_modifiers() {
        for (prefix, mods) in [
            ("Ctrl", Modifiers { ctrl: true, ..NO_MODS }),
            ("Alt", Modifiers { alt: true, ..NO_MODS }),
            ("Shift", Modifiers { shift: true, ..NO_MODS }),
            ("Win", Modifiers { win: true, ..NO_MODS }),
        ] {
            let hotkey = Cfg::parse(&format!("{prefix}+-")).unwrap();
            let combo = to_combo(Action::Toggle, &hotkey).unwrap();
            assert_eq!(combo.modifiers, mods, "{prefix}");
            assert_eq!(combo.vk, Some(keys::VK_OEM_MINUS));
        }
    }

    #[test]
    fn modifier_only_chords_fire_on_release() {
        let ctrl = Modifiers { ctrl: true, ..NO_MODS };
        let alt = Modifiers { alt: true, ..NO_MODS };
        let shift = Modifiers { shift: true, ..NO_MODS };
        let win = Modifiers { win: true, ..NO_MODS };
        let both = |a: Modifiers, b: Modifiers| Modifiers {
            ctrl: a.ctrl || b.ctrl,
            alt: a.alt || b.alt,
            shift: a.shift || b.shift,
            win: a.win || b.win,
        };
        let cases = [
            ("Ctrl+Alt", ctrl, keys::VK_LCONTROL, alt, keys::VK_LMENU),
            ("Alt+Shift", alt, keys::VK_LMENU, shift, keys::VK_LSHIFT),
            ("Shift+Win", shift, keys::VK_LSHIFT, win, keys::VK_LWIN),
            ("Ctrl+Win", ctrl, keys::VK_LCONTROL, win, keys::VK_LWIN),
            ("Ctrl+Shift", ctrl, keys::VK_LCONTROL, shift, keys::VK_LSHIFT),
            ("Alt+Win", alt, keys::VK_LMENU, win, keys::VK_LWIN),
        ];
        for (text, a, vk_a, b, vk_b) in cases {
            let mut matcher = only(text, ToggleMode::Release, |s, h| s.toggle = h);
            assert_eq!(matcher.len(), 1, "{text} compiles");
            assert_eq!(matcher.on_key(vk_a, true, a, true), None, "{text}");
            assert_eq!(matcher.on_key(vk_b, true, both(a, b), true), None, "{text}");
            assert_eq!(matcher.on_key(vk_b, false, a, true), Some(Action::Toggle), "{text}");
        }
    }

    #[test]
    fn a_non_ascii_character_cannot_be_a_hotkey_key() {
        // What Option+- would have recorded before the Mac UI used event.code.
        let hotkey = Cfg::parse("Alt+\u{2013}").unwrap();
        assert!(to_combo(Action::Toggle, &hotkey).is_none());
    }

    #[test]
    fn every_mac_default_hotkey_compiles() {
        let matcher = compile(&HotkeySet::default());
        // toggle + restore_word + open_settings always; F-keys only off macOS.
        let expected = if cfg!(target_os = "macos") { 3 } else { 8 };
        assert_eq!(matcher.len(), expected);
    }
}
