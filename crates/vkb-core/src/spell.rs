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
//! accepted when a trailing coda is legal. The onset must also agree with the
//! nucleus (`ngi`, `ce`, `ka` are impossible), per A9.
//!
//! One English shape *is* a well-formed Vietnamese syllable: `test` composes to
//! `tét`. [`english_stop_cluster`] rejects the typing shape (a tone key stranded
//! inside a stop-coda cluster) so such words pass through raw, which is the
//! dictionary-free half of the English passthrough.
//!
//! Known gap (see `70-impl-notes-engine.md`): no dictionary, so no `hỏi`/`ngã`
//! word-sense check, which EVKey has.

use crate::letters::{Letter, Mark};

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
            if onset_takes_nucleus(onset, rest) && parse_nucleus_and_coda(rest) {
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

/// `true` when `letters` is a legal onset (C1) — the "only onset letters typed"
/// state the Telex `w` rule needs (`tw` → `tư`, but `btw` keeps its `w`).
pub fn is_onset(letters: &[Letter]) -> bool {
    let mut buf = [0u8; 3];
    let mut len = 0usize;
    for letter in letters {
        if !letter.ch.is_ascii_alphabetic() || len == buf.len() {
            return false;
        }
        buf[len] = letter.base() as u8;
        len += 1;
    }
    if len == 0 {
        return true;
    }
    let word = as_str(&buf[..len]);
    // `q` on its own is not an onset: only `qu` is (and that ends in a vowel).
    word != "q" && ONSETS.iter().any(|onset| onset.starts_with(word))
}

/// Orthographic agreement between an onset and the nucleus that follows it
/// (A9): `k`, `gh`, `ngh` only before `i e ê y`; `c`, `g`, `ng` not before
/// them (`gi` is matched as its own onset, so a bare `g` + `i` is `gì`, `gìn`);
/// `q` is only legal as `qu`.
fn onset_takes_nucleus(onset: &str, rest: &str) -> bool {
    let Some(first) = rest.chars().next() else {
        // Still inside the onset: every prefix is allowed.
        return true;
    };
    if !first.is_ascii() || !is_vowel(first as u8) {
        return true;
    }
    let front = matches!(first, 'i' | 'e' | 'y');
    match onset {
        "k" | "gh" | "ngh" => front,
        "c" | "ng" => !front,
        "g" => first != 'e',
        "q" => false,
        _ => true,
    }
}

/// `true` when `raw` is an English consonant cluster a tone key was typed into:
/// a *plain* vowel, the tone key, then a stop coda (`test` = t-e-**s**-t). A
/// Vietnamese syllable types such a tone after the coda (`tét` = `tets`) or
/// carries a modification mark (`việc` = `vieejc`), so this shape is rejected
/// and the word passes through as raw text.
///
/// This is the dictionary-free half of the English passthrough: the structural
/// test above already rejects `chrome`, `github` and `select`; `test` is a
/// well-formed Vietnamese syllable (`tét`), and this catches its typing shape.
pub fn english_stop_cluster(raw: &[char], letters: &[Letter]) -> bool {
    if letters.iter().any(|letter| letter.mark != Mark::None) {
        return false;
    }
    let Some(pos) = raw.iter().rposition(|ch| {
        matches!(ch.to_ascii_lowercase(), 's' | 'f' | 'r' | 'x' | 'j')
    }) else {
        return false;
    };
    let tail = &raw[pos + 1..];
    if tail.is_empty() || tail.len() > 2 {
        return false;
    }
    if !raw[..pos].iter().any(|ch| {
        let lower = ch.to_ascii_lowercase();
        lower.is_ascii() && is_vowel(lower as u8)
    }) {
        return false;
    }
    let mut coda = [0u8; 2];
    for (idx, ch) in tail.iter().enumerate() {
        let lower = ch.to_ascii_lowercase();
        if !lower.is_ascii_alphabetic() || is_vowel(lower as u8) {
            return false;
        }
        coda[idx] = lower as u8;
    }
    matches!(&coda[..tail.len()], b"c" | b"ch" | b"p" | b"t")
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
