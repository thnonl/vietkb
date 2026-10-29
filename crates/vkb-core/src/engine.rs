//! [`Engine`]: the public state machine plus the action diffing.
//!
//! Every key recomposes the word from the raw keystroke history and renders it
//! twice — the previous rendering is kept in `text`, so the engine can emit the
//! smallest action that turns one into the other:
//!
//! * the key just appends its own character → `Noop`, the app types it itself;
//! * the word gained one character that differs → `Commit` that character;
//! * the word was rewritten (tone moved, mark applied) → `Replace`;
//! * nothing visible changed → `Swallow`.
//!
//! `Noop` is the steady path and must not allocate; it is checked by
//! `tests/no_alloc.rs` with a counting allocator.

use smallvec::SmallVec;

use crate::charset::{push_encoded, TextBuf};
use crate::letters::{compose as compose_letter, Letter, Tone};
use crate::macros::MacroTable;
use crate::method::{self, Composed};
use crate::tone as place;
use crate::{spell, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine, MAX_WORD};

/// Windows virtual-key codes the engine reacts to even without a character.
pub const VK_BACK: u16 = 0x08;
pub const VK_TAB: u16 = 0x09;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_SPACE: u16 = 0x20;

/// Raw keystrokes the reference implementation treated as a wrong word at a
/// boundary, ported verbatim as a compat shim.
const COMPAT_RESTORE_RAW: &[&str] = &["of", "oj", "awf", "owf", "uwf", "aww", "oww", "uww"];

/// The word a boundary key just committed, kept only until the next key so a
/// Backspace can re-open it. One word, not a history.
struct CommittedWord {
    raw: SmallVec<[char; MAX_WORD]>,
    composed: Composed,
    text: TextBuf,
    literal_from: usize,
}

/// Default engine.
pub struct Engine {
    options: EngineOptions,
    raw: SmallVec<[char; MAX_WORD]>,
    composed: Composed,
    /// What the engine last put on screen, in charset units.
    text: TextBuf,
    /// Raw index from which the caller forced literal text (restore-word).
    literal_from: usize,
    macros: MacroTable,
    caps_pending: bool,
    /// The last character the engine produced, used by the raw-zone rules.
    last_emitted: Option<char>,
    /// A raw-zone opener was just typed; the next key decides whether the word
    /// that follows is typed literally (spec §A9).
    raw_pending: bool,
    /// The word the last separator committed, live for exactly one key:
    /// `Tha` + space + Backspace + `s` = `Thá`.
    reopen: Option<CommittedWord>,
}

impl Engine {
    pub fn new(options: EngineOptions) -> Self {
        Self {
            options,
            raw: SmallVec::new(),
            composed: Composed::default(),
            text: TextBuf::new(),
            literal_from: usize::MAX,
            macros: MacroTable::new(),
            caps_pending: false,
            last_emitted: None,
            raw_pending: false,
            reopen: None,
        }
    }

    pub fn options(&self) -> EngineOptions {
        self.options
    }

    /// Raw keystrokes of the current word, for tests and the CLI simulator.
    pub fn buffer(&self) -> &[char] {
        &self.raw
    }

    /// The text the engine believes is on screen for the current word.
    pub fn current_text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn macros(&self) -> &MacroTable {
        &self.macros
    }

    pub fn set_macros(&mut self, macros: MacroTable) {
        self.macros = macros;
        self.reset_word();
    }

    pub fn upsert_macro(&mut self, trigger: &str, expansion: &str) {
        self.macros.insert(trigger, expansion);
    }

    /// `Phục hồi từ gõ sai`: put the raw keystrokes back on screen and keep the
    /// rest of the word literal.
    pub fn restore_word(&mut self) -> EngineAction {
        if self.raw.is_empty() || self.composed.literal || self.literal_from == 0 {
            return EngineAction::Noop;
        }
        self.literal_from = 0;
        method::compose(&self.raw, &self.options, self.literal_from, &mut self.composed);
        let mut next = TextBuf::new();
        self.render_into(&self.composed.letters, self.composed.tone, self.composed.tone_slot, &mut next);
        let action = if next.as_slice() == self.text.as_slice() {
            EngineAction::Swallow
        } else {
            EngineAction::Replace {
                delete_units: self.text.len() as u32,
                insert: next.iter().collect(),
            }
        };
        self.text = next;
        action
    }

    fn reset_word(&mut self) {
        self.raw.clear();
        self.composed.clear();
        self.text.clear();
        self.literal_from = usize::MAX;
    }

    /// Compose `letters` with the word tone placed per the tone rules.
    ///
    /// `tone_slot` pins the tone to one letter (`free_tone = false`); `None`
    /// re-places it per A4 on every render.
    fn render_into(
        &self,
        letters: &[Letter],
        tone: Tone,
        tone_slot: Option<usize>,
        out: &mut TextBuf,
    ) {
        out.clear();
        let target =
            tone_slot.or_else(|| place::target(letters, self.options.modern_tone));
        for (idx, letter) in letters.iter().enumerate() {
            let letter_tone = if Some(idx) == target { tone } else { Tone::None };
            let composed =
                compose_letter(letter.ch, letter.mark, letter_tone).unwrap_or(letter.ch);
            push_encoded(composed, self.options.charset, out);
        }
    }

    fn diff(&self, next: &TextBuf, typed: Option<char>) -> EngineAction {
        let old = self.text.as_slice();
        let new = next.as_slice();
        if new == old {
            return EngineAction::Swallow;
        }
        if new.len() == old.len() + 1 && new[..old.len()] == *old {
            let appended = new[new.len() - 1];
            return match typed {
                Some(ch) if ch == appended => EngineAction::Noop,
                _ => EngineAction::Commit(appended.to_string()),
            };
        }
        EngineAction::Replace {
            delete_units: old.len() as u32,
            insert: new.iter().collect(),
        }
    }

    fn word_key(&mut self, ch: char) -> EngineAction {
        if self.raw_pending {
            // The opener was followed straight by a letter: this whole word is
            // raw, so freeze it from its first keystroke (spec §A9).
            self.raw_pending = false;
            if self.literal_from == usize::MAX {
                self.literal_from = self.raw.len();
            }
        }
        self.last_emitted = Some(ch);
        let committed = if self.raw.len() >= MAX_WORD {
            // Commit the capped word, then start the new word with this key.
            self.reset_word();
            true
        } else {
            false
        };
        let ch = if self.caps_pending && ch.is_ascii_alphabetic() {
            ch.to_ascii_uppercase()
        } else {
            ch
        };
        self.caps_pending = false;
        self.raw.push(ch);
        method::compose(&self.raw, &self.options, self.literal_from, &mut self.composed);
        let mut next = TextBuf::new();
        self.render_into(&self.composed.letters, self.composed.tone, self.composed.tone_slot, &mut next);
        let action = self.diff(&next, Some(ch));
        self.text = next;
        if committed {
            EngineAction::Replace {
                delete_units: 0,
                insert: self.text.iter().collect(),
            }
        } else {
            action
        }
    }

    /// One displayed character per press: the last composed *letter*, not the
    /// last keystroke (`tiếng` − `g` = `tiến`, and `ế` goes in one press even
    /// though `ee` + `s` typed it).
    ///
    /// The raw history stays honest instead of being recomposed from a shorter
    /// prefix: every keystroke that fed the deleted letter is dropped, found by
    /// replaying [`method::compose`] over the growing prefixes (see
    /// [`Engine::keystroke_owners`]). What is left is the keystroke sequence the
    /// user would have typed to produce the surviving text, so further typing
    /// composes on top of it. When the replay cannot reproduce the text on
    /// screen — a doubled-trigger escape (`aaa`, `ass`) rewrote the history —
    /// the surviving characters are frozen as literal keystrokes instead: the
    /// screen never moves, at the cost of that word no longer composing further.
    fn backspace(&mut self, reopening: Option<CommittedWord>) -> EngineAction {
        // Backspace removes the opener itself: the pending raw zone is dead.
        self.raw_pending = false;
        if self.raw.is_empty() {
            if let Some(word) = reopening {
                // The app deletes the boundary itself; the word comes back so
                // the next keystroke composes onto it (`Tha` + space + BS + `s`
                // = `Thá`). A second Backspace then deletes one character.
                self.caps_pending = false;
                self.raw = word.raw;
                self.composed = word.composed;
                self.text = word.text;
                self.literal_from = word.literal_from;
            }
            // Nothing of ours: let the app delete whatever came before.
            return EngineAction::Noop;
        }
        if self.composed.letters.is_empty() {
            return EngineAction::Noop;
        }

        let target = self.text_without_last_character();
        let kept = self.raw_without_letter(self.composed.letters.len() - 1);

        let mut literal_from = self.literal_from;
        if literal_from != usize::MAX && literal_from > kept.len() {
            literal_from = usize::MAX;
        }
        let mut composed = Composed::default();
        let mut next = TextBuf::new();
        method::compose(&kept, &self.options, literal_from, &mut composed);
        self.render_into(
            &composed.letters,
            composed.tone,
            composed.tone_slot,
            &mut next,
        );

        if next.as_slice() == target.as_slice() {
            self.raw = kept;
        } else {
            let mut frozen: SmallVec<[char; MAX_WORD]> = SmallVec::new();
            for &ch in &target {
                frozen.push(ch);
            }
            literal_from = 0;
            method::compose(&frozen, &self.options, literal_from, &mut composed);
            self.render_into(
                &composed.letters,
                composed.tone,
                composed.tone_slot,
                &mut next,
            );
            self.raw = frozen;
        }

        self.literal_from = literal_from;
        self.composed = composed;
        self.settle_deletion(next)
    }

    /// The text on screen without its last displayed character.
    ///
    /// One character, not one charset unit: `ế` is a single scalar in `unicode`
    /// and three in `unicode_nfd`, and both must lose the whole letter.
    fn text_without_last_character(&self) -> TextBuf {
        let letters = &self.composed.letters;
        let last = letters.len() - 1;
        let carrier = self
            .composed
            .tone_slot
            .or_else(|| place::target(letters, self.options.modern_tone));
        let last_tone = if carrier == Some(last) {
            self.composed.tone
        } else {
            Tone::None
        };
        let mut tail = TextBuf::new();
        self.render_into(&letters[last..], last_tone, Some(0), &mut tail);
        let keep = self.text.len().saturating_sub(tail.len());
        let mut target = TextBuf::new();
        target.extend_from_slice(&self.text[..keep]);
        target
    }

    /// The raw keystrokes left when every keystroke that fed letter `drop` goes.
    fn raw_without_letter(&self, drop: usize) -> SmallVec<[char; MAX_WORD]> {
        let owners = self.keystroke_owners();
        let dropped = drop as u8;
        let mut kept: SmallVec<[char; MAX_WORD]> = SmallVec::new();
        for (idx, &ch) in self.raw.iter().enumerate() {
            if owners.get(idx) != Some(&dropped) {
                kept.push(ch);
            }
        }
        kept
    }

    /// For each raw keystroke, the letter it produced or modified.
    ///
    /// Recomputed by replaying [`method::compose`] over the growing prefixes, so
    /// it always agrees with what the engine actually composed — there is no
    /// parallel bookkeeping that could drift from the state machine.
    fn keystroke_owners(&self) -> SmallVec<[u8; MAX_WORD]> {
        let mut owners: SmallVec<[u8; MAX_WORD]> = SmallVec::new();
        let mut prev = Composed::default();
        let mut cur = Composed::default();
        for end in 0..self.raw.len() {
            method::compose(&self.raw[..=end], &self.options, self.literal_from, &mut cur);
            owners.push(owner_of(&prev, &cur, self.options.modern_tone));
            std::mem::swap(&mut prev, &mut cur);
        }
        owners
    }

    /// Emit the smallest action that turns `self.text` into `next`, then adopt
    /// `next`. A deletion that changes nothing visible swallows the key instead
    /// of emitting a stray `Replace`.
    fn settle_deletion(&mut self, next: TextBuf) -> EngineAction {
        let old = self.text.as_slice();
        let new = next.as_slice();
        let action = if new == old {
            EngineAction::Swallow
        } else if new.len() < old.len() && old[..new.len()] == *new {
            let units = old.len() - new.len();
            if units == 1 {
                // One unit: the app's own backspace deletes exactly that.
                EngineAction::Noop
            } else {
                // One character, several units (`unicode_nfd`): delete them all.
                EngineAction::Replace {
                    delete_units: units as u32,
                    insert: String::new(),
                }
            }
        } else {
            EngineAction::Replace {
                delete_units: old.len() as u32,
                insert: new.iter().collect(),
            }
        };
        self.text = next;
        action
    }

    fn boundary_key(&mut self, ch: char) -> EngineAction {
        if self.options.auto_capitalize && matches!(ch, '.' | '!' | '?' | '\r' | '\n') {
            self.caps_pending = true;
        }
        // Raw zone (spec §A9): the opener only arms; the next key decides. An
        // ellipsis or a digit-dot never arms, and whitespace disarms it again.
        let previous = self.last_emitted;
        self.last_emitted = Some(ch);
        let opener = matches!(ch, '.' | '/' | ':' | '@' | '_' | '\\' | '#' | '=' | '?')
            && !(ch == '.' && previous == Some('.'))
            && !(ch == '.' && previous.is_some_and(|c| c.is_ascii_digit()));
        self.raw_pending = self.options.raw_zone && opener;
        if self.raw.is_empty() {
            return EngineAction::Noop;
        }
        let old_len = self.text.len();

        if self.options.macro_enabled {
            if let Some(expansion) = self.macros.lookup_chars(&self.raw).map(str::to_owned) {
                let mut insert = TextBuf::new();
                self.push_expansion(&mut insert, &expansion);
                push_encoded(ch, self.options.charset, &mut insert);
                self.reset_word();
                return EngineAction::Replace {
                    delete_units: old_len as u32,
                    insert: insert.iter().collect(),
                };
            }
        }

        if self.options.restore_wrong_word && self.should_restore() {
            let mut insert = TextBuf::new();
            for &raw_ch in &self.raw {
                push_encoded(raw_ch, self.options.charset, &mut insert);
            }
            push_encoded(ch, self.options.charset, &mut insert);
            self.reset_word();
            return EngineAction::Replace {
                delete_units: old_len as u32,
                insert: insert.iter().collect(),
            };
        }

        // The word on screen is already correct: let the boundary key through.
        //
        // A separator commits the word but leaves it reachable for exactly one
        // key: a Backspace that removes the separator re-opens it (`Tha` + space
        // + BS + `s` = `Thá`). Which keys are separators instead of hard resets
        // is EVKey's character class 1: `FUN_14003d480` appends a boundary entry
        // and keeps the word, while class 3 (`0x140039740`) zeroes the word. The
        // class table initialiser `FUN_140036b10` puts space, Tab and ASCII
        // punctuation in class 1, and Enter with the other control characters in
        // class 3 — so Enter ends the line and never re-opens.
        if is_reopening_separator(ch) {
            self.reopen = Some(CommittedWord {
                raw: std::mem::take(&mut self.raw),
                composed: std::mem::take(&mut self.composed),
                text: std::mem::take(&mut self.text),
                literal_from: self.literal_from,
            });
        }
        self.reset_word();
        EngineAction::Noop
    }

    fn push_expansion(&self, out: &mut TextBuf, expansion: &str) {
        let capitalize = self.options.macro_case_follow
            && self.raw.first().is_some_and(|ch| ch.is_ascii_uppercase());
        let mut pending_caps = capitalize;
        for ch in expansion.chars() {
            let ch = if pending_caps && ch.is_alphabetic() {
                pending_caps = false;
                ch.to_uppercase().next().unwrap_or(ch)
            } else {
                pending_caps = false;
                ch
            };
            push_encoded(ch, self.options.charset, out);
        }
    }

    fn should_restore(&self) -> bool {
        if !self.options.spell_check || self.composed.literal || self.literal_from != usize::MAX || self.raw.len() < 2 {
            return false;
        }
        if raw_matches(&self.raw, COMPAT_RESTORE_RAW) {
            return true;
        }
        !spell::is_valid_syllable(&self.composed.letters, self.options.jfwz_as_consonant)
    }
}

impl VietnameseEngine for Engine {
    fn reset(&mut self) {
        self.reset_word();
        // A caret move or focus change is a boundary the memory must not cross.
        self.reopen = None;
        self.caps_pending = false;
        self.last_emitted = None;
        self.raw_pending = false;
    }

    fn set_options(&mut self, options: EngineOptions) {
        if self.options.method != options.method || self.options.charset != options.charset {
            self.reset_word();
            self.reopen = None;
        }
        self.options = options;
    }

    fn set_macros(&mut self, macros: MacroTable) {
        Engine::set_macros(self, macros);
    }

    fn process_key(&mut self, key: KeyInput) -> EngineAction {
        // Key-up and non-text chords are none of our business.
        if !key.down {
            return EngineAction::Noop;
        }
        if key.modifiers.ctrl || key.modifiers.alt || key.modifiers.win {
            return EngineAction::Noop;
        }
        // The just-committed word is reachable from one key only: whatever this
        // key is (Backspace, a letter, a navigation key), the memory is spent.
        let reopening = self.reopen.take();
        match key.vk {
            VK_BACK => return self.backspace(reopening),
            VK_RETURN => return self.boundary_key('\r'),
            VK_TAB => {
                let ch = key.ch.unwrap_or('\t');
                return self.boundary_key(ch);
            }
            // Escape is class 3 in EVKey: a hard reset, never a reopenable word.
            VK_ESCAPE => return self.boundary_key(key.ch.unwrap_or('\u{1b}')),
            _ => {}
        }
        let ch = match key.ch {
            Some(ch) => ch,
            None => return EngineAction::Noop,
        };
        match ch {
            '\u{8}' | '\u{7f}' => self.backspace(reopening),
            '\r' | '\n' => self.boundary_key('\r'),
            _ if ch.is_control() => self.boundary_key(ch),
            _ if is_word_key(self.options.method, ch) => self.word_key(ch),
            _ if ch.is_ascii_digit() => {
                // A digit joins the word only when it starts a numeric run or
                // the word is already e-mail shaped; otherwise it ends it.
                if method::digit_joins_word(&self.raw) {
                    self.word_key(ch)
                } else {
                    self.boundary_key(ch)
                }
            }
            _ => self.boundary_key(ch),
        }
    }

    fn boundary(&mut self) {
        self.reset_word();
        self.reopen = None;
    }
}
/// EVKey character class 1: keys that end the word but leave it re-openable
/// after a Backspace. Taken from the class table initialiser `FUN_140036b10`
/// (`DAT_1400a28e8` plus the single Tab entry): space, Tab, and every ASCII
/// punctuation character. Enter and the other control characters are class 3
/// (`0x140039740`) and hard-reset the word, so they never re-open.
fn is_reopening_separator(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch.is_ascii_punctuation()
}

fn is_word_key(method: InputMethod, ch: char) -> bool {

    if ch.is_ascii_alphabetic() {
        return true;
    }
    match method {
        InputMethod::Telex | InputMethod::Legacy => false,
        InputMethod::Vni => ch.is_ascii_digit(),
    }
}

/// Which letter the keystroke that turned `prev` into `cur` belongs to.
///
/// A keystroke either created a letter, or transformed one already on the
/// buffer: a mark lands on the letter whose mark changed, a tone on the letter
/// that carries it right now (`ế` owns the `s` of `tiếng`; the `g` does not).
fn owner_of(prev: &Composed, cur: &Composed, modern_tone: bool) -> u8 {
    let Some(last) = cur.letters.len().checked_sub(1) else {
        return 0;
    };
    if cur.letters.len() == prev.letters.len() + 1 {
        return last as u8;
    }
    if let Some(idx) = cur
        .letters
        .iter()
        .zip(prev.letters.iter())
        .position(|(now, before)| now != before)
    {
        return idx as u8;
    }
    if cur.tone != prev.tone {
        if let Some(slot) = cur.tone_slot {
            return slot as u8;
        }
        if let Some(idx) = place::target(&cur.letters, modern_tone) {
            return idx as u8;
        }
    }
    last as u8
}

/// Case-insensitive comparison of raw keystrokes against a literal shape.
fn raw_matches(raw: &[char], shapes: &[&str]) -> bool {
    shapes.iter().any(|shape| {
        let mut chars = shape.chars();
        let mut count = 0usize;
        for &ch in raw {
            match chars.next() {
                Some(expected) if expected == ch.to_ascii_lowercase() => count += 1,
                _ => return false,
            }
        }
        chars.next().is_none() && count == raw.len()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::letters::Mark;

    fn typer() -> Engine {
        Engine::new(EngineOptions::default())
    }

    #[test]
    fn autocompose_is_a_noop_for_plain_append() {
        let mut engine = typer();
        assert_eq!(engine.process_key(KeyInput::char('t')), EngineAction::Noop);
        assert_eq!(engine.process_key(KeyInput::char('a')), EngineAction::Noop);
    }

    #[test]
    fn mark_rewrites_the_word() {
        let mut engine = typer();
        engine.process_key(KeyInput::char('a'));
        assert_eq!(
            engine.process_key(KeyInput::char('a')),
            EngineAction::Replace {
                delete_units: 1,
                insert: "â".to_string()
            }
        );
    }

    #[test]
    fn key_up_and_chords_do_nothing() {
        let mut engine = typer();
        let mut key = KeyInput::char('a');
        key.down = false;
        assert_eq!(engine.process_key(key), EngineAction::Noop);
        let mut key = KeyInput::char('s');
        key.modifiers.ctrl = true;
        assert_eq!(engine.process_key(key), EngineAction::Noop);
    }

    #[test]
    fn restore_word_gives_the_raw_keystrokes_back() {
        let mut engine = typer();
        for ch in "tieengs".chars() {
            engine.process_key(KeyInput::char(ch));
        }
        assert_eq!(engine.current_text(), "tiếng");
        assert_eq!(
            engine.restore_word(),
            EngineAction::Replace {
                delete_units: 5,
                insert: "tieengs".to_string()
            }
        );
        assert_eq!(engine.current_text(), "tieengs");
    }

    #[test]
    fn tone_moves_when_the_syllable_grows() {
        let mut engine = typer();
        for ch in "hoafn".chars() {
            engine.process_key(KeyInput::char(ch));
        }
        assert_eq!(engine.current_text(), "hoàn");
    }

    #[test]
    fn composed_letters_carry_marks() {
        let mut engine = typer();
        for ch in "dduwowngf".chars() {
            engine.process_key(KeyInput::char(ch));
        }
        let letters = &engine.composed.letters;
        assert_eq!(letters[0].mark, Mark::Bar);
        assert_eq!(letters[1].mark, Mark::Horn);
        assert_eq!(letters[2].mark, Mark::Horn);
        assert_eq!(engine.composed.tone, Tone::Grave);
    }
}

