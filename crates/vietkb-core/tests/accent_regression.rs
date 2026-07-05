use vietkb_core::{
    Charset, EngineAction, EngineOptions, InputMethod, KeyCode, KeyInput, SimpleVietnameseEngine,
    VietnameseEngine,
};

fn apply_action(output: &mut String, action: EngineAction) {
    match action {
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

fn simulate(method: InputMethod, input: &str) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_input_method(method);
    engine.set_charset(Charset::Unicode);
    engine.set_options(EngineOptions::default());
    let mut output = String::new();

    for ch in input.chars() {
        apply_action(&mut output, engine.process_key(KeyInput::from_char(ch)));
    }

    output
}

fn simulate_keys(method: InputMethod, keys: &[KeyInput]) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_input_method(method);
    engine.set_charset(Charset::Unicode);
    engine.set_options(EngineOptions::default());
    let mut output = String::new();

    for key in keys {
        apply_action(&mut output, engine.process_key(*key));
    }

    output
}

fn chars(input: &str) -> Vec<KeyInput> {
    input.chars().map(KeyInput::from_char).collect()
}

#[test]
fn supplied_stress_words_keep_expected_tone_positions() {
    let cases = [
        (InputMethod::Telex, "khuyur", "khuỷu"),
        (InputMethod::Telex, "nguyts", "nguýt"),
        (InputMethod::Telex, "uynhf", "uỳnh"),
        (InputMethod::Telex, "uychs", "uých"),
        (InputMethod::Telex, "uychj", "uỵch"),
        (InputMethod::Telex, "uynhs", "uýnh"),
        (InputMethod::Vni, "khuyu3", "khuỷu"),
        (InputMethod::Vni, "nguyt1", "nguýt"),
        (InputMethod::Vni, "uynh2", "uỳnh"),
        (InputMethod::Vni, "uych1", "uých"),
        (InputMethod::Vni, "uych5", "uỵch"),
        (InputMethod::Vni, "uynh1", "uýnh"),
    ];

    for (method, input, expected) in cases {
        assert_eq!(simulate(method, input), expected, "method={method:?} input={input}");
    }
}

#[test]
fn backspace_recompute_preserves_original_case() {
    let mut keys = chars("To6i1");
    keys.push(KeyInput::plain(KeyCode::Backspace));
    assert_eq!(simulate_keys(InputMethod::Vni, &keys), "Tôi");

    let mut keys = chars("Hoas");
    keys.push(KeyInput::plain(KeyCode::Backspace));
    assert_eq!(simulate_keys(InputMethod::Telex, &keys), "Hoa");
}

#[test]
fn backspace_after_wrong_tone_or_letter_restores_previous_word() {
    let mut keys = chars("nghieng2");
    keys.push(KeyInput::plain(KeyCode::Backspace));
    assert_eq!(simulate_keys(InputMethod::Vni, &keys), "nghieng");

    let mut keys = chars("nghieengxq");
    keys.push(KeyInput::plain(KeyCode::Backspace));
    assert_eq!(simulate_keys(InputMethod::Telex, &keys), "nghiễng");
}

#[test]
fn vni_zero_removes_previous_tone_in_current_word() {
    assert_eq!(simulate(InputMethod::Vni, "nghie6ng0"), "nghieng");
    assert_eq!(simulate(InputMethod::Vni, "di90"), "di");
    assert_eq!(simulate(InputMethod::Vni, "ta8t10"), "tat");
}
