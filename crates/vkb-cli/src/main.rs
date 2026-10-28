use anyhow::Result;
use clap::{Parser, Subcommand};
use vkb_core::{Engine, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

#[derive(Parser)]
#[command(name = "vkb", about = "VietKB headless tools", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Type a string through the engine and print the resulting actions.
    Simulate {
        #[arg(long, default_value = "telex")]
        method: String,
        /// Raw keystrokes, e.g. "tieengs Vieetj"
        text: String,
    },
    /// Run the engine over a corpus and report timing.
    Bench {
        /// Number of keys to process.
        #[arg(long, default_value_t = 100_000)]
        keys: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Simulate { method, text } => simulate(&method, &text),
        Command::Bench { keys } => bench(keys),
    }
}

fn options_for(method: &str) -> EngineOptions {
    EngineOptions {
        method: InputMethod::from_str(method).unwrap_or(InputMethod::Telex),
        ..EngineOptions::default()
    }
}

fn simulate(method: &str, text: &str) -> Result<()> {
    let mut engine = Engine::new(options_for(method));
    for ch in text.chars() {
        let action = engine.process_key(KeyInput::char(ch));
        match action {
            EngineAction::Noop => print!("{ch}"),
            EngineAction::Commit(s) | EngineAction::Replace { insert: s, .. } => print!("{s}"),
            EngineAction::Swallow => {}
            EngineAction::ToggleEnabled => println!("\n<toggle>"),
            EngineAction::Reject => println!("\n<reject>"),
        }
    }
    println!();
    println!("-- buffer: {}", engine.buffer().iter().collect::<String>());
    Ok(())
}

fn bench(keys: usize) -> Result<()> {
    // Sprint 1 replaces this with a criterion bench; the CLI version exists so the
    // engine can be measured on any machine without a bench harness.
    let mut engine = Engine::new(EngineOptions::default());
    let _ = &mut engine;
    let _ = keys;
    anyhow::bail!("bench is wired once the engine state machine lands (sprint 1)")
}
