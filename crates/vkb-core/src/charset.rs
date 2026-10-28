//! Charset output.
//!
//! The engine composes NFC Vietnamese text; this module renders it in the two
//! shapes the product ships:
//!
//! | shape | charset | `EngineAction` text |
//! |---|---|---|
//! | Unicode scalar values (NFC) | `unicode` | the text itself |
//! | canonical decomposition | `unicode_nfd` | base + combining marks |
//!
//! `unicode_nfd` is reachable only from `config.toml`; the UI offers `unicode`.
//! The legacy 8-bit tables (TCVN3, VNI Windows, VISCII, VPS, BK HCM1) and VIQR
//! were removed with the 2026-10 product decision, not hidden behind a flag.

use crate::letters::{combining_marks, decompose, Mark};
use crate::Charset;
use smallvec::SmallVec;

/// Rendered word, in the charset's own units.
pub type TextBuf = SmallVec<[char; 128]>;

impl Charset {
    /// Bytes for `text` in this charset.
    pub fn encode(self, text: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(text.len());
        for ch in text.chars() {
            match self {
                // The sentinel never reaches rendering (`AppConfig::migrated()`
                // normalises it); treat it as the default if it ever does.
                Charset::Unicode | Charset::Legacy => {
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
                Charset::UnicodeNfd => {
                    let mut buf = TextBuf::new();
                    push_nfd(ch, &mut buf);
                    for c in buf {
                        let mut raw = [0u8; 4];
                        out.extend_from_slice(c.encode_utf8(&mut raw).as_bytes());
                    }
                }
            }
        }
        out
    }
}

/// Append the charset's rendering of one composed character to `out`.
///
/// Never allocates: `out` is an inline-capacity buffer.
pub(crate) fn push_encoded(ch: char, charset: Charset, out: &mut TextBuf) {
    match charset {
        Charset::Unicode | Charset::Legacy => out.push(ch),
        Charset::UnicodeNfd => push_nfd(ch, out),
    }
}

fn push_nfd(ch: char, out: &mut TextBuf) {
    match decompose(ch) {
        // `đ` has no canonical decomposition.
        Some((_, Mark::Bar, _)) => out.push(ch),
        Some((base, mark, tone)) => {
            out.push(if ch.is_uppercase() {
                base.to_ascii_uppercase()
            } else {
                base
            });
            for mark in combining_marks(mark, tone) {
                out.push(*mark);
            }
        }
        None => out.push(ch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfd_decomposes_mark_before_tone() {
        assert_eq!(
            Charset::UnicodeNfd.encode("ế"),
            "\u{65}\u{302}\u{301}".as_bytes()
        );
        assert_eq!(
            Charset::UnicodeNfd.encode("ờ"),
            "\u{6F}\u{31B}\u{300}".as_bytes()
        );
        assert_eq!(Charset::UnicodeNfd.encode("đ"), "đ".as_bytes());
    }

    #[test]
    fn unicode_is_nfc_bytes() {
        assert_eq!(Charset::Unicode.encode("Tiếng Việt"), "Tiếng Việt".as_bytes());
        // The removed-slug sentinel renders like the default.
        assert_eq!(Charset::Legacy.encode("Tiếng Việt"), "Tiếng Việt".as_bytes());
    }

    #[test]
    fn ascii_passes_through_both_charsets() {
        for charset in Charset::ALL {
            let bytes = charset.encode("abc 123");
            let text = String::from_utf8_lossy(&bytes);
            assert!(text.contains("abc"), "{charset:?}");
            assert!(text.contains("123"), "{charset:?}");
        }
    }
}
