//! macOS virtual keycodes (`kVK_*`, layout independent) -> the Windows virtual-key
//! space that `vkb_core::KeyInput::vk` and the hotkey `Matcher` speak, plus
//! modifier bookkeeping from `CGEventFlags`.
//!
//! Everything here is pure so it is unit-tested without an event tap.

use vkb_core::Modifiers;

// Windows virtual-key codes (the engine's key space).
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
pub const VK_INSERT: u16 = 0x2D;
pub const VK_DELETE: u16 = 0x2E;
pub const VK_LWIN: u16 = 0x5B;
pub const VK_RWIN: u16 = 0x5C;
pub const VK_APPS: u16 = 0x5D;
pub const VK_NUMPAD0: u16 = 0x60;
pub const VK_NUMPAD9: u16 = 0x69;
pub const VK_MULTIPLY: u16 = 0x6A;
pub const VK_ADD: u16 = 0x6B;
pub const VK_SUBTRACT: u16 = 0x6D;
pub const VK_DECIMAL: u16 = 0x6E;
pub const VK_DIVIDE: u16 = 0x6F;
pub const VK_F1: u16 = 0x70;
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
/// Key with no Windows equivalent. The engine treats it as "not a text key".
pub const VK_NONE: u16 = 0xFF;

// macOS keycodes the backend names explicitly.
pub const MAC_RETURN: u16 = 36;
pub const MAC_TAB: u16 = 48;
pub const MAC_SPACE: u16 = 49;
/// The Backspace key (named "Delete" on Apple keyboards). Used for injection.
pub const MAC_BACKSPACE: u16 = 51;
pub const MAC_ESCAPE: u16 = 53;
pub const MAC_RIGHT_COMMAND: u16 = 54;
pub const MAC_COMMAND: u16 = 55;
pub const MAC_SHIFT: u16 = 56;
pub const MAC_CAPS_LOCK: u16 = 57;
pub const MAC_OPTION: u16 = 58;
pub const MAC_CONTROL: u16 = 59;
pub const MAC_RIGHT_SHIFT: u16 = 60;
pub const MAC_RIGHT_OPTION: u16 = 61;
pub const MAC_RIGHT_CONTROL: u16 = 62;
pub const MAC_FUNCTION: u16 = 63;

/// `CGEventFlags` bits.
pub mod flags {
    pub const ALPHA_SHIFT: u64 = 0x0001_0000;
    pub const SHIFT: u64 = 0x0002_0000;
    pub const CONTROL: u64 = 0x0004_0000;
    pub const ALTERNATE: u64 = 0x0008_0000;
    pub const COMMAND: u64 = 0x0010_0000;
    // Device-dependent (IOKit `NX_DEVICE*KEYMASK`) bits: which physical side.
    pub const DEV_LCTRL: u64 = 0x0000_0001;
    pub const DEV_LSHIFT: u64 = 0x0000_0002;
    pub const DEV_RSHIFT: u64 = 0x0000_0004;
    pub const DEV_LCMD: u64 = 0x0000_0008;
    pub const DEV_RCMD: u64 = 0x0000_0010;
    pub const DEV_LALT: u64 = 0x0000_0020;
    pub const DEV_RALT: u64 = 0x0000_0040;
    pub const DEV_RCTRL: u64 = 0x0000_2000;
    pub const DEV_ALL: u64 = DEV_LCTRL
        | DEV_LSHIFT
        | DEV_RSHIFT
        | DEV_LCMD
        | DEV_RCMD
        | DEV_LALT
        | DEV_RALT
        | DEV_RCTRL;
}

/// Windows VK for a macOS keycode (ANSI/ISO/JIS physical keys). `VK_NONE` when
/// the key has no counterpart.
pub fn vk_for_keycode(code: u16) -> u16 {
    match code {
        // Letters.
        0 => 0x41,  // A
        11 => 0x42, // B
        8 => 0x43,  // C
        2 => 0x44,  // D
        14 => 0x45, // E
        3 => 0x46,  // F
        5 => 0x47,  // G
        4 => 0x48,  // H
        34 => 0x49, // I
        38 => 0x4A, // J
        40 => 0x4B, // K
        37 => 0x4C, // L
        46 => 0x4D, // M
        45 => 0x4E, // N
        31 => 0x4F, // O
        35 => 0x50, // P
        12 => 0x51, // Q
        15 => 0x52, // R
        1 => 0x53,  // S
        17 => 0x54, // T
        32 => 0x55, // U
        9 => 0x56,  // V
        13 => 0x57, // W
        7 => 0x58,  // X
        16 => 0x59, // Y
        6 => 0x5A,  // Z
        // Digit row.
        29 => 0x30,
        18 => 0x31,
        19 => 0x32,
        20 => 0x33,
        21 => 0x34,
        23 => 0x35,
        22 => 0x36,
        26 => 0x37,
        28 => 0x38,
        25 => 0x39,
        // Punctuation (US positions).
        27 => VK_OEM_MINUS,
        24 => VK_OEM_PLUS,
        33 => VK_OEM_4,
        30 => VK_OEM_6,
        41 => VK_OEM_1,
        39 => VK_OEM_7,
        42 => VK_OEM_5,
        43 => VK_OEM_COMMA,
        47 => VK_OEM_PERIOD,
        44 => VK_OEM_2,
        50 | 10 => VK_OEM_3, // ` (ANSI) and the ISO key left of Z
        93 => VK_OEM_5,      // JIS yen
        94 => VK_OEM_2,      // JIS underscore
        // Editing and whitespace.
        36 | 76 => VK_RETURN, // Return and keypad Enter
        48 => VK_TAB,
        49 => VK_SPACE,
        51 => VK_BACK,
        53 => VK_ESCAPE,
        117 => VK_DELETE, // forward delete
        114 => VK_INSERT, // Help
        110 => VK_APPS,   // context-menu key
        // Navigation.
        115 => VK_HOME,
        119 => VK_END,
        116 => VK_PRIOR,
        121 => VK_NEXT,
        123 => VK_LEFT,
        124 => VK_RIGHT,
        125 => VK_DOWN,
        126 => VK_UP,
        // Keypad.
        82 => VK_NUMPAD0,
        83 => VK_NUMPAD0 + 1,
        84 => VK_NUMPAD0 + 2,
        85 => VK_NUMPAD0 + 3,
        86 => VK_NUMPAD0 + 4,
        87 => VK_NUMPAD0 + 5,
        88 => VK_NUMPAD0 + 6,
        89 => VK_NUMPAD0 + 7,
        91 => VK_NUMPAD0 + 8,
        92 => VK_NUMPAD9,
        67 => VK_MULTIPLY,
        69 => VK_ADD,
        78 => VK_SUBTRACT,
        65 => VK_DECIMAL,
        75 => VK_DIVIDE,
        81 => VK_OEM_PLUS, // keypad =
        // Function keys.
        122 => VK_F1,
        120 => VK_F1 + 1,
        99 => VK_F1 + 2,
        118 => VK_F1 + 3,
        96 => VK_F1 + 4,
        97 => VK_F1 + 5,
        98 => VK_F1 + 6,
        100 => VK_F1 + 7,
        101 => VK_F1 + 8,
        109 => VK_F1 + 9,
        103 => VK_F1 + 10,
        111 => VK_F1 + 11,
        105 => VK_F1 + 12, // F13
        107 => VK_F1 + 13,
        113 => VK_F1 + 14,
        106 => VK_F1 + 15,
        64 => VK_F1 + 16, // F17
        79 => VK_F1 + 17,
        80 => VK_F1 + 18,
        90 => VK_F1 + 19, // F20
        // Modifiers (flagsChanged).
        MAC_SHIFT => VK_LSHIFT,
        MAC_RIGHT_SHIFT => VK_RSHIFT,
        MAC_CONTROL => VK_LCONTROL,
        MAC_RIGHT_CONTROL => VK_RCONTROL,
        MAC_OPTION => VK_LMENU,
        MAC_RIGHT_OPTION => VK_RMENU,
        MAC_COMMAND => VK_LWIN,
        MAC_RIGHT_COMMAND => VK_RWIN,
        MAC_CAPS_LOCK => VK_CAPITAL,
        _ => VK_NONE,
    }
}

/// macOS keycode of the primary key for a Windows VK (aliases such as keypad
/// Enter are not returned). `None` for keys with no physical equivalent.
pub fn keycode_for_vk(vk: u16) -> Option<u16> {
    if vk == VK_NONE {
        return None;
    }
    (0u16..128)
        .filter(|code| !matches!(*code, 76 | 10 | 93 | 94 | 81))
        .find(|code| vk_for_keycode(*code) == vk)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModifierKey {
    Ctrl,
    Alt,
    Shift,
    Win,
}

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

/// Is the modifier key `keycode` pressed, judging by a `flagsChanged` event's
/// flags? The device-dependent side bits tell left from right; when the sender
/// supplied none (synthetic events), the generic mask is all there is.
pub fn modifier_key_down(keycode: u16, event_flags: u64) -> bool {
    use flags::*;
    let (device, generic) = match keycode {
        MAC_SHIFT => (DEV_LSHIFT, SHIFT),
        MAC_RIGHT_SHIFT => (DEV_RSHIFT, SHIFT),
        MAC_CONTROL => (DEV_LCTRL, CONTROL),
        MAC_RIGHT_CONTROL => (DEV_RCTRL, CONTROL),
        MAC_OPTION => (DEV_LALT, ALTERNATE),
        MAC_RIGHT_OPTION => (DEV_RALT, ALTERNATE),
        MAC_COMMAND => (DEV_LCMD, COMMAND),
        MAC_RIGHT_COMMAND => (DEV_RCMD, COMMAND),
        _ => return false,
    };
    if event_flags & generic == 0 {
        return false;
    }
    if event_flags & DEV_ALL != 0 {
        event_flags & device != 0
    } else {
        true
    }
}

/// Bits of [`ModifierState::shift_keys`].
pub const SHIFT_L: u8 = 1;
pub const SHIFT_R: u8 = 2;

/// Physical modifier state. `CGEventFlags` carries the full state on every
/// event, so [`ModifierState::sync`] rebuilds it from them and a lost event can
/// never leave a modifier latched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierState {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub shift_keys: u8,
    /// Command (the `Win` slot of the shared config: decision M2).
    pub win: bool,
    pub caps_lock: bool,
}

impl ModifierState {
    /// Rebuild from the flags of the event being handled.
    pub fn sync(&mut self, event_flags: u64) {
        use flags::*;
        self.ctrl = event_flags & CONTROL != 0;
        self.alt = event_flags & ALTERNATE != 0;
        self.win = event_flags & COMMAND != 0;
        self.caps_lock = event_flags & ALPHA_SHIFT != 0;
        self.shift = event_flags & SHIFT != 0;
        if !self.shift {
            self.shift_keys = 0;
        } else if event_flags & (DEV_LSHIFT | DEV_RSHIFT) != 0 {
            self.shift_keys = (if event_flags & DEV_LSHIFT != 0 { SHIFT_L } else { 0 })
                | (if event_flags & DEV_RSHIFT != 0 { SHIFT_R } else { 0 });
        } else if self.shift_keys == 0 {
            // No side information: assume the left key.
            self.shift_keys = SHIFT_L;
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
}

/// The character the Shift / Caps Lock *flags* of an event call for, when the
/// event's Unicode string disagrees with them (D-MAC-01).
///
/// Hardware events carry a string that matches their flags. Synthetic ones (UI
/// automation, remote-desktop and remapping tools) get a string translated from
/// the session's modifier state at some other moment: `V` arrives as `v`, and a
/// digit typed right after a Shift release arrives as `^` with clean flags. The
/// app applies the flags, so the engine must follow them too, or `VIE6` never
/// sees its mark key.
///
/// Only when the string is one of the two US characters of that very key is it
/// swapped for the one the flags select; any other string (non-US layouts,
/// Option combinations, dead keys) is the layout's own answer and is kept.
pub fn reconcile_ascii(vk: u16, typed: char, shift: bool, caps: bool) -> char {
    if !typed.is_ascii() {
        return typed;
    }
    let (Some(plain), Some(shifted)) = (us_char(vk, false, false), us_char(vk, true, false)) else {
        return typed;
    };
    if typed != plain && typed != shifted {
        return typed;
    }
    if plain.is_ascii_alphabetic() {
        return if shift ^ caps { shifted } else { plain };
    }
    // Digits and punctuation: only the stale-shift direction. A string equal to
    // the key's plain US character is never promoted to the shifted one, because
    // on layouts such as AZERTY the Shift+digit-row string *is* that character.
    if typed == shifted && !shift {
        plain
    } else {
        typed
    }
}

/// US-layout character for a VK. Used to build hotkey tables; typing uses the
/// character the OS put in the event.
pub fn us_char(vk: u16, shift: bool, caps: bool) -> Option<char> {
    match vk {
        0x41..=0x5A => {
            let letter = (b'a' + (vk - 0x41) as u8) as char;
            Some(if shift ^ caps { letter.to_ascii_uppercase() } else { letter })
        }
        0x30..=0x39 => {
            const SHIFTED: [char; 10] = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
            Some(if shift {
                SHIFTED[(vk - 0x30) as usize]
            } else {
                (b'0' + (vk - 0x30) as u8) as char
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

/// Inverse of [`us_char`]: character -> (VK, needs shift).
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

/// Keys the engine handles itself even though they type no ordinary character.
pub fn is_word_boundary(vk: u16) -> bool {
    matches!(vk, VK_SPACE | VK_RETURN | VK_TAB | VK_BACK)
}

/// Keys that move the caret or act as commands: the word is over.
pub fn is_caret_mover(vk: u16) -> bool {
    matches!(
        vk,
        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_DELETE
            | VK_ESCAPE
    )
}

/// Keypad, F-keys and other keys that never become text, whatever the event's
/// character says (the keypad reports digits; Windows treats it as non-text too).
pub fn never_text(vk: u16) -> bool {
    (VK_NUMPAD0..=VK_DIVIDE).contains(&vk)
        || (VK_F1..=VK_F1 + 23).contains(&vk)
        || matches!(vk, VK_APPS | VK_INSERT)
}

/// The single character an event typed, from its UTF-16 string. `None` for no
/// string, several characters (dead-key composites, IME), and the private-use
/// codes macOS reports for arrows and function keys.
pub fn typed_char(units: &[u16]) -> Option<char> {
    let mut chars = char::decode_utf16(units.iter().copied());
    let first = chars.next()?.ok()?;
    if chars.next().is_some() {
        return None;
    }
    if ('\u{F700}'..='\u{F8FF}').contains(&first) {
        return None;
    }
    Some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_digits_and_punctuation_map_to_windows_vks() {
        assert_eq!(vk_for_keycode(0), 0x41, "A");
        assert_eq!(vk_for_keycode(6), 0x5A, "Z");
        assert_eq!(vk_for_keycode(46), 0x4D, "M");
        assert_eq!(vk_for_keycode(18), 0x31, "1");
        assert_eq!(vk_for_keycode(29), 0x30, "0");
        assert_eq!(vk_for_keycode(47), VK_OEM_PERIOD);
        assert_eq!(vk_for_keycode(41), VK_OEM_1);
        assert_eq!(vk_for_keycode(50), VK_OEM_3);
    }

    #[test]
    fn control_and_navigation_keys_map() {
        assert_eq!(vk_for_keycode(51), VK_BACK);
        assert_eq!(vk_for_keycode(36), VK_RETURN);
        assert_eq!(vk_for_keycode(76), VK_RETURN, "keypad Enter");
        assert_eq!(vk_for_keycode(48), VK_TAB);
        assert_eq!(vk_for_keycode(49), VK_SPACE);
        assert_eq!(vk_for_keycode(53), VK_ESCAPE);
        assert_eq!(vk_for_keycode(117), VK_DELETE);
        assert_eq!(vk_for_keycode(123), VK_LEFT);
        assert_eq!(vk_for_keycode(126), VK_UP);
        assert_eq!(vk_for_keycode(115), VK_HOME);
        assert_eq!(vk_for_keycode(121), VK_NEXT);
        assert_eq!(vk_for_keycode(122), VK_F1);
        assert_eq!(vk_for_keycode(111), VK_F1 + 11, "F12");
        assert_eq!(vk_for_keycode(63), VK_NONE, "fn has no VK");
        assert_eq!(vk_for_keycode(200), VK_NONE);
    }

    #[test]
    fn every_ansi_text_key_has_a_distinct_vk() {
        let mut seen = std::collections::HashSet::new();
        for code in 0u16..=50 {
            if code == 10 {
                continue; // ISO alias of the grave key
            }
            let vk = vk_for_keycode(code);
            assert_ne!(vk, VK_NONE, "keycode {code} unmapped");
            assert!(seen.insert(vk), "keycode {code} collides on vk {vk:#x}");
        }
    }

    #[test]
    fn keycode_for_vk_inverts_the_primary_keys() {
        for code in [0u16, 6, 18, 29, 36, 48, 49, 51, 53, 117, 123, 122, 41, 50] {
            assert_eq!(keycode_for_vk(vk_for_keycode(code)), Some(code), "code {code}");
        }
        assert_eq!(keycode_for_vk(VK_NONE), None);
    }

    #[test]
    fn modifier_keys_classify_by_side() {
        assert_eq!(modifier_of(vk_for_keycode(MAC_SHIFT)), Some(ModifierKey::Shift));
        assert_eq!(modifier_of(vk_for_keycode(MAC_RIGHT_OPTION)), Some(ModifierKey::Alt));
        assert_eq!(modifier_of(vk_for_keycode(MAC_COMMAND)), Some(ModifierKey::Win));
        assert_eq!(modifier_of(vk_for_keycode(MAC_RIGHT_CONTROL)), Some(ModifierKey::Ctrl));
        assert_eq!(modifier_of(vk_for_keycode(MAC_CAPS_LOCK)), None, "caps lock is no chord modifier");
        assert!(!is_modifier(vk_for_keycode(0)));
    }

    #[test]
    fn modifier_key_down_uses_side_bits() {
        use flags::*;
        // Both shifts were held, the left one was released.
        assert!(!modifier_key_down(MAC_SHIFT, SHIFT | DEV_RSHIFT));
        assert!(modifier_key_down(MAC_RIGHT_SHIFT, SHIFT | DEV_RSHIFT));
        assert!(modifier_key_down(MAC_SHIFT, SHIFT | DEV_LSHIFT));
        // Synthetic event without side bits: the generic mask decides.
        assert!(modifier_key_down(MAC_CONTROL, CONTROL));
        assert!(!modifier_key_down(MAC_CONTROL, 0));
        assert!(!modifier_key_down(0, SHIFT), "a letter is not a modifier");
    }

    #[test]
    fn modifier_state_rebuilds_from_flags() {
        use flags::*;
        let mut m = ModifierState::default();
        m.sync(CONTROL | SHIFT | DEV_LCTRL | DEV_RSHIFT);
        assert_eq!(
            m.modifiers(),
            Modifiers { shift: true, ctrl: true, alt: false, win: false }
        );
        assert_eq!(m.shift_keys, SHIFT_R);
        assert!(m.has_shortcut_modifier());
        m.sync(0);
        assert_eq!(m.modifiers(), Modifiers::NONE);
        assert_eq!(m.shift_keys, 0);
        assert!(!m.has_shortcut_modifier());
    }

    #[test]
    fn command_is_the_win_slot_and_caps_lock_follows_alpha_shift() {
        use flags::*;
        let mut m = ModifierState::default();
        m.sync(COMMAND | DEV_LCMD);
        assert!(m.win && m.has_shortcut_modifier());
        m.sync(ALPHA_SHIFT);
        assert!(m.caps_lock && !m.win);
        m.sync(0);
        assert!(!m.caps_lock);
    }

    #[test]
    fn shift_without_side_bits_assumes_left() {
        let mut m = ModifierState::default();
        m.sync(flags::SHIFT);
        assert!(m.shift);
        assert_eq!(m.shift_keys, SHIFT_L);
    }

    #[test]
    fn typed_char_accepts_one_printable_character() {
        assert_eq!(typed_char(&['a' as u16]), Some('a'));
        assert_eq!(typed_char(&[0x00E1]), Some('á'));
        assert_eq!(typed_char(&[0xD83D, 0xDE00]), Some('😀'), "surrogate pair");
        assert_eq!(typed_char(&[]), None);
        assert_eq!(typed_char(&['a' as u16, 'b' as u16]), None);
        assert_eq!(typed_char(&[0xD800]), None, "lone surrogate");
        assert_eq!(typed_char(&[0xF702]), None, "left-arrow private-use code");
    }

    #[test]
    fn keypad_and_function_keys_are_never_text() {
        assert!(never_text(vk_for_keycode(82)));
        assert!(never_text(vk_for_keycode(122)));
        assert!(!never_text(vk_for_keycode(0)));
        assert!(!never_text(VK_SPACE));
    }

    #[test]
    fn reconcile_follows_flags_for_the_keys_own_us_characters() {
        // Letters: case from Shift xor Caps Lock, whatever the string says.
        assert_eq!(reconcile_ascii(0x56, 'v', true, false), 'V');
        assert_eq!(reconcile_ascii(0x56, 'V', false, false), 'v');
        assert_eq!(reconcile_ascii(0x56, 'v', false, true), 'V');
        assert_eq!(reconcile_ascii(0x56, 'V', true, true), 'v');
        // Digits: a stale shifted string with clean flags is the digit again.
        assert_eq!(reconcile_ascii(0x36, '^', false, false), '6');
        assert_eq!(reconcile_ascii(0x35, '%', false, true), '5', "Caps Lock leaves digits alone");
        // Never promote a plain string to the shifted one: AZERTY Shift+`&` key gives `1`.
        assert_eq!(reconcile_ascii(0x31, '1', true, false), '1');
        assert_eq!(reconcile_ascii(0x36, '6', true, false), '6');
        assert_eq!(reconcile_ascii(0x32, '2', true, false), '2');
        assert_eq!(reconcile_ascii(0x31, '1', false, false), '1');
        assert_eq!(reconcile_ascii(0x31, '!', true, false), '!', "US Shift+1 untouched");
        // AZERTY: unshifted digit-row strings are symbols and stay as typed.
        assert_eq!(reconcile_ascii(0x31, '&', false, false), '&');
        assert_eq!(reconcile_ascii(0x32, 'é', false, false), 'é');
        assert_eq!(reconcile_ascii(0x33, '"', false, false), '"');
        // Anything that is not one of the key's two US characters is the layout's own.
        assert_eq!(reconcile_ascii(0x36, '§', false, false), '§');
        assert_eq!(reconcile_ascii(0x41, 'ă', false, false), 'ă');
        assert_eq!(reconcile_ascii(0x31, '-', false, false), '-', "AZERTY-style digit row is kept");
        assert_eq!(reconcile_ascii(VK_NONE, '6', false, false), '6');
    }

    #[test]
    fn us_char_and_vk_round_trip() {
        for ch in "abcXYZ019 .,;'[]{}\\-=`/~!@#$%^&*()_+<>?:\"|".chars() {
            let (vk, shift) = us_vk_for_char(ch).unwrap_or_else(|| panic!("no vk for {ch:?}"));
            assert_eq!(us_char(vk, shift, false), Some(ch), "round trip {ch:?}");
        }
        assert_eq!(us_char(0x41, false, true), Some('A'));
        assert_eq!(us_char(0x41, true, true), Some('a'));
        assert_eq!(us_vk_for_char('á'), None);
    }

    #[test]
    fn boundary_and_mover_sets() {
        assert!(is_word_boundary(VK_SPACE) && is_word_boundary(VK_BACK));
        assert!(!is_word_boundary(0x41));
        assert!(is_caret_mover(VK_LEFT) && is_caret_mover(VK_ESCAPE));
        assert!(!is_caret_mover(VK_SPACE));
    }
}
