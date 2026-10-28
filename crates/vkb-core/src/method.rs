//! The composition state machine.
//!
//! The whole word is a *pure function of the raw keystrokes*: [`compose`]
//! replays the buffer from the first key on every call. That buys three things
//! incremental mutation makes hard — backspace over the raw history,
//! `restore_wrong_word`, and a tone that re-places itself as the syllable grows
//! (`hoà` + `n` → `hoàn`) — for O(32) work per key.

use crate::letters::{Letter, LetterBuf, Mark, Tone};
use crate::tone as place;
use crate::{EngineOptions, InputMethod};

/// Composed state of the current word.
#[derive(Debug, Default)]
pub struct Composed {
    /// Letters, case preserved as typed, marks attached.
    pub letters: LetterBuf,
    /// The word's single tone; its position is computed at render time.
    pub tone: Tone,
    /// The word is raw text from here on (numeric bypass, failed spell check).
    pub literal: bool,
    /// VIQR: a `\` is waiting for the character it protects.
    pub escape_pending: bool,
    /// A modifier keystroke had to become literal (`ass`, `aww`, `aaa`). The
    /// word is no longer a candidate for the spell check's raw rewrite.
    pub escaped: bool,
}

impl Composed {
    pub fn clear(&mut self) {
        self.letters.clear();
        self.tone = Tone::None;
        self.literal = false;
        self.escape_pending = false;
        self.escaped = false;
    }
}

/// Replay `raw` and return the composed word.
///
/// `literal_from` is the index from which the caller already forced raw text
/// (the "phục hồi từ" hotkey); `usize::MAX` disables it.
pub fn compose(raw: &[char], options: &EngineOptions, literal_from: usize, out: &mut Composed) {
    out.clear();
    let mut escape = false;

    for (i, &ch) in raw.iter().enumerate() {
        if escape {
            escape = false;
            out.letters.push(Letter::literal(ch));
            continue;
        }
        // VIQR `\` protects the next character and is never part of the text,
        // even inside a literal (raw) word: `how are you\?`.
        if options.method == InputMethod::Viqr && ch == '\\' {
            escape = true;
            continue;
        }
        if out.literal || i >= literal_from {
            out.literal = true;
            out.letters.push(Letter::literal(ch));
            continue;
        }
        // A modifier keystroke that had to become literal (`ass`, `aww`): the
        // rest of the word stays literal too, so `docker` cannot turn into
        // `dỏcke` at the `r`.
        if out.escaped {
            out.letters.push(Letter::literal(ch));
            continue;
        }

        // Compat shim, ported from the reference implementation: with
        // `f j w z` allowed as consonants these shapes stay raw.
        if options.method == InputMethod::Telex
            && options.jfwz_as_consonant
            && compat_telex_literal(&raw[..=i])
        {
            literalize(out, raw, i);
            continue;
        }

        if ch.is_ascii_digit() {
            step_digit(out, raw, i, options);
        } else if !ch.is_ascii_alphabetic() {
            if options.method == InputMethod::Viqr {
                step_viqr_symbol(out, ch, options);
            } else {
                out.letters.push(Letter::literal(ch));
            }
        } else {
            match options.method {
                InputMethod::Telex => step_telex(out, ch, options),
                InputMethod::Vni => step_vni(out, ch),
                InputMethod::Viqr => step_viqr(out, ch),
            }
        }

        // A word that stops looking Vietnamese shows the raw keystrokes instead
        // of a wrong accent (`Kiểm tra chính tả`), which is both what the user
        // typed and what `restore_wrong_word` would produce at the boundary.
        if options.spell_check
            && !out.literal
            && !out.escaped
            && !crate::spell::is_valid_syllable(&out.letters, options.jfwz_as_consonant)
        {
            literalize(out, raw, i);
        }
    }
    out.escape_pending = escape;
}

/// `true` when a digit typed now joins the word instead of ending it.
pub(crate) fn digit_joins_word(raw: &[char]) -> bool {
    raw.is_empty() || bypass_digit_suffix(raw, raw.len())
}

/// Raw shapes the reference implementation kept literal when `f j w z` are
/// allowed as consonants (`"of"`, `"oj"` would otherwise become `ò`, `ọ`).
fn compat_telex_literal(prefix: &[char]) -> bool {
    if prefix.len() > 2 {
        return false;
    }
    let mut buf = [0u8; 2];
    let mut len = 0;
    for &ch in prefix {
        buf[len] = ch.to_ascii_lowercase() as u8;
        len += 1;
    }
    matches!(&buf[..len], [b'o', b'f'] | [b'o', b'j'])
}

// ---------------------------------------------------------------------------
// Digits: numeric bypass, e-mail guard, VNI triggers
// ---------------------------------------------------------------------------

fn step_digit(out: &mut Composed, raw: &[char], i: usize, options: &EngineOptions) {
    let digit = raw[i];
    let at_start = out.letters.is_empty();

    if options.method != InputMethod::Vni || at_start || bypass_digit_suffix(raw, i) {
        literalize(out, raw, i);
        return;
    }
    match digit {
        '0' => {
            for letter in out.letters.iter_mut() {
                letter.mark = Mark::None;
            }
            out.tone = Tone::None;
        }
        '1' | '2' | '3' | '4' | '5' => {
            let tone = tone_of(digit);
            apply_tone(out, tone, digit, options);
        }
        '6' => mark_last(out, &['a', 'e', 'o'], Mark::Circumflex, digit),
        '7' => mark_last(out, &['o', 'u'], Mark::Horn, digit),
        '8' => mark_last(out, &['a'], Mark::Breve, digit),
        '9' => mark_last(out, &['d'], Mark::Bar, digit),
        _ => out.letters.push(Letter::literal(digit)),
    }
}

/// `pndquyens203@gmail.com`: three or more leading consonants means the user is
/// not typing a syllable, so the word reverts to raw keystrokes.
fn bypass_digit_suffix(raw: &[char], i: usize) -> bool {
    if i < 3 {
        return false;
    }
    let mut run = 0usize;
    for &ch in &raw[..i] {
        let lower = ch.to_ascii_lowercase();
        if matches!(lower, 'a' | 'e' | 'i' | 'o' | 'u' | 'y') || !lower.is_ascii_alphabetic() {
            break;
        }
        run += 1;
    }
    run >= 3 && !raw_prefix_is_ngh(&raw[..i])
}

fn raw_prefix_is_ngh(prefix: &[char]) -> bool {
    prefix.len() >= 3
        && prefix[0].eq_ignore_ascii_case(&'n')
        && prefix[1].eq_ignore_ascii_case(&'g')
        && prefix[2].eq_ignore_ascii_case(&'h')
}

fn literalize(out: &mut Composed, raw: &[char], upto: usize) {
    out.letters.clear();
    out.tone = Tone::None;
    out.literal = true;
    for &ch in &raw[..=upto] {
        out.letters.push(Letter::literal(ch));
    }
}

// ---------------------------------------------------------------------------
// Telex
// ---------------------------------------------------------------------------

fn step_telex(out: &mut Composed, ch: char, options: &EngineOptions) {
    let lower = ch.to_ascii_lowercase();

    // `f j w z` as consonants: only in the onset.
    if options.jfwz_as_consonant && matches!(lower, 'f' | 'j' | 'w' | 'z') && !has_vowel(out) {
        out.letters.push(Letter::literal(ch));
        return;
    }

    match lower {
        's' | 'f' | 'r' | 'x' | 'j' => apply_tone(out, tone_of(lower), ch, options),
        'w' => apply_w(out, ch),
        'a' | 'e' | 'o' => {
            let doubled = out
                .letters
                .last()
                .is_some_and(|last| last.base() == lower && last.mark == Mark::None);
            if doubled {
                if let Some(last) = out.letters.last_mut() {
                    last.mark = Mark::Circumflex;
                }
            } else {
                // A doubled key that could not modify: `aaa` keeps both letters.
                if out.letters.last().is_some_and(|last| last.base() == lower) {
                    out.escaped = true;
                }
                out.letters.push(Letter::literal(ch));
            }
        }
        'd' => {
            let doubled = out
                .letters
                .last()
                .is_some_and(|last| last.base() == 'd' && last.mark == Mark::None);
            if doubled {
                if let Some(last) = out.letters.last_mut() {
                    last.mark = Mark::Bar;
                }
            } else {
                if out.letters.last().is_some_and(|last| last.base() == 'd') {
                    out.escaped = true;
                }
                out.letters.push(Letter::literal(ch));
            }
        }
        _ => out.letters.push(Letter::literal(ch)),
    }
}

/// Telex `w`: `aw` → `ă`, `ow` → `ơ`, `uw` → `ư`, `uow` → `ươ`.
fn apply_w(out: &mut Composed, key: char) {
    if let Some(idx) = w_target(&out.letters) {
        if out.letters[idx].base() == 'o' {
            if let Some(u) = out.letters[..idx]
                .iter()
                .rposition(|letter| letter.base() == 'u')
            {
                out.letters[u].mark = Mark::Horn;
                out.letters[idx].mark = Mark::Horn;
                return;
            }
        }
        if out.letters[idx].mark == Mark::None {
            out.letters[idx].mark = match out.letters[idx].base() {
                'a' => Mark::Breve,
                _ => Mark::Horn,
            };
            return;
        }
    }
    out.escaped = true;
    out.letters.push(Letter::literal(key));
}

fn w_target(letters: &[Letter]) -> Option<usize> {
    let last = letters
        .iter()
        .rposition(|letter| matches!(letter.base(), 'a' | 'o' | 'u'))?;
    if letters[last].base() == 'u' {
        if let Some(prev) = letters[..last]
            .iter()
            .rposition(|letter| letter.base() == 'u')
        {
            return Some(prev);
        }
    }
    Some(last)
}

// ---------------------------------------------------------------------------
// VNI and VIQR
// ---------------------------------------------------------------------------

fn step_vni(out: &mut Composed, ch: char) {
    out.letters.push(Letter::literal(ch));
}

fn step_viqr(out: &mut Composed, ch: char) {
    let doubled = ch.eq_ignore_ascii_case(&'d')
        && out
            .letters
            .last()
            .is_some_and(|last| last.base() == 'd' && last.mark == Mark::None);
    if doubled {
        if let Some(last) = out.letters.last_mut() {
            last.mark = Mark::Bar;
        }
        return;
    }
    out.letters.push(Letter::literal(ch));
}

/// VIQR punctuation: tone marks, `^ ( + -` modifiers.
fn step_viqr_symbol(out: &mut Composed, ch: char, options: &EngineOptions) {
    match ch {
        '\'' | '`' | '?' | '~' | '.' => apply_tone(out, tone_of(ch), ch, options),
        '^' => mark_last(out, &['a', 'e', 'o'], Mark::Circumflex, ch),
        '(' => mark_last(out, &['a'], Mark::Breve, ch),
        '+' => mark_last(out, &['o', 'u'], Mark::Horn, ch),
        '-' => mark_last(out, &['d'], Mark::Bar, ch),
        _ => out.letters.push(Letter::literal(ch)),
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn tone_of(ch: char) -> Tone {
    match ch.to_ascii_lowercase() {
        's' | '\'' | '1' => Tone::Acute,
        'f' | '`' | '2' => Tone::Grave,
        'r' | '?' | '3' => Tone::Hook,
        'x' | '~' | '4' => Tone::Tilde,
        'j' | '.' | '5' => Tone::Dot,
        _ => Tone::None,
    }
}

fn has_vowel(out: &Composed) -> bool {
    out.letters.iter().any(|letter| letter.is_vowel())
}

fn apply_tone(out: &mut Composed, tone: Tone, key: char, options: &EngineOptions) {
    if tone == Tone::None {
        out.letters.push(Letter::literal(key));
        return;
    }
    match place::target(&out.letters, options.old_tone_style) {
        None => out.letters.push(Letter::literal(key)),
        Some(_) => {
            if out.tone == tone {
                // Same tone typed twice: the keystroke wins, the accent drops.
                out.tone = Tone::None;
                out.escaped = true;
                out.letters.push(Letter::literal(key));
            } else {
                out.tone = tone;
            }
        }
    }
}

fn mark_last(out: &mut Composed, bases: &[char], mark: Mark, key: char) {
    if let Some(idx) = out
        .letters
        .iter()
        .rposition(|letter| bases.contains(&letter.base()) && letter.mark == Mark::None)
    {
        out.letters[idx].mark = mark;
    } else {
        out.escaped = true;
        out.letters.push(Letter::literal(key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(options: &EngineOptions, input: &str) -> String {
        let raw: Vec<char> = input.chars().collect();
        let mut composed = Composed::default();
        compose(&raw, options, usize::MAX, &mut composed);
        composed
            .letters
            .iter()
            .map(|letter| {
                crate::letters::compose(letter.ch, letter.mark, Tone::None)
                    .unwrap_or(letter.ch)
            })
            .collect()
    }

    #[test]
    fn render_marks_and_tones() {
        let options = EngineOptions::default();
        assert_eq!(render(&options, "aa"), "â");
        assert_eq!(render(&options, "aw"), "ă");
        assert_eq!(render(&options, "ow"), "ơ");
        assert_eq!(render(&options, "uw"), "ư");
        assert_eq!(render(&options, "dd"), "đ");
    }

    #[test]
    fn telex_uow_marks_both() {
        let options = EngineOptions::default();
        let raw: Vec<char> = "uow".chars().collect();
        let mut composed = Composed::default();
        compose(&raw, &options, usize::MAX, &mut composed);
        assert_eq!(composed.letters[0].mark, Mark::Horn);
        assert_eq!(composed.letters[1].mark, Mark::Horn);
    }

    #[test]
    fn leading_digit_turns_the_word_literal() {
        let options = EngineOptions::default();
        assert_eq!(render(&options, "1thuongwf"), "1thuongwf");
    }
}
