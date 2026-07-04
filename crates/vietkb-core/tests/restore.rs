use vietkb_core::{EngineAction, KeyCode, KeyInput, SimpleVietnameseEngine, VietnameseEngine};

fn apply(output: &mut String, action: EngineAction) {
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

fn type_chars(engine: &mut SimpleVietnameseEngine, output: &mut String, input: &str) {
    for ch in input.chars() {
        apply(output, engine.process_key(KeyInput::from_char(ch)));
    }
}

fn backspace(engine: &mut SimpleVietnameseEngine, output: &mut String) {
    apply(
        output,
        engine.process_key(KeyInput::plain(KeyCode::Backspace)),
    );
}

#[test]
fn backspace_recomputes_transformed_word_from_raw() {
    let mut engine = SimpleVietnameseEngine::new();
    let mut output = String::new();

    type_chars(&mut engine, &mut output, "tieengs");
    assert_eq!(output, "tiếng");

    backspace(&mut engine, &mut output);
    assert_eq!(output, "tiêng");

    backspace(&mut engine, &mut output);
    assert_eq!(output, "tiên");
}

#[test]
fn restore_word_reverts_to_raw_keystrokes() {
    let mut engine = SimpleVietnameseEngine::new();
    let mut output = String::new();

    type_chars(&mut engine, &mut output, "tieengs");
    assert_eq!(output, "tiếng");

    apply(&mut output, engine.restore_word());
    assert_eq!(output, "tieengs");
}
