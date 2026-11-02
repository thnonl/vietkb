//! D10 completeness of the hand review (0.2.9). Every syllable a listed English word, or an
//! inflection of one, composes to in Telex (auto-restore off) must be either:
//! - attested by the corpus (count > 0: the rule never fires on it), or
//! - a real Vietnamese syllable in `data/real_syllables.rs`, or
//! - reviewed as not real in `tests/fixtures/d10_not_vietnamese.txt`, or
//! - not a plausible syllable at all (the engine already calls such words English).
//! A new count-zero plausible key that appears in neither list fails this test.

use std::collections::BTreeSet;

use vkb_core::letters::Tone;
use vkb_core::lexicon::{self, syllable_key, vi_count};
use vkb_core::method::{self, Composed};
use vkb_core::{spell, EngineOptions};

static ENGLISH: &str = include_str!("../data/english.txt");
static REVIEW: &str = include_str!("fixtures/d10_not_vietnamese.txt");

const SUFFIXES: &[&str] = &["ers", "ing", "es", "ed", "er", "s"];

/// Syllable key of the Telex composition of `word` and whether the engine could use it
/// (plausible structure). `None` when the word does not compose to a different text.
fn composed_key(word: &str) -> Option<(String, bool)> {
    let raw: Vec<char> = word.chars().collect();
    let mut composed = Composed::default();
    method::compose(&raw, &EngineOptions::default(), usize::MAX, &mut composed);
    let changed = composed.tone != Tone::None
        || composed.letters.len() != raw.len()
        || composed.letters.iter().zip(&raw).any(|(l, &c)| l.mark != vkb_core::letters::Mark::None || l.ch != c);
    // A literal composition (the spell check kept the raw text) is never looked up by the engine.
    if !changed || composed.literal {
        return None;
    }
    let mut buf = [0u8; 48];
    let len = syllable_key(&composed.letters, composed.tone, &mut buf)?;
    let plausible = spell::is_plausible_syllable(&composed.letters, composed.tone, true);
    Some((String::from_utf8_lossy(&buf[..len]).into_owned(), plausible))
}

fn reviewed_keys() -> BTreeSet<&'static str> {
    REVIEW.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split('\t').next()).collect()
}

/// Every listed word and every suffix inflection of a listed, non-closed, non-identifier stem.
fn candidates() -> Vec<String> {
    let mut words = Vec::new();
    let mut stems = Vec::new();
    for line in ENGLISH.lines() {
        let (word, tag) = line.split_at(line.len() - 1);
        words.push(word.to_string());
        // Identifier words (tags h..n) are code tokens, not inflection bases.
        if !("h"..="n").contains(&tag) {
            stems.push(word.to_string());
        }
    }
    let listed: BTreeSet<String> = words.iter().cloned().collect();
    let mut all: BTreeSet<String> = listed.clone();
    for stem in &stems {
        for suffix in SUFFIXES {
            all.insert(format!("{stem}{suffix}"));
            if let Some(base) = stem.strip_suffix('e') {
                all.insert(format!("{base}{suffix}"));
            }
        }
    }
    all.into_iter().collect()
}

#[test]
fn every_composed_key_of_a_listed_or_inflected_word_is_reviewed() {
    let reviewed = reviewed_keys();
    let mut unreviewed = BTreeSet::new();
    for word in candidates() {
        let Some((key, plausible)) = composed_key(&word) else { continue };
        if !plausible || vi_count(key.as_bytes()) > 0 || lexicon::real_syllable(key.as_bytes()) {
            continue;
        }
        if !reviewed.contains(key.as_str()) {
            unreviewed.insert(format!("{key} (from {word})"));
        }
    }
    assert!(
        unreviewed.is_empty(),
        "{} plausible count-zero keys are in neither real_syllables.rs nor the review fixture:\n{}",
        unreviewed.len(),
        unreviewed.iter().take(40).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn the_review_fixture_has_no_real_or_attested_key() {
    for key in reviewed_keys() {
        assert!(!lexicon::real_syllable(key.as_bytes()), "{key} is both real and reviewed not real");
        assert_eq!(vi_count(key.as_bytes()), 0, "{key} is attested in vi.txt");
    }
}
