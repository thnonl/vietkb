//! Macro table (gõ tắt).
//!
//! Triggers are matched against the raw keystrokes of the word, ASCII
//! case-insensitively, and only on a word boundary — the engine expands, then
//! types the boundary character. The table is kept sorted by the lowercase
//! trigger so a lookup is `O(log n · len)` and never allocates.
//!
//! Capacity target from the charter: at least 5000 entries.

use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct Entry {
    /// The trigger as the user wrote it, returned by [`MacroTable::iter`].
    trigger: String,
    /// Lowercase form, the sort key.
    lower: String,
    expansion: String,
}

/// Ordered macro table. Detection of duplicates is case-insensitive.
#[derive(Debug, Clone, Default)]
pub struct MacroTable {
    entries: Vec<Entry>,
}

impl MacroTable {
    pub const RECOMMENDED_CAPACITY: usize = 5000;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut table = Self::new();
        for (trigger, expansion) in pairs {
            table.insert(trigger.as_ref(), expansion.as_ref());
        }
        table
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add or replace a macro. Empty triggers are ignored.
    pub fn insert(&mut self, trigger: &str, expansion: &str) {
        let trigger = trigger.trim();
        if trigger.is_empty() {
            return;
        }
        let lower = trigger.to_lowercase();
        match self
            .entries
            .binary_search_by(|entry| entry.lower.as_str().cmp(lower.as_str()))
        {
            Ok(idx) => {
                self.entries[idx].trigger = trigger.to_string();
                self.entries[idx].expansion = expansion.to_string();
            }
            Err(idx) => self.entries.insert(
                idx,
                Entry {
                    trigger: trigger.to_string(),
                    lower,
                    expansion: expansion.to_string(),
                },
            ),
        }
    }

    /// Remove a macro by trigger. Returns whether one was removed.
    pub fn remove(&mut self, trigger: &str) -> bool {
        let lower = trigger.trim().to_lowercase();
        match self
            .entries
            .binary_search_by(|entry| entry.lower.as_str().cmp(lower.as_str()))
        {
            Ok(idx) => {
                self.entries.remove(idx);
                true
            }
            Err(_) => false,
        }
    }

    pub fn get(&self, trigger: &str) -> Option<&str> {
        let lower = trigger.trim().to_lowercase();
        self.entries
            .binary_search_by(|entry| entry.lower.as_str().cmp(lower.as_str()))
            .ok()
            .map(|idx| self.entries[idx].expansion.as_str())
    }

    /// Expansion for the raw keystrokes of the word, if any. Allocates nothing;
    /// used on every word boundary, so it must stay cheap.
    pub fn lookup_chars(&self, raw: &[char]) -> Option<&str> {
        if raw.is_empty() {
            return None;
        }
        self.entries
            .binary_search_by(|entry| compare_trigger(raw, &entry.lower))
            .ok()
            .map(|idx| self.entries[idx].expansion.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries
            .iter()
            .map(|entry| (entry.trigger.as_str(), entry.expansion.as_str()))
    }

    pub fn triggers(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|entry| entry.trigger.as_str())
    }
}

/// Case-insensitive comparison of raw keystrokes against a lowercase trigger.
///
/// Returns the ordering **of the entry** relative to `raw`, which is what
/// `slice::binary_search_by` expects.
fn compare_trigger(raw: &[char], lower: &str) -> Ordering {
    let mut index = 0usize;
    for expected in lower.chars() {
        if index == raw.len() {
            // The trigger is longer than what was typed: it sorts after.
            return Ordering::Greater;
        }
        match expected.cmp(&raw[index].to_ascii_lowercase()) {
            Ordering::Equal => {}
            other => return other,
        }
        index += 1;
    }
    if index == raw.len() {
        Ordering::Equal
    } else {
        Ordering::Less
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(input: &str) -> Vec<char> {
        input.chars().collect()
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let table = MacroTable::from_pairs([("btw", "by the way"), ("vn", "Việt Nam")]);
        assert_eq!(table.lookup_chars(&chars("btw")), Some("by the way"));
        assert_eq!(table.lookup_chars(&chars("BTW")), Some("by the way"));
        assert_eq!(table.lookup_chars(&chars("Vn")), Some("Việt Nam"));
        assert_eq!(table.lookup_chars(&chars("vnx")), None);
        assert_eq!(table.lookup_chars(&chars("v")), None);
    }

    #[test]
    fn insert_replaces_and_remove_works() {
        let mut table = MacroTable::new();
        table.insert("btw", "by the way");
        table.insert("BTW", "by the way!");
        assert_eq!(table.len(), 1);
        assert_eq!(table.get("btw"), Some("by the way!"));
        assert!(table.remove("bTw"));
        assert!(table.is_empty());
        assert_eq!(table.get("btw"), None);
    }

    #[test]
    fn keeps_five_thousand_entries() {
        let mut table = MacroTable::new();
        for idx in 0..5000 {
            table.insert(&format!("m{idx:04}"), &format!("x{idx}"));
        }
        assert_eq!(table.len(), 5000);
        assert_eq!(table.lookup_chars(&chars("m4999")), Some("x4999"));
    }
}
