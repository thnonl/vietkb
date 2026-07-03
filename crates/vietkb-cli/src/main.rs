use vietkb_core::{EngineAction, KeyInput, SimpleVietnameseEngine, VietnameseEngine};
use vietkb_linux_common::LinuxEnvironment;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("usage: vietkb-cli <simulate|doctor> [text]");
        std::process::exit(2);
    };

    match command.as_str() {
        "simulate" => simulate(args.collect::<Vec<_>>().join(" ")),
        "doctor" => doctor(),
        _ => {
            eprintln!("unknown command: {command}");
            std::process::exit(2);
        }
    }
}

fn simulate(input: String) {
    let mut engine = SimpleVietnameseEngine::new();
    let mut output = String::new();

    for ch in input.chars() {
        let action = engine.process_key(KeyInput::from_char(ch));
        apply_action(&mut output, action);
    }

    println!("{output}");
}

fn doctor() {
    let env = LinuxEnvironment::detect_from(|key| std::env::var(key).ok());
    println!("session={:?}", env.session);
    println!("framework={:?}", env.preferred_framework());
}

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
