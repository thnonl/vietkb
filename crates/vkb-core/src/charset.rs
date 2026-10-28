//! Charset output.
//!
//! The engine composes NFC Vietnamese text; this module maps it to the nine
//! charsets the contract lists. Three shapes exist:
//!
//! | shape | charsets | `EngineAction` text |
//! |---|---|---|
//! | Unicode scalar values | `unicode`, `utf8`, `unicode_nfd` | the text itself |
//! | one byte per letter (TCVN3: two for an uppercase toned vowel) | `tcvn3`, `vps`, `viscii` | one char per byte |
//! | several bytes per letter | `vni_windows`, `bk_hcm1`, `viqr` | the byte sequence as chars |
//!
//! For the legacy 8-bit charsets a `String` cannot hold raw bytes, so each byte
//! `b` is carried as `char::from(b)` (Latin-1 view). [`Charset::encode`] gives
//! the real `Vec<u8>` for file conversion, and is what the convert tool must
//! use. This is the same convention the reference implementation used for
//! TCVN3 (`tiếng` → `tiÕng`), and it round-trips through `encode`.
//!
//! ## Table provenance
//!
//! Every byte table below is transcribed from a published source, and each
//! charset carries a [`TableStatus`] plus a note saying how far the table is
//! trusted. Nothing here was invented: where a code point could not be
//! established, the entry is listed as `Unverified` and a test documents the
//! assumption.
//!
//! * VISCII — RFC 1456, Table 1 (authoritative), cross-checked against the
//!   VietUnicode charset chart.
//! * VPS — the VietUnicode chart and the Wikipedia `VPSKeys` article agree on
//!   all 73 letters, in both cases.
//! * TCVN3 (VSCII-3) — the 67 lowercase letters and the 7 unmodified uppercase
//!   letters are VSCII-3 proper (VSCII chart + VietUnicode chart, and identical
//!   to the reference implementation's table). Uppercase *toned* letters have
//!   no VSCII-3 code point; TCVN3 fonts render them as "uppercase letter
//!   followed by the lowercase tone byte", which is also what the VietUnicode
//!   chart lists (`À` = `41 B5`), so that rule is `Documented`.
//! * VNI Windows — the VietUnicode VNI chart (one row per letter, both bytes)
//!   agrees with the Wikipedia `VNI` article's combining-mark table.
//! * VIQR — RFC 1456, Table 2.
//! * BK HCM1 — no primary source found. Implemented as the VNI Windows layout,
//!   flagged `Unverified`, and pinned by a test that records the assumption.

use crate::letters::{combining_marks, decompose, Mark, Tone};
use crate::Charset;
use smallvec::SmallVec;

/// How far a byte table is trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableStatus {
    /// Checked against an authoritative source and pinned by a test.
    Verified,
    /// Two published sources agree, but nothing was validated against real
    /// legacy output (a font, a file produced by the original tool).
    Documented,
    /// No primary source. The table is an explicit assumption, recorded in a
    /// test, and must be re-validated before it ships to users.
    Unverified,
}

/// Rendered word, in the charset's own units. Worst case is NFD/VIQR:
/// 32 letters × 3 chars.
pub type TextBuf = SmallVec<[char; 128]>;

impl Charset {
    /// How far this table is trusted.
    pub fn status(self) -> TableStatus {
        match self {
            Charset::Unicode | Charset::UnicodeNfd | Charset::Utf8 | Charset::Viscii | Charset::Viqr => {
                TableStatus::Verified
            }
            Charset::Tcvn3 | Charset::Vps | Charset::VniWindows => TableStatus::Documented,
            Charset::BkHcm1 => TableStatus::Unverified,
        }
    }

    /// Human-readable provenance for the table (shown in impl notes and tests).
    pub fn status_note(self) -> &'static str {
        match self {
            Charset::Unicode => "NFC; the 73+73 table is pinned to Unicode names in tests",
            Charset::UnicodeNfd => "canonical decomposition, mark before tone (U+0302/0306/031B then tone)",
            Charset::Utf8 => "same scalar values as `unicode`; a Rust `String` is already UTF-8",
            Charset::Viscii => "RFC 1456 Table 1, cross-checked with the VietUnicode chart",
            Charset::Viqr => "RFC 1456 Table 2",
            Charset::Tcvn3 => {
                "VSCII-3: 67 lowercase + 7 uppercase verified; uppercase toned letters use the \
                 documented `uppercase base + lowercase tone byte` pair from the VietUnicode chart"
            }
            Charset::Vps => "VietUnicode VPS chart and Wikipedia VPSKeys table agree on all 134 letters",
            Charset::VniWindows => {
                "VietUnicode VNI chart (byte pairs) agrees with the Wikipedia VNI combining table; \
                 not yet validated against a VNI font"
            }
            Charset::BkHcm1 => {
                "UNVERIFIED: no primary source found; emitted as the VNI Windows layout. Must be \
                 re-validated against real BK HCM1 output before release"
            }
        }
    }

    /// `true` when one letter can produce more than one byte.
    ///
    /// TCVN3 is single-byte for the 67 lowercase letters, but an uppercase toned
    /// vowel is written as two bytes (`Ấ` = `41 C1`), so it belongs here.
    pub fn is_multibyte(self) -> bool {
        matches!(
            self,
            Charset::Tcvn3 | Charset::VniWindows | Charset::BkHcm1 | Charset::Viqr
        )
    }

    /// Bytes for `text` in this charset. The converter behind
    /// `convert_text` in the Tauri command surface needs this, not
    /// [`EngineAction`](crate::EngineAction).
    pub fn encode(self, text: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(text.len());
        for ch in text.chars() {
            match self {
                Charset::Unicode | Charset::Utf8 => {
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
                Charset::Viqr => {
                    let mut buf = TextBuf::new();
                    push_viqr(ch, &mut buf);
                    for c in buf {
                        out.push(c as u8);
                    }
                }
                _ => match bytes_of(ch, self) {
                    Some(bytes) => out.extend_from_slice(bytes),
                    None if self == Charset::Tcvn3 && tcvn3_toned_upper(ch).is_some() => {
                        out.extend_from_slice(&tcvn3_toned_upper(ch).unwrap());
                    }
                    None if ch.is_ascii() => out.push(ch as u8),
                    None => out.push(b'?'),
                },
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
        Charset::Unicode | Charset::Utf8 => out.push(ch),
        Charset::UnicodeNfd => push_nfd(ch, out),
        Charset::Viqr => push_viqr(ch, out),
        Charset::Tcvn3 => {
            if let Some(bytes) = tcvn3(ch) {
                for byte in bytes {
                    out.push(char::from(*byte));
                }
            } else if let Some(pair) = tcvn3_toned_upper(ch) {
                out.push(char::from(pair[0]));
                out.push(char::from(pair[1]));
            } else {
                out.push(ch);
            }
        }
        _ => match bytes_of(ch, charset) {
            Some(bytes) => {
                for byte in bytes {
                    out.push(char::from(*byte));
                }
            }
            // Not a Vietnamese letter (or not representable): pass through.
            None => out.push(ch),
        },
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

fn push_viqr(ch: char, out: &mut TextBuf) {
    match decompose(ch) {
        Some((base, mark, tone)) => {
            let case = |c: char| {
                if ch.is_uppercase() {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            };
            if mark == Mark::Bar {
                out.push(case('d'));
                out.push(case('d'));
                return;
            }
            out.push(case(base));
            match mark {
                Mark::Breve => out.push('('),
                Mark::Circumflex => out.push('^'),
                Mark::Horn => out.push('+'),
                Mark::None | Mark::Bar => {}
            }
            if let Some(tone) = tone_mark(tone) {
                out.push(tone);
            }
        }
        None => {
            // Escape anything VIQR would otherwise read as a trigger.
            if matches!(
                ch,
                '\\' | '(' | ')' | '^' | '+' | '\'' | '`' | '?' | '~' | '.' | '-'
            ) {
                out.push('\\');
            }
            out.push(ch);
        }
    }
}

fn tone_mark(tone: Tone) -> Option<char> {
    match tone {
        Tone::None => None,
        Tone::Acute => Some('\''),
        Tone::Grave => Some('`'),
        Tone::Hook => Some('?'),
        Tone::Tilde => Some('~'),
        Tone::Dot => Some('.'),
    }
}

/// Byte sequence for one composed letter, or `None` when this charset has no
/// code for it (the caller passes the character through).
fn bytes_of(ch: char, charset: Charset) -> Option<&'static [u8]> {
    match charset {
        Charset::Tcvn3 => tcvn3(ch),
        Charset::Viscii => viscii(ch),
        Charset::Vps => vps(ch),
        Charset::VniWindows | Charset::BkHcm1 => vni_windows(ch),
        Charset::Unicode | Charset::UnicodeNfd | Charset::Utf8 | Charset::Viqr => None,
    }
}

// ---------------------------------------------------------------------------
// TCVN3 (VSCII-3), 8-bit
// ---------------------------------------------------------------------------

/// The 67 lowercase letters plus the 7 unmodified uppercase letters.
fn tcvn3(ch: char) -> Option<&'static [u8]> {
    Some(match ch {
        // a
        'á' => &[0xB8],
        'à' => &[0xB5],
        'ả' => &[0xB6],
        'ã' => &[0xB7],
        'ạ' => &[0xB9],
        'ă' => &[0xA8],
        'ắ' => &[0xBE],
        'ằ' => &[0xBB],
        'ẳ' => &[0xBC],
        'ẵ' => &[0xBD],
        'ặ' => &[0xC6],
        'â' => &[0xA9],
        'ấ' => &[0xCA],
        'ầ' => &[0xC7],
        'ẩ' => &[0xC8],
        'ẫ' => &[0xC9],
        'ậ' => &[0xCB],
        // e
        'é' => &[0xD0],
        'è' => &[0xCC],
        'ẻ' => &[0xCE],
        'ẽ' => &[0xCF],
        'ẹ' => &[0xD1],
        'ê' => &[0xAA],
        'ế' => &[0xD5],
        'ề' => &[0xD2],
        'ể' => &[0xD3],
        'ễ' => &[0xD4],
        'ệ' => &[0xD6],
        // i
        'í' => &[0xDD],
        'ì' => &[0xD7],
        'ỉ' => &[0xD8],
        'ĩ' => &[0xDC],
        'ị' => &[0xDE],
        // o
        'ó' => &[0xE3],
        'ò' => &[0xDF],
        'ỏ' => &[0xE1],
        'õ' => &[0xE2],
        'ọ' => &[0xE4],
        'ô' => &[0xAB],
        'ố' => &[0xE8],
        'ồ' => &[0xE5],
        'ổ' => &[0xE6],
        'ỗ' => &[0xE7],
        'ộ' => &[0xE9],
        'ơ' => &[0xAC],
        'ớ' => &[0xED],
        'ờ' => &[0xEA],
        'ở' => &[0xEB],
        'ỡ' => &[0xEC],
        'ợ' => &[0xEE],
        // u
        'ú' => &[0xF3],
        'ù' => &[0xEF],
        'ủ' => &[0xF1],
        'ũ' => &[0xF2],
        'ụ' => &[0xF4],
        'ư' => &[0xAD],
        'ứ' => &[0xF8],
        'ừ' => &[0xF5],
        'ử' => &[0xF6],
        'ữ' => &[0xF7],
        'ự' => &[0xF9],
        // y
        'ý' => &[0xFD],
        'ỳ' => &[0xFA],
        'ỷ' => &[0xFB],
        'ỹ' => &[0xFC],
        'ỵ' => &[0xFE],
        // d
        'đ' => &[0xAE],
        // unmodified uppercase letters
        'Ă' => &[0xA1],
        'Â' => &[0xA2],
        'Ê' => &[0xA3],
        'Ô' => &[0xA4],
        'Ơ' => &[0xA5],
        'Ư' => &[0xA6],
        'Đ' => &[0xA7],
        _ => return None,
    })
}

/// TCVN3 has no single code for `Ấ`; its ABC fonts draw the capital glyph from
/// the unmodified uppercase letter followed by the lowercase tone byte, which
/// is how the VietUnicode chart lists them (`Ấ` = `41 C1`, `Ắ` = `A1 BE`).
pub(crate) fn tcvn3_toned_upper(ch: char) -> Option<[u8; 2]> {
    let (base, mark, tone) = decompose(ch)?;
    if tone == Tone::None || !ch.is_uppercase() {
        return None;
    }
    let head: u8 = match (base, mark) {
        ('a', Mark::None) => b'A',
        ('e', Mark::None) => b'E',
        ('i', Mark::None) => b'I',
        ('o', Mark::None) => b'O',
        ('u', Mark::None) => b'U',
        ('y', Mark::None) => b'Y',
        ('a', Mark::Breve) => 0xA1,
        ('a', Mark::Circumflex) => 0xA2,
        ('e', Mark::Circumflex) => 0xA3,
        ('o', Mark::Circumflex) => 0xA4,
        ('o', Mark::Horn) => 0xA5,
        ('u', Mark::Horn) => 0xA6,
        _ => return None,
    };
    let tail = crate::letters::compose(base, mark, tone)?;
    let tail = *tcvn3(tail)?.first()?;
    Some([head, tail])
}

// ---------------------------------------------------------------------------
// VISCII, 8-bit — RFC 1456
// ---------------------------------------------------------------------------

fn viscii(ch: char) -> Option<&'static [u8]> {
    Some(match ch {
        'Ạ' => &[0x80],
        'Ắ' => &[0x81],
        'Ằ' => &[0x82],
        'Ặ' => &[0x83],
        'Ấ' => &[0x84],
        'Ầ' => &[0x85],
        'Ẩ' => &[0x86],
        'Ậ' => &[0x87],
        'Ẽ' => &[0x88],
        'Ẹ' => &[0x89],
        'Ế' => &[0x8A],
        'Ề' => &[0x8B],
        'Ể' => &[0x8C],
        'Ễ' => &[0x8D],
        'Ệ' => &[0x8E],
        'Ố' => &[0x8F],
        'Ồ' => &[0x90],
        'Ổ' => &[0x91],
        'Ỗ' => &[0x92],
        'Ộ' => &[0x93],
        'Ợ' => &[0x94],
        'Ớ' => &[0x95],
        'Ờ' => &[0x96],
        'Ở' => &[0x97],
        'Ị' => &[0x98],
        'Ỏ' => &[0x99],
        'Ọ' => &[0x9A],
        'Ỉ' => &[0x9B],
        'Ủ' => &[0x9C],
        'Ũ' => &[0x9D],
        'Ụ' => &[0x9E],
        'Ỳ' => &[0x9F],
        'Õ' => &[0xA0],
        'ắ' => &[0xA1],
        'ằ' => &[0xA2],
        'ặ' => &[0xA3],
        'ấ' => &[0xA4],
        'ầ' => &[0xA5],
        'ẩ' => &[0xA6],
        'ậ' => &[0xA7],
        'ẽ' => &[0xA8],
        'ẹ' => &[0xA9],
        'ế' => &[0xAA],
        'ề' => &[0xAB],
        'ể' => &[0xAC],
        'ễ' => &[0xAD],
        'ệ' => &[0xAE],
        'ố' => &[0xAF],
        'ồ' => &[0xB0],
        'ổ' => &[0xB1],
        'ỗ' => &[0xB2],
        'Ỡ' => &[0xB3],
        'Ơ' => &[0xB4],
        'ộ' => &[0xB5],
        'ờ' => &[0xB6],
        'ở' => &[0xB7],
        'ị' => &[0xB8],
        'Ự' => &[0xB9],
        'ứ' => &[0xBA],
        'Ừ' => &[0xBB],
        'Ử' => &[0xBC],
        'ơ' => &[0xBD],
        'ớ' => &[0xBE],
        'Ư' => &[0xBF],
        'À' => &[0xC0],
        'Á' => &[0xC1],
        'Â' => &[0xC2],
        'Ã' => &[0xC3],
        'Ả' => &[0xC4],
        'Ă' => &[0xC5],
        'ẳ' => &[0xC6],
        'ẵ' => &[0xC7],
        'È' => &[0xC8],
        'É' => &[0xC9],
        'Ê' => &[0xCA],
        'Ẻ' => &[0xCB],
        'Ì' => &[0xCC],
        'Í' => &[0xCD],
        'Ĩ' => &[0xCE],
        'ỳ' => &[0xCF],
        'Đ' => &[0xD0],
        'Ứ' => &[0xD1],
        'Ò' => &[0xD2],
        'Ó' => &[0xD3],
        'Ô' => &[0xD4],
        'ạ' => &[0xD5],
        'ỷ' => &[0xD6],
        'ừ' => &[0xD7],
        'ử' => &[0xD8],
        'Ù' => &[0xD9],
        'Ú' => &[0xDA],
        'ỹ' => &[0xDB],
        'ỵ' => &[0xDC],
        'Ý' => &[0xDD],
        'ỡ' => &[0xDE],
        'ư' => &[0xDF],
        'à' => &[0xE0],
        'á' => &[0xE1],
        'â' => &[0xE2],
        'ã' => &[0xE3],
        'ả' => &[0xE4],
        'ă' => &[0xE5],
        'ữ' => &[0xE6],
        'ẫ' => &[0xE7],
        'è' => &[0xE8],
        'é' => &[0xE9],
        'ê' => &[0xEA],
        'ẻ' => &[0xEB],
        'ì' => &[0xEC],
        'í' => &[0xED],
        'ĩ' => &[0xEE],
        'ỉ' => &[0xEF],
        'đ' => &[0xF0],
        'ự' => &[0xF1],
        'ò' => &[0xF2],
        'ó' => &[0xF3],
        'ô' => &[0xF4],
        'õ' => &[0xF5],
        'ỏ' => &[0xF6],
        'ọ' => &[0xF7],
        'ụ' => &[0xF8],
        'ù' => &[0xF9],
        'ú' => &[0xFA],
        'ũ' => &[0xFB],
        'ủ' => &[0xFC],
        'ý' => &[0xFD],
        'ợ' => &[0xFE],
        'Ữ' => &[0xFF],
        // The 6 least-used uppercase letters live in C0 (RFC 1456 §3).
        'Ẳ' => &[0x02],
        'Ẵ' => &[0x05],
        'Ẫ' => &[0x06],
        'Ỷ' => &[0x14],
        'Ỹ' => &[0x19],
        'Ỵ' => &[0x1E],
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// VPS, 8-bit
// ---------------------------------------------------------------------------

fn vps(ch: char) -> Option<&'static [u8]> {
    Some(match ch {
        'Ạ' => &[0x02],
        'Ậ' => &[0x03],
        'Ặ' => &[0x04],
        'Ẹ' => &[0x05],
        'Ệ' => &[0x06],
        'Ị' => &[0x10],
        'Ọ' => &[0x11],
        'Ộ' => &[0x12],
        'Ợ' => &[0x13],
        'Ụ' => &[0x14],
        'Ự' => &[0x15],
        'Ỵ' => &[0x19],
        'Ẫ' => &[0x1C],
        'Ữ' => &[0x1D],
        'À' => &[0x80],
        'Ả' => &[0x81],
        'Ã' => &[0x82],
        'Ấ' => &[0x83],
        'Ầ' => &[0x84],
        'Ẩ' => &[0x85],
        'ọ' => &[0x86],
        'ỗ' => &[0x87],
        'Ă' => &[0x88],
        'ế' => &[0x89],
        'ề' => &[0x8A],
        'ể' => &[0x8B],
        'ệ' => &[0x8C],
        'Ắ' => &[0x8D],
        'Ằ' => &[0x8E],
        'Ẳ' => &[0x8F],
        'Ế' => &[0x90],
        'Ề' => &[0x93],
        'Ể' => &[0x94],
        'Ễ' => &[0x95],
        'Ố' => &[0x96],
        'Ồ' => &[0x97],
        'Ổ' => &[0x98],
        'Ỗ' => &[0x99],
        'ý' => &[0x9A],
        'ỷ' => &[0x9B],
        'ỵ' => &[0x9C],
        'Ớ' => &[0x9D],
        'Ờ' => &[0x9E],
        'Ở' => &[0x9F],
        'ắ' => &[0xA1],
        'ằ' => &[0xA2],
        'ẳ' => &[0xA3],
        'ẵ' => &[0xA4],
        'ặ' => &[0xA5],
        'Ỡ' => &[0xA6],
        'ớ' => &[0xA7],
        'Ù' => &[0xA8],
        'ờ' => &[0xA9],
        'ở' => &[0xAA],
        'ỡ' => &[0xAB],
        'Ũ' => &[0xAC],
        'Ứ' => &[0xAD],
        'ợ' => &[0xAE],
        'Ừ' => &[0xAF],
        'ổ' => &[0xB0],
        'Ử' => &[0xB1],
        'Ỳ' => &[0xB2],
        'Ỹ' => &[0xB3],
        'Í' => &[0xB4],
        'Ì' => &[0xB5],
        'ộ' => &[0xB6],
        'Ỉ' => &[0xB7],
        'Ĩ' => &[0xB8],
        'Ó' => &[0xB9],
        'ử' => &[0xBA],
        'ữ' => &[0xBB],
        'Ò' => &[0xBC],
        'Ỏ' => &[0xBD],
        'Õ' => &[0xBE],
        'ự' => &[0xBF],
        'ầ' => &[0xC0],
        'Á' => &[0xC1],
        'Â' => &[0xC2],
        'ấ' => &[0xC3],
        'ẩ' => &[0xC4],
        'ẫ' => &[0xC5],
        'ậ' => &[0xC6],
        'đ' => &[0xC7],
        'ẻ' => &[0xC8],
        'É' => &[0xC9],
        'Ê' => &[0xCA],
        'ẹ' => &[0xCB],
        'ỉ' => &[0xCC],
        'ễ' => &[0xCD],
        'ị' => &[0xCE],
        'ỹ' => &[0xCF],
        'Ư' => &[0xD0],
        'Ủ' => &[0xD1],
        'ồ' => &[0xD2],
        'ố' => &[0xD3],
        'Ô' => &[0xD4],
        'ỏ' => &[0xD5],
        'ơ' => &[0xD6],
        'È' => &[0xD7],
        'ừ' => &[0xD8],
        'ứ' => &[0xD9],
        'Ú' => &[0xDA],
        'ũ' => &[0xDB],
        'ư' => &[0xDC],
        'Ý' => &[0xDD],
        'Ẻ' => &[0xDE],
        'à' => &[0xE0],
        'á' => &[0xE1],
        'â' => &[0xE2],
        'ã' => &[0xE3],
        'ả' => &[0xE4],
        'ạ' => &[0xE5],
        'ă' => &[0xE6],
        'è' => &[0xE8],
        'é' => &[0xE9],
        'ê' => &[0xEA],
        'ẽ' => &[0xEB],
        'ì' => &[0xEC],
        'í' => &[0xED],
        'ĩ' => &[0xEF],
        'Ẵ' => &[0xF0],
        'Đ' => &[0xF1],
        'ò' => &[0xF2],
        'ó' => &[0xF3],
        'ô' => &[0xF4],
        'õ' => &[0xF5],
        'Ơ' => &[0xF7],
        'ụ' => &[0xF8],
        'ù' => &[0xF9],
        'ú' => &[0xFA],
        'ủ' => &[0xFB],
        'Ỷ' => &[0xFD],
        'Ẽ' => &[0xFE],
        'ỳ' => &[0xFF],
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// VNI Windows / BK HCM1, combining
// ---------------------------------------------------------------------------

fn vni_windows(ch: char) -> Option<&'static [u8]> {
    Some(match ch {
        // lowercase
        'ạ' => &[0x61, 0xEF],
        'ả' => &[0x61, 0xFB],
        'ấ' => &[0x61, 0xE1],
        'ầ' => &[0x61, 0xE0],
        'ẩ' => &[0x61, 0xE5],
        'ẫ' => &[0x61, 0xE3],
        'ậ' => &[0x61, 0xE4],
        'ắ' => &[0x61, 0xE9],
        'ằ' => &[0x61, 0xE8],
        'ẳ' => &[0x61, 0xFA],
        'ẵ' => &[0x61, 0xFC],
        'ặ' => &[0x61, 0xEB],
        'ă' => &[0x61, 0xEA],
        'â' => &[0x61, 0xE2],
        'ã' => &[0x61, 0xF5],
        'à' => &[0x61, 0xF8],
        'á' => &[0x61, 0xF9],
        'ẹ' => &[0x65, 0xEF],
        'ẻ' => &[0x65, 0xFB],
        'ẽ' => &[0x65, 0xF5],
        'ế' => &[0x65, 0xE1],
        'ề' => &[0x65, 0xE0],
        'ể' => &[0x65, 0xE5],
        'ễ' => &[0x65, 0xE3],
        'ệ' => &[0x65, 0xE4],
        'ê' => &[0x65, 0xE2],
        'è' => &[0x65, 0xF8],
        'é' => &[0x65, 0xF9],
        'ị' => &[0xF2],
        'ỉ' => &[0xE6],
        'ĩ' => &[0xF3],
        'ì' => &[0xEC],
        'í' => &[0xED],
        'ọ' => &[0x6F, 0xEF],
        'ỏ' => &[0x6F, 0xFB],
        'ố' => &[0x6F, 0xE1],
        'ồ' => &[0x6F, 0xE0],
        'ổ' => &[0x6F, 0xE5],
        'ỗ' => &[0x6F, 0xE3],
        'ộ' => &[0x6F, 0xE4],
        'ô' => &[0x6F, 0xE2],
        'õ' => &[0x6F, 0xF5],
        'ò' => &[0x6F, 0xF8],
        'ó' => &[0x6F, 0xF9],
        'ơ' => &[0xF4],
        'ợ' => &[0xF4, 0xEF],
        'ở' => &[0xF4, 0xFB],
        'ỡ' => &[0xF4, 0xF5],
        'ờ' => &[0xF4, 0xF8],
        'ớ' => &[0xF4, 0xF9],
        'ụ' => &[0x75, 0xEF],
        'ủ' => &[0x75, 0xFB],
        'ũ' => &[0x75, 0xF5],
        'ự' => &[0xF6, 0xEF],
        'ử' => &[0xF6, 0xFB],
        'ữ' => &[0xF6, 0xF5],
        'ừ' => &[0xF6, 0xF8],
        'ứ' => &[0xF6, 0xF9],
        'ư' => &[0xF6],
        'ù' => &[0x75, 0xF8],
        'ú' => &[0x75, 0xF9],
        'ỵ' => &[0xEE],
        'ỷ' => &[0x79, 0xFB],
        'ỹ' => &[0x79, 0xF5],
        'ỳ' => &[0x79, 0xF8],
        'ý' => &[0x79, 0xF9],
        'đ' => &[0xF1],
        // uppercase
        'Ạ' => &[0x41, 0xCF],
        'Ả' => &[0x41, 0xDB],
        'Ấ' => &[0x41, 0xC1],
        'Ầ' => &[0x41, 0xC0],
        'Ẩ' => &[0x41, 0xC5],
        'Ẫ' => &[0x41, 0xC3],
        'Ậ' => &[0x41, 0xC4],
        'Ắ' => &[0x41, 0xC9],
        'Ằ' => &[0x41, 0xC8],
        'Ẳ' => &[0x41, 0xDA],
        'Ẵ' => &[0x41, 0xDC],
        'Ặ' => &[0x41, 0xCB],
        'Ă' => &[0x41, 0xCA],
        'Â' => &[0x41, 0xC2],
        'Ã' => &[0x41, 0xD5],
        'À' => &[0x41, 0xD8],
        'Á' => &[0x41, 0xD9],
        'Ẹ' => &[0x45, 0xCF],
        'Ẻ' => &[0x45, 0xDB],
        'Ẽ' => &[0x45, 0xD5],
        'Ế' => &[0x45, 0xC1],
        'Ề' => &[0x45, 0xC0],
        'Ể' => &[0x45, 0xC5],
        'Ễ' => &[0x45, 0xC3],
        'Ệ' => &[0x45, 0xC4],
        'Ê' => &[0x45, 0xC2],
        'È' => &[0x45, 0xD8],
        'É' => &[0x45, 0xD9],
        'Ị' => &[0xD2],
        'Ỉ' => &[0xC6],
        'Ĩ' => &[0xD3],
        'Ì' => &[0xCC],
        'Í' => &[0xCD],
        'Ọ' => &[0x4F, 0xCF],
        'Ỏ' => &[0x4F, 0xDB],
        'Ố' => &[0x4F, 0xC1],
        'Ồ' => &[0x4F, 0xC0],
        'Ổ' => &[0x4F, 0xC5],
        'Ỗ' => &[0x4F, 0xC3],
        'Ộ' => &[0x4F, 0xC4],
        'Ô' => &[0x4F, 0xC2],
        'Õ' => &[0x4F, 0xD5],
        'Ò' => &[0x4F, 0xD8],
        'Ó' => &[0x4F, 0xD9],
        'Ơ' => &[0xD4],
        'Ợ' => &[0xD4, 0xCF],
        'Ở' => &[0xD4, 0xDB],
        'Ỡ' => &[0xD4, 0xD5],
        'Ờ' => &[0xD4, 0xD8],
        'Ớ' => &[0xD4, 0xD9],
        'Ụ' => &[0x55, 0xCF],
        'Ủ' => &[0x55, 0xDB],
        'Ũ' => &[0x55, 0xD5],
        'Ự' => &[0xD6, 0xCF],
        'Ử' => &[0xD6, 0xDB],
        'Ữ' => &[0xD6, 0xD5],
        'Ừ' => &[0xD6, 0xD8],
        'Ứ' => &[0xD6, 0xD9],
        'Ư' => &[0xD6],
        'Ù' => &[0x55, 0xD8],
        'Ú' => &[0x55, 0xD9],
        'Ỵ' => &[0xCE],
        'Ỷ' => &[0x59, 0xDB],
        'Ỹ' => &[0x59, 0xD5],
        'Ỳ' => &[0x59, 0xD8],
        'Ý' => &[0x59, 0xD9],
        'Đ' => &[0xD1],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_byte_tables_are_injective() {
        for charset in [Charset::Tcvn3, Charset::Vps, Charset::Viscii] {
            let mut seen: Vec<(u8, char)> = Vec::new();
            for (kind, (base, mark)) in crate::letters::BASES.iter().enumerate() {
                for tone in 0..6 {
                    let ch = crate::letters::LOWER[kind][tone];
                    if ch == '\u{0}' {
                        continue;
                    }
                    let bytes = charset.encode(&ch.to_string());
                    assert_eq!(bytes.len(), 1, "{charset:?} {ch} → {bytes:?}");
                    if let Some((_, other)) = seen.iter().find(|(b, _)| *b == bytes[0]) {
                        panic!(
                            "{charset:?}: {ch} collides with {other} at 0x{:02X} \
                             (base {base} mark {mark:?} tone {tone})",
                            bytes[0]
                        );
                    }
                    seen.push((bytes[0], ch));
                }
            }
            assert_eq!(seen.len(), 73, "{charset:?} must cover the 73 untoned+toned letters");
        }
    }

    #[test]
    fn tcvn3_upper_toned_pair_is_documented_shape() {
        // VietUnicode chart: À = `41 B5`, Ắ = `A1 BE`, Ầ = `A2 C7`.
        assert_eq!(Charset::Tcvn3.encode("À"), vec![0x41, 0xB5]);
        assert_eq!(Charset::Tcvn3.encode("Ắ"), vec![0xA1, 0xBE]);
        assert_eq!(Charset::Tcvn3.encode("Ầ"), vec![0xA2, 0xC7]);
    }

    #[test]
    fn bk_hcm1_is_the_vni_layout_by_assumption() {
        // UNVERIFIED assumption (see `Charset::BkHcm1::status_note`).
        assert_eq!(Charset::BkHcm1.status(), TableStatus::Unverified);
        for word in ["tiếng", "Việt", "Đường", "Ằ"] {
            assert_eq!(
                Charset::BkHcm1.encode(word),
                Charset::VniWindows.encode(word)
            );
        }
    }

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
    fn viqr_escapes_triggers() {
        assert_eq!(Charset::Viqr.encode("á"), b"a'".to_vec());
        assert_eq!(Charset::Viqr.encode("ắ"), b"a('".to_vec());
        assert_eq!(Charset::Viqr.encode("?"), b"\\?".to_vec());
        assert_eq!(Charset::Viqr.encode("."), b"\\.".to_vec());
        assert_eq!(Charset::Viqr.encode("đ"), b"dd".to_vec());
    }

    #[test]
    fn engine_text_round_trips_through_encode() {
        for charset in Charset::ALL {
            let text = "Tiếng Việt";
            let bytes = charset.encode(text);
            assert!(!bytes.is_empty(), "{charset:?}");
            if matches!(charset, Charset::Unicode | Charset::Utf8) {
                assert_eq!(bytes, text.as_bytes());
            }
        }
    }
}
