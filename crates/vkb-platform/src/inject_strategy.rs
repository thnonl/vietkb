//! Injection strategies for the D-WIN-09 measurement build (dev channel only).
//!
//! Pure: this module only names and parses a strategy. Building the `INPUT`
//! records and sending them lives in `vkb-windows` (`input_dev.rs`), which reads
//! `VKB_INJECT_STRATEGY` only when compiled with its `dev-channel` feature.

/// Largest `split-delay-N` accepted, in milliseconds.
pub const MAX_DELAY_MS: u8 = 50;

/// How a `Replace { deletes, text }` is handed to `SendInput` on the hook path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InjectStrategy {
    /// Backspaces as their own call, then the text (the current behaviour).
    #[default]
    Split,
    /// Control: everything in one call (0.2.6 behaviour).
    Single,
    /// `Split`, with a pause of N ms before the text call.
    SplitDelay(u8),
    /// `Split`, with each character of the text in its own call.
    PerChar,
    /// `Split`, Shift via `KEYEVENTF_SCANCODE` + `VK_LSHIFT` + scan 0x2A.
    ScancodeShift,
    /// Backspace call, then the whole text as `KEYEVENTF_UNICODE` packets.
    UnicodeAfterSplit,
    /// `Split`, Shift as plain `VK_LSHIFT` (+ scan 0x2A) instead of `VK_SHIFT`.
    CapslockFree,
}

impl InjectStrategy {
    /// Parse a `VKB_INJECT_STRATEGY` value (case-insensitive, trimmed).
    /// `None` for an unknown name or a delay outside `1..=50`.
    pub fn parse(raw: &str) -> Option<Self> {
        let s = raw.trim().to_ascii_lowercase();
        match s.as_str() {
            "split" => Some(Self::Split),
            "single" => Some(Self::Single),
            "per-char" => Some(Self::PerChar),
            "scancode-shift" => Some(Self::ScancodeShift),
            "unicode-after-split" => Some(Self::UnicodeAfterSplit),
            "capslock-free" => Some(Self::CapslockFree),
            _ => {
                let n = s.strip_prefix("split-delay-")?;
                // Digits only: no sign, no whitespace.
                if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
                    return None;
                }
                let n: u8 = n.parse().ok()?;
                (1..=MAX_DELAY_MS).contains(&n).then_some(Self::SplitDelay(n))
            }
        }
    }

    /// The name `parse` accepts, for the keys.log trace.
    pub fn label(self) -> String {
        match self {
            Self::Split => "split".into(),
            Self::Single => "single".into(),
            Self::SplitDelay(n) => format!("split-delay-{n}"),
            Self::PerChar => "per-char".into(),
            Self::ScancodeShift => "scancode-shift".into(),
            Self::UnicodeAfterSplit => "unicode-after-split".into(),
            Self::CapslockFree => "capslock-free".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_name() {
        assert_eq!(InjectStrategy::parse("split"), Some(InjectStrategy::Split));
        assert_eq!(InjectStrategy::parse(" Single "), Some(InjectStrategy::Single));
        assert_eq!(InjectStrategy::parse("PER-CHAR"), Some(InjectStrategy::PerChar));
        assert_eq!(InjectStrategy::parse("scancode-shift"), Some(InjectStrategy::ScancodeShift));
        assert_eq!(
            InjectStrategy::parse("unicode-after-split"),
            Some(InjectStrategy::UnicodeAfterSplit)
        );
        assert_eq!(InjectStrategy::parse("capslock-free"), Some(InjectStrategy::CapslockFree));
    }

    #[test]
    fn delay_range_is_1_to_50() {
        assert_eq!(InjectStrategy::parse("split-delay-1"), Some(InjectStrategy::SplitDelay(1)));
        assert_eq!(InjectStrategy::parse("split-delay-50"), Some(InjectStrategy::SplitDelay(50)));
        for bad in [
            "split-delay-0",
            "split-delay-51",
            "split-delay-",
            "split-delay--3",
            "split-delay-+3",
            "split-delay-999",
            "split-delay-5x",
        ] {
            assert_eq!(InjectStrategy::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn unknown_is_none_and_labels_round_trip() {
        assert_eq!(InjectStrategy::parse(""), None);
        assert_eq!(InjectStrategy::parse("fast"), None);
        for s in [
            InjectStrategy::Split,
            InjectStrategy::Single,
            InjectStrategy::SplitDelay(7),
            InjectStrategy::PerChar,
            InjectStrategy::ScancodeShift,
            InjectStrategy::UnicodeAfterSplit,
            InjectStrategy::CapslockFree,
        ] {
            assert_eq!(InjectStrategy::parse(&s.label()), Some(s));
        }
    }
}
