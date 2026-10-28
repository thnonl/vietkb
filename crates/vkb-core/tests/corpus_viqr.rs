//! VIQR corpus.
//!
//! `^ (` ` +` modifiers, ` ' \` ? ~ .` tones, `\` escape, `dd` for `đ`.

use vkb_core::{simulate, EngineOptions, InputMethod};

fn viqr() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Viqr,
        ..EngineOptions::default()
    }
}

fn check(options: EngineOptions, cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_eq!(simulate(options, input), *expected, "input = {input:?}");
    }
}

#[test]
fn basic_words() {
    check(
        viqr(),
        &[
            ("Tie^'ng Vie^.t", "Tiếng Việt"),
            ("Vie^.t", "Việt"),
            ("d-u+o+`ng", "đường"),
            ("hoa`", "hòa"),
            ("thuy`", "thùy"),
            ("thuy?", "thủy"),
            ("ddi", "đi"),
            ("TIE^'NG", "TIẾNG"),
        ],
    );
}

#[test]
fn modifier_keys() {
    check(
        viqr(),
        &[
            ("a^", "â"),
            ("e^", "ê"),
            ("o^", "ô"),
            ("a(", "ă"),
            ("o+", "ơ"),
            ("u+", "ư"),
        ],
    );
}

#[test]
fn tone_keys() {
    check(
        viqr(),
        &[
            ("a'", "á"),
            ("a`", "à"),
            ("a?", "ả"),
            ("a~", "ã"),
            ("a.", "ạ"),
            ("u+o+`ng", "ường"),
        ],
    );
}

#[test]
fn backslash_escapes_the_next_character() {
    check(
        viqr(),
        &[
            ("how are you\\?", "how are you?"),
            ("nga`y\\.", "ngày."),
            ("caf\\~", "caf~"),
            ("100\\%", "100%"),
        ],
    );
}

#[test]
fn numeric_words_are_not_transformed() {
    check(
        viqr(),
        &[
            ("123 1d-u+o+`ng 2Tie^'ng", "123 1d-u+o+`ng 2Tie^'ng"),
            ("2026", "2026"),
        ],
    );
}

#[test]
fn english_words_survive() {
    check(viqr(), &[("hello", "hello"), ("google", "google")]);
}

#[test]
fn punctuation_that_is_not_a_trigger_ends_the_word() {
    check(
        viqr(),
        &[
            ("hoa`,", "hòa,"),
            ("hoa`!", "hòa!"),
            ("hoa` ", "hòa "),
        ],
    );
}
