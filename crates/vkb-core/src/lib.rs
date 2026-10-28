//! Pure Vietnamese typing engine.
//!
//! Invariants (see `docs/evkey-clone-work/20-spec.md`):
//! - no I/O, no threads, no OS calls anywhere in this crate
//! - `process_key` performs no heap allocation on the `EngineAction::Noop` path
//! - the engine owns no window/caret knowledge: the platform layer tells it about
//!   boundaries via [`VietnameseEngine::boundary`]

#![forbid(unsafe_code)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
}

// ---------------------------------------------------------------------------
// Engine output
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineAction {
    /// Hook consumes nothing; the key reaches the app untouched.
    Noop,
    /// Hook swallows the key and the backend types `text` once.
    Commit(String),
    /// Hook swallows the key; delete `delete_graphemes` chars backwards, then type `insert`.
    Replace { delete_graphemes: u32, insert: String },
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
    ($name:ident, $($variant:ident => $slug:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $slug),+
                }
            }

            #[allow(clippy::should_implement_trait)]
            pub fn from_str(input: &str) -> Option<Self> {
                let needle = input.trim();
                $(if needle.eq_ignore_ascii_case($slug) { return Some($name::$variant); })+
                None
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

string_enum!(
    InputMethod,
    Telex => "telex",
    Vni => "vni",
    Viqr => "viqr",
);

string_enum!(
    Charset,
    Unicode => "unicode",
    UnicodeNfd => "unicode_nfd",
    Tcvn3 => "tcvn3",
    VniWindows => "vni_windows",
    Viscii => "viscii",
    Viqr => "viqr",
    BkHcm1 => "bk_hcm1",
    Vps => "vps",
    Utf8 => "utf8",
);

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EngineOptions {
    pub method: InputMethod,
    pub charset: Charset,
    /// `true` = kiểu cũ (`hoà`), `false` = kiểu mới (`hòa`).
    pub old_tone_style: bool,
    pub spell_check: bool,
    pub restore_wrong_word: bool,
    pub jfwz_as_consonant: bool,
    pub macro_enabled: bool,
    pub auto_capitalize: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            method: InputMethod::Telex,
            charset: Charset::Unicode,
            old_tone_style: false,
            spell_check: false,
            restore_wrong_word: true,
            jfwz_as_consonant: false,
            macro_enabled: true,
            auto_capitalize: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub trait VietnameseEngine: Send {
    fn reset(&mut self);
    fn set_options(&mut self, options: EngineOptions);
    /// Pure and allocation-free on the `Noop` path. O(1) in word length up to [`MAX_WORD`].
    fn process_key(&mut self, key: KeyInput) -> EngineAction;
    /// The caret moved / focus changed: flush history but emit nothing.
    fn boundary(&mut self);
}

/// Default engine. Sprint 1 fills the state machine; the shape is frozen.
pub struct Engine {
    options: EngineOptions,
    word: smallvec::SmallVec<[char; MAX_WORD]>,
}

impl Engine {
    pub fn new(options: EngineOptions) -> Self {
        Self {
            options,
            word: smallvec::SmallVec::new(),
        }
    }

    pub fn options(&self) -> EngineOptions {
        self.options
    }

    /// Current raw keystroke buffer, for tests and the CLI simulator.
    pub fn buffer(&self) -> &[char] {
        &self.word
    }
}

impl VietnameseEngine for Engine {
    fn reset(&mut self) {
        self.word.clear();
    }

    fn set_options(&mut self, options: EngineOptions) {
        if self.options.method != options.method || self.options.charset != options.charset {
            self.word.clear();
        }
        self.options = options;
    }

    fn process_key(&mut self, _key: KeyInput) -> EngineAction {
        todo!("sprint 1: telex/vni/viqr state machine")
    }

    fn boundary(&mut self) {
        self.word.clear();
    }
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
        assert!(!d.old_tone_style);
        assert!(d.restore_wrong_word);
    }
}
