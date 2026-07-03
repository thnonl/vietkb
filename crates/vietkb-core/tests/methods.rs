use vietkb_core::{EngineAction, InputMethod, KeyInput, SimpleVietnameseEngine, VietnameseEngine};

fn simulate(method: InputMethod, input: &str) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_input_method(method);
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
fn vni_basic_words() {
    let cases = [
        ("Tie6ng1 Vie6t5", "Tiếng Việt"),
        ("d9u7o7ng2", "đường"),
        ("hoa2", "hoà"),
    ];

    for (input, expected) in cases {
        assert_eq!(simulate(InputMethod::Vni, input), expected, "input={input}");
    }
}

#[test]
fn vni_repeated_tone_key_escapes_to_raw_text() {
    assert_eq!(simulate(InputMethod::Vni, "a11"), "a1");
}

#[test]
fn viqr_basic_words() {
    let cases = [
        ("Tie^'ng Vie^.t", "Tiếng Việt"),
        ("d-u+o+`ng", "đường"),
        ("hoa`", "hoà"),
    ];

    for (input, expected) in cases {
        assert_eq!(
            simulate(InputMethod::Viqr, input),
            expected,
            "input={input}"
        );
    }
}
