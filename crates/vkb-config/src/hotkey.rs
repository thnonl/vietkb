//! Hotkey value type: parse `Ctrl+Shift+V` style strings into modifiers + key.
//!
//! Canonical form is `Ctrl+Alt+Shift+Win+<Key>`. Parsing is case-insensitive and
//! order-insensitive. A modifier-only string (`Ctrl+Shift`) is a valid combo
//! hotkey with no key: EVKey's default toggle is exactly that shape.

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
        win: false,
    };

    pub fn is_empty(&self) -> bool {
        !(self.ctrl || self.alt || self.shift || self.win)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HotkeyKey {
    /// A single printable character, already normalized to uppercase for letters.
    Char(char),
    /// A named key: F1..F24, Space, Enter, Tab, Escape, arrows, Home, End, PageUp, ...
    Named(&'static str),
}

impl HotkeyKey {
    pub fn as_str(&self) -> String {
        match self {
            HotkeyKey::Char(c) => c.to_string(),
            HotkeyKey::Named(n) => (*n).to_string(),
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        let upper = token.trim().to_ascii_uppercase();
        if upper.is_empty() {
            return None;
        }
        if upper.chars().count() == 1 {
            let c = upper.chars().next().unwrap();
            return Some(HotkeyKey::Char(c));
        }
        if let Some(rest) = upper.strip_prefix('F') {
            if let Ok(n) = rest.parse::<u8>() {
                if (1..=24).contains(&n) {
                    // Leak-free: map onto a fixed table instead of allocating.
                    return Some(HotkeyKey::Named(FKEYS[(n - 1) as usize]));
                }
            }
        }
        let named = match upper.as_str() {
            "SPACE" | "SPACEBAR" => "Space",
            "ENTER" | "RETURN" => "Enter",
            "TAB" => "Tab",
            "ESC" | "ESCAPE" => "Escape",
            "BACKSPACE" | "BKSP" => "Backspace",
            "DELETE" | "DEL" => "Delete",
            "INSERT" | "INS" => "Insert",
            "HOME" => "Home",
            "END" => "End",
            "PAGEUP" | "PGUP" => "PageUp",
            "PAGEDOWN" | "PGDN" => "PageDown",
            "UP" => "Up",
            "DOWN" => "Down",
            "LEFT" => "Left",
            "RIGHT" => "Right",
            "`" | "BACKQUOTE" => "`",
            "-" | "MINUS" => "-",
            "=" | "EQUAL" => "=",
            "," | "COMMA" => ",",
            "." | "PERIOD" => ".",
            "/" | "SLASH" => "/",
            ";" | "SEMICOLON" => ";",
            "[" | "LBRACKET" => "[",
            "\\" | "BACKSLASH" => "\\",
            "]" | "RBRACKET" => "]",
            "'" | "QUOTE" => "'",
            _ => return None,
        };
        Some(HotkeyKey::Named(named))
    }
}

static FKEYS: [&str; 24] = [
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13", "F14",
    "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hotkey {
    pub modifiers: Modifiers,
    pub key: Option<HotkeyKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HotkeyError {
    #[error("phím tắt rỗng")]
    Empty,
    #[error("tổ hợp chỉ có phím bổ trợ, thiếu phím chính: {0}")]
    ModifierOnly(String),
    #[error("không nhận ra phím: {0}")]
    UnknownKey(String),
    #[error("phím tắt cần ít nhất một phím Ctrl/Alt/Shift/Win: {0}")]
    NoModifier(String),
}

impl Hotkey {
    pub fn parse(input: &str) -> Result<Self, HotkeyError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(HotkeyError::Empty);
        }

        let mut modifiers = Modifiers::NONE;
        let mut key: Option<HotkeyKey> = None;
        let mut saw_modifier_after_key = false;

        for raw in trimmed.split('+') {
            let token = raw.trim();
            if token.is_empty() {
                // Supports a literal '+' as the final key: "Ctrl++"
                continue;
            }
            match token.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                "win" | "windows" | "super" | "meta" => modifiers.win = true,
                _ => {
                    if key.is_some() {
                        return Err(HotkeyError::UnknownKey(trimmed.to_string()));
                    }
                    key = Some(HotkeyKey::from_token(token).ok_or_else(|| {
                        HotkeyError::UnknownKey(token.to_string())
                    })?);
                }
            }
            if key.is_some() && (modifiers.ctrl || modifiers.alt || modifiers.shift || modifiers.win)
            {
                // A modifier appearing after the key is legal ("V+Ctrl"); normalize later.
                saw_modifier_after_key = true;
            }
        }
        let _ = saw_modifier_after_key;

        if modifiers.is_empty() {
            // A bare key is only meaningful for function keys, which EVKey uses by default.
            match key {
                Some(HotkeyKey::Named(n)) if n.starts_with('F') => {}
                _ => return Err(HotkeyError::NoModifier(trimmed.to_string())),
            }
        }

        if key.is_none() && modifiers.is_empty() {
            return Err(HotkeyError::ModifierOnly(trimmed.to_string()));
        }

        Ok(Self { modifiers, key })
    }

    pub fn as_str(&self) -> String {
        let mut out = String::new();
        if self.modifiers.ctrl {
            out.push_str("Ctrl+");
        }
        if self.modifiers.alt {
            out.push_str("Alt+");
        }
        if self.modifiers.shift {
            out.push_str("Shift+");
        }
        if self.modifiers.win {
            out.push_str("Win+");
        }
        if let Some(key) = &self.key {
            out.push_str(&key.as_str());
        } else if out.ends_with('+') {
            out.pop();
        }
        out
    }

    /// Human label for the UI: `Ctrl + Shift` reads better than `Ctrl+Shift`.
    pub fn display_pretty(&self) -> String {
        self.as_str().replace('+', " + ")
    }

    pub fn matches(&self, modifiers: Modifiers, key: Option<HotkeyKey>) -> bool {
        self.modifiers == modifiers && self.key == key
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl Serialize for Hotkey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_str())
    }
}

struct HotkeyVisitor;

impl<'de> Visitor<'de> for HotkeyVisitor {
    type Value = Hotkey;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a hotkey string such as \"Ctrl+Shift\"")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Hotkey, E> {
        Hotkey::parse(v).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for Hotkey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(HotkeyVisitor)
    }
}

/// Serde helpers for `Option<Hotkey>`, where an empty string means "unbound"
/// and a missing key means "use the documented default".
pub mod opt {
    use super::*;

    pub fn serialize<S: Serializer>(value: &Option<Hotkey>, s: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(hk) => s.serialize_str(&hk.as_str()),
            None => s.serialize_str(""),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Hotkey>, D::Error> {
        let raw = String::deserialize(d)?;
        if raw.trim().is_empty() || raw.trim().eq_ignore_ascii_case("none") {
            return Ok(None);
        }
        Hotkey::parse(&raw).map(Some).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canon(input: &str) -> String {
        Hotkey::parse(input).unwrap().as_str()
    }

    #[test]
    fn parses_modifier_combos() {
        assert_eq!(canon("Ctrl+Shift"), "Ctrl+Shift");
        assert_eq!(canon("shift+ctrl"), "Ctrl+Shift");
        assert_eq!(canon("Ctrl + Shift "), "Ctrl+Shift");
        assert_eq!(canon("ctrl+alt+v"), "Ctrl+Alt+V");
        assert_eq!(canon("V+Ctrl"), "Ctrl+V");
    }

    #[test]
    fn parses_function_and_named_keys() {
        assert_eq!(canon("F1"), "F1");
        assert_eq!(canon("f12"), "F12");
        assert_eq!(canon("Ctrl+F11"), "Ctrl+F11");
        assert_eq!(canon("Alt+Space"), "Alt+Space");
        assert_eq!(canon("Ctrl+PageDown"), "Ctrl+PageDown");
        assert_eq!(canon("Win+Up"), "Win+Up");
    }

    #[test]
    fn rejects_junk() {
        assert!(Hotkey::parse("").is_err());
        assert!(Hotkey::parse("Hype+Q").is_err());
        assert!(Hotkey::parse("A").is_err(), "bare letter needs a modifier");
        assert!(Hotkey::parse("Ctrl+Shift+A+B").is_err());
    }

    #[test]
    fn serde_is_canonical() {
        let hk = Hotkey::parse("shift+ctrl+z").unwrap();
        let json = serde_json::to_string(&hk).unwrap();
        assert_eq!(json, "\"Ctrl+Shift+Z\"");
        let back: Hotkey = serde_json::from_str(&json).unwrap();
        assert_eq!(back, hk);
    }

    #[test]
    fn pretty_display() {
        let hk = Hotkey::parse("Ctrl+Shift").unwrap();
        assert_eq!(hk.display_pretty(), "Ctrl + Shift");
    }
}
