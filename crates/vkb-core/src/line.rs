//! The line history: what a Backspace may reopen, kept the way EVKey keeps it.
//!
//! EVKey holds the whole line's letters in one array and appends a boundary
//! entry for every separator. Backspace pops the boundary (the app deletes the
//! separator) and the previous word is the tail again. VietKB keeps the same
//! shape, bounded and inline:
//!
//! * one ring of raw keystrokes, [`LINE_KEYS`] characters, holding the keys of
//!   every live word in order;
//! * one ring of [`Entry`] headers, [`LINE_ENTRIES`] long, each a committed word
//!   (its keys sit in the ring) or one separator mark (the app typed one
//!   separator character here).
//!
//! Nothing is composed or rendered here: a word is rebuilt from its keys by the
//! engine when a Backspace reaches it. Pushing, dropping and popping are O(1)
//! (a word's keys are copied only on the Backspace that reopens it), and the
//! structure never touches the heap.

use smallvec::SmallVec;

use crate::MAX_WORD;

/// Keystroke ring capacity in characters (a power of two, see [`slot`]).
pub const LINE_KEYS: usize = 256;
/// Header ring capacity: committed words plus separator marks kept reachable.
pub const LINE_ENTRIES: usize = 32;

const NO_LITERAL: u8 = u8::MAX;
const NO_SLOT: u8 = u8::MAX;

/// One header: 8 bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// `true`: a committed word. `false`: one separator the app typed.
    pub word: bool,
    /// Keystrokes the word occupies in the ring (0 for a mark).
    pub len: u8,
    /// The word's `literal_from` (`NO_LITERAL` = `usize::MAX`).
    literal: u8,
    /// The word's pinned tone letter (`NO_SLOT` = none).
    slot: u8,
    /// FNV-1a of the text the word showed on screen when it was committed.
    pub hash: u32,
}

impl Entry {
    /// Header for a word committed with this `literal_from`, pinned tone and
    /// on-screen text. `None` when the values do not fit a header (never for
    /// words within [`MAX_WORD`]): the caller then forgets the line.
    pub fn word(literal_from: usize, tone_slot: Option<usize>, text: &[char]) -> Option<Entry> {
        let literal = if literal_from == usize::MAX {
            NO_LITERAL
        } else if literal_from <= MAX_WORD {
            literal_from as u8
        } else {
            return None;
        };
        let slot = match tone_slot {
            None => NO_SLOT,
            Some(slot) if slot < MAX_WORD => slot as u8,
            Some(_) => return None,
        };
        Some(Entry { word: true, len: 0, literal, slot, hash: text_hash(text) })
    }

    /// Separator mark header.
    pub fn mark() -> Entry {
        Entry { word: false, len: 0, literal: NO_LITERAL, slot: NO_SLOT, hash: 0 }
    }

    /// The word's `literal_from` as the engine stores it.
    pub fn literal_from(&self) -> usize {
        if self.literal == NO_LITERAL {
            usize::MAX
        } else {
            usize::from(self.literal)
        }
    }

    /// The word's pinned tone letter, if any.
    pub fn tone_slot(&self) -> Option<usize> {
        (self.slot != NO_SLOT).then_some(usize::from(self.slot))
    }
}

/// FNV-1a over the characters of a word's on-screen text.
pub fn text_hash(text: &[char]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for &ch in text {
        hash ^= ch as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[inline]
fn slot(pos: u32) -> usize {
    pos as usize & (LINE_KEYS - 1)
}

/// The bounded line history. Positions are wrapping `u32` counters; the live
/// keystrokes are always `key_begin..key_end`, at most [`LINE_KEYS`] of them.
pub struct LineHistory {
    keys: [char; LINE_KEYS],
    key_begin: u32,
    key_end: u32,
    entries: [Entry; LINE_ENTRIES],
    head: u8,
    count: u8,
    /// "Gõ dấu đè": on-screen characters between the caret and the end of the remembered line
    /// (Left/Right/End move it; 0 = the caret is at the end, where new text goes).
    caret_back: u16,
    /// The word (counted from the newest entry) being re-toned in place, if any.
    editing: u8,
}

const NOT_EDITING: u8 = u8::MAX;

impl Default for LineHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl LineHistory {
    pub const fn new() -> Self {
        Self {
            keys: ['\0'; LINE_KEYS],
            key_begin: 0,
            key_end: 0,
            entries: [Entry { word: false, len: 0, literal: NO_LITERAL, slot: NO_SLOT, hash: 0 }; LINE_ENTRIES],
            head: 0,
            count: 0,
            caret_back: 0,
            editing: NOT_EDITING,
        }
    }

    /// Forget the whole line. O(1).
    pub fn clear(&mut self) {
        self.key_begin = 0;
        self.key_end = 0;
        self.head = 0;
        self.count = 0;
        self.caret_back = 0;
        self.editing = NOT_EDITING;
    }

    /// Number of headers (words and separator marks) remembered.
    pub fn len(&self) -> usize {
        usize::from(self.count)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn caret_back(&self) -> usize {
        usize::from(self.caret_back)
    }

    pub fn set_caret_back(&mut self, back: usize) {
        self.caret_back = back.min(usize::from(u16::MAX)) as u16;
    }

    /// The word being re-toned in place, counted from the newest entry.
    pub fn editing(&self) -> Option<usize> {
        (self.editing != NOT_EDITING).then_some(usize::from(self.editing))
    }

    pub fn set_editing(&mut self, nth: Option<usize>) {
        self.editing = nth.map_or(NOT_EDITING, |n| n as u8);
    }

    /// Header `nth` counted from the newest (0 = newest).
    pub fn entry_from_newest(&self, nth: usize) -> Option<Entry> {
        if nth >= usize::from(self.count) {
            return None;
        }
        Some(self.entries[(usize::from(self.head) + usize::from(self.count) - 1 - nth) % LINE_ENTRIES])
    }

    /// The keys of word `nth` (counted from the newest) into `out`, with its header.
    pub fn word_from_newest(&self, nth: usize, out: &mut SmallVec<[char; MAX_WORD]>) -> Option<Entry> {
        let entry = self.entry_from_newest(nth).filter(|e| e.word)?;
        let newer: u32 = (0..nth).filter_map(|n| self.entry_from_newest(n)).map(|e| u32::from(e.len)).sum();
        let first = self.key_end.wrapping_sub(newer).wrapping_sub(u32::from(entry.len));
        out.clear();
        for offset in 0..u32::from(entry.len) {
            out.push(self.keys[slot(first.wrapping_add(offset))]);
        }
        Some(entry)
    }

    /// Put `keys`/`entry` in place of word `nth` (counted from the newest), keeping every other
    /// entry, the caret and the order. Rare (one call per in-place re-tone): rebuilt by copying.
    pub fn replace_word_from_newest(&mut self, nth: usize, keys: &[char], entry: Entry) {
        let count = usize::from(self.count);
        if nth >= count || !self.entry_from_newest(nth).is_some_and(|e| e.word) {
            return;
        }
        let mut all_keys = [['\0'; MAX_WORD]; LINE_ENTRIES];
        let mut all_entries = [Entry::mark(); LINE_ENTRIES];
        let mut scratch: SmallVec<[char; MAX_WORD]> = SmallVec::new();
        // Oldest first.
        for i in 0..count {
            let n = count - 1 - i;
            let e = self.entry_from_newest(n).unwrap_or_default();
            if e.word {
                let _ = self.word_from_newest(n, &mut scratch);
                all_keys[i][..scratch.len()].copy_from_slice(&scratch);
            }
            all_entries[i] = e;
        }
        let (caret_back, editing) = (self.caret_back, self.editing);
        self.clear();
        for i in 0..count {
            let n = count - 1 - i;
            if n == nth {
                self.push_word(keys, entry);
            } else if all_entries[i].word {
                let len = usize::from(all_entries[i].len);
                self.push_word(&all_keys[i][..len], all_entries[i]);
            } else {
                self.push_mark();
            }
        }
        self.caret_back = caret_back;
        self.editing = editing;
    }

    /// The newest header, if any.
    pub fn top(&self) -> Option<Entry> {
        if self.count == 0 {
            return None;
        }
        Some(self.entries[self.newest_index()])
    }

    /// Append a committed word whose keys are `keys`. Drops the oldest entries
    /// as needed to stay within both rings. A word longer than the key ring
    /// (cannot happen with [`MAX_WORD`]) clears the line instead.
    pub fn push_word(&mut self, keys: &[char], mut entry: Entry) {
        if keys.len() > LINE_KEYS {
            self.clear();
            return;
        }
        while usize::from(self.count) == LINE_ENTRIES {
            self.drop_oldest();
        }
        while self.key_end.wrapping_sub(self.key_begin) as usize + keys.len() > LINE_KEYS {
            self.drop_oldest();
        }
        for (offset, &ch) in keys.iter().enumerate() {
            self.keys[slot(self.key_end.wrapping_add(offset as u32))] = ch;
        }
        self.key_end = self.key_end.wrapping_add(keys.len() as u32);
        entry.word = true;
        entry.len = keys.len() as u8;
        self.push_entry(entry);
    }

    /// Append one separator mark.
    pub fn push_mark(&mut self) {
        if usize::from(self.count) == LINE_ENTRIES {
            self.drop_oldest();
        }
        self.push_entry(Entry::mark());
    }

    /// Remove the newest header, which must be a mark, and report whether it
    /// was one. Used when the app deletes that separator.
    pub fn pop_mark(&mut self) -> bool {
        match self.top() {
            Some(entry) if !entry.word => {
                self.pop_entry();
                true
            }
            _ => false,
        }
    }

    /// If the newest header is a word, copy its keys into `out`, remove it and
    /// return its header. Otherwise leave the line alone and return `None`.
    pub fn pop_word(&mut self, out: &mut SmallVec<[char; MAX_WORD]>) -> Option<Entry> {
        let entry = self.top().filter(|entry| entry.word)?;
        let first = self.key_end.wrapping_sub(u32::from(entry.len));
        out.clear();
        for offset in 0..u32::from(entry.len) {
            out.push(self.keys[slot(first.wrapping_add(offset))]);
        }
        self.pop_entry();
        Some(entry)
    }

    fn newest_index(&self) -> usize {
        (usize::from(self.head) + usize::from(self.count) - 1) % LINE_ENTRIES
    }

    fn push_entry(&mut self, entry: Entry) {
        let index = (usize::from(self.head) + usize::from(self.count)) % LINE_ENTRIES;
        self.entries[index] = entry;
        self.count += 1;
    }

    fn pop_entry(&mut self) {
        let entry = self.entries[self.newest_index()];
        self.key_end = self.key_end.wrapping_sub(u32::from(entry.len));
        self.count -= 1;
    }

    fn drop_oldest(&mut self) {
        if self.count == 0 {
            return;
        }
        let entry = self.entries[usize::from(self.head)];
        self.key_begin = self.key_begin.wrapping_add(u32::from(entry.len));
        self.head = ((usize::from(self.head) + 1) % LINE_ENTRIES) as u8;
        self.count -= 1;
    }
}
