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
    /// The next tone key must not be read as a repeated trigger: the tone on
    /// screen sits where A4 would no longer place it (the last character was
    /// deleted, §A7 step 4). Set by the engine for the first key after such a
    /// Backspace; consumed by the first same-tone press.
    pub(crate) tone_stale: bool,
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
        self.tone_stale = false;
    }
}

/// Replay `raw` and return the composed word.
///
/// `literal_from` is the index from which the caller already forced raw text
/// (the "phục hồi từ" hotkey); `usize::MAX` disables it.
pub fn compose(raw: &[char], options: &EngineOptions, literal_from: usize, out: &mut Composed) {
    compose_word(raw, options, literal_from, false, usize::MAX, out);
}

/// [`compose`] with the engine's per-word notes: `tone_stale` marks a tone that
/// the screen shows where A4 would no longer place it (§A7 step 4). `edit_len`
/// is how many leading keystrokes survived the latest Backspace (`usize::MAX` =
/// none): marks those keystrokes made stay when a later key only appends, while
/// marks typed after that Backspace get the same verdict as fresh typing
/// (`D-BS-LIT-01`).
pub(crate) fn compose_word(
    raw: &[char],
    options: &EngineOptions,
    literal_from: usize,
    tone_stale: bool,
    edit_len: usize,
    out: &mut Composed,
) {
    out.clear();
    out.tone_stale = tone_stale;

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

        // Marks the surviving keystrokes (before the edit) already put on screen.
        let protected = edit_len != usize::MAX
            && ((out.mark_at != usize::MAX && out.mark_at < edit_len)
                || (out.tone_at != usize::MAX && out.tone_at < edit_len));
        // Snapshot for the post-deletion refusal below: a doubled key that would
        // leave an invalid syllable is typed as text instead of applied.
        let before = (options.spell_check && protected && !out.literal && !out.escaped).then(|| {
            (
                out.letters.clone(),
                out.tone,
                out.tone_at,
                out.mark_at,
                out.tone_slot,
            )
        });

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
        if options.auto_horn_uo && !out.literal && out.tone == Tone::None && i > 0 && matches!(ch.to_ascii_lowercase(), 'i' | 'u' | 'c' | 'h' | 'm' | 'n' | 'p' | 't') {
            if let Some(o) = out.letters.iter().rposition(|letter| letter.base() == 'o' && letter.mark == Mark::None) {
                if o > 0 && out.letters[o - 1].base() == 'u' && out.letters[o - 1].mark == Mark::Horn {
                    out.letters[o].mark = Mark::Horn;
                    out.mark_at = i;
                }
            }
        }

        if options.spell_check && !out.literal && !out.escaped {
            let transformed = out.tone_at == i || out.mark_at == i;
            let doubled = i > 0 && raw[i - 1].eq_ignore_ascii_case(&ch);
            let keeps_the_marks = protected
                && !transformed
                && {
                    // A plain letter appends: a vowel, a new coda after a vowel,
                    // or a doubled coda. A third consonant after an existing coda
                    // is a different word (`tiếnt`), so it shows the raw keys.
                    let vowel = Letter::literal(ch).is_vowel();
                    let previous = out
                        .letters
                        .len()
                        .checked_sub(2)
                        .map(|idx| out.letters[idx]);
                    vowel
                        || previous.is_some_and(|letter| {
                            letter.is_vowel() || letter.base() == ch.to_ascii_lowercase()
                        })
                };
            let invalid =
                !crate::spell::is_valid_syllable(&out.letters, options.jfwz_as_consonant)
                    || (matches!(options.method, InputMethod::Telex | InputMethod::Legacy)
                        && crate::spell::english_stop_cluster(&raw[..=i], &out.letters));
            if invalid && !keeps_the_marks {
                if transformed && doubled && protected {
                    // §A1 rule 5 inside a word that survived a deletion: the
                    // doubled key is literal text, the marks already on screen
                    // stay (`Nguyễ` + `ee` = `Nguyễee`).
                    if let Some((letters, tone, tone_at, mark_at, tone_slot)) = before {
                        out.letters = letters;
                        out.tone = tone;
                        out.tone_at = tone_at;
                        out.mark_at = mark_at;
                        out.tone_slot = tone_slot;
                        out.literal = true;
                    }
                    out.letters.push(Letter::literal(ch));
                } else {
                    literalize(out, raw, i);
                }
            }
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
        // `0` removes the tone only (§A2); the circumflex/breve/horn/bar are
        // independent of it (§A5), so `a60` is `â`, not `a` (D-TN-03).
        // With no tone and no mark there is nothing to remove: the digit is
        // literal text (§A2 rule 1), `a0` stays `a0`.
        '0' if out.tone != Tone::None
            || out.letters.iter().any(|letter| letter.mark != Mark::None) =>
        {
            remove_tone(out)
        }
        '0' => literal_trigger(out, digit),
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

/// The raw index to drop when a transform is escaped by repeating its trigger.
///
/// The escape is the *immediately repeated* press (§A1 escape table, §A2 "same
/// digit repeated immediately after its own transform"): a trigger typed after
/// other keys is a new transform, not an escape (`baajtj` keeps `bật`).
fn drop_for(raw: &[char], i: usize, key: char, at: usize) -> Option<usize> {
    if at != usize::MAX && at + 1 == i && raw[at].eq_ignore_ascii_case(&key) {
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
        // §A1: `z` removes the tone; with no tone present it is a plain letter
        // (`az` stays `az`). D-TN-02.
        'z' if out.tone != Tone::None => remove_tone(out),
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
        // The same *letter key* pressed again — right after the press that made
        // the mark — escapes the whole word to its literal keystrokes. A letter
        // typed later, after a tone key or a coda, is a plain letter and appends
        // (`D-TN-08`: `Nguyễ` + `e` is `Nguyễe`, not `Nguyexe`).
        if out.mark_at != usize::MAX && out.mark_at + 1 == i {
            let drop = drop_for(raw, i, base, out.mark_at);
            escape_to_raw(out, raw, i, drop);
            return;
        }
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

/// Drop the word's tone and nothing else: every letter keeps its mark.
fn remove_tone(out: &mut Composed) {
    out.tone = Tone::None;
    out.tone_at = usize::MAX;
    out.tone_slot = None;
}

/// A trigger that cannot apply (§A1 rule 5 / §A2 rule 1) is text: it is
/// appended as the literal key and the word on screen stays as it is. The rest
/// of the word stays literal too, so later keys cannot rewrite what is shown.
fn literal_trigger(out: &mut Composed, key: char) {
    out.letters.push(Letter::literal(key));
    out.escaped = true;
}

/// A9 coda check: `c ch p t` take only sắc and nặng, so huyền/hỏi/ngã on a word
/// that ends in one of them would spell something that cannot exist.
fn tone_illegal_before_coda(letters: &[Letter], tone: Tone) -> bool {
    if !matches!(tone, Tone::Grave | Tone::Hook | Tone::Tilde) {
        return false;
    }
    let n = letters.len();
    let Some(last) = letters.last() else {
        return false;
    };
    match last.base() {
        'c' | 'p' | 't' => true,
        'h' => n >= 2 && letters[n - 2].base() == 'c',
        _ => false,
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
        // The tone is already in force. After a Backspace the display can be
        // stale (`toán` BS shows `toá`, the deleted coda no longer forces T3):
        // §A7 step 4 says the next key re-runs A4, so re-place instead of
        // escaping.
        if out.tone_stale {
            out.tone_stale = false;
            out.tone_slot = None;
            return;
        }
        // The word reverts to the literal keystrokes only when the trigger is
        // *immediately repeated* (§A1 escape table, §A2 "same digit repeated
        // immediately after its own transform"): `ass` escapes, but a tone key
        // typed after other keys (`baajtj`) leaves the word alone.
        if let Some(drop) = drop_for(raw, i, key, out.tone_at) {
            escape_to_raw(out, raw, i, Some(drop));
            return;
        }
        return;
    }
    // §A1 rule 5 + A9: a tone that cannot stand before the coda is not applied;
    // the key is literal text and the word stays as it is (D-TN-04: `bậtf`).
    if options.spell_check && tone_illegal_before_coda(&out.letters, tone) {
        literal_trigger(out, key);
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
        // C1 `qu`: its `u` belongs to the onset, so 7 horns only the `o`.
        // Use raw history to distinguish this from a true `uo` vowel cluster.
        let qu_onset = i >= 3
            && raw[i - 3].eq_ignore_ascii_case(&'q')
            && raw[i - 2].eq_ignore_ascii_case(&'u');
        let u = len - 2;
        if qu_onset {
            out.letters[len - 1].mark = Mark::Horn;
        } else {
            out.letters[u].mark = Mark::Horn;
            out.letters[len - 1].mark = Mark::Horn;
        }
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
    // The same digit pressed again right after its own transform escapes to the
    // raw keystrokes, never a half transform (`a66` → `a6`).
    let drop = drop_for(raw, i, key, out.mark_at);
    if drop.is_some() {
        escape_to_raw(out, raw, i, drop);
        return;
    }
    // Otherwise the word offers no target: the digit is literal text and the
    // word stays composed (§A2 rule 1; D-TN-05: `ba65t6` → `bật6`).
    literal_trigger(out, key);
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
