//! Spell check and `Tự động khôi phục phím với từ sai`.

use vkb_core::{simulate, EngineOptions};

fn with_restore() -> EngineOptions {
    EngineOptions {
        restore_wrong_word: true,
        jfwz_as_consonant: false,
        ..EngineOptions::default()
    }
}

#[test]
fn invalid_words_keep_their_raw_keystrokes() {
    assert_eq!(simulate(EngineOptions::default(), "bcdf"), "bcdf");
    assert_eq!(simulate(EngineOptions::default(), "qwerty"), "qwerty");
    // `of` is kept raw by the `f j w z` compat rule; `ò` alone would be valid.
    assert_eq!(simulate(EngineOptions::default(), "of"), "of");
    assert_eq!(
        simulate(
            EngineOptions {
                jfwz_as_consonant: false,
                ..EngineOptions::default()
            },
            "of"
        ),
        "ò"
    );
}

#[test]
fn an_invalid_suffix_shows_the_raw_keystrokes() {
    // Spell check on: an invalid suffix makes the word show what was typed.
    assert_eq!(simulate(EngineOptions::default(), "tieengk"), "tieengk");
    assert_eq!(simulate(EngineOptions::default(), "nghieengxq"), "nghieengxq");
    // (`z` is not a suffix any more: Telex `z` removes the tone, D-TN-02.)
    assert_eq!(simulate(EngineOptions::default(), "hoafnk"), "hoafnk");
    assert_eq!(simulate(EngineOptions::default(), "hoafnz"), "hoan");
}

#[test]
fn restore_wrong_word_rewrites_the_word_at_the_boundary() {
    assert_eq!(simulate(with_restore(), "of "), "of ");
    assert_eq!(simulate(with_restore(), "aww "), "aww ");
    assert_eq!(simulate(with_restore(), "tieengsk "), "tieengsk ");
}

#[test]
fn restore_wrong_word_off_keeps_the_transformation() {
    let options = EngineOptions {
        spell_check: false,
        restore_wrong_word: false,
        jfwz_as_consonant: false,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, "of "), "ò ");
    assert_eq!(simulate(options, "tieengsk "), "tiếngk ");
}

#[test]
fn restore_never_touches_a_correct_word() {
    assert_eq!(simulate(with_restore(), "tieengs "), "tiếng ");
    assert_eq!(simulate(with_restore(), "hoaf "), "hòa ");
    assert_eq!(simulate(with_restore(), "quas "), "quá ");
}

#[test]
fn restore_does_not_fire_when_the_word_is_already_raw() {
    let options = EngineOptions {
        restore_wrong_word: true,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, "xyz "), "xyz ");
}

#[test]
fn spell_check_off_keeps_transforming_english_words() {
    let options = EngineOptions {
        spell_check: false,
        ..EngineOptions::default()
    };
    assert_ne!(simulate(options, "windows"), "windows");
    // …and restore is inert without the spell checker (ruling D4), so the
    // transform stands instead of being reverted to the raw word.
    let options = EngineOptions {
        spell_check: false,
        restore_wrong_word: true,
        jfwz_as_consonant: false,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, "of "), "ò ");
}

#[test]
fn tone_key_without_a_vowel_stays_literal() {
    assert_eq!(simulate(EngineOptions::default(), "bs"), "bs");
    assert_eq!(simulate(EngineOptions::default(), "bs "), "bs ");
}
