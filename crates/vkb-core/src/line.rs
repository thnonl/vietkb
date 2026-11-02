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
}

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
        }
    }

    /// Forget the whole line. O(1).
    pub fn clear(&mut self) {
        self.key_begin = 0;
        self.key_end = 0;
        self.head = 0;
        self.count = 0;
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

    /// Storage the line occupies, as a constant: the rings are inline.
    pub const fn footprint() -> usize {
        LINE_KEYS + LINE_ENTRIES
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
