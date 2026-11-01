//! Headless tools: `simulate` and `bench`.
//!
//! `vkb simulate --method telex "tieengs Vieetj"` prints the produced text;
//! `vkb bench` measures charter P1 latency percentiles and throughput.

use std::time::Instant;

use anyhow::Result;
use clap::{Parser, Subcommand};
use vkb_core::{
    apply_action, simulate, AppKind, Charset, Engine, EngineOptions, InputMethod, KeyInput,
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
        /// Stop treating `j f w z` as consonants, so `w` alone becomes `ư`.
        #[arg(long)]
        no_jfwz_consonant: bool,
        /// `Tự nhận biết tiếng Anh và code` (D10). On by default; this flag only says so.
        #[arg(long)]
        auto_english: bool,
        /// Turn D10 off: exactly the 0.2.7 behaviour.
        #[arg(long, conflicts_with = "auto_english")]
        no_auto_english: bool,
        /// The app in front: `neutral` or `code` (editor / terminal profile).
        #[arg(long, default_value = "neutral")]
        app: String,
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
        /// Turn `Tự nhận biết tiếng Anh và code` (D10) off, to compare.
        #[arg(long)]
        no_auto_english: bool,
        /// The app in front: `neutral` or `code`.
        #[arg(long, default_value = "neutral")]
        app: String,
        /// What to type: `vi`, `en`, `code` or `mixed`.
        #[arg(long, default_value = "vi")]
        text: String,
        /// Soak mode: type this file (keystrokes, one paragraph per line, each
        /// ended with Enter) instead of `--text`, `--loops` times.
        #[arg(long)]
        text_file: Option<String>,
        /// Soak mode: how many times to type the file through one engine.
        #[arg(long, default_value_t = 1)]
        loops: usize,
        /// Print peak and current RSS before typing, after the first loop and at the end.
        #[arg(long)]
        report_rss: bool,
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
            no_jfwz_consonant,
            auto_english: _,
            no_auto_english,
            app,
            actions,
            text,
        } => {
            let options = EngineOptions {
                // `EngineOptions::modern_tone` is the inverse of the old flag name.
                modern_tone: !old_tone_style,
                spell_check: !no_spell_check,
                free_tone: !no_free_tone,
                jfwz_as_consonant: !no_jfwz_consonant,
                auto_restore_english: !no_auto_english,
                app_kind: parse_app(&app)?,
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
        Command::Bench { keys, method, json, no_auto_english, app, text, text_file, loops, report_rss } => {
            if let Some(path) = text_file {
                let options = EngineOptions {
                    auto_restore_english: !no_auto_english,
                    app_kind: parse_app(&app)?,
                    ..options_for(&method)?
                };
                return soak(&path, loops.max(1), options, report_rss);
            }
            bench(keys, &method, json, !no_auto_english, &app, &text)
        }
    }
}

/// Peak and current resident set size in MiB (`None` where unsupported).
fn rss_mib() -> (Option<f64>, Option<f64>) {
    #[cfg(unix)]
    {
        // SAFETY: `getrusage` only writes into the zeroed struct we pass.
        let peak = unsafe {
            let mut usage: libc::rusage = std::mem::zeroed();
            (libc::getrusage(libc::RUSAGE_SELF, &mut usage) == 0).then(|| {
                // macOS reports bytes, Linux kilobytes.
                let raw = usage.ru_maxrss as f64;
                if cfg!(target_os = "macos") {
                    raw / 1048576.0
                } else {
                    raw / 1024.0
                }
            })
        };
        let current = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()
            .and_then(|out| String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().ok())
            .map(|kib| kib / 1024.0);
        (peak, current)
    }
    #[cfg(not(unix))]
    {
        (None, None)
    }
}

fn print_rss(label: &str) {
    let (peak, current) = rss_mib();
    let show = |v: Option<f64>| v.map_or("n/a".to_string(), |v| format!("{v:.2} MiB"));
    println!("rss {label:<12} peak {}  current {}", show(peak), show(current));
}

/// Soak: one engine, the file typed `loops` times. Latency goes into a fixed
/// histogram so the measurement itself allocates nothing per key.
fn soak(path: &str, loops: usize, options: EngineOptions, report_rss: bool) -> Result<()> {
    let text = std::fs::read_to_string(path)?;
    let mut keys: Vec<KeyInput> = Vec::new();
    for line in text.lines().filter(|l| !l.is_empty()) {
        keys.extend(line.chars().map(KeyInput::char));
        keys.push(KeyInput::enter());
    }
    anyhow::ensure!(!keys.is_empty(), "{path} has no keystrokes");
    const BUCKETS: usize = 200_001; // 1 ns each up to 200 us
    let mut histogram = vec![0u64; BUCKETS];
    let mut max_ns = 0u64;
    let mut engine = Engine::new(options);
    let mut screen = String::with_capacity(8192);
    if report_rss {
        print_rss("start");
    }
    let started = Instant::now();
    for round in 1..=loops {
        for key in &keys {
            let key_start = Instant::now();
            let action = engine.process_key(*key);
            let elapsed = key_start.elapsed().as_nanos() as u64;
            max_ns = max_ns.max(elapsed);
            histogram[(elapsed as usize).min(BUCKETS - 1)] += 1;
            if screen.len() > 4096 {
                screen.clear();
            }
            apply_action(&mut screen, action, *key);
        }
        if report_rss && round == 1 {
            print_rss("after loop 1");
        }
    }
    let total = started.elapsed();
    let count: u64 = histogram.iter().sum();
    let pick = |quantile: f64| -> u64 {
        let target = (count as f64 * quantile).ceil() as u64;
        let mut seen = 0u64;
        for (ns, n) in histogram.iter().enumerate() {
            seen += n;
            if seen >= target {
                return ns as u64;
            }
        }
        0
    };
    let (p50, p99) = (pick(0.50), pick(0.99));
    println!("file:      {path}");
    println!("keys:      {} x {} loops = {count}", keys.len(), loops);
    println!("app:       {:?}  auto-english: {}", options.app_kind, options.auto_restore_english);
    println!("p50:       {p50} ns");
    println!("p99:       {p99} ns   budget {P99_BUDGET_NS} ns");
    println!("max:       {max_ns} ns");
    println!("throughput:{:>10.0} keys/sec", count as f64 / total.as_secs_f64());
    let (caps, spilled) = engine.buffer_footprint();
    println!("engine buffers {caps:?} spilled: {spilled}");
    if report_rss {
        print_rss("end");
    }
    if p99 > P99_BUDGET_NS {
        std::process::exit(1);
    }
    Ok(())
}

fn parse_method(input: &str) -> Result<InputMethod> {
    InputMethod::from_str(input).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown method {input:?}; expected one of {}",
            slugs(InputMethod::ALL.iter().map(|m| m.as_str()))
        )
    })
}

fn parse_app(input: &str) -> Result<AppKind> {
    match input.trim().to_ascii_lowercase().as_str() {
        "neutral" => Ok(AppKind::Neutral),
        "code" => Ok(AppKind::Code),
        other => anyhow::bail!("unknown app {other:?}; expected neutral or code"),
    }
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

fn bench(keys: usize, method: &str, json: bool, auto_english: bool, app: &str, text: &str) -> Result<()> {
    if keys == 0 {
        anyhow::bail!("--keys must be greater than zero");
    }
    let options = EngineOptions {
        auto_restore_english: auto_english,
        app_kind: parse_app(app)?,
        ..options_for(method)?
    };
    let corpus = match text {
        "vi" => {
            "tieengs Vieetj dduwowngf hoaf thuyeenf quas giaf nguyeenx khoong hoir sasci \
             nguowif tuowngr Vieetj hoafng tieengs"
        }
        "en" => {
            "user use were here there order why twitter what where with world swift strong \
             script schema through export query the quick brown fox jumps over"
        }
        "code" => {
            "user_id getValue foo.bar std::io a->b item2 utf8 HttpClient self.fieldd \
             let mut x = vec.len(); return Ok(x);"
        }
        "mixed" => {
            "tooi hay mix nhacj chung vowis tieengs oonf user_id cos nhaf tuooir teen \
             if x == nos then return Vieetj Nam laf nos"
        }
        other => anyhow::bail!("unknown text {other:?}; expected vi, en, code or mixed"),
    };
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
