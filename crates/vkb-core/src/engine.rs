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

    fn diff(&self, next: &TextBuf, typed: Option<char>, allow_single_delete: bool) -> EngineAction {
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
        if allow_single_delete && new.len() + 1 == old.len() && old[..new.len()] == *new {
            // The app's own backspace removes exactly the last character.
            return EngineAction::Noop;
        }
        EngineAction::Replace {
            delete_units: old.len() as u32,
            insert: new.iter().collect(),
        }
    }

    fn word_key(&mut self, ch: char) -> EngineAction {
        if self.raw.len() >= MAX_WORD {
            // Beyond the cap: commit what is on screen and start over.
            self.reset_word();
        }
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
        let action = self.diff(&next, Some(ch), false);
        self.text = next;
        action
    }

    fn backspace(&mut self) -> EngineAction {
        if self.raw.is_empty() {
            // Nothing of ours: let the app delete whatever came before.
            return EngineAction::Noop;
        }
        self.raw.pop();
        if self.raw.is_empty() || self.literal_from > self.raw.len() {
            self.literal_from = usize::MAX;
        }
        method::compose(&self.raw, &self.options, self.literal_from, &mut self.composed);
        let mut next = TextBuf::new();
        self.render_into(&self.composed.letters, self.composed.tone, self.composed.tone_slot, &mut next);
        let action = self.diff(&next, None, true);
        self.text = next;
        action
    }

    fn boundary_key(&mut self, ch: char) -> EngineAction {
        if self.options.auto_capitalize && matches!(ch, '.' | '!' | '?' | '\r' | '\n') {
            self.caps_pending = true;
        }
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
        self.reset_word();
        EngineAction::Noop
    }

    fn push_expansion(&self, out: &mut TextBuf, expansion: &str) {
        let capitalize = self
            .raw
            .first()
            .is_some_and(|ch| ch.is_ascii_uppercase());
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
        if self.composed.literal || self.literal_from != usize::MAX || self.raw.len() < 2 {
            return false;
        }
        if raw_matches(&self.raw, COMPAT_RESTORE_RAW) {
            return true;
        }
        self.options.spell_check
            && !spell::is_valid_syllable(&self.composed.letters, self.options.jfwz_as_consonant)
    }
}

impl VietnameseEngine for Engine {
    fn reset(&mut self) {
        self.reset_word();
        self.caps_pending = false;
    }

    fn set_options(&mut self, options: EngineOptions) {
        if self.options.method != options.method || self.options.charset != options.charset {
            self.reset_word();
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
        match key.vk {
            VK_BACK => return self.backspace(),
            VK_RETURN => return self.boundary_key('\r'),
            VK_TAB | VK_ESCAPE => {
                let ch = key.ch.unwrap_or('\t');
                return self.boundary_key(ch);
            }
            _ => {}
        }
        let ch = match key.ch {
            Some(ch) => ch,
            None => return EngineAction::Noop,
        };
        match ch {
            '\u{8}' | '\u{7f}' => self.backspace(),
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
    }
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
