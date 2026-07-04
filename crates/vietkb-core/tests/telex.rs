use vietkb_core::{
    EngineAction, EngineOptions, KeyInput, SimpleVietnameseEngine, VietnameseEngine,
};

fn simulate(input: &str) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    simulate_with_engine(input, &mut engine)
}

fn simulate_with_engine(input: &str, engine: &mut SimpleVietnameseEngine) -> String {
    let mut output = String::new();

    for ch in input.chars() {
        match engine.process_key(KeyInput::from_char(ch)) {
            EngineAction::Noop
            | EngineAction::Preedit(_)
            | EngineAction::ToggleVietnamese
            | EngineAction::BeepOrReject => {}
            EngineAction::Commit(text) => output.push_str(&text),
            EngineAction::Replace {
                delete_graphemes,
                insert,
            } => {
                for _ in 0..delete_graphemes {
                    output.pop();
                }
                output.push_str(&insert);
            }
        }
    }

    output
}

#[test]
fn telex_basic_words() {
    let cases = [
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
    ];

    for (input, expected) in cases {
        assert_eq!(simulate(input), expected, "input={input}");
    }
}

#[test]
fn telex_marks() {
    let cases = [
        ("aa", "â"),
        ("aw", "ă"),
        ("ee", "ê"),
        ("oo", "ô"),
        ("ow", "ơ"),
        ("uw", "ư"),
        ("dd", "đ"),
    ];

    for (input, expected) in cases {
        assert_eq!(simulate(input), expected, "input={input}");
    }
}

#[test]
fn telex_tones() {
    let cases = [
        ("as", "á"),
        ("af", "à"),
        ("ar", "ả"),
        ("ax", "ã"),
        ("aj", "ạ"),
        ("uws", "ứ"),
        ("ows", "ớ"),
    ];

    for (input, expected) in cases {
        assert_eq!(simulate(input), expected, "input={input}");
    }
}

#[test]
fn repeated_tone_key_escapes_to_raw_text() {
    assert_eq!(simulate("ass"), "as");
}

#[test]
fn boundaries_clear_word_state() {
    assert_eq!(simulate("tieengs Vieetj"), "tiếng Việt");
    assert_eq!(simulate("a,b"), "a,b");
    assert_eq!(simulate("Thasi Thasi Thasi"), "Thái Thái Thái");
}

#[test]
fn can_skip_common_english_words() {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_skip_english_words(true);
    let mut output = String::new();

    for ch in "google windows Tieengs".chars() {
        match engine.process_key(KeyInput::from_char(ch)) {
            EngineAction::Noop
            | EngineAction::Preedit(_)
            | EngineAction::ToggleVietnamese
            | EngineAction::BeepOrReject => {}
            EngineAction::Commit(text) => output.push_str(&text),
            EngineAction::Replace {
                delete_graphemes,
                insert,
            } => {
                for _ in 0..delete_graphemes {
                    output.pop();
                }
                output.push_str(&insert);
            }
        }
    }

    assert_eq!(output, "google windows Tiếng");
}

#[test]
fn unikey_style_oa_uy_option_places_tone_on_second_vowel() {
    let cases = [
        ("hoaf", "hoà"),
        ("hoef", "hoè"),
        ("thuyf", "thuỳ"),
        ("HOAF", "HOÀ"),
        ("Thuyf", "Thuỳ"),
    ];

    for (input, expected) in cases {
        let mut engine = SimpleVietnameseEngine::new();
        engine.set_options(EngineOptions {
            old_oa_uy: true,
            ..EngineOptions::default()
        });
        assert_eq!(
            simulate_with_engine(input, &mut engine),
            expected,
            "input={input}"
        );
    }
}

#[test]
fn unikey_style_tone_target_keeps_qu_gi_and_trailing_consonant_rules() {
    let cases = [
        ("quas", "quá"),
        ("giaf", "già"),
        ("toans", "toán"),
        ("thuyeenf", "thuyền"),
        ("nguyeenx", "nguyễn"),
    ];

    for (input, expected) in cases {
        assert_eq!(simulate(input), expected, "input={input}");
    }
}

#[test]
fn use_j_f_w_as_consonants_keeps_invalid_telex_positions_raw() {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_options(EngineOptions {
        use_j_f_w_as_consonants: true,
        ..EngineOptions::default()
    });

    assert_eq!(simulate_with_engine("of af uw", &mut engine), "of à ư");
}

#[test]
fn disabled_use_j_f_w_as_consonants_preserves_legacy_telex_tone_keys() {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_options(EngineOptions {
        use_j_f_w_as_consonants: false,
        ..EngineOptions::default()
    });

    assert_eq!(simulate_with_engine("of", &mut engine), "ò");
}

#[test]
fn disabled_spell_check_keeps_telex_transforming_english_words() {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_options(EngineOptions {
        spell_check: false,
        skip_english_words: true,
        ..EngineOptions::default()
    });

    assert_ne!(simulate_with_engine("windows", &mut engine), "windows");
}

#[test]
fn restore_wrong_word_restores_invalid_transformed_word_at_boundary() {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_options(EngineOptions {
        use_j_f_w_as_consonants: false,
        restore_wrong_word: true,
        ..EngineOptions::default()
    });

    assert_eq!(simulate_with_engine("of tieengs", &mut engine), "of tiếng");
}
