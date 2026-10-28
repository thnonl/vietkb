//! Headless tools: `simulate` and `bench`.
//!
//! `vkb simulate --method telex "tieengs Vieetj"` prints the produced text;
//! `vkb bench` measures charter P1 latency percentiles and throughput.

use std::time::Instant;

use anyhow::Result;
use clap::{Parser, Subcommand};
use vkb_core::{
    apply_action, simulate, Charset, Engine, EngineOptions, InputMethod, KeyInput,
    VietnameseEngine,
};

/// Charter P1 budget for `process_key`.
const P99_BUDGET_NS: u64 = 50_000;

#[derive(Parser)]
#[command(name = "vkb", about = "VietKB headless tools", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Type a string through the engine and print the produced text.
    Simulate {
        #[arg(long, default_value = "telex")]
        method: String,
        #[arg(long, default_value = "unicode")]
        charset: String,
        /// Old tone style (kiểu cũ): `hoà` instead of `hòa`.
        #[arg(long)]
        old_tone_style: bool,
        /// Disable `Kiểm tra chính tả`.
        #[arg(long)]
        no_spell_check: bool,
        /// Disable `Đặt dấu tự do` (the tone sticks to the vowel it hit).
        #[arg(long)]
        no_free_tone: bool,
        /// Print every EngineAction instead of the produced text.
        #[arg(long)]
        actions: bool,
        /// Raw keystrokes, e.g. "tieengs Vieetj".
        text: String,
    },
    /// Measure process_key latency (charter P1) and throughput.
    Bench {
        /// Number of keys to process.
        #[arg(long, default_value_t = 100_000)]
        keys: usize,
        #[arg(long, default_value = "telex")]
        method: String,
        /// Machine-readable output.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Simulate {
            method,
            charset,
            old_tone_style,
            no_spell_check,
            no_free_tone,
            actions,
            text,
        } => {
            let options = EngineOptions {
                // `EngineOptions::modern_tone` is the inverse of the old flag name.
                modern_tone: !old_tone_style,
                spell_check: !no_spell_check,
                free_tone: !no_free_tone,
                ..options_for(&method)?
            };
            if actions {
                print_actions(options, &text);
                Ok(())
            } else {
                let charset = parse_charset(&charset)?;
                let options = EngineOptions { charset, ..options };
                println!("{}", simulate(options, &text));
                Ok(())
            }
        }
        Command::Bench { keys, method, json } => bench(keys, &method, json),
    }
}

fn parse_method(input: &str) -> Result<InputMethod> {
    InputMethod::from_str(input).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown method {input:?}; expected one of {}",
            slugs(InputMethod::ALL.iter().map(|m| m.as_str()))
        )
    })
}

fn parse_charset(input: &str) -> Result<Charset> {
    Charset::from_str(input).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown charset {input:?}; expected one of {}",
            slugs(Charset::ALL.iter().map(|c| c.as_str()))
        )
    })
}

fn slugs<'a>(items: impl Iterator<Item = &'a str>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

fn options_for(method: &str) -> Result<EngineOptions> {
    Ok(EngineOptions {
        method: parse_method(method)?,
        ..EngineOptions::default()
    })
}

fn print_actions(options: EngineOptions, text: &str) {
    let mut engine = Engine::new(options);
    let mut out = String::new();
    for ch in text.chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        println!("{ch:?} -> {action:?}");
        apply_action(&mut out, action, key);
    }
    println!("-- text: {out}");
    println!("-- raw:  {}", engine.buffer().iter().collect::<String>());
}

fn bench(keys: usize, method: &str, json: bool) -> Result<()> {
    if keys == 0 {
        anyhow::bail!("--keys must be greater than zero");
    }
    let options = options_for(method)?;
    let corpus = "tieengs Vieetj dduwowngf hoaf thuyeenf quas giaf nguyeenx khoong hoir sasci \
                  nguowif tuowngr Vieetj hoafng tieengs";
    let chars: Vec<char> = corpus.chars().cycle().take(keys).collect();

    let mut engine = Engine::new(options);
    for ch in chars.iter().take(4096) {
        std::hint::black_box(engine.process_key(KeyInput::char(*ch)));
    }
    engine.reset();

    let mut samples: Vec<u64> = Vec::with_capacity(keys);
    let started = Instant::now();
    for ch in &chars {
        let key_start = Instant::now();
        let action = engine.process_key(KeyInput::char(*ch));
        let elapsed = key_start.elapsed().as_nanos() as u64;
        std::hint::black_box(action);
        samples.push(elapsed);
    }
    let total = started.elapsed();

    samples.sort_unstable();
    let pick = |quantile: f64| -> u64 {
        let idx = ((samples.len() - 1) as f64 * quantile).round() as usize;
        samples[idx.min(samples.len() - 1)]
    };
    let p50 = pick(0.50);
    let p95 = pick(0.95);
    let p99 = pick(0.99);
    let max = *samples.last().unwrap_or(&0);
    let per_second = keys as f64 / total.as_secs_f64();

    if json {
        println!(
            "{{\"keys\":{keys},\"method\":\"{}\",\"p50_ns\":{p50},\"p95_ns\":{p95},\
             \"p99_ns\":{p99},\"max_ns\":{max},\"keys_per_sec\":{per_second:.0},\
             \"p99_budget_ns\":{P99_BUDGET_NS},\"p1_pass\":{}}}",
            options.method.as_str(),
            p99 <= P99_BUDGET_NS
        );
    } else {
        println!("keys:      {keys}");
        println!("method:    {}", options.method.as_str());
        println!("p50:       {p50} ns  ({:.3} µs)", p50 as f64 / 1000.0);
        println!("p95:       {p95} ns  ({:.3} µs)", p95 as f64 / 1000.0);
        println!("p99:       {p99} ns  ({:.3} µs)   budget {P99_BUDGET_NS} ns", p99 as f64 / 1000.0);
        println!("max:       {max} ns  ({:.3} µs)", max as f64 / 1000.0);
        println!("throughput:{per_second:>10.0} keys/sec");
        println!(
            "P1 (p99 ≤ {} µs): {}",
            P99_BUDGET_NS / 1000,
            if p99 <= P99_BUDGET_NS { "PASS" } else { "FAIL" }
        );
    }

    if p99 > P99_BUDGET_NS {
        std::process::exit(1);
    }
    Ok(())
}
