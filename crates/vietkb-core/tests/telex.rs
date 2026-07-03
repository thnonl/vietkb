use vietkb_core::{EngineAction, KeyInput, SimpleVietnameseEngine, VietnameseEngine};

fn simulate(input: &str) -> String {
    let mut engine = SimpleVietnameseEngine::new();
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
        ("hoaf", "hoà"),
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
fn boundaries_clear_word_state() {
    assert_eq!(simulate("tieengs Vieetj"), "tiếng Việt");
    assert_eq!(simulate("a,b"), "a,b");
    assert_eq!(simulate("Thasi Thasi Thasi"), "Thái Thái Thái");
}
