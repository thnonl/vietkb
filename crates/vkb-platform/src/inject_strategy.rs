//! Injection strategies for the D-WIN-09 measurement build (dev channel only).
//!
//! Pure: this module only names and parses a strategy. Building the `INPUT`
//! records and sending them lives in `vkb-windows` (`input_dev.rs`), which reads
//! `VKB_INJECT_STRATEGY` only when compiled with its `dev-channel` feature.

/// Largest `split-delay-N` accepted, in milliseconds.
pub const MAX_DELAY_MS: u8 = 50;

/// Largest `boundary-hold-N` accepted, in milliseconds (below the hook's
/// 150 ms wait cap).
pub const MAX_HOLD_MS: u8 = 120;

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
    /// Select the N characters with Shift+Left x N (real VKs) instead of N
    /// Backspaces, then type the text (VK path) in a second call. Falls back to
    /// `Split` when a selection is not safe.
    SelectReplace,
    /// `SelectReplace`, but the text goes out as `KEYEVENTF_UNICODE` packets.
    SelectReplaceUnicode,
    /// Delete the whole current word and retype the whole new word, in ONE call.
    WordRetype,
    /// `WordRetype`, with the Backspace block as its own call.
    WordRetypeSplit,
    /// Backspace call, a pause of N ms, then the text as `KEYEVENTF_UNICODE`.
    UnicodeAfterSplitDelay(u8),
    /// `Split`, and the boundary key (space/punctuation) that follows our text
    /// is forwarded N ms after the text.
    BoundaryHold(u8),
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
            "select-replace" => Some(Self::SelectReplace),
            "select-replace-unicode" => Some(Self::SelectReplaceUnicode),
            "word-retype" => Some(Self::WordRetype),
            "word-retype-split" => Some(Self::WordRetypeSplit),
            _ => {
                if let Some(n) = s.strip_prefix("split-delay-") {
                    return delay(n, MAX_DELAY_MS).map(Self::SplitDelay);
                }
                if let Some(n) = s.strip_prefix("unicode-after-split-delay-") {
                    return delay(n, MAX_DELAY_MS).map(Self::UnicodeAfterSplitDelay);
                }
                let n = s.strip_prefix("boundary-hold-")?;
                delay(n, MAX_HOLD_MS).map(Self::BoundaryHold)
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
            Self::SelectReplace => "select-replace".into(),
            Self::SelectReplaceUnicode => "select-replace-unicode".into(),
            Self::WordRetype => "word-retype".into(),
            Self::WordRetypeSplit => "word-retype-split".into(),
            Self::UnicodeAfterSplitDelay(n) => format!("unicode-after-split-delay-{n}"),
            Self::BoundaryHold(n) => format!("boundary-hold-{n}"),
        }
    }
}

/// Digits only (no sign, no whitespace), `1..=max`.
fn delay(n: &str, max: u8) -> Option<u8> {
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u8 = n.parse().ok()?;
    (1..=max).contains(&n).then_some(n)
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
            InjectStrategy::SelectReplace,
            InjectStrategy::SelectReplaceUnicode,
            InjectStrategy::WordRetype,
            InjectStrategy::WordRetypeSplit,
            InjectStrategy::UnicodeAfterSplitDelay(12),
            InjectStrategy::BoundaryHold(100),
        ] {
            assert_eq!(InjectStrategy::parse(&s.label()), Some(s));
        }
    }

    #[test]
    fn parses_the_notepad_strategies() {
        assert_eq!(InjectStrategy::parse("select-replace"), Some(InjectStrategy::SelectReplace));
        assert_eq!(
            InjectStrategy::parse("Select-Replace-Unicode"),
            Some(InjectStrategy::SelectReplaceUnicode)
        );
        assert_eq!(InjectStrategy::parse("word-retype"), Some(InjectStrategy::WordRetype));
        assert_eq!(
            InjectStrategy::parse("word-retype-split"),
            Some(InjectStrategy::WordRetypeSplit)
        );
        assert_eq!(
            InjectStrategy::parse("unicode-after-split-delay-20"),
            Some(InjectStrategy::UnicodeAfterSplitDelay(20))
        );
        assert_eq!(InjectStrategy::parse("boundary-hold-80"), Some(InjectStrategy::BoundaryHold(80)));
        assert_eq!(InjectStrategy::parse("unicode-after-split"), Some(InjectStrategy::UnicodeAfterSplit));
    }

    #[test]
    fn new_delays_are_range_checked() {
        for bad in [
            "unicode-after-split-delay-0",
            "unicode-after-split-delay-51",
            "unicode-after-split-delay-",
            "boundary-hold-0",
            "boundary-hold-121",
            "boundary-hold-",
            "boundary-hold--1",
            "boundary-hold-9x",
            "boundary-hold",
            "word-retype-",
            "select-replace-delay-5",
        ] {
            assert_eq!(InjectStrategy::parse(bad), None, "{bad}");
        }
        assert_eq!(InjectStrategy::parse("boundary-hold-120"), Some(InjectStrategy::BoundaryHold(120)));
    }
}
