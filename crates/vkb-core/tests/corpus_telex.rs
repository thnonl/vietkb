//! Telex corpus: the reference implementation's cases (adapted to the frozen
//! API) plus the cases sprint 1 added.

use vkb_core::{simulate, Charset, EngineOptions, InputMethod, KeyInput};

fn telex() -> EngineOptions {
    EngineOptions::default()
}

fn old_style() -> EngineOptions {
    EngineOptions {
        modern_tone: false,
        ..EngineOptions::default()
    }
}

fn check(options: EngineOptions, cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_eq!(
            simulate(options, input),
            *expected,
            "input = {input:?}"
        );
    }
}

#[test]
fn basic_words() {
    check(
        telex(),
        &[
            ("Tieengs Vieetj", "Tiếng Việt"),
            ("tieengs", "tiếng"),
            ("Vieetj", "Việt"),
            ("dduwowngf", "đường"),
            ("hoaf", "hòa"),
            ("thuyeenf", "thuyền"),
            ("quas", "quá"),
            ("giaf", "già"),
            ("thasi", "thái"),
            ("Thasi", "Thái"),
            ("nguyeenx", "nguyễn"),
            ("toans", "toán"),
        ],
    );
}

#[test]
fn mark_keys() {
    check(
        telex(),
        &[
            ("aa", "â"),
            ("aw", "ă"),
            ("ee", "ê"),
            ("oo", "ô"),
            ("ow", "ơ"),
            ("uw", "ư"),
            ("dd", "đ"),
        ],
    );
}

#[test]
fn tone_keys() {
    check(
        telex(),
        &[
            ("as", "á"),
            ("af", "à"),
            ("ar", "ả"),
            ("ax", "ã"),
            ("aj", "ạ"),
            ("uws", "ứ"),
            ("ows", "ớ"),
            ("aws", "ắ"),
            ("ddi", "đi"),
        ],
    );
}

#[test]
fn w_marks_both_u_and_o_in_uo_compounds() {
    check(
        telex(),
        &[
            ("thuongwf", "thường"),
            ("tuongwr", "tưởng"),
            ("duongws", "dướng"),
            ("Thuongwf", "Thường"),
            ("TUONGWF", "TƯỜNG"),
            ("nguowif", "người"),
            ("muowif", "mười"),
        ],
    );
}

#[test]
fn uppercase_propagation() {
    check(
        telex(),
        &[
            ("HOAF", "HÒA"),
            ("VIEETJ", "VIỆT"),
            ("Tieengs", "Tiếng"),
            ("HOAS", "HÓA"),
            ("DDUWOWNG", "ĐƯƠNG"),
        ],
    );
}

#[test]
fn modern_tone_places_the_tone_on_the_second_vowel() {
    check(
        old_style(),
        &[
            ("hoaf", "hoà"),
            ("hoef", "hoè"),
            ("thuyf", "thuỳ"),
            ("HOAF", "HOÀ"),
            ("Thuyf", "Thuỳ"),
            ("hoafn", "hoàn"),
            ("khoef", "khoè"),
            ("khoer", "khoẻ"),
        ],
    );
}

#[test]
fn modern_style_is_the_default() {
    check(
        telex(),
        &[
            ("hoaf", "hòa"),
            ("hoef", "hòe"),
            ("thuyf", "thùy"),
            ("thuyr", "thủy"),
            ("khoef", "khòe"),
            ("khoer", "khỏe"),
            ("toans", "toán"),
        ],
    );
}

#[test]
fn tone_key_typed_twice_becomes_literal() {
    check(telex(), &[("ass", "as"), ("aff", "af")]);
}

#[test]
fn repeated_mark_key_escapes_to_the_literal_keystrokes() {
    // A1's escape rule: the key that produced a mark, typed again, reverts the
    // whole word to the raw keys without that trigger (`aww` → `aw`).
    check(telex(), &[("aww", "aw"), ("aaa", "aa"), ("ddd", "dd")]);
}

#[test]
fn calls_f_j_w_z_as_consonants() {
    check(
        telex(),
        &[
            ("of", "of"),
            ("oj", "oj"),
            ("af", "à"),
            ("uw", "ư"),
            ("fa", "fa"),
            ("jazz", "jazz"),
            ("zalo", "zalo"),
        ],
    );
}

#[test]
fn jfwz_off_lets_the_tone_keys_win() {
    let options = EngineOptions {
        jfwz_as_consonant: false,
        ..EngineOptions::default()
    };
    check(options, &[("of", "ò"), ("af", "à")]);
}

#[test]
fn boundaries_and_punctuation() {
    check(
        telex(),
        &[
            ("a,b", "a,b"),
            ("tieengs,", "tiếng,"),
            ("Thasi Thasi Thasi", "Thái Thái Thái"),
            ("a a a", "a a a"),
            ("tieengs.", "tiếng."),
            ("hoaf!", "hòa!"),
        ],
    );
}

#[test]
fn numeric_words_are_not_transformed() {
    check(
        telex(),
        &[
            ("123 1thuongwf 2Tieengs", "123 1thuongwf 2Tieengs"),
            ("2026", "2026"),
            ("x2", "x2"),
        ],
    );
}

#[test]
fn digit_suffix_restores_e_mail_shaped_words() {
    check(
        telex(),
        &[
            ("pndquyens203@gmail.com", "pndquyens203@gmail.com"),
            ("pndquyen203@gmail.com", "pndquyen203@gmail.com"),
        ],
    );
}

#[test]
fn english_words_survive_spell_check() {
    check(
        telex(),
        &[
            ("Microsoft Google Windows", "Microsoft Google Windows"),
            ("google", "google"),
            ("windows", "windows"),
            ("install", "install"),
            ("javascript", "javascript"),
            ("docker", "docker"),
        ],
    );
}

#[test]
fn spell_check_off_transforms_english_looking_words() {
    let options = EngineOptions {
        spell_check: false,
        ..EngineOptions::default()
    };
    assert_ne!(simulate(options, "windows"), "windows");
    assert_ne!(simulate(options, "google"), "google");
}

#[test]
fn charset_tcvn3_matches_the_reference_table() {
    let options = EngineOptions {
        charset: Charset::Tcvn3,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, "tieengs"), "tiÕng");
}

#[test]
fn keys_that_are_not_text_are_noop() {
    use vkb_core::VietnameseEngine;
    let mut engine = vkb_core::Engine::new(telex());
    let mut key = KeyInput::char('a');
    key.down = false;
    assert!(engine.process_key(key).is_noop());
    let mut key = KeyInput::char('s');
    key.modifiers.ctrl = true;
    assert!(engine.process_key(key).is_noop());
    let mut key = KeyInput::char('s');
    key.modifiers.alt = true;
    assert!(engine.process_key(key).is_noop());
    assert!(engine.process_key(KeyInput::key(0x25, None)).is_noop());
}

#[test]
fn methods_round_trip_through_the_option_slug() {
    for method in InputMethod::ALL {
        let options = EngineOptions {
            method: *method,
            ..EngineOptions::default()
        };
        assert_eq!(options.method.as_str(), method.as_str());
    }
}
