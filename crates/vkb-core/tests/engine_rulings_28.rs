//! Regression pins for the engine defects ruled on in
//! `docs/evkey-clone-work/28-engine-rulings.md` (D1, D2, D3, D3b, D5, D6, D7).
//!
//! One test per ruling so a regression names the defect it reopens. The full
//! corpus row set lives in `spec_51_engine.rs`; these are the minimal cases.

use vkb_core::letters::Letter;
use vkb_core::{simulate, spell, EngineOptions, InputMethod};

fn telex(jfwz: bool) -> EngineOptions {
    EngineOptions {
        jfwz_as_consonant: jfwz,
        ..EngineOptions::default()
    }
}

fn vni() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    }
}

fn letters(word: &str) -> Vec<Letter> {
    word.chars().map(Letter::literal).collect()
}

/// D1 (P0): `đ` is a consonant, never a tone target. The bar and the tone must
/// both survive.
#[test]
fn d1_d_bar_never_takes_the_tone() {
    assert_eq!(simulate(telex(false), "ddepj"), "đẹp");
    assert_eq!(simulate(telex(false), "ddoanf"), "đoàn");
    assert_eq!(simulate(telex(false), "ddos"), "đó");
    assert_eq!(simulate(vni(), "d9a4"), "đã");
    assert_eq!(simulate(vni(), "d9ung1"), "đúng");
}

/// D2 (P1): a standalone `w` is `ư` when `f j w z` are tone keys, and a plain
/// letter when they are consonants.
#[test]
fn d2_standalone_w_is_u_horn_unless_jfwz_are_consonants() {
    let off = telex(false);
    assert_eq!(simulate(off, "w"), "ư");
    assert_eq!(simulate(off, "w "), "ư ");
    assert_eq!(simulate(off, "tw"), "tư");
    assert_eq!(simulate(off, "nhw"), "như");

    let on = telex(true);
    assert_eq!(simulate(on, "w"), "w");
    assert_eq!(simulate(on, "tw"), "tw");
}

/// D3 (P1): repeating the trigger that produced the tone/mark reverts the whole
/// word to the literal keystrokes, minus the trigger that was consumed.
#[test]
fn d3_repeated_trigger_escapes_to_the_raw_keys() {
    let options = telex(false);
    for (input, expected) in [
        ("asss", "ass"),
        ("dddd", "ddd"),
        ("oooo", "ooo"),
        ("aass", "aas"),
        ("thuowngff", "thuowngf"),
        ("aww", "aw"),
        ("ww", "w"),
    ] {
        assert_eq!(simulate(options, input), expected, "telex {input:?}");
    }
    assert_eq!(simulate(vni(), "a11"), "a1");
    assert_eq!(simulate(vni(), "a66"), "a6");
}

/// D3b (P2): a mark key that cannot complete a transform reverts the word to
/// the raw keystrokes instead of leaving a half-transformed prefix.
#[test]
fn d3b_failed_mark_key_never_half_transforms() {
    assert_eq!(simulate(telex(false), "await"), "await");
}

/// D5 (P2): the validator rejects an onset that cannot take the nucleus after
/// it (`ng`/`c` before `i e ê`, `k`/`gh`/`ngh` before anything else).
#[test]
fn d5_impossible_onsets_are_invalid() {
    assert!(!spell::is_valid_syllable(&letters("ngie"), false));
    assert!(!spell::is_valid_syllable(&letters("ngien"), false));
    assert!(!spell::is_valid_syllable(&letters("ce"), false));
    assert!(!spell::is_valid_syllable(&letters("ka"), false));
    assert!(!spell::is_valid_syllable(&letters("kgh"), false));
    assert!(!spell::is_valid_syllable(&letters("phg"), false));
    // …and the legal partners still parse.
    for word in ["nghi", "nghie", "ghe", "ke", "gia", "nguoi", "co"] {
        assert!(spell::is_valid_syllable(&letters(word), false), "{word}");
    }
    // The row this closes.
    assert_eq!(simulate(telex(false), "ngieengs"), "ngieengs");
}

/// D6 (P2): the acceptance list from the ruling — English words pass through
/// untouched with the default options, Vietnamese still transforms.
#[test]
fn d6_english_passthrough_with_defaults() {
    let options = EngineOptions::default();
    for word in ["test", "chrome", "github", "select"] {
        assert_eq!(simulate(options, word), word, "{word} must stay raw");
    }
    assert_eq!(simulate(options, "tieengs"), "tiếng");
    assert_eq!(simulate(options, "dduwowngf"), "đường");
    // The escape hatch keeps working: with the spell check off, transforms
    // apply as before.
    let unchecked = EngineOptions {
        spell_check: false,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(unchecked, "test"), "tét");
}

/// D7 (P1): `Đặt dấu tự do`. On (default): the tone may be typed early and
/// re-places itself as the syllable grows. Off: it stays on the vowel it hit.
#[test]
fn d7_free_tone_is_honoured() {
    // On (default): the tone re-places itself as the syllable grows.
    let free = EngineOptions::default();
    assert_eq!(simulate(free, "hoafn"), "hoàn");
    assert_eq!(simulate(free, "thasi"), "thái");

    // Off: the mark sticks to the vowel it hit when the key was typed.
    let pinned = EngineOptions {
        free_tone: false,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(pinned, "hoafn"), "hòan");
    // Marks are positional either way.
    assert_eq!(simulate(pinned, "vieejt"), "việt");
    assert_eq!(simulate(pinned, "dduwowngf"), "đường");
}
