use vietkb_core::{
    EngineAction, EngineOptions, InputMethod, KeyInput, SimpleVietnameseEngine, VietnameseEngine,
};

fn simulate(method: InputMethod, input: &str) -> String {
    simulate_with_options(method, input, EngineOptions::default())
}

fn simulate_with_options(method: InputMethod, input: &str, options: EngineOptions) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_input_method(method);
    engine.set_options(options);
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
        ("hoa2", "hòa"),
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
fn vni_repeated_tone_changes_keep_word_stable() {
    assert_eq!(simulate(InputMethod::Vni, "thai1212"), "thài");
}

#[test]
fn viqr_basic_words() {
    let cases = [
        ("Tie^'ng Vie^.t", "Tiếng Việt"),
        ("d-u+o+`ng", "đường"),
        ("hoa`", "hòa"),
    ];

    for (input, expected) in cases {
        assert_eq!(
            simulate(InputMethod::Viqr, input),
            expected,
            "input={input}"
        );
    }
}

#[test]
fn smart_mark_option_applies_to_all_input_methods() {
    let smart = EngineOptions {
        smart_mark: true,
        ..EngineOptions::default()
    };
    let free = EngineOptions {
        smart_mark: false,
        ..EngineOptions::default()
    };

    assert_eq!(
        simulate_with_options(InputMethod::Telex, "thuyf", smart),
        "thùy"
    );
    assert_eq!(
        simulate_with_options(InputMethod::Telex, "thuyf", free),
        "thuỳ"
    );
    assert_eq!(
        simulate_with_options(InputMethod::Vni, "thuy2", smart),
        "thùy"
    );
    assert_eq!(
        simulate_with_options(InputMethod::Vni, "thuy2", free),
        "thuỳ"
    );
    assert_eq!(
        simulate_with_options(InputMethod::Viqr, "thuy`", smart),
        "thùy"
    );
    assert_eq!(
        simulate_with_options(InputMethod::Viqr, "thuy`", free),
        "thuỳ"
    );
}
