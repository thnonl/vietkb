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
    /// A keystroke escaped to the literal raw text (`ass`, `await`, `aaa`). The
    /// rest of the word stays literal too.
    pub escaped: bool,
    /// Raw index of the keystroke that produced [`Composed::tone`]
    /// (`usize::MAX` = no tone).
    pub tone_at: usize,
    /// Raw index of the keystroke that applied the last mark (`usize::MAX` = the
    /// mark came from something else, e.g. the inserted `ư` of a lone `w`).
    pub mark_at: usize,
    /// Letter index the tone is pinned to. `None` = re-place it per A4 on every
    /// render (`free_tone = true`); `Some` = `free_tone = false`, the tone stays
    /// on the vowel it hit when the key was typed.
    pub tone_slot: Option<usize>,
}

impl Composed {
    pub fn clear(&mut self) {
        self.letters.clear();
        self.tone = Tone::None;
        self.literal = false;
        self.escaped = false;
        self.tone_at = usize::MAX;
        self.mark_at = usize::MAX;
        self.tone_slot = None;
    }
}

/// Replay `raw` and return the composed word.
///
/// `literal_from` is the index from which the caller already forced raw text
/// (the "phục hồi từ" hotkey); `usize::MAX` disables it.
pub fn compose(raw: &[char], options: &EngineOptions, literal_from: usize, out: &mut Composed) {
    out.clear();

    for (i, &ch) in raw.iter().enumerate() {
        // Compat shim, ported from the reference implementation: with
        // `f j w z` allowed as consonants these shapes stay raw.
        if matches!(options.method, InputMethod::Telex | InputMethod::Legacy)
            && options.jfwz_as_consonant
            && compat_telex_literal(&raw[..=i])
        {
            literalize(out, raw, i);
            continue;
        }
        if out.literal || i >= literal_from {
            out.literal = true;
            out.letters.push(Letter::literal(ch));
            continue;
        }
        // A keystroke that escaped to the raw text (`ass`, `await`): the rest of
        // the word stays literal too, so `docker` cannot turn into `dỏcke`.
        if out.escaped {
            out.letters.push(Letter::literal(ch));
            continue;
        }

        if ch.is_ascii_digit() {
            step_digit(out, raw, i, options);
        } else if !ch.is_ascii_alphabetic() {
            out.letters.push(Letter::literal(ch));
        } else {
            match options.method {
                InputMethod::Telex | InputMethod::Legacy => step_telex(out, raw, i, options),
                InputMethod::Vni => out.letters.push(Letter::literal(ch)),
            }
        }

        // Promote `ưo` only once a legal C2 (or `i`/`u` continuation) follows.
        if options.auto_horn_uo && !out.literal && i > 0 && matches!(ch.to_ascii_lowercase(), 'i' | 'u' | 'c' | 'h' | 'm' | 'n' | 'p' | 't') {
            if let Some(o) = out.letters.iter().rposition(|letter| letter.base() == 'o' && letter.mark == Mark::None) {
                if o > 0 && out.letters[o - 1].base() == 'u' && out.letters[o - 1].mark == Mark::Horn {
                    out.letters[o].mark = Mark::Horn;
                }
            }
        }

        // A word that stops looking Vietnamese shows the raw keystrokes instead
        // of a wrong accent (`Kiểm tra chính tả`), which is both what the user
        // typed and what `restore_wrong_word` would produce at the boundary.
        if options.spell_check
            && !out.literal
            && !out.escaped
            && (!crate::spell::is_valid_syllable(&out.letters, options.jfwz_as_consonant)
                || (matches!(options.method, InputMethod::Telex | InputMethod::Legacy)
                    && crate::spell::english_stop_cluster(&raw[..=i], &out.letters)))
        {
            literalize(out, raw, i);
        }
    }
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

/// `true` when a digit typed now joins the word instead of ending it.
pub(crate) fn digit_joins_word(raw: &[char]) -> bool {
    raw.is_empty() || bypass_digit_suffix(raw, raw.len())
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
            out.tone_at = usize::MAX;
            out.mark_at = usize::MAX;
            out.tone_slot = None;
        }
        '1' | '2' | '3' | '4' | '5' => {
            let tone = tone_of(digit);
            apply_tone(out, raw, i, tone, digit, options);
        }
        '6' => mark_last(out, raw, i, &['a', 'e', 'o'], Mark::Circumflex, digit),
        '7' => mark_vni_horn(out, raw, i, digit),
        '8' => mark_last(out, raw, i, &['a'], Mark::Breve, digit),
        '9' => mark_last(out, raw, i, &['d'], Mark::Bar, digit),
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
    out.tone_at = usize::MAX;
    out.mark_at = usize::MAX;
    out.tone_slot = None;
    for &ch in &raw[..=upto] {
        out.letters.push(Letter::literal(ch));
    }
}

/// The keystroke cannot complete a transform: the word falls back to the
/// literal keystrokes, never to a half-transformed prefix (`await` stays
/// `await`, `ass` shows `as`, `a^^` shows `a^`).
///
/// `drop` is the raw index of the trigger keystroke that produced the transform
/// being escaped; the literal text is the raw keystrokes without it (the first
/// press consumed the character, the second one is the text).
fn escape_to_raw(out: &mut Composed, raw: &[char], upto: usize, drop: Option<usize>) {
    out.letters.clear();
    out.tone = Tone::None;
    out.tone_at = usize::MAX;
    out.mark_at = usize::MAX;
    out.tone_slot = None;
    out.escaped = true;
    for (idx, &ch) in raw[..=upto].iter().enumerate() {
        if Some(idx) == drop {
            continue;
        }
        out.letters.push(Letter::literal(ch));
    }
}

/// Raw index of the most recent `key` typed before `upto`.
fn previous_trigger(raw: &[char], upto: usize, key: char) -> Option<usize> {
    raw[..upto]
        .iter()
        .rposition(|ch| ch.eq_ignore_ascii_case(&key))
}

/// The raw index to drop when a transform is escaped by repeating its trigger.
fn drop_for(raw: &[char], i: usize, key: char, at: usize) -> Option<usize> {
    if at != usize::MAX && at < i && raw[at].eq_ignore_ascii_case(&key) {
        Some(at)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Telex
// ---------------------------------------------------------------------------

fn step_telex(out: &mut Composed, raw: &[char], i: usize, options: &EngineOptions) {
    let ch = raw[i];
    let lower = ch.to_ascii_lowercase();

    match lower {
        's' | 'f' | 'r' | 'x' | 'j' => apply_tone(out, raw, i, tone_of(lower), ch, options),
        'w' if options.jfwz_as_consonant && !has_vowel(out) => {
            out.letters.push(Letter::literal(ch));
        }
        'w' => apply_w(out, raw, i, ch, options),
        'a' | 'e' | 'o' => double_letter(out, raw, i, lower, Mark::Circumflex),
        'd' => double_letter(out, raw, i, 'd', Mark::Bar),
        _ => out.letters.push(Letter::literal(ch)),
    }
}

/// `aa`/`ee`/`oo` → `â ê ô`, `dd` → `đ`. A third press escapes to the raw text.
fn double_letter(out: &mut Composed, raw: &[char], i: usize, base: char, mark: Mark) {
    let ch = raw[i];
    let doubled = out
        .letters
        .last()
        .is_some_and(|last| last.base() == base && last.mark == Mark::None);
    if doubled {
        if let Some(last) = out.letters.last_mut() {
            last.mark = mark;
        }
        out.mark_at = i;
        return;
    }
    if out.letters.last().is_some_and(|last| last.base() == base) {
        // The same key again: escape the whole word to its literal keystrokes.
        let drop = drop_for(raw, i, base, out.mark_at);
        escape_to_raw(out, raw, i, drop);
        return;
    }
    out.letters.push(Letter::literal(ch));
}

/// Telex `w`: `aw` → `ă`, `ow` → `ơ`, `uw` → `ư`, `uow` → `ươ`, and a lone `w`
/// at a syllable start → `ư` (A1 W-rules 1–7).
fn apply_w(out: &mut Composed, raw: &[char], i: usize, key: char, options: &EngineOptions) {
    if let Some(idx) = w_target(&out.letters) {
        // Rule 1: `uo` → `ươ`.
        if out.letters[idx].base() == 'o' {
            if let Some(u) = out.letters[..idx]
                .iter()
                .rposition(|letter| letter.base() == 'u')
            {
                if out.letters[u].mark == Mark::Horn && out.letters[idx].mark == Mark::Horn {
                    let drop = drop_for(raw, i, key, out.mark_at);
                    escape_to_raw(out, raw, i, drop);
                    return;
                }
                out.letters[u].mark = Mark::Horn;
                out.letters[idx].mark = Mark::Horn;
                out.mark_at = i;
                return;
            }
        }
        match out.letters[idx].mark {
            Mark::None => {
                out.letters[idx].mark = if out.letters[idx].base() == 'a' {
                    Mark::Breve
                } else {
                    Mark::Horn
                };
                out.mark_at = i;
                return;
            }
            // Rule 3: `aaw` → `ă`, the breve replaces the circumflex.
            Mark::Circumflex if out.letters[idx].base() == 'a' => {
                out.letters[idx].mark = Mark::Breve;
                out.mark_at = i;
                return;
            }
            // Already transformed by a previous `w`: escape.
            _ => {
                let drop = drop_for(raw, i, key, out.mark_at);
                escape_to_raw(out, raw, i, drop);
                return;
            }
        }
    }
    // Rule 6: V is empty → insert `ư` (`w` → `ư`, `tw` → `tư`, `nhw` → `như`).
    if !options.jfwz_as_consonant
        && !has_vowel(out)
        && crate::spell::is_onset(&out.letters)
        && w_is_the_whole_tail(raw, i)
    {
        out.letters.push(Letter::new(
            if key.is_uppercase() { 'U' } else { 'u' },
            Mark::Horn,
        ));
        out.mark_at = i;
        return;
    }
    // Rule 7: otherwise `w` is literal and the word stops transforming
    // (`hwllo`, `qwerty`, `btw`).
    out.escaped = true;
    out.letters.push(Letter::literal(key));
}

/// A lone `w` is only `ư` when nothing else has been typed after it yet: with
/// a plain letter following (`hwllo`, `qwerty`) the `w` is a letter, and a
/// following `w` is its own escape (`ww` → `w`).
fn w_is_the_whole_tail(raw: &[char], i: usize) -> bool {
    raw[i + 1..]
        .iter()
        .all(|ch| matches!(ch.to_ascii_lowercase(), 'w' | 's' | 'f' | 'r' | 'x' | 'j'))
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
// Shared helpers
// ---------------------------------------------------------------------------

fn tone_of(ch: char) -> Tone {
    match ch.to_ascii_lowercase() {
        's' | '1' => Tone::Acute,
        'f' | '2' => Tone::Grave,
        'r' | '3' => Tone::Hook,
        'x' | '4' => Tone::Tilde,
        'j' | '5' => Tone::Dot,
        _ => Tone::None,
    }
}

fn has_vowel(out: &Composed) -> bool {
    out.letters.iter().any(|letter| letter.is_vowel())
}

fn apply_tone(out: &mut Composed, raw: &[char], i: usize, tone: Tone, key: char, options: &EngineOptions) {
    if tone == Tone::None {
        out.letters.push(Letter::literal(key));
        return;
    }
    let Some(slot) = place::target(&out.letters, options.modern_tone) else {
        // No vowel to carry it (`s`, `str`): the key is a letter.
        out.letters.push(Letter::literal(key));
        return;
    };
    if out.tone == tone {
        // The same trigger again: the word reverts to the literal keystrokes.
        let drop = drop_for(raw, i, key, out.tone_at).or_else(|| previous_trigger(raw, i, key));
        escape_to_raw(out, raw, i, drop);
        return;
    }
    out.tone = tone;
    out.tone_at = i;
    // `Đặt dấu tự do` off: the tone stays on the vowel it hit right now.
    out.tone_slot = if options.free_tone { None } else { Some(slot) };
}

fn mark_vni_horn(out: &mut Composed, raw: &[char], i: usize, key: char) {
    // VNI horn applied after an adjacent `uo` cluster marks both letters.
    // Other sequences retain mark_last behavior (`u7a` must not fuse).
    let len = out.letters.len();
    if len >= 2
        && out.letters[len - 2].base() == 'u'
        && out.letters[len - 2].mark == Mark::None
        && out.letters[len - 1].base() == 'o'
        && out.letters[len - 1].mark == Mark::None
    {
        out.letters[len - 2].mark = Mark::Horn;
        out.letters[len - 1].mark = Mark::Horn;
        out.mark_at = i;
        return;
    }
    mark_last(out, raw, i, &['o', 'u'], Mark::Horn, key);
}

fn mark_last(out: &mut Composed, raw: &[char], i: usize, bases: &[char], mark: Mark, key: char) {
    if let Some(idx) = out
        .letters
        .iter()
        .rposition(|letter| bases.contains(&letter.base()) && letter.mark == Mark::None)
    {
        out.letters[idx].mark = mark;
        out.mark_at = i;
        return;
    }
    // No letter can take it: escape to the raw keystrokes, never a half
    // transform (`a^^` → `a^`, `await` → `await`).
    let drop = drop_for(raw, i, key, out.mark_at);
    escape_to_raw(out, raw, i, drop);
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
