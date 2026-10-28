//! Virtual-key bookkeeping: modifier tracking, US-layout character mapping.
//!
//! Everything here except [`seed_from_os`] is pure so the mapping tables are
//! unit-tested; the hook's hot path never calls into the OS to resolve a letter.

use vkb_core::Modifiers;

// Virtual-key codes (Win32). Kept local so the pure code has no windows-crate
// dependency and the tests stay fast.
pub const VK_BACK: u16 = 0x08;
pub const VK_TAB: u16 = 0x09;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12;
pub const VK_CAPITAL: u16 = 0x14;
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_SPACE: u16 = 0x20;
pub const VK_PRIOR: u16 = 0x21;
pub const VK_NEXT: u16 = 0x22;
pub const VK_END: u16 = 0x23;
pub const VK_HOME: u16 = 0x24;
pub const VK_LEFT: u16 = 0x25;
pub const VK_UP: u16 = 0x26;
pub const VK_RIGHT: u16 = 0x27;
pub const VK_DOWN: u16 = 0x28;
pub const VK_DELETE: u16 = 0x2E;
pub const VK_LWIN: u16 = 0x5B;
pub const VK_RWIN: u16 = 0x5C;
pub const VK_LSHIFT: u16 = 0xA0;
pub const VK_RSHIFT: u16 = 0xA1;
pub const VK_LCONTROL: u16 = 0xA2;
pub const VK_RCONTROL: u16 = 0xA3;
pub const VK_LMENU: u16 = 0xA4;
pub const VK_RMENU: u16 = 0xA5;
pub const VK_OEM_1: u16 = 0xBA;
pub const VK_OEM_PLUS: u16 = 0xBB;
pub const VK_OEM_COMMA: u16 = 0xBC;
pub const VK_OEM_MINUS: u16 = 0xBD;
pub const VK_OEM_PERIOD: u16 = 0xBE;
pub const VK_OEM_2: u16 = 0xBF;
pub const VK_OEM_3: u16 = 0xC0;
pub const VK_OEM_4: u16 = 0xDB;
pub const VK_OEM_5: u16 = 0xDC;
pub const VK_OEM_6: u16 = 0xDD;
pub const VK_OEM_7: u16 = 0xDE;
pub const VK_PACKET: u16 = 0xE7;

pub const VK_F1: u16 = 0x70;
pub const VK_F2: u16 = 0x71;
pub const VK_F13: u16 = 0x7C;
pub const VK_F24: u16 = 0x87;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModifierKey {
    Ctrl,
    Alt,
    Shift,
    Win,
}

/// Classify a virtual key as a modifier.
pub fn modifier_of(vk: u16) -> Option<ModifierKey> {
    match vk {
        VK_CONTROL | VK_LCONTROL | VK_RCONTROL => Some(ModifierKey::Ctrl),
        VK_MENU | VK_LMENU | VK_RMENU => Some(ModifierKey::Alt),
        VK_SHIFT | VK_LSHIFT | VK_RSHIFT => Some(ModifierKey::Shift),
        VK_LWIN | VK_RWIN => Some(ModifierKey::Win),
        _ => None,
    }
}

pub fn is_modifier(vk: u16) -> bool {
    modifier_of(vk).is_some()
}

/// Physical modifier state, tracked from the hook stream (one source of truth,
/// so a stuck `GetKeyState` cannot leave the toggle latched).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierState {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    pub caps_lock: bool,
}

impl ModifierState {
    /// Apply one transition. Caps Lock toggles on key-down.
    pub fn apply(&mut self, vk: u16, down: bool) {
        match modifier_of(vk) {
            Some(ModifierKey::Ctrl) => self.ctrl = down,
            Some(ModifierKey::Alt) => self.alt = down,
            Some(ModifierKey::Shift) => self.shift = down,
            Some(ModifierKey::Win) => self.win = down,
            None => {
                if vk == VK_CAPITAL && down {
                    self.caps_lock = !self.caps_lock;
                }
            }
        }
    }

    pub fn modifiers(&self) -> Modifiers {
        Modifiers {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            win: self.win,
        }
    }

    /// A shortcut chord: the engine must not see these keys as text.
    pub fn has_shortcut_modifier(&self) -> bool {
        self.ctrl || self.alt || self.win
    }

    /// Forget everything. Used when the hook is re-installed and the physical
    /// state is unknown; the OS is re-read once via [`seed_from_os`].
    pub fn reset(&mut self) {
        *self = ModifierState::default();
    }
}

/// Physical state at hook install time, so a modifier already held when the app
/// starts is not mistaken for "released".
pub fn seed_from_os() -> ModifierState {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyState};

    let pressed = |vk: u16| -> bool { (unsafe { GetAsyncKeyState(vk as i32) } as u16) & 0x8000 != 0 };
    ModifierState {
        ctrl: pressed(VK_CONTROL) || pressed(VK_LCONTROL) || pressed(VK_RCONTROL),
        alt: pressed(VK_MENU) || pressed(VK_LMENU) || pressed(VK_RMENU),
        shift: pressed(VK_SHIFT) || pressed(VK_LSHIFT) || pressed(VK_RSHIFT),
        win: pressed(VK_LWIN) || pressed(VK_RWIN),
        caps_lock: (unsafe { GetKeyState(VK_CAPITAL as i32) } & 1) != 0,
    }
}

/// Last-resort US-layout mapping: returns the character a key produces.
///
/// Only used when the foreground layout is US; otherwise the backend asks
/// `ToUnicodeEx` because the mapping belongs to the layout.
pub fn us_char(vk: u16, shift: bool, caps: bool) -> Option<char> {
    match vk {
        0x41..=0x5A => {
            let letter = (b'a' + (vk - 0x41) as u8) as char;
            Some(if shift ^ caps {
                letter.to_ascii_uppercase()
            } else {
                letter
            })
        }
        0x30..=0x39 => {
            const SHIFTED: [char; 10] = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
            let digit = b'0' + (vk - 0x30) as u8;
            Some(if shift {
                SHIFTED[(vk - 0x30) as usize]
            } else {
                digit as char
            })
        }
        VK_SPACE => Some(' '),
        VK_OEM_1 => Some(if shift { ':' } else { ';' }),
        VK_OEM_PLUS => Some(if shift { '+' } else { '=' }),
        VK_OEM_COMMA => Some(if shift { '<' } else { ',' }),
        VK_OEM_MINUS => Some(if shift { '_' } else { '-' }),
        VK_OEM_PERIOD => Some(if shift { '>' } else { '.' }),
        VK_OEM_2 => Some(if shift { '?' } else { '/' }),
        VK_OEM_3 => Some(if shift { '~' } else { '`' }),
        VK_OEM_4 => Some(if shift { '{' } else { '[' }),
        VK_OEM_5 => Some(if shift { '|' } else { '\\' }),
        VK_OEM_6 => Some(if shift { '}' } else { ']' }),
        VK_OEM_7 => Some(if shift { '"' } else { '\'' }),
        _ => None,
    }
}

/// Inverse of [`us_char`] for the `send_standard` path: character -> (VK, needs shift).
pub fn us_vk_for_char(ch: char) -> Option<(u16, bool)> {
    match ch {
        'a'..='z' => Some(((ch as u8 - b'a') as u16 + 0x41, false)),
        'A'..='Z' => Some(((ch as u8 - b'A') as u16 + 0x41, true)),
        '0'..='9' => Some(((ch as u8 - b'0') as u16 + 0x30, false)),
        ' ' => Some((VK_SPACE, false)),
        ';' => Some((VK_OEM_1, false)),
        ':' => Some((VK_OEM_1, true)),
        '=' => Some((VK_OEM_PLUS, false)),
        '+' => Some((VK_OEM_PLUS, true)),
        ',' => Some((VK_OEM_COMMA, false)),
        '<' => Some((VK_OEM_COMMA, true)),
        '-' => Some((VK_OEM_MINUS, false)),
        '_' => Some((VK_OEM_MINUS, true)),
        '.' => Some((VK_OEM_PERIOD, false)),
        '>' => Some((VK_OEM_PERIOD, true)),
        '/' => Some((VK_OEM_2, false)),
        '?' => Some((VK_OEM_2, true)),
        '`' => Some((VK_OEM_3, false)),
        '~' => Some((VK_OEM_3, true)),
        '[' => Some((VK_OEM_4, false)),
        '{' => Some((VK_OEM_4, true)),
        '\\' => Some((VK_OEM_5, false)),
        '|' => Some((VK_OEM_5, true)),
        ']' => Some((VK_OEM_6, false)),
        '}' => Some((VK_OEM_6, true)),
        '\'' => Some((VK_OEM_7, false)),
        '"' => Some((VK_OEM_7, true)),
        ')' => Some((0x30, true)),
        '!' => Some((0x31, true)),
        '@' => Some((0x32, true)),
        '#' => Some((0x33, true)),
        '$' => Some((0x34, true)),
        '%' => Some((0x35, true)),
        '^' => Some((0x36, true)),
        '&' => Some((0x37, true)),
        '*' => Some((0x38, true)),
        '(' => Some((0x39, true)),
        _ => None,
    }
}

/// Keys that end or change the current word: the engine decides what to do,
/// the backend only decides whether the key is a text key at all.
pub fn is_word_boundary(vk: u16) -> bool {
    matches!(vk, VK_SPACE | VK_RETURN | VK_TAB | VK_BACK)
}

/// Keys that move the caret or act as commands: the engine's word buffer must
/// be dropped, and the key passes through untouched.
pub fn is_caret_mover(vk: u16) -> bool {
    matches!(
        vk,
        VK_LEFT
            | VK_RIGHT
            | VK_UP
            | VK_DOWN
            | VK_HOME
            | VK_END
            | VK_PRIOR
            | VK_NEXT
            | VK_DELETE
            | VK_ESCAPE
    )
}

/// Scan code (set 1) for a virtual key, for the `send_standard` path.
///
/// A static table instead of `MapVirtualKeyW`: no API round-trip on the inject
/// path, and the mapping is unit-tested. `0` means "no scancode".
pub fn scan_for_vk(vk: u16) -> u16 {
    match vk {
        VK_ESCAPE => 0x01,
        0x31..=0x39 => 0x02 + (vk - 0x31),
        0x30 => 0x0B,
        VK_OEM_MINUS => 0x0C,
        VK_OEM_PLUS => 0x0D,
        VK_BACK => 0x0E,
        VK_TAB => 0x0F,
        0x51 => 0x10, // Q
        0x57 => 0x11, // W
        0x45 => 0x12, // E
        0x52 => 0x13, // R
        0x54 => 0x14, // T
        0x59 => 0x15, // Y
        0x55 => 0x16, // U
        0x49 => 0x17, // I
        0x4F => 0x18, // O
        0x50 => 0x19, // P
        VK_OEM_4 => 0x1A,
        VK_OEM_6 => 0x1B,
        VK_RETURN => 0x1C,
        VK_CONTROL | VK_LCONTROL => 0x1D,
        0x41 => 0x1E, // A
        0x53 => 0x1F, // S
        0x44 => 0x20, // D
        0x46 => 0x21, // F
        0x47 => 0x22, // G
        0x48 => 0x23, // H
        0x4A => 0x24, // J
        0x4B => 0x25, // K
        0x4C => 0x26, // L
        VK_OEM_1 => 0x27,
        VK_OEM_7 => 0x28,
        VK_OEM_3 => 0x29,
        VK_SHIFT | VK_LSHIFT => 0x2A,
        VK_OEM_5 => 0x2B,
        0x5A => 0x2C, // Z
        0x58 => 0x2D, // X
        0x43 => 0x2E, // C
        0x56 => 0x2F, // V
        0x42 => 0x30, // B
        0x4E => 0x31, // N
        0x4D => 0x32, // M
        VK_OEM_COMMA => 0x33,
        VK_OEM_PERIOD => 0x34,
        VK_OEM_2 => 0x35,
        VK_RSHIFT => 0x36,
        VK_MENU | VK_LMENU => 0x38,
        VK_SPACE => 0x39,
        VK_CAPITAL => 0x3A,
        VK_F1 => 0x3B,
        VK_F24 => 0x76,
        0x70..=0x7B => 0x3B + (vk - VK_F1), // F1..F12
        VK_F13..=0x87 => 0x64 + (vk - 0x7C), // F13..F24
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_follow_shift_and_caps() {
        assert_eq!(us_char(0x41, false, false), Some('a'));
        assert_eq!(us_char(0x41, true, false), Some('A'));
        assert_eq!(us_char(0x41, false, true), Some('A'), "caps alone uppercases");
        assert_eq!(us_char(0x41, true, true), Some('a'), "shift+caps lowercases");
        assert_eq!(us_char(0x5A, false, false), Some('z'));
    }

    #[test]
    fn digits_map_to_shifted_symbols() {
        assert_eq!(us_char(0x31, false, false), Some('1'));
        assert_eq!(us_char(0x31, true, false), Some('!'));
        assert_eq!(us_char(0x30, true, false), Some(')'));
        assert_eq!(us_char(0x39, true, false), Some('('));
    }

    #[test]
    fn punctuation_and_control_keys() {
        assert_eq!(us_char(VK_OEM_PERIOD, false, false), Some('.'));
        assert_eq!(us_char(VK_OEM_2, true, false), Some('?'));
        assert_eq!(us_char(VK_SPACE, false, false), Some(' '));
        assert_eq!(us_char(VK_RETURN, false, false), None);
        assert_eq!(us_char(VK_BACK, false, false), None);
        assert_eq!(us_char(VK_F1, false, false), None);
    }

    #[test]
    fn us_char_and_vk_round_trip() {
        for ch in "abcXYZ019 .,;'[]{}\\-=`/~!@#$%^&*()_+<>?:\"|".chars() {
            let (vk, shift) = us_vk_for_char(ch).unwrap_or_else(|| panic!("no vk for {ch:?}"));
            assert_eq!(
                us_char(vk, shift, false),
                Some(ch),
                "round-trip failed for {ch:?}"
            );
        }
    }

    #[test]
    fn non_ascii_has_no_standard_vk() {
        assert_eq!(us_vk_for_char('á'), None);
        assert_eq!(us_vk_for_char('Đ'), None);
    }

    #[test]
    fn modifier_classification() {
        assert_eq!(modifier_of(VK_LCONTROL), Some(ModifierKey::Ctrl));
        assert_eq!(modifier_of(VK_RMENU), Some(ModifierKey::Alt));
        assert_eq!(modifier_of(VK_RSHIFT), Some(ModifierKey::Shift));
        assert_eq!(modifier_of(VK_RWIN), Some(ModifierKey::Win));
        assert_eq!(modifier_of(0x41), None);
        assert!(is_modifier(VK_CONTROL) && !is_modifier(VK_SPACE));
    }

    #[test]
    fn modifier_state_tracks_press_and_release() {
        let mut m = ModifierState::default();
        m.apply(VK_LCONTROL, true);
        m.apply(VK_RSHIFT, true);
        assert_eq!(
            m.modifiers(),
            Modifiers {
                shift: true,
                ctrl: true,
                alt: false,
                win: false
            }
        );
        assert!(m.has_shortcut_modifier());
        m.apply(VK_LCONTROL, false);
        assert!(!m.ctrl && m.shift, "releasing left ctrl keeps right shift");
        m.apply(VK_RSHIFT, false);
        assert_eq!(m.modifiers(), Modifiers::NONE);
        assert!(!m.has_shortcut_modifier());
    }

    #[test]
    fn caps_lock_toggles_only_on_key_down() {
        let mut m = ModifierState::default();
        m.apply(VK_CAPITAL, true);
        assert!(m.caps_lock);
        m.apply(VK_CAPITAL, false);
        assert!(m.caps_lock, "key-up must not toggle");
        m.apply(VK_CAPITAL, true);
        assert!(!m.caps_lock);
    }

    #[test]
    fn boundary_and_mover_sets() {
        assert!(is_word_boundary(VK_SPACE));
        assert!(is_word_boundary(VK_BACK));
        assert!(!is_word_boundary(0x41));
        assert!(is_caret_mover(VK_LEFT));
        assert!(is_caret_mover(VK_HOME));
        assert!(!is_caret_mover(VK_SPACE));
    }

    #[test]
    fn scancodes_cover_the_typing_alphabet() {
        assert_eq!(scan_for_vk(0x41), 0x1E, "A");
        assert_eq!(scan_for_vk(0x5A), 0x2C, "Z");
        assert_eq!(scan_for_vk(0x31), 0x02, "1");
        assert_eq!(scan_for_vk(0x30), 0x0B, "0");
        assert_eq!(scan_for_vk(VK_SPACE), 0x39);
        assert_eq!(scan_for_vk(VK_BACK), 0x0E);
        assert_eq!(scan_for_vk(VK_SHIFT), 0x2A);
        assert_eq!(scan_for_vk(VK_F1), 0x3B);
        assert_eq!(scan_for_vk(VK_F24), 0x76);
        assert_eq!(scan_for_vk(0xFF), 0, "unknown key has no scancode");
    }

    #[test]
    fn every_standard_mode_char_has_a_scancode() {
        for ch in "abcXYZ019 .,;'[]{}\\-=`/~!@#$%^&*()_+<>?:\"|".chars() {
            let (vk, _) = us_vk_for_char(ch).unwrap();
            assert_ne!(scan_for_vk(vk), 0, "no scancode for {ch:?} (vk {vk:#x})");
        }
    }
}
