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

    #[test]
    fn every_mac_default_hotkey_compiles() {
        let matcher = compile(&HotkeySet::default());
        // toggle + restore_word + open_settings always; F-keys only off macOS.
        let expected = if cfg!(target_os = "macos") { 3 } else { 8 };
        assert_eq!(matcher.len(), expected);
    }
}
