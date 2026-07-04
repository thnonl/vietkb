use vietkb_core::{Charset, EngineAction, KeyInput, SimpleVietnameseEngine, VietnameseEngine};

fn simulate(charset: Charset, input: &str) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_charset(charset);
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
fn unicode_charset_keeps_unicode_output() {
    assert_eq!(simulate(Charset::Unicode, "tieengs"), "tiếng");
}

#[test]
fn legacy_charsets_convert_engine_output() {
    assert_eq!(simulate(Charset::VniWindows, "tieengs"), "tieng");
    assert_eq!(simulate(Charset::Tcvn3, "tieengs"), "tiÕng");
}
