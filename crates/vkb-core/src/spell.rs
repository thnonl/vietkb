//! Spell check: is the word being typed a well-formed Vietnamese syllable?
//!
//! This is a *structural* check (onset + nucleus + coda), not a dictionary. A
//! word it rejects is left as raw keystrokes, which is what `Kiểm tra chính tả`
//! does in EVKey, and is also how English input survives: `google`, `windows`
//! and `install` all fail the nucleus/coda test. The cost of a false rejection
//! is that a rare-but-valid word keeps its raw spelling, so the tables below
//! lean permissive.
//!
//! The onset is matched longest-first and backtracked (`gi` + `ương` must parse
//! as onset `gi` + nucleus `ương`, not `g` + `iương`), and a vowel run is only
//! accepted when a trailing coda is legal.
//!
//! Known gap (see `70-impl-notes-engine.md`): no dictionary, so no `hỏi`/`ngã`
//! word-sense check, which EVKey has.

use crate::letters::Letter;

/// Vietnamese onsets, longest first.
const ONSETS: &[&str] = &[
    "ngh", "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr", "b", "c", "d", "g", "h",
    "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x",
];

const JFWZ: &[&str] = &["f", "j", "w", "z"];

/// Vietnamese codas, longest first.
const CODAS: &[&str] = &["ch", "ng", "nh", "c", "m", "n", "p", "t"];

/// One-vowel nuclei.
const NUCLEI1: &[&str] = &["a", "e", "i", "o", "u", "y"];

/// Two-vowel nuclei, base letters only (`ươ` → `uo`, `iê` → `ie`).
///
/// `oo` is deliberately absent: it only occurs in loanwords (`xoong`), and
/// including it makes `google` compose to `gô` before the spell check can
/// reject it. See `70-impl-notes-engine.md`.
const NUCLEI2: &[&str] = &[
    "ai", "ao", "au", "ay", "eo", "eu", "ia", "ie", "iu", "oa", "oe", "oi", "ua", "ue", "ui",
    "uo", "uu", "uy", "ya", "ye",
];

/// Three-vowel nuclei, base letters only (`uyê` → `uye`, `ươi` → `uoi`).
const NUCLEI3: &[&str] = &[
    "ieu", "yeu", "oai", "oao", "oay", "oeo", "uay", "uoi", "uou", "uya", "uye", "uyu",
];

/// `true` when `letters` forms a syllable Vietnamese spelling allows.
///
/// `allow_jfwz` mirrors the `Cho phép 'f j w z' làm phụ âm` option.
pub fn is_valid_syllable(letters: &[Letter], allow_jfwz: bool) -> bool {
    if letters.is_empty() {
        return true;
    }
    let mut buf = [0u8; 32];
    let mut len = 0usize;
    for letter in letters {
        if !letter.ch.is_ascii_alphabetic() || len == buf.len() {
            return false;
        }
        buf[len] = letter.base() as u8;
        len += 1;
    }
    let word = as_str(&buf[..len]);

    for onset in ONSETS {
        if let Some(rest) = word.strip_prefix(onset) {
            if parse_nucleus_and_coda(rest) {
                return true;
            }
        }
    }
    if allow_jfwz {
        for onset in JFWZ {
            if let Some(rest) = word.strip_prefix(onset) {
                if parse_nucleus_and_coda(rest) {
                    return true;
                }
            }
        }
    }
    parse_nucleus_and_coda(word)
}

/// `rest` starts where the onset ended. An empty `rest` means the user is still
/// inside the onset, which every syllable prefix allows.
fn parse_nucleus_and_coda(rest: &str) -> bool {
    if rest.is_empty() {
        return true;
    }
    let bytes = rest.as_bytes();
    let mut end = 0usize;
    while end < bytes.len() && is_vowel(bytes[end]) {
        end += 1;
    }
    if end == 0 {
        // A consonant after the onset, with no vowel between.
        return false;
    }
    let nucleus = &rest[..end];
    let ok = match nucleus.len() {
        1 => NUCLEI1.contains(&nucleus),
        2 => NUCLEI2.contains(&nucleus),
        3 => NUCLEI3.contains(&nucleus),
        _ => false,
    };
    if !ok {
        return false;
    }
    let coda = &rest[end..];
    coda.is_empty() || CODAS.contains(&coda)
}

fn is_vowel(byte: u8) -> bool {
    matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
}

fn as_str(bytes: &[u8]) -> &str {
    // Only ASCII letters are ever copied into the buffer; stay total anyway.
    std::str::from_utf8(bytes).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::letters::LetterBuf;

    fn check(word: &str, allow_jfwz: bool) -> bool {
        let mut buf = LetterBuf::new();
        for ch in word.chars() {
            buf.push(Letter::literal(ch));
        }
        is_valid_syllable(&buf, allow_jfwz)
    }

    #[test]
    fn accepts_vietnamese_words() {
        for word in [
            "tieng", "viet", "duong", "nguyen", "thuyen", "khuynh", "nghieng", "quoc", "gia",
            "giuong", "oai", "hoac", "truong", "a", "khuya", "khuyu", "chuyen", "quyet",
            "nguoi", "nghiep", "khong", "thuong",
        ] {
            assert!(check(word, true), "{word} should be valid");
        }
    }

    #[test]
    fn rejects_english_words() {
        for word in ["google", "windows", "install", "schema", "function"] {
            assert!(!check(word, true), "{word} should be invalid");
        }
    }

    #[test]
    fn onset_prefixes_are_valid_while_typing() {
        assert!(check("ngh", true));
        assert!(check("ng", true));
        assert!(check("tr", true));
        assert!(check("n", true));
        assert!(!check("btw", true));
    }

    #[test]
    fn gi_is_an_onset_not_a_nucleus() {
        assert!(check("giuong", true));
        assert!(check("giu", true));
        assert!(check("gia", true));
    }

    #[test]
    fn jfwz_toggle_changes_onset_acceptance() {
        assert!(check("w", true));
        assert!(!check("w", false));
        assert!(check("za", true));
        assert!(!check("za", false));
    }
}
