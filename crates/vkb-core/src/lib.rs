//! Pure Vietnamese typing engine.
//!
//! Invariants (see `docs/evkey-clone-work/25-contract.md` §3 and
//! `70-impl-notes-engine.md`):
//!
//! - no I/O, no threads, no OS calls anywhere in this crate
//! - `process_key` performs no heap allocation on the `EngineAction::Noop` path
//!   (checked by `tests/no_alloc.rs`)
//! - the engine owns no window/caret knowledge: the platform layer tells it
//!   about boundaries via [`VietnameseEngine::boundary`]
//!
//! Layout:
//!
//! | module | owns |
//! |---|---|
//! | [`letters`] | the 67 lowercase + 67 uppercase letters as base/mark/tone |
//! | [`tone`] | tone placement (modern and old style, `qu`/`gi`, coda rules) |
//! | [`method`] | replaying raw keystrokes into marks and a tone |
//! | [`spell`] | structural syllable validation |
//! | [`charset`] | the two Unicode output shapes (NFC and NFD) |
//! | [`macros`] | the `gõ tắt` table |
//! | [`engine`] | [`Engine`], action diffing, backspace, boundary |

#![forbid(unsafe_code)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod charset;
pub mod engine;
pub mod letters;
pub mod macros;
pub mod method;
pub mod spell;
pub mod tone;

pub use engine::{Engine, VK_BACK, VK_ESCAPE, VK_RETURN, VK_SPACE, VK_TAB};
pub use macros::MacroTable;

/// Hard cap on the current word buffer. Longer input is committed and the engine resets.
pub const MAX_WORD: usize = 32;

// ---------------------------------------------------------------------------
// Input model
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        win: false,
    };
}

/// One physical key transition as delivered by the OS hook.
///
/// Lives in `vkb-core` (not `vkb-platform`) so the engine trait stays OS-free;
/// `vkb-platform` re-exports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyInput {
    /// Windows virtual-key code. The macOS backend maps CGEvent keycodes into this space.
    pub vk: u16,
    pub scan: u16,
    /// Layout-resolved character, already reflecting the shift state.
    pub ch: Option<char>,
    pub modifiers: Modifiers,
    pub down: bool,
    pub repeat: bool,
}

impl KeyInput {
    /// A plain text keystroke.
    pub fn char(ch: char) -> Self {
        Self {
            vk: 0,
            scan: 0,
            ch: Some(ch),
            modifiers: Modifiers::NONE,
            down: true,
            repeat: false,
        }
    }

    /// Any key, with or without a character.
    pub fn key(vk: u16, ch: Option<char>) -> Self {
        Self {
            vk,
            scan: 0,
            ch,
            modifiers: Modifiers::NONE,
            down: true,
            repeat: false,
        }
    }

    pub fn backspace() -> Self {
        Self::key(VK_BACK, None)
    }

    pub fn enter() -> Self {
        Self::key(VK_RETURN, Some('\r'))
    }

    pub fn space() -> Self {
        Self::key(VK_SPACE, Some(' '))
    }
}

// ---------------------------------------------------------------------------
// Engine output
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineAction {
    /// Hook consumes nothing; the key must reach the app untouched.
    Noop,
    /// Hook swallows the key and the backend types `text` once.
    Commit(String),
    /// Hook swallows the key; delete `delete_units` chars backwards, then type `insert`.
    ///
    /// The count is in the charset's own units — Unicode scalar values for the
    /// Unicode charsets, bytes for the legacy 8-bit ones — which is exactly what
    /// the engine emitted for the word, so "delete N, insert S" always restores
    /// a consistent state.
    Replace { delete_units: u32, insert: String },
    /// Same as [`EngineAction::Replace`], then the original key still reaches the app.
    /// Macros committed with Enter/Tab need this: the expansion is typed and the key acts.
    ReplaceThenPass { delete_units: u32, insert: String },
    /// Swallow the key and type nothing (a pure modifier trigger).
    Swallow,
    /// Toggle Vietnamese on/off; handled by the app, not by the engine.
    ToggleEnabled,
    /// Invalid keystroke: backend beeps and swallows.
    Reject,
}

impl EngineAction {
    pub fn is_noop(&self) -> bool {
        matches!(self, EngineAction::Noop)
    }
}

// ---------------------------------------------------------------------------
// Input method + charset
// ---------------------------------------------------------------------------

macro_rules! string_enum {
    ($name:ident, $sentinel:ident, $fallback:literal, $($variant:ident => $slug:literal),+ $(,)?) => {
        // The sentinel is a real value, not the non-exhaustive marker: serde
        // must be able to construct it in another crate.
        #[allow(clippy::manual_non_exhaustive)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant,)+
            /// Serde sentinel: a slug a removed variant left in an old config.
            /// Never offered by the UI or the CLI; `AppConfig::migrated()`
            /// normalises it to a real value before anything reads it.
            #[doc(hidden)]
            $sentinel,
        }

        impl $name {
            /// The values the UI and the CLI offer. The sentinel is not one.
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $slug,)+
                    // Never written to a file after `migrated()`; keep the
                    // fallback serialisable so a manual construct stays valid.
                    $name::$sentinel => $fallback,
                }
            }

            #[allow(clippy::should_implement_trait)]
            pub fn from_str(input: &str) -> Option<Self> {
                let needle = input.trim();
                $(if needle.eq_ignore_ascii_case($slug) { return Some($name::$variant); })+
                None
            }

            /// `true` when the value came from a removed slug in an old file.
            pub fn is_legacy(self) -> bool {
                matches!(self, $name::$sentinel)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        #[cfg(feature = "serde")]
        impl Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = String::deserialize(deserializer)?;
                Ok(Self::from_str(&raw).unwrap_or(Self::$sentinel))
            }
        }
    };
}

string_enum!(
    InputMethod,
    Legacy,
    "telex",
    Telex => "telex",
    Vni => "vni",
);

string_enum!(
    Charset,
    Legacy,
    "unicode",
    Unicode => "unicode",
    UnicodeNfd => "unicode_nfd",
);

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Defaults mirror `vkb-config`'s `TypingConfig`/`InputConfig` defaults, which
/// are the EVKey-compatible ones (spell check on, `f j w z` as consonants on,
/// automatic restore off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EngineOptions {
    pub method: InputMethod,
    pub charset: Charset,
    /// `true` = kiểu mới (`hòa`, `thúy`) · `false` = kiểu cũ (`hoà`, `thuý`).
    pub modern_tone: bool,
    pub spell_check: bool,
    pub restore_wrong_word: bool,
    pub jfwz_as_consonant: bool,
    pub macro_enabled: bool,
    pub auto_capitalize: bool,
    /// `Đặt dấu tự do`. `true`: the tone may be typed after the vowel run or at
    /// the end of the word, and it re-places itself as the syllable grows
    /// (`hoaf` + `n` → `hoàn`). `false`: the tone sticks to the vowel it hit
    /// when the key was typed (`hoafn` → `hòan`).
    pub free_tone: bool,
    pub auto_horn_uo: bool,
    /// Reserved: URL/code raw zone (spec §A9) is not implemented yet; the value is ignored.
    pub raw_zone: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            method: InputMethod::Telex,
            charset: Charset::Unicode,
            modern_tone: true,
            spell_check: true,
            restore_wrong_word: false,
            jfwz_as_consonant: true,
            macro_enabled: true,
            auto_capitalize: false,
            free_tone: true,
            auto_horn_uo: true,
            raw_zone: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Engine trait
// ---------------------------------------------------------------------------

pub trait VietnameseEngine: Send {
    fn reset(&mut self);
    fn set_options(&mut self, options: EngineOptions);
    /// Swap the macro table. One `Arc` store; valid from the next keystroke.
    fn set_macros(&mut self, macros: MacroTable);
    /// Pure and allocation-free on the `Noop` path. O(1) in word length up to [`MAX_WORD`].
    fn process_key(&mut self, key: KeyInput) -> EngineAction;
    /// The caret moved / focus changed: flush history but emit nothing.
    fn boundary(&mut self);
}

// ---------------------------------------------------------------------------
// Test/CLI harness
// ---------------------------------------------------------------------------

/// Apply an action the way a backend would, so tests and the CLI observe the
/// same text the user would see.
pub fn apply_action(out: &mut String, action: EngineAction, key: KeyInput) {
    match action {
        EngineAction::Noop => {
            // The key reaches the application: the app inserts its character,
            // or deletes one for backspace.
            if key.vk == VK_BACK || matches!(key.ch, Some('\u{8}') | Some('\u{7f}')) {
                out.pop();
            } else if let Some(ch) = key.ch {
                out.push(ch);
            }
        }
        EngineAction::Commit(text) => out.push_str(&text),
        EngineAction::Replace {
            delete_units,
            insert,
        } => {
            for _ in 0..delete_units {
                if out.pop().is_none() {
                    break;
                }
            }
            out.push_str(&insert);
        }
        EngineAction::ReplaceThenPass {
            delete_units,
            insert,
        } => {
            for _ in 0..delete_units {
                if out.pop().is_none() {
                    break;
                }
            }
            out.push_str(&insert);
            if let Some(ch) = key.ch {
                out.push(ch);
            }
        }
        EngineAction::Swallow | EngineAction::ToggleEnabled | EngineAction::Reject => {}
    }
}

/// Type `input` through a fresh engine and return the text produced.
pub fn simulate(options: EngineOptions, input: &str) -> String {
    let keys: Vec<KeyInput> = input.chars().map(KeyInput::char).collect();
    simulate_keys(options, &keys)
}

/// Same as [`simulate`], for inputs that mix characters with control keys.
pub fn simulate_keys(options: EngineOptions, keys: &[KeyInput]) -> String {
    let mut engine = Engine::new(options);
    let mut out = String::new();
    for key in keys {
        let action = engine.process_key(*key);
        apply_action(&mut out, action, *key);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_roundtrip() {
        for m in InputMethod::ALL {
            assert_eq!(InputMethod::from_str(m.as_str()), Some(*m));
        }
    }

    #[test]
    fn charset_roundtrip() {
        for c in Charset::ALL {
            assert_eq!(Charset::from_str(c.as_str()), Some(*c));
        }
    }

    #[test]
    fn defaults_are_evkey_like() {
        let d = EngineOptions::default();
        assert_eq!(d.method, InputMethod::Telex);
        assert_eq!(d.charset, Charset::Unicode);
        assert!(d.modern_tone, "kiểu mới is the default");
        assert!(d.spell_check);
        assert!(!d.restore_wrong_word);
        assert!(d.jfwz_as_consonant);
        assert!(d.macro_enabled);
        assert!(d.free_tone, "`Đặt dấu tự do` defaults on");
    }

    #[test]
    fn simulate_telex_smoke() {
        assert_eq!(simulate(EngineOptions::default(), "tieengs Vieetj"), "tiếng Việt");
    }
}
