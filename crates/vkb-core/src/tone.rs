//! Tone placement.
//!
//! One tone per word. The position is recomputed from the whole word on every
//! keystroke, so a tone typed early moves when the syllable grows
//! (`hoà` + `n` → `hoàn`, see `tests/tone_placement.rs`).
//!
//! Rules implemented (EVKey/UniKey "đặt dấu kiểu mới" for `old_style = false`):
//!
//! 1. A vowel carrying a modification mark wins (`thuyền` → `ê`, `được` → `ơ`).
//!    When several carry one, the last one wins (`ươ` → `ơ`).
//! 2. Three-vowel nuclei take the middle vowel (`oai` → `a`, `khuỷu` → `y`).
//! 3. `oa`, `oe`, `uy` take the last vowel when a coda follows, and — in the
//!    modern style only — the first vowel when the syllable ends in the vowel
//!    (`hòa` modern / `hoà` old, `thủy` / `thuỳ`).
//! 4. Otherwise the first vowel takes the tone (`thái`, `mùa`, `gửi`).
//! 5. `qu` and `gi` are onsets: their `u`/`i` is not a tone target when another
//!    vowel follows (`quá`, `già`), but is when it is the only vowel (`gì`).

use crate::letters::{Letter, Mark};

/// Index into `letters` of the letter that carries the tone.
pub fn target(letters: &[Letter], old_style: bool) -> Option<usize> {
    let mut vowels: [usize; 32] = [0; 32];
    let mut vowel_count = 0usize;
    for (idx, letter) in letters.iter().enumerate() {
        if vowel_count < vowels.len() && letter.is_vowel() {
            vowels[vowel_count] = idx;
            vowel_count += 1;
        }
    }
    if vowel_count == 0 {
        return None;
    }

    // `qu` / `gi` onsets: drop the onset vowel when a nucleus follows.
    let mut start = 0usize;
    if vowel_count >= 2 {
        let first = vowels[0];
        let base = letters[first].base();
        let prev = first.checked_sub(1).map(|i| letters[i].base());
        if matches!((base, prev), ('u', Some('q')) | ('i', Some('g'))) {
            start = 1;
        }
    }

    let candidates = &vowels[start..vowel_count];
    if candidates.is_empty() {
        return None;
    }

    // 1. A modified vowel wins; the last one of several (`ươ` → `ơ`).
    let mut marked: Option<usize> = None;
    for &idx in candidates {
        if letters[idx].mark != Mark::None {
            marked = Some(idx);
        }
    }
    if let Some(idx) = marked {
        return Some(idx);
    }

    // 2. / 3. / 4.
    match candidates.len() {
        1 => Some(candidates[0]),
        2 => {
            let first = candidates[0];
            let last = candidates[1];
            let pair = (letters[first].base(), letters[last].base());
            let open_pair = matches!(pair, ('o', 'a') | ('o', 'e') | ('u', 'y'));
            if open_pair {
                let has_coda = letters
                    .get(last + 1)
                    .map(|_| letters[last + 1..].iter().any(|l| l.ch.is_ascii_alphabetic()))
                    .unwrap_or(false);
                if has_coda || old_style {
                    Some(last)
                } else {
                    Some(first)
                }
            } else {
                Some(first)
            }
        }
        n => Some(candidates[n - 2]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::letters::LetterBuf;

    fn word(input: &str) -> LetterBuf {
        let mut buf = LetterBuf::new();
        for ch in input.chars() {
            buf.push(Letter::literal(ch));
        }
        buf
    }

    #[test]
    fn single_vowel() {
        assert_eq!(target(&word("a"), true), Some(0));
        assert_eq!(target(&word("ng"), true), None);
    }

    #[test]
    fn picks_last_modified_vowel() {
        let mut letters = word("uo");
        letters[0].mark = Mark::Horn;
        letters[1].mark = Mark::Horn;
        assert_eq!(target(&letters, false), Some(1));
    }

    #[test]
    fn triple_takes_middle() {
        assert_eq!(target(&word("oai"), false), Some(1));
        assert_eq!(target(&word("uyu"), false), Some(1));
    }

    #[test]
    fn open_pairs_depend_on_coda_and_style() {
        assert_eq!(target(&word("hoa"), false), Some(1));
        assert_eq!(target(&word("hoa"), true), Some(2));
        assert_eq!(target(&word("hoan"), false), Some(2));
        assert_eq!(target(&word("hoan"), true), Some(2));
    }

    #[test]
    fn qu_and_gi_onsets_are_skipped() {
        assert_eq!(target(&word("qua"), false), Some(2));
        assert_eq!(target(&word("gia"), false), Some(2));
        assert_eq!(target(&word("gi"), false), Some(1));
    }
}
