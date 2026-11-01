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
use crate::lexicon::{self, Left};

use crate::{spell, AppKind, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine, MAX_WORD};

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

/// The word before the one being decided (D10 left context), kept as numbers:
/// no text, no allocation.
#[derive(Clone, Copy)]
struct PrevWord {
    kind: PrevKind,
    /// Corpus count of the Vietnamese syllable (`PrevKind::Vietnamese`).
    count: u32,
    /// FNV state after `key + " "`, so the pair hash is one `fnv_extend` away.
    hash: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrevKind {
    /// Start of line or sentence, or the context was cut.
    None,
    English,
    Vietnamese,
    /// A word we know nothing about (names, numbers, rare syllables).
    Other,
    /// Right after a comma, semicolon or colon.
    Clause,
    /// Right after a code operator or bracket.
    Code,
    /// The word was an English function word (`the`, `my`, `with`).
    Function,
}

impl PrevWord {
    const NONE: PrevWord = PrevWord { kind: PrevKind::None, count: 0, hash: 0 };
}

/// A Vietnamese word kept over an English one at the space after it, kept for
/// exactly one more word: it may be rewritten once if the next word is clearly
/// English (amendment 4).
struct Back {
    /// The Vietnamese word on screen.
    shown: TextBuf,
    /// The English reading (the typed keys).
    alternate: TextBuf,
    /// The typed keys, lowercase.
    typed: [u8; MAX_WORD],
    typed_len: usize,
}

/// What the break that just ended a word left behind, for the one-word-back check.
struct Scratch {
    /// Units the word occupied on screen before the break.
    old_len: usize,
    old_text: TextBuf,
    /// The word as it stands after the break (restored or composed).
    final_text: TextBuf,
    right: PrevWord,
    /// The typed keys of the word, lowercase ASCII letters (empty otherwise).
    right_raw: [u8; MAX_WORD],
    right_len: usize,
}

/// The last automatic correction, kept until the next key so the restore hotkey
/// can take it back (and put it again): the text now on screen and its other
/// reading, both with the break key that ended the word, if any.
struct Correction {
    shown: TextBuf,
    alternate: TextBuf,
    /// Made mid-word (no break key yet): toggling also moves the engine state.
    mid_word: bool,
    /// `shown` is the typed keys (`alternate` is the Vietnamese word).
    showing_raw: bool,
}

/// A word followed by `.`, `-` or `:` that may turn out to be code (`foo.bar`,
/// `a->b`, `std::io`): decided by the next key (a letter confirms).
#[derive(Clone)]
struct PendingCode {
    raw: SmallVec<[char; MAX_WORD]>,
    /// Charset units the word occupies on screen.
    units: usize,
    /// The word on screen is composed and must go back to `raw` on confirm.
    differs: bool,
    chain: SmallVec<[char; 2]>,
    /// `.` only counts after a word that is not a common Vietnamese syllable.
    gate_ok: bool,
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
    /// The opener that just ended a word armed the raw zone: the next key decides
    /// whether the word that follows is typed literally (spec §A9 amendment).
    raw_pending: bool,
    /// The tone on screen is not where A4 would place it: set by a Backspace that
    /// kept a stale tone slot, read by the very next key (spec §A7 step 4).
    tone_stale: bool,
    /// How many leading keystrokes survived the *latest* Backspace (`usize::MAX`
    /// = the word was never edited): every Backspace resets it to the kept
    /// length, so keystrokes that survived any Backspace are protected. A later
    /// key that only appends keeps the marks those keystrokes made instead of
    /// reverting the word (`D-TN-07`/`D-TN-08`); keys typed after the edit get
    /// the verdict fresh typing would give.
    edit_len: usize,
    /// The word so far is only leading digits (plain text, not in `raw`): a
    /// following opener still arms the raw zone (`2/3`).
    digit_prefix: bool,
    /// The word the last separator committed, live for exactly one key:
    /// `Tha` + space + Backspace + `s` = `Thá`.
    reopen: Option<CommittedWord>,
    /// Vietnamese is off, but gõ tắt must still expand (`macro_when_disabled`).
    macro_only: bool,
    /// D10 left context: the word the last plain space ended.
    prev_word: PrevWord,
    /// D10 code shapes waiting for the next key.
    pending_code: Option<PendingCode>,
    /// D10 undo: the last automatic correction.
    correction: Option<Correction>,
    /// The user took the correction of the word being typed back: leave it alone.
    auto_suppressed: bool,
    /// The word started right after `/`, `~` or `\\` inside a path token.
    after_path_sep: bool,
    /// The current space-delimited token started as a path (`/usr`, `~/`, `C:\\`, `./`).
    path_token: bool,
    /// Separators seen in the current path token (a lone `/x` is not yet a path).
    path_segments: u8,
    /// The path token began at the start of a space-delimited token.
    path_from_start: bool,
    /// The word is the part after an inner-capital split: capitals never act as keys here.
    split_token: bool,
    /// A close call at the last space, valid until the next word ends.
    back: Option<Back>,
    scratch: Option<Scratch>,
    /// A one-word-back candidate is waiting: record what this break leaves.
    want_scratch: bool,
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
            tone_stale: false,
            edit_len: usize::MAX,
            digit_prefix: false,
            reopen: None,
            macro_only: false,
            prev_word: PrevWord::NONE,
            pending_code: None,
            correction: None,
            auto_suppressed: false,
            after_path_sep: false,
            path_token: false,
            path_segments: 0,
            path_from_start: false,
            split_token: false,
            back: None,
            scratch: None,
            want_scratch: false,
        }
    }

    /// Capacities of every growable buffer the engine owns, and whether any of
    /// them has spilled to the heap. For the leak soak test (`tests/tech_mixed_leak.rs`):
    /// after any amount of typing the numbers must equal what they were after
    /// the first pass.
    #[doc(hidden)]
    pub fn buffer_footprint(&self) -> ([usize; 10], bool) {
        let reopen = self.reopen.as_ref();
        let correction = self.correction.as_ref();
        let pending = self.pending_code.as_ref();
        let caps = [
            self.raw.capacity(),
            self.text.capacity(),
            self.composed.letters.capacity(),
            reopen.map_or(0, |r| r.raw.capacity() + r.text.capacity()),
            correction.map_or(0, |c| c.shown.capacity() + c.alternate.capacity()),
            pending.map_or(0, |p| p.raw.capacity() + p.chain.capacity()),
            self.macros.len(),
            std::mem::size_of::<Self>(),
            self.back.as_ref().map_or(0, |b| b.shown.capacity() + b.alternate.capacity()),
            self.scratch.as_ref().map_or(0, |s| s.old_text.capacity() + s.final_text.capacity()),
        ];
        let spilled = self.raw.spilled()
            || self.text.spilled()
            || self.composed.letters.spilled()
            || reopen.is_some_and(|r| r.raw.spilled() || r.text.spilled())
            || correction.is_some_and(|c| c.shown.spilled() || c.alternate.spilled())
            || pending.is_some_and(|p| p.raw.spilled() || p.chain.spilled())
            || self.back.as_ref().is_some_and(|b| b.shown.spilled() || b.alternate.spilled())
            || self.scratch.as_ref().is_some_and(|s| s.old_text.spilled() || s.final_text.spilled());
        (caps, spilled)
    }

    pub fn options(&self) -> EngineOptions {
        self.options
    }

    /// Raw keystrokes of the current word, for tests and the CLI simulator.
    pub fn buffer(&self) -> &[char] {
        &self.raw
    }

    /// The text on screen for the current word, in charset units, without
    /// allocating. A backend snapshots it before a key to learn which part of a
    /// `Replace` the app already shows.
    pub fn on_screen(&self) -> &[char] {
        &self.text
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

    /// While on, the engine only expands macros: every other key passes through
    /// and nothing is composed, so Vietnamese stays off.
    pub fn set_macro_only(&mut self, on: bool) {
        self.macro_only = on;
        self.reset_word();
        // The armed raw zone never survives a mode switch (spec §A9 §7): the
        // next word is composed by the new mode, not frozen by the old opener.
        self.raw_pending = false;
        self.pending_code = None;
        self.prev_word = PrevWord::NONE;
    }

    fn macro_only_key(&mut self, key: KeyInput) -> EngineAction {
        if key.vk == VK_BACK {
            self.raw.pop();
            return EngineAction::Noop;
        }
        let Some(ch) = key.ch else {
            return EngineAction::Noop;
        };
        let is_boundary = matches!(ch, ' ' | '\t' | '\r' | '\n') || is_reopening_separator(ch);
        if !is_boundary {
            self.raw.push(ch);
            return EngineAction::Noop;
        }
        // `raw` holds exactly the characters the app already typed, so deleting
        // that many units is safe even in Unicode NFD.
        let delete_units = self.raw.len() as u32;
        let expansion = if self.options.macro_enabled {
            self.macros.lookup_chars(&self.raw).map(str::to_owned)
        } else {
            None
        };
        self.raw.clear();
        match expansion {
            Some(text) => {
                let mut insert = TextBuf::new();
                self.push_expansion(&mut insert, &text);
                push_encoded(ch, self.options.charset, &mut insert);
                EngineAction::Replace {
                    delete_units,
                    insert: insert.iter().collect(),
                }
            }
            None => EngineAction::Noop,
        }
    }

    pub fn upsert_macro(&mut self, trigger: &str, expansion: &str) {
        self.macros.insert(trigger, expansion);
    }

    /// `Phục hồi từ gõ sai`: put the raw keystrokes back on screen and keep the
    /// rest of the word literal.
    pub fn restore_word(&mut self) -> EngineAction {
        self.back = None;
        if let Some(correction) = self.correction.take() {
            return self.toggle_correction(correction);
        }
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

    /// Take an automatic correction back (or put it again).
    fn toggle_correction(&mut self, correction: Correction) -> EngineAction {
        let action = EngineAction::Replace {
            delete_units: correction.shown.len() as u32,
            insert: correction.alternate.iter().collect(),
        };
        if correction.mid_word {
            if correction.showing_raw {
                // Back to the Vietnamese reading; auto-restore stays out of this word.
                self.literal_from = usize::MAX;
                self.auto_suppressed = true;
            } else {
                self.literal_from = 0;
            }
            method::compose(&self.raw, &self.options, self.literal_from, &mut self.composed);
            self.text = correction.alternate.clone();
        }
        self.correction = Some(Correction {
            shown: correction.alternate,
            alternate: correction.shown,
            mid_word: correction.mid_word,
            showing_raw: !correction.showing_raw,
        });
        action
    }

    fn reset_word(&mut self) {
        self.auto_suppressed = false;
        self.split_token = false;
        self.raw.clear();
        self.composed.clear();
        self.text.clear();
        self.literal_from = usize::MAX;
        self.tone_stale = false;
        self.edit_len = usize::MAX;
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
        self.digit_prefix = false;
        if self.raw_pending {
            // The opener was followed straight by a letter: this whole word is
            // raw, so freeze it from its first keystroke (spec §A9).
            self.raw_pending = false;
            if self.literal_from == usize::MAX {
                self.literal_from = self.raw.len();
            }
        }
        self.last_emitted = Some(ch);
        // Owner rule: no Vietnamese word starts with f, j or z: such a word is
        // English or code and is typed exactly as the keys, from the first key on,
        // in every app. `w` is not in the rule (Telex `w` is ư): it follows the
        // normal rules, where doubt means Vietnamese.
        if self.options.auto_restore_english
            && self.raw.is_empty()
            && self.literal_from == usize::MAX
            && matches!(ch.to_ascii_lowercase(), 'f' | 'j' | 'z')
        {
            self.literal_from = 0;
        }
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
        // D10 code shape: a capital inside a word (`getValue`, `HttpClient`,
        // `iPhone`) is not Vietnamese typing; the whole word stops composing.
        if self.options.auto_restore_english
            && ch.is_ascii_uppercase()
            && self.literal_from == usize::MAX
            && self.raw.last().is_some_and(|last| last.is_ascii_lowercase())
        {
            if self.split_token {
                // Already after a split: capitals are letters here, never keys.
                self.literal_from = 0;
            } else if self.capital_is_a_key(ch) && self.prefix_could_be_vietnamese() {
                // `NguyeenX`, `dD`: Shift+X is the tone key, a doubled `D` is `đ`, as in 0.2.7.
            } else if self.inner_capital_splits() {
                // `VieetjNam`: the part before is a Vietnamese word of its own;
                // the capital starts the next one.
                self.reset_word();
                self.split_token = true;
            } else {
                // `getValue`, `iOS`, `macOS`, `eBay`: the word stays as typed.
                self.literal_from = 0;
            }
        }
        self.raw.push(ch);
        // §A7 step 4: the first key after a deletion that left a stale tone
        // display re-runs A4 instead of being read as a repeated tone trigger.
        let tone_stale = std::mem::take(&mut self.tone_stale);
        method::compose_word(
            &self.raw,
            &self.options,
            self.literal_from,
            tone_stale,
            self.edit_len,
            &mut self.composed,
        );
        let mut mid_word_restore = false;
        if self.escaped_english() || self.listed_word_with_spell_check_off() {
            // `pass`, `error`: the doubled tone key was swallowed as an undo and
            // the keys are an English word: put them back now.
            mid_word_restore = true;
            self.literal_from = 0;
            method::compose(&self.raw, &self.options, self.literal_from, &mut self.composed);
        }
        let mut next = TextBuf::new();
        self.render_into(&self.composed.letters, self.composed.tone, self.composed.tone_slot, &mut next);
        let action = self.diff(&next, Some(ch));
        if mid_word_restore {
            // The Vietnamese reading is what the engine showed before this fix.
            let mut vietnamese = TextBuf::new();
            let saved = self.literal_from;
            self.literal_from = usize::MAX;
            method::compose(&self.raw, &self.options, self.literal_from, &mut self.composed);
            self.render_into(&self.composed.letters, self.composed.tone, self.composed.tone_slot, &mut vietnamese);
            self.literal_from = saved;
            method::compose(&self.raw, &self.options, self.literal_from, &mut self.composed);
            self.correction = Some(Correction {
                shown: next.clone(),
                alternate: vietnamese,
                mid_word: true,
                showing_raw: true,
            });
        }
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
    /// the surviving text is typed fresh when it is plain letters that compose
    /// to themselves, and is otherwise frozen as literal keystrokes: the screen
    /// does not move, at the cost of that word no longer composing further.
    ///
    /// `D-BS-LIT-01` (re-judge): a word the spell check or a typo left as raw
    /// text is judged again from the kept keystrokes, so the screen may change
    /// by more than one character (`gifk` + BS = `gì`, `d9e6i` + BS = `đê`).
    /// The bound: the re-judged screen is exactly the fresh typing of the kept
    /// keystrokes and is no longer than the screen it replaces; otherwise the
    /// frozen path above applies.
    fn backspace(&mut self, reopening: Option<CommittedWord>) -> EngineAction {
        // Backspace removes the opener itself: the pending raw zone is dead.
        self.raw_pending = false;
        self.digit_prefix = false;
        self.tone_stale = false;
        if self.raw.is_empty() {
            // The text left of the caret changed under us: the context is gone.
            self.prev_word = PrevWord::NONE;
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
        // The letter that carries the tone on screen right now (before the press).
        let carrier = self
            .composed
            .tone_slot
            .or_else(|| place::target(&self.composed.letters, self.options.modern_tone));
        let old_valid =
            spell::is_valid_syllable(&self.composed.letters, self.options.jfwz_as_consonant);
        // `D-BS-LIT-01`: a word the spell check (or a typo) turned into raw text
        // is judged again once a keystroke is gone: `gifk` + BS is `gì`.
        let reevaluate = self.literal_from == usize::MAX
            && !self.composed.escaped
            && (self.composed.literal || !old_valid);

        let was_literal = self.composed.literal;
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

        // The tone on screen can sit on a letter A4 would no longer choose: the
        // deleted character may have been the coda that forced T3 (`toán` BS →
        // `toá`, while `toa` alone takes the tone on `o`). §A7 step 4 keeps the
        // display unchanged, so pin the tone to the letter that shows it; the
        // next key re-runs A4 from raw anyway, and the word keeps composing. A
        // word that was not a syllable to begin with (`hoafk`) has no display
        // worth keeping: it re-runs A4 now.
        if old_valid && next.as_slice() != target.as_slice() && composed.tone != Tone::None {
            if let Some(slot) = carrier.filter(|&slot| slot < composed.letters.len()) {
                let mut pinned = TextBuf::new();
                self.render_into(&composed.letters, composed.tone, Some(slot), &mut pinned);
                if pinned.as_slice() == target.as_slice() {
                    composed.tone_slot = Some(slot);
                    next = pinned;
                    self.tone_stale = true;
                }
            }
        }

        // `D-BS-LIT-01`: `next` is the fresh typing of `kept`; the re-judge only
        // stands when it does not grow the word.
        let rejudged = reevaluate && next.len() <= self.text.len();
        if next.as_slice() == target.as_slice() || rejudged {
            self.raw = kept;
        } else {
            // The kept keystrokes cannot reproduce the screen (a doubled-trigger
            // escape rewrote the history). If the surviving text is plain
            // letters that type themselves, it is just that word, typed fresh.
            // Otherwise the characters are frozen as literal keystrokes: the
            // screen never moves, at the cost of no further composing.
            let mut chars: SmallVec<[char; MAX_WORD]> = SmallVec::new();
            for &ch in &target {
                chars.push(ch);
            }
            literal_from = usize::MAX;
            let retypable = chars.iter().all(|ch| ch.is_ascii_alphabetic());
            if retypable {
                method::compose(&chars, &self.options, literal_from, &mut composed);
                self.render_into(&composed.letters, composed.tone, composed.tone_slot, &mut next);
            }
            if !retypable || next.as_slice() != target.as_slice() {
                literal_from = 0;
                method::compose(&chars, &self.options, literal_from, &mut composed);
                self.render_into(&composed.letters, composed.tone, composed.tone_slot, &mut next);
            }
            self.raw = chars;
        }

        self.literal_from = literal_from;
        self.composed = composed;
        if self.raw.is_empty() {
            // A fully deleted word is a fresh word (`D-BS-LIT-01`).
            self.literal_from = usize::MAX;
            self.edit_len = usize::MAX;
        } else {
            // The word survived an edit: later keys keep the marks the surviving
            // keystrokes made when they only append (`D-TN-07`/`D-TN-08`).
            self.edit_len = self.raw.len();
            if rejudged && was_literal {
                // Nothing of the literal word was composed on screen, so there
                // are no marks to protect: the kept keys are typed fresh.
                self.edit_len = usize::MAX;
            }
        }
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
            method::compose_word(
                &self.raw[..=end],
                &self.options,
                self.literal_from,
                false,
                self.edit_len,
                &mut cur,
            );
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
        let back = self.back.take();
        self.want_scratch = back.is_some();
        self.scratch = None;
        let action = self.boundary_inner(ch);
        match back {
            Some(back) if self.options.auto_restore_english => self.one_word_back(back, ch, action),
            _ => action,
        }
    }

    fn boundary_inner(&mut self, ch: char) -> EngineAction {
        if self.options.auto_capitalize && matches!(ch, '.' | '!' | '?' | '\r' | '\n') {
            self.caps_pending = true;
        }
        // D10 left context: only a plain space keeps the previous word alive.
        let prev = self.prev_word;
        if ch != ' ' {
            self.prev_word = if matches!(ch, ',' | ';' | ':') {
                PrevWord { kind: PrevKind::Clause, count: 0, hash: 0 }
            } else if matches!(ch, '=' | '(' | '[' | '{' | '<' | '>' | '+' | '*' | '%' | '&' | '|') {
                PrevWord { kind: PrevKind::Code, count: 0, hash: 0 }
            } else {
                PrevWord::NONE
            };
        }
        // A path is a token that *starts* with a separator (`/usr`, `~/`, `./`, `C:\\`),
        // never a slash glued to a word (`xanh/đỏ/tím`).
        let path = self.after_path_sep;
        let starts_token = self.raw.is_empty()
            && matches!(self.last_emitted, None | Some(' ') | Some('\t') | Some('\r') | Some('\n'));
        match ch {
            ' ' | '\t' | '\r' | '\n' => {
                self.path_token = false;
                self.path_segments = 0;
                self.path_from_start = false;
            }
            '/' | '\\' | '~' => {
                // `://` of a URL and `C:\\` of a drive also start a path.
                if self.path_token || starts_token || (ch != '~' && self.last_emitted == Some(':')) {
                    if !self.path_token {
                        self.path_from_start = starts_token;
                        self.path_segments = 0;
                    }
                    if ch == '/' && self.last_emitted == Some('/') && self.path_from_start && self.path_segments == 1 {
                        // `//` at the start of a token is a comment marker, not a path.
                        self.path_token = false;
                        self.path_segments = 0;
                        self.path_from_start = false;
                    } else {
                        self.path_token = true;
                        self.path_segments = self.path_segments.saturating_add(1);
                    }
                }
            }
            '.' if starts_token => {
                self.path_token = true;
                self.path_from_start = true;
            }
            _ => {}
        }
        self.after_path_sep = self.path_token && matches!(ch, '/' | '\\' | '~');
        // Raw zone (spec §A9 + amendment 2026-10-30): the opener only arms; the
        // next key decides. The decision is taken at the end of this key, on the
        // *final* left text, because the macro and A10 restore branches below
        // rewrite what the left token shows.
        let previous = self.last_emitted;
        let digits_left = std::mem::take(&mut self.digit_prefix);
        self.last_emitted = Some(ch);
        let opener = is_raw_zone_opener(ch, previous);
        // §2b: the previous opener armed and nothing has been typed since.
        let arm_prev = self.raw_pending;
        let chain_ready = self.options.raw_zone
            && matches!(ch, '\\' | '/')
            && arm_prev
            && matches!(previous, Some(':') | Some('\\') | Some('/'));
        if self.raw.is_empty() {
            // Empty left token (start of line, after whitespace, after another
            // opener): never arms, so `/tắt`, `bật//tắt`, `bật /tắt` compose —
            // unless this opener continues a path (`C:\`, `C:/`, `http://`).
            self.raw_pending = opener && (chain_ready || (digits_left && self.options.raw_zone));
            return EngineAction::Noop;
        }
        let old_len = self.text.len();

        if self.options.macro_enabled {
            if let Some(expansion) = self.macros.lookup_chars(&self.raw).map(str::to_owned) {
                let mut insert = TextBuf::new();
                self.push_expansion(&mut insert, &expansion);
                push_encoded(ch, self.options.charset, &mut insert);
                // The expansion is text, not a syllable, and it is what the left
                // token shows once this key is handled.
                self.raw_pending = self.arm_raw_zone(
                    ch,
                    opener,
                    expansion.is_empty(),
                    expansion.is_ascii(),
                    false,
                    chain_ready && expansion.is_empty(),
                );
                self.prev_word = PrevWord::NONE;
                self.reset_word();
                return EngineAction::Replace {
                    delete_units: old_len as u32,
                    insert: insert.iter().collect(),
                };
            }
        }

        let (auto, score) = self.auto_decide(ch, prev, path, self.path_segments);
        if auto || (self.options.restore_wrong_word && self.should_restore()) {
            let mut insert = TextBuf::new();
            for &raw_ch in &self.raw {
                push_encoded(raw_ch, self.options.charset, &mut insert);
            }
            let raw_units = insert.len();
            push_encoded(ch, self.options.charset, &mut insert);
            if auto && !ch.is_control() {
                // Keep the other reading for the restore hotkey.
                let mut vietnamese: TextBuf = self.text.clone();
                push_encoded(ch, self.options.charset, &mut vietnamese);
                self.correction = Some(Correction {
                    shown: insert.clone(),
                    alternate: vietnamese,
                    mid_word: false,
                    showing_raw: true,
                });
            }
            if self.options.auto_restore_english {
                self.pending_code = self.new_pending(ch, raw_units, false);
                if ch == ' ' {
                    self.prev_word = self.summarize_word(true);
                }
                self.note_boundary(ch, score, true, old_len);
            }
            // A10 put the raw keystrokes back: that is the final left text, and a
            // restored word is by definition not a valid syllable.
            self.raw_pending = self.arm_raw_zone(
                ch,
                opener,
                self.raw.is_empty(),
                plain_ascii(&self.raw),
                false,
                chain_ready && self.raw.is_empty(),
            );
            // Code shapes keep the word that follows literal even with the raw
            // zone switched off.
            if auto && matches!(ch, '_' | '@') {
                self.raw_pending = true;
            }
            self.reset_word();
            return EngineAction::Replace {
                delete_units: old_len as u32,
                insert: insert.iter().collect(),
            };
        }

        if self.options.auto_restore_english {
            self.pending_code = self.new_pending(ch, old_len, self.raw_differs());
            if ch == ' ' {
                self.prev_word = self.summarize_word(false);
            }
            self.note_boundary(ch, score, false, old_len);
        }

        // Nothing rewrote the word, so the final left text is `self.text` as it
        // renders from the composed letters (NFC, before charset encoding). Taken
        // before the reopen block below moves the word out.
        let left_empty = self.composed.letters.is_empty();
        let left_plain_ascii = self.composed_plain_ascii();
        let left_valid = !left_empty
            && spell::is_valid_syllable(&self.composed.letters, self.options.jfwz_as_consonant);

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
        if is_reopening_separator(ch) || ch.is_ascii_digit() {
            self.reopen = Some(CommittedWord {
                raw: std::mem::take(&mut self.raw),
                composed: std::mem::take(&mut self.composed),
                text: std::mem::take(&mut self.text),
                literal_from: self.literal_from,
            });
        }
        // Nothing rewrote the word, so the final left text is `self.text` as it
        // renders from the composed letters (NFC, before charset encoding).
        self.raw_pending = self.arm_raw_zone(
            ch,
            opener,
            left_empty,
            left_plain_ascii,
            left_valid,
            chain_ready && left_empty,
        );
        self.reset_word();
        EngineAction::Noop
    }

    /// Spec §A9 amendment 2026-10-30: the raw zone arms only when the token left
    /// of the opener is visibly not Vietnamese.
    ///
    /// `empty`, `plain_ascii` and `valid_syllable` describe the **final** left
    /// text for this boundary key — the macro expansion or A10 restore output
    /// when those fired, the rendered word otherwise. `chained` is §2b: this
    /// opener continues a path run (`C:\`, `C:/`, `http://`) after an armed
    /// opener with nothing typed since, so the empty left token is allowed.
    fn arm_raw_zone(
        &self,
        ch: char,
        opener: bool,
        empty: bool,
        plain_ascii: bool,
        valid_syllable: bool,
        chained: bool,
    ) -> bool {
        self.options.raw_zone
            && opener
            && (!empty || chained)
            && plain_ascii
            && !(needs_syllable_test(ch) && valid_syllable)
    }

    /// The word on screen is plain ASCII: every letter renders to a `≤ U+007F`
    /// char, so no composed Vietnamese letter (`ă â đ ê ô ơ ư`) and no tone.
    fn composed_plain_ascii(&self) -> bool {
        let letters = &self.composed.letters;
        let target = self
            .composed
            .tone_slot
            .or_else(|| place::target(letters, self.options.modern_tone));
        letters.iter().enumerate().all(|(idx, letter)| {
            let tone = if Some(idx) == target {
                self.composed.tone
            } else {
                Tone::None
            };
            compose_letter(letter.ch, letter.mark, tone)
                .unwrap_or(letter.ch)
                .is_ascii()
        })
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

    /// An inner capital after a part that is a whole Vietnamese word
    /// (`Việt|Nam`, `i|Phone`) starts a new word; anything else (`getValue`,
    /// `useEffect`) freezes the word as typed.
    fn inner_capital_splits(&self) -> bool {
        // A one-letter prefix (`iOS`, `eBay`, `iMac`) is never a word of its own.
        if self.composed.literal || self.composed.escaped || self.raw.len() < 2 || self.raw.len() > 8 {
            return false;
        }
        let mut word = [0u8; MAX_WORD];
        for (slot, &typed) in word.iter_mut().zip(self.raw.iter()) {
            *slot = typed.to_ascii_lowercase() as u8;
        }
        if lexicon::english_word(&word[..self.raw.len()]).is_some() {
            return false;
        }
        let mut key = [0u8; 48];
        let key_len = self.composed_key(&mut key);
        self.plausibly_vietnamese(key_len.map(|len| &key[..len]))
    }

    /// The word so far is a valid Vietnamese syllable prefix and not an English
    /// word (`d`, `Nguyên`; not `use`).
    fn prefix_could_be_vietnamese(&self) -> bool {
        if self.composed.literal || self.composed.escaped || self.raw.len() > 8 {
            return false;
        }
        let mut word = [0u8; MAX_WORD];
        for (slot, &typed) in word.iter_mut().zip(self.raw.iter()) {
            *slot = typed.to_ascii_lowercase() as u8;
        }
        if lexicon::english_word(&word[..self.raw.len()]).is_some() {
            return false;
        }
        let jfwz = self.options.jfwz_as_consonant;
        spell::is_valid_syllable(&self.composed.letters, jfwz) && spell::is_feasible_prefix(&self.composed.letters, jfwz)
    }

    /// The capital would act as a Telex key on the word so far (tone, `w`, or a
    /// doubling of the last letter), so the word is left to compose as in 0.2.7.
    fn capital_is_a_key(&self, ch: char) -> bool {
        if self.options.method == InputMethod::Vni {
            return false;
        }
        let lower = ch.to_ascii_lowercase();
        if matches!(lower, 's' | 'f' | 'r' | 'x' | 'j' | 'w' | 'z') {
            return true;
        }
        matches!(lower, 'a' | 'e' | 'o' | 'd')
            && self.raw.last().is_some_and(|last| last.to_ascii_lowercase() == lower)
    }

    /// Spell check off: the structural trigger is gone, so a typed word that is
    /// exactly a listed English word whose Telex form is not a Vietnamese word
    /// (`order`, `teacher`) goes raw as soon as it is complete.
    fn listed_word_with_spell_check_off(&self) -> bool {
        if !self.options.auto_restore_english
            || self.options.spell_check
            || self.auto_suppressed
            || self.composed.literal
            || self.literal_from != usize::MAX
            || self.edit_len != usize::MAX
            || self.raw.len() < 3
            || !self.raw_differs()
        {
            return false;
        }
        let mut word = [0u8; MAX_WORD];
        for (slot, &typed) in word.iter_mut().zip(self.raw.iter()) {
            if !typed.is_ascii_alphabetic() {
                return false;
            }
            *slot = typed.to_ascii_lowercase() as u8;
        }
        if lexicon::english_word(&word[..self.raw.len()]).is_none() {
            return false;
        }
        // A prefix that could still become a Vietnamese syllable (`baa` on the
        // way to `bậtf`, `moot` on the way to `mootj`) keeps composing; the
        // break decides.
        let letters = &self.composed.letters;
        let jfwz = self.options.jfwz_as_consonant;
        !(spell::is_valid_syllable(letters, jfwz) && spell::is_feasible_prefix(letters, jfwz))
    }

    /// The doubled tone key of an English word was swallowed (`pass` shows
    /// `pas`) and the keys spell an English word the Telex undo idiom does not
    /// need (`ass` is left alone, see `scripts/wordlists/escape-allow.txt`).
    fn escaped_english(&self) -> bool {
        if !self.options.auto_restore_english
            || self.auto_suppressed
            || !self.composed.escaped
            || self.edit_len != usize::MAX
            || self.literal_from != usize::MAX
            || self.raw.len() < 3
        {
            return false;
        }
        let mut word = [0u8; MAX_WORD];
        for (slot, &typed) in word.iter_mut().zip(self.raw.iter()) {
            if !typed.is_ascii_alphabetic() {
                return false;
            }
            *slot = typed.to_ascii_lowercase() as u8;
        }
        // A code identifier (`arr`, `err`) keeps the Telex undo (`arr` = `ar`).
        if !lexicon::english_word(&word[..self.raw.len()]).is_some_and(|w| !w.ident) {
            return false;
        }
        let mut key = [0u8; 48];
        let len = self.composed_key(&mut key);
        !self.plausibly_vietnamese(len.map(|len| &key[..len]))
    }

    /// The composed word is a common Vietnamese word or could be a real
    /// syllable (`ngỗng`, `gỏi`): code rules must leave it alone. Gated on the
    /// spelling, not only on the corpus count, so rare words keep their marks.
    fn plausibly_vietnamese(&self, key: Option<&[u8]>) -> bool {
        key.is_some_and(|key| lexicon::vi_count(key) > 0)
            || spell::is_plausible_syllable(&self.composed.letters, self.composed.tone, self.options.jfwz_as_consonant)
    }

    /// The composed word on screen is not the typed keys.
    fn raw_differs(&self) -> bool {
        let letters = &self.composed.letters;
        self.composed.tone != Tone::None
            || letters.len() != self.raw.len()
            || letters
                .iter()
                .zip(self.raw.iter())
                .any(|(letter, &typed)| letter.mark != crate::letters::Mark::None || letter.ch != typed)
    }

    /// Syllable key of the composed word, for the lexicon lookups.
    fn composed_key(&self, out: &mut [u8; 48]) -> Option<usize> {
        lexicon::syllable_key(&self.composed.letters, self.composed.tone, out)
    }

    /// VNI: a word with digits is code, not Vietnamese typed with tone keys, when
    /// it sits in a path or URL, is a known acronym (`CO2`), ends in two digits
    /// that start with a tone key (`Win10`, `A101`, `A52`), or is one letter and
    /// one digit that is not a vowel (`x1`, `y2`) or is a capital (`A1`, `O2`).
    fn vni_digits_are_code(&self) -> bool {
        if self.path_token {
            return true;
        }
        let mut token = [0u8; MAX_WORD];
        let mut ascii = true;
        for (slot, &typed) in token.iter_mut().zip(self.raw.iter()) {
            ascii &= typed.is_ascii_alphanumeric();
            *slot = typed as u8;
        }
        let raw = &self.raw;
        if ascii && lexicon::is_acronym(&token[..raw.len().min(MAX_WORD)]) {
            return true;
        }
        let trailing = raw.iter().rev().take_while(|c| c.is_ascii_digit()).count();
        if trailing >= 2 && raw.len() > trailing {
            let first = raw[raw.len() - trailing];
            // `a11` is the doubled-tone undo of VNI, not a number.
            let repeated = raw[raw.len() - trailing..].iter().all(|&c| c == first);
            // `ba12` (a tone changed) is Vietnamese typing: the letters before
            // the digits are a plausible lowercase syllable. `Win10`, `A52`,
            // `Top10` are not.
            let mut letters = Composed::default();
            method::compose(&raw[..raw.len() - trailing], &self.options, usize::MAX, &mut letters);
            let vietnamese_letters = raw[0].is_ascii_lowercase()
                && spell::is_plausible_syllable(&letters.letters, letters.tone, self.options.jfwz_as_consonant);
            if matches!(first, '0'..='5') && !repeated && !vietnamese_letters {
                return true;
            }
        }
        false
    }

    /// D10: at a word break, should the typed keys replace the composed word?
    /// The second value is the English-versus-Vietnamese score when the English
    /// list decided (positive = English), for the one-word-back correction.
    ///
    /// Code shapes first (`_` and `@` right after a composed word, and a digit
    /// after a word that is not a common Vietnamese syllable), then the English
    /// word list scored against the Vietnamese reading (`lexicon.rs`).
    fn auto_decide(&self, ch: char, prev: PrevWord, path: bool, segments: u8) -> (bool, Option<f32>) {
        if !self.options.auto_restore_english
            || self.auto_suppressed
            || self.raw.is_empty()
            || self.composed.literal
            || self.literal_from != usize::MAX
            || !self.raw_differs()
        {
            return (false, None);
        }
        // Acronyms typed in capitals (`USA`, `UX`, `OS`, `CO2`).
        let mut token = [0u8; MAX_WORD];
        let mut alnum = true;
        for (slot, &typed) in token.iter_mut().zip(self.raw.iter()) {
            alnum &= typed.is_ascii_alphanumeric();
            *slot = typed as u8;
        }
        if alnum && self.raw.len() <= 5 && lexicon::is_acronym(&token[..self.raw.len()]) {
            return (true, None);
        }
        if self.options.method == InputMethod::Vni && self.raw.iter().any(|c| c.is_ascii_digit()) {
            return (self.vni_digits_are_code(), None);
        }
        let mut key = [0u8; 48];
        let key_len = self.composed_key(&mut key);
        let common_vietnamese = self.plausibly_vietnamese(key_len.map(|len| &key[..len]));
        if path {
            // `cd /usr/bin`: a word right after `/`, `~` or `\` is a path
            // unless it is a common Vietnamese word (`bật/tắt`).
            // A lone separator before a plausible Vietnamese word is not a path
            // (`/Ừ`, `~Nguyễn`); two segments (`/usr/bin`) or an impossible word are.
            let common = key_len.is_some_and(|len| lexicon::vi_count(&key[..len]) >= 20);
            if !common_vietnamese || segments >= 2 {
                return (!(common && common_vietnamese), None);
            }
        }
        if ch.is_ascii_digit() {
            // `utf8`, `h264`: letters then a digit with no space (Telex only: in
            // VNI a digit is a tone key and never reaches a boundary here).
            return (self.raw.len() >= 3 && !common_vietnamese, None);
        }
        let mut word = [0u8; MAX_WORD];
        let mut ascii = self.raw.len() >= 2;
        for (slot, &typed) in word.iter_mut().zip(self.raw.iter()) {
            ascii &= typed.is_ascii_alphabetic();
            *slot = typed.to_ascii_lowercase() as u8;
        }
        let english = if ascii { lexicon::english_word(&word[..self.raw.len()]) } else { None };
        if matches!(ch, '_' | '@' | '(') {
            // `_`, `@` and a `(` glued to a word (`cos(x)`) are code shapes: a word that is not a
            // common Vietnamese syllable, or whose keys spell an English word
            // (`is_empty`, `as_str`), goes back to the typed keys.
            return (!common_vietnamese || english.is_some(), None);
        }
        let Some(english) = english else {
            return (false, None);
        };
        let Some(len) = key_len else {
            return (true, None);
        };
        let left = match prev.kind {
            PrevKind::English => Left::English,
            PrevKind::Function => Left::Function,
            PrevKind::Vietnamese => Left::Vietnamese { count: prev.count, hash: prev.hash },
            PrevKind::Clause => Left::Clause,
            PrevKind::Code => Left::Code,
            PrevKind::None | PrevKind::Other => Left::Start,
        };
        let plausible = self.plausibly_vietnamese(Some(&key[..len]));
        let wins = lexicon::english_wins(
            &key[..len],
            &word[..self.raw.len()],
            english,
            left,
            self.options.app_kind == AppKind::Code,
            plausible,
            self.raw.first().is_some_and(char::is_ascii_uppercase),
        );
        // A Vietnamese word kept over an English one that is not among the most
        // common Vietnamese words may still be corrected once (amendment 4).
        // Only the first word of an explicit phrase (`Max speed`, `Six pack`) is
        // ever corrected one word back; everything else stays Vietnamese.
        let candidate = !wins
            && plausible
            && matches!(left, Left::Start)
            && lexicon::back_first(&word[..self.raw.len()]);
        (wins, candidate.then_some(1.0))
    }

    /// Record what the word that just ended means for the next word
    /// (`scratch`) and, after a space, keep it as a one-word-back candidate when
    /// the call was close (amendment 4). `restored`: the typed keys are now on
    /// screen; otherwise the composed word is.
    fn note_boundary(&mut self, ch: char, score: Option<f32>, restored: bool, old_len: usize) {
        let candidate = ch == ' ' && !restored && !self.raw.is_empty() && score.is_some();
        if !self.want_scratch && !candidate {
            return;
        }
        let mut raw_text = TextBuf::new();
        for &raw_ch in &self.raw {
            push_encoded(raw_ch, self.options.charset, &mut raw_text);
        }
        let composed_text = self.text.clone();
        let mut lower = [0u8; MAX_WORD];
        let mut lower_len = self.raw.len();
        for (slot, &typed) in lower.iter_mut().zip(self.raw.iter()) {
            if !typed.is_ascii_alphabetic() {
                lower_len = 0;
                break;
            }
            *slot = typed.to_ascii_lowercase() as u8;
        }
        if self.want_scratch {
            let right = self.summarize_word(restored);
            self.scratch = Some(Scratch {
                old_len,
                old_text: composed_text.clone(),
                final_text: if restored { raw_text.clone() } else { composed_text.clone() },
                right,
                right_raw: lower,
                right_len: lower_len,
            });
        }
        if candidate {
            self.back = Some(Back {
                shown: composed_text,
                alternate: raw_text,
                typed: lower,
                typed_len: lower_len,
            });
        }
    }

    /// Amendment 4: the word before the one that just ended may be rewritten, once,
    /// when the right context settles a close call. One `Replace` over the old
    /// word, the space and the new word; the restore hotkey takes it back.
    fn one_word_back(&mut self, back: Back, ch: char, action: EngineAction) -> EngineAction {
        let Some(scratch) = self.scratch.take() else {
            return action;
        };
        if ch.is_control() {
            return action;
        }
        // `Max speed`, `Six pack`: the next word is English that cannot be
        // Vietnamese. An unusual Vietnamese word on the right (`mĩ miều`) is not
        // evidence; neither is anything else.
        let right = &scratch.right_raw[..scratch.right_len];
        let rewrite = scratch.right.kind == PrevKind::English
            && lexicon::back_pair(&back.typed[..back.typed_len], right);
        if !rewrite {
            return action;
        }
        let charset = self.options.charset;
        let mut old_region = back.shown.clone();
        push_encoded(' ', charset, &mut old_region);
        old_region.extend(scratch.old_text.iter().copied());
        push_encoded(ch, charset, &mut old_region);
        let mut new_region = back.alternate.clone();
        push_encoded(' ', charset, &mut new_region);
        new_region.extend(scratch.final_text.iter().copied());
        push_encoded(ch, charset, &mut new_region);
        let delete_units = (back.shown.len() + 1 + scratch.old_len) as u32;
        let insert: String = new_region.iter().collect();
        self.correction = Some(Correction {
            shown: new_region,
            alternate: old_region,
            mid_word: false,
            showing_raw: false,
        });
        EngineAction::Replace { delete_units, insert }
    }

    /// What the word that is ending means for the next word's context.
    fn summarize_word(&self, restored: bool) -> PrevWord {
        let mut word = [0u8; MAX_WORD];
        let ascii = self.raw.len() >= 2
            && self.raw.iter().zip(word.iter_mut()).all(|(&typed, slot)| {
                *slot = typed.to_ascii_lowercase() as u8;
                typed.is_ascii_alphabetic()
            });
        // The pronoun `I` opens an English clause as the function words do.
        if self.raw.as_slice() == ['I'] && !restored {
            return PrevWord { kind: PrevKind::Function, count: 0, hash: 0 };
        }
        // Only when the word stays as typed: `how` shown as `hơ` is Vietnamese.
        if ascii && !restored && !self.raw_differs() && lexicon::is_function_word(&word[..self.raw.len()]) {
            return PrevWord { kind: PrevKind::Function, count: 0, hash: 0 };
        }
        let english = restored || (ascii && (self.composed.literal || self.literal_from != usize::MAX));
        if english {
            return PrevWord { kind: PrevKind::English, count: 0, hash: 0 };
        }
        let mut key = [0u8; 48];
        if let Some(len) = self.composed_key(&mut key) {
            let count = lexicon::vi_count(&key[..len]);
            if count > 0 {
                let hash = lexicon::fnv_extend(lexicon::fnv_extend(lexicon::FNV_BASIS, &key[..len]), b" ");
                return PrevWord { kind: PrevKind::Vietnamese, count, hash };
            }
        }
        PrevWord { kind: PrevKind::Other, count: 0, hash: 0 }
    }

    /// `foo.`, `a-`, `std:` — a word followed by a code separator that the next
    /// key may confirm. `units` is what the word occupies on screen.
    fn new_pending(&self, ch: char, units: usize, differs: bool) -> Option<PendingCode> {
        if !matches!(ch, '.' | '-' | ':') || self.raw.is_empty() {
            return None;
        }
        let gate_ok = ch != '.' || {
            let mut key = [0u8; 48];
            let len = self.composed_key(&mut key);
            let mut word = [0u8; MAX_WORD];
            let mut ascii = self.raw.len() >= 2;
            for (slot, &typed) in word.iter_mut().zip(self.raw.iter()) {
                ascii &= typed.is_ascii_alphabetic();
                *slot = typed.to_ascii_lowercase() as u8;
            }
            let ident = ascii && lexicon::english_word(&word[..self.raw.len()]).is_some_and(|w| w.ident);
            ident || !self.plausibly_vietnamese(len.map(|len| &key[..len]))
        };
        let mut chain = SmallVec::new();
        chain.push(ch);
        Some(PendingCode { raw: self.raw.clone(), units, differs, chain, gate_ok })
    }

    /// The key after a pending code separator. `Some` = handled.
    fn chain_key(&mut self, mut pending: PendingCode, ch: char) -> Option<EngineAction> {
        if ch.is_ascii_alphabetic() {
            let complete = matches!(pending.chain.as_slice(), ['.'] | ['-', '>'] | [':', ':']);
            if !(complete && pending.gate_ok) {
                return None;
            }
            // Code: the word before keeps (or gets back) the typed keys and the
            // word that starts now is literal.
            self.caps_pending = false;
            self.literal_from = 0;
            let typed = self.word_key(ch);
            if !pending.differs {
                return Some(typed);
            }
            let mut insert = TextBuf::new();
            for &raw_ch in &pending.raw {
                push_encoded(raw_ch, self.options.charset, &mut insert);
            }
            for &sep in &pending.chain {
                push_encoded(sep, self.options.charset, &mut insert);
            }
            insert.extend(self.text.iter().copied());
            return Some(EngineAction::Replace {
                delete_units: (pending.units + pending.chain.len()) as u32,
                insert: insert.iter().collect(),
            });
        }
        let extends = matches!((pending.chain.as_slice(), ch), (['-'], '>') | ([':'], ':'));
        if !extends {
            return None;
        }
        pending.chain.push(ch);
        let action = self.boundary_key(ch);
        self.pending_code = Some(pending);
        Some(action)
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
        self.digit_prefix = false;
        self.prev_word = PrevWord::NONE;
        self.pending_code = None;
        self.correction = None;
        self.back = None;
        self.after_path_sep = false;
        self.path_token = false;
        self.path_segments = 0;
        self.path_from_start = false;
    }

    fn set_options(&mut self, options: EngineOptions) {
        if self.options.method != options.method || self.options.charset != options.charset {
            self.reset_word();
            self.reopen = None;
        }
        // Config change is a boundary for the armed flag: turning `raw_zone` off
        // (or changing anything else) must not freeze the next word (spec §A9 §7).
        self.raw_pending = false;
        self.pending_code = None;
        self.correction = None;
        self.back = None;
        if self.options.app_kind != options.app_kind || !options.auto_restore_english {
            // A focus change: the words typed in the other app mean nothing here.
            self.prev_word = PrevWord::NONE;
        }
        self.options = options;
    }

    fn set_macros(&mut self, macros: MacroTable) {
        Engine::set_macros(self, macros);
    }

    fn set_macro_only(&mut self, on: bool) {
        Engine::set_macro_only(self, on);
    }

    fn on_screen(&self) -> &[char] {
        Engine::on_screen(self)
    }

    fn process_key(&mut self, key: KeyInput) -> EngineAction {
        // Key-up and non-text chords are none of our business.
        if !key.down {
            return EngineAction::Noop;
        }
        if key.modifiers.ctrl || key.modifiers.alt || key.modifiers.win {
            // A chord (Ctrl+Z, Ctrl+V ...) may change the text under us.
            self.back = None;
            self.correction = None;
            return EngineAction::Noop;
        }
        if self.macro_only {
            return self.macro_only_key(key);
        }
        // The just-committed word is reachable from one key only: whatever this
        // key is (Backspace, a letter, a navigation key), the memory is spent.
        let reopening = self.reopen.take();
        let pending = self.pending_code.take();
        // Any typed or editing key ends the chance to take a correction back;
        // modifier and navigation presses (no character) do not.
        if key.ch.is_some() || matches!(key.vk, VK_BACK | VK_RETURN | VK_TAB | VK_ESCAPE) {
            self.correction = None;
        }
        // The one-word-back chance ends at Backspace, Enter, Tab, Escape and any
        // navigation or command key (not a bare modifier).
        if matches!(key.vk, VK_BACK | VK_RETURN | VK_TAB | VK_ESCAPE)
            || (key.ch.is_none() && !is_modifier_vk(key.vk))
        {
            self.back = None;
        }
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
            None => {
                // A key with no character and no word/boundary role of its own —
                // arrows, Home/End, PgUp/PgDn, Delete, F-keys — is a caret move or
                // a window command: it ends the armed window (spec §A9 §7), so an
                // opener typed before it cannot freeze the next word.
                self.raw_pending = false;
                return EngineAction::Noop;
            }
        };
        if let Some(pending) = pending {
            if let Some(action) = self.chain_key(pending, ch) {
                return action;
            }
        }
        match ch {
            '\u{8}' | '\u{7f}' => self.backspace(reopening),
            '\r' | '\n' => self.boundary_key('\r'),
            _ if ch.is_control() => self.boundary_key(ch),
            // Owner ruling: digits typed before any letter are plain text, in VNI
            // too (`1duowcj` = `1được`); the letters after them are a fresh
            // word. Only an armed raw zone (`C:\` + digit) still swallows them.
            _ if ch.is_ascii_digit() && self.raw.is_empty() && !self.raw_pending => {
                self.caps_pending = false;
                let action = self.boundary_key(ch);
                self.digit_prefix = true;
                action
            }
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

    fn restore_word(&mut self) -> EngineAction {
        Engine::restore_word(self)
    }

    fn can_restore(&self) -> bool {
        if self.correction.is_some() {
            return true;
        }
        // `D-BS-LIT-01`: only when the restore would change the screen, so a
        // bare `d` lets Ctrl+Shift+Z through to the application.
        if self.raw.is_empty() || self.composed.literal || self.literal_from == 0 {
            return false;
        }
        let mut raw_word = Composed::default();
        method::compose(&self.raw, &self.options, 0, &mut raw_word);
        let mut raw_text = TextBuf::new();
        self.render_into(&raw_word.letters, raw_word.tone, raw_word.tone_slot, &mut raw_text);
        raw_text.as_slice() != self.text.as_slice()
    }

    fn boundary(&mut self) {
        self.reset_word();
        self.reopen = None;
        self.prev_word = PrevWord::NONE;
        self.pending_code = None;
        self.correction = None;
        self.back = None;
        self.after_path_sep = false;
        self.path_token = false;
        self.path_segments = 0;
        self.path_from_start = false;
    }
}
/// EVKey character class 1: keys that end the word but leave it re-openable
/// after a Backspace. Taken from the class table initialiser `FUN_140036b10`
/// (`DAT_1400a28e8` plus the single Tab entry): space, Tab, and every ASCII
/// punctuation character. Enter and the other control characters are class 3
/// (`0x140039740`) and hard-reset the word, so they never re-open.
/// A bare modifier or lock key: pressing it changes no text.
fn is_modifier_vk(vk: u16) -> bool {
    matches!(vk, 0x10 | 0x11 | 0x12 | 0x14 | 0x5B | 0x5C | 0xA0..=0xA5)
}

fn is_reopening_separator(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch.is_ascii_punctuation()
}

/// The raw-zone opener set (spec §A9), minus the two shapes that never arm: an
/// ellipsis (`...`) and a digit-dot (`3.5`).
fn is_raw_zone_opener(ch: char, previous: Option<char>) -> bool {
    matches!(ch, '.' | '/' | ':' | '@' | '_' | '\\' | '#' | '=' | '?')
        && !(ch == '.' && previous == Some('.'))
        && !(ch == '.' && previous.is_some_and(|c| c.is_ascii_digit()))
}

/// The three openers a Vietnamese typist also puts between two words. For these
/// only, a valid syllable on the left keeps composing (`bật/tắt`, `ban.`, `sao?`).
fn needs_syllable_test(ch: char) -> bool {
    matches!(ch, '.' | '/' | '?')
}

/// Every char of a fixed left text is `≤ U+007F`.
fn plain_ascii(chars: &[char]) -> bool {
    chars.iter().all(|c| c.is_ascii())
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
    // A literal word (spell-check freeze) shows one letter per keystroke, even
    // when the key that froze it rewrote the letters before it.
    if cur.literal || cur.letters.len() == prev.letters.len() + 1 {
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

    // -- raw-zone predicate (spec §A9 amendment 2026-10-30) ------------------

    fn vni() -> EngineOptions {
        EngineOptions {
            method: InputMethod::Vni,
            ..EngineOptions::default()
        }
    }

    fn type_keys(engine: &mut Engine, keys: &str) {
        for ch in keys.chars() {
            engine.process_key(KeyInput::char(ch));
        }
    }

    /// Type `keys` and report the armed flag the last key left behind — the flag
    /// the next `word_key` consumes.
    fn armed_after(options: EngineOptions, keys: &str) -> bool {
        let mut engine = Engine::new(options);
        type_keys(&mut engine, keys);
        engine.raw_pending
    }

    #[test]
    fn raw_zone_never_arms_on_an_empty_left_token() {
        // `/tắt`: the opener is the first key, so there is nothing on its left.
        assert!(!armed_after(EngineOptions::default(), "/"));
        // `bật /tắt`: the space already reset the word.
        assert!(!armed_after(EngineOptions::default(), "baajt /"));
        // `bật//tắt`: the second `/` sees an empty left token as well.
        assert!(!armed_after(vni(), "ba65t//"));
    }

    #[test]
    fn raw_zone_never_arms_after_a_vietnamese_letter() {
        // `bật` renders a composed letter, so it is not plain ASCII — for every
        // opener, not just the `. / ?` three.
        for opener in ['.', '/', '?', '_', '@', ':', '=', '#', '\\'] {
            assert!(
                !armed_after(EngineOptions::default(), &format!("baajt{opener}")),
                "{opener} must not arm after `bật`"
            );
        }
    }

    #[test]
    fn sentence_punctuation_keeps_a_valid_syllable_vietnamese() {
        // `ban` is a well-formed syllable: `. / ?` leave it Vietnamese.
        for opener in ['.', '/', '?'] {
            assert!(!armed_after(EngineOptions::default(), &format!("ban{opener}")), "{opener}");
        }
        // The same left token under a code-only opener still arms: those three
        // are the only characters a Vietnamese typist also puts between words.
        for opener in ['_', '@', ':', '=', '#', '\\'] {
            assert!(armed_after(EngineOptions::default(), &format!("ban{opener}")), "{opener}");
        }
    }

    #[test]
    fn raw_zone_arms_after_a_plain_ascii_identifier() {
        for opener in ['_', '@', ':', '=', '#', '\\'] {
            assert!(armed_after(vni(), &format!("file{opener}")), "{opener}");
        }
        // `github` is ASCII and not a syllable, so `/` arms too.
        assert!(armed_after(vni(), "github/"));
        // A digits-only left token is not a syllable either (`2/3`).
        assert!(armed_after(vni(), "2/"));
    }

    #[test]
    fn digit_dot_and_ellipsis_never_arm() {
        // Digit-dot: a decimal separator, never a word boundary.
        assert!(!armed_after(vni(), "3."));
        assert!(!armed_after(vni(), "1."));
        // Ellipsis: the second and third dot. `github` arms on the first dot, so
        // the drop is the ellipsis rule and not the syllable test.
        assert!(armed_after(vni(), "github."));
        assert!(!armed_after(vni(), "github.."));
        assert!(!armed_after(vni(), "github..."));
    }

    #[test]
    fn the_syllable_term_covers_exactly_dot_slash_question() {
        for opener in ['.', '/', '?'] {
            assert!(needs_syllable_test(opener), "{opener}");
        }
        for opener in [':', '@', '_', '\\', '#', '=', 'a', ' ', '-'] {
            assert!(!needs_syllable_test(opener), "{opener}");
        }
        // Ellipsis and digit-dot are opener-level exclusions, not predicate terms.
        assert!(is_raw_zone_opener('.', Some('a')));
        assert!(!is_raw_zone_opener('.', Some('.')));
        assert!(!is_raw_zone_opener('.', Some('5')));
        assert!(is_raw_zone_opener('/', Some('a')));
        assert!(!is_raw_zone_opener(',', Some('a')));
    }
}

