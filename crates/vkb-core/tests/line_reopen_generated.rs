//! Generated edit scenarios for the line history (Backspace reopen across
//! separators), Telex and VNI, seeded and deterministic.
//!
//! A case types a short sentence of common Vietnamese words with separators,
//! presses Backspace to cut the line, then types a continuation (a tone or
//! modifier key, or a word). The oracle is fresh typing: the screen after the
//! cut must equal `simulate` of the kept keystrokes, and the final screen must
//! equal `simulate` of the kept keystrokes followed by the continuation.
//!
//! Cuts are taken on boundaries only. A cut inside a word leaves a partial word
//! whose keystroke ownership is not well defined, so those cases are counted as
//! skipped and not run.

use vkb_core::{apply_action, simulate, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

/// Telex and VNI key sequences for common Vietnamese words (display forms are
/// not needed: each word's display length is taken from fresh typing).
const WORDS: &[(&str, &str)] = &[
    ("Tha", "Tha"),
    ("tieengs", "tie6ng1"),
    ("vieetj", "vie6t5"),
    ("nam", "nam"),
    ("nguwowif", "nguo7i2"),
    ("khoongf", "kho6ng2"),
    ("laf", "la2"),
    ("cos", "co1"),
    ("vaf", "va2"),
    ("dduwowcj", "d9uo7c5"),
    ("chaof", "chao2"),
    ("trowif", "tro7i2"),
    ("ddepj", "d9e5p"),
    ("ddi", "d9i"),
    ("awn", "a8n"),
    ("minhf", "minh2"),
    ("ngafy", "nga2y"),
    ("tooi", "toi6"),
    ("banj", "ba5n"),
    ("hocj", "ho5c"),
    ("sachs", "sa1ch"),
    ("nhaf", "nha2"),
    ("dduwowngf", "d9uo7ng2"),
    ("xanh", "xanh"),
    ("ddor", "d9o3"),
    ("mowis", "mo7i1"),
    ("aams", "a6m1"),
    ("ddaayf", "d9a6y2"),
    ("quas", "qua1"),
    ("hoom", "ho6m"),
    ("nay", "nay"),
    ("phowr", "pho73"),
    ("toi", "toi"),
    ("ban", "ban"),
    ("viet", "viet"),
    ("bowf", "bo72"),
    ("tuwj", "tu75"),
    ("hoas", "hoa1"),
    ("ddaau", "d9a6u"),
    ("cuoois", "cuo6i1"),
    ("ddeen", "d9e6n"),
    ("gioongf", "gio2ng"),
    ("lamf", "lam2"),
];

/// Separators the app can type between words (each character is one press).
const SEPARATORS: &[&str] = &[" ", ", ", ". ", "  ", "; ", "\t", ","];

/// Continuations that add a tone or modifier to the reopened word.
const TELEX_TONES: &[&str] = &["s", "f", "r", "x", "j", "w", "aa", "ee", "oo", "dd"];
const VNI_TONES: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9"];

const CASES: usize = 2000;
const SEED: u64 = 0x5EED_1EE7_0F20_26AB;

/// splitmix64: a small deterministic generator, no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
}

fn options(vni: bool) -> EngineOptions {
    EngineOptions {
        method: if vni { InputMethod::Vni } else { InputMethod::Telex },
        ..EngineOptions::default()
    }
}

fn press(engine: &mut Engine, out: &mut String, key: KeyInput) {
    let action = engine.process_key(key);
    apply_action(out, action, key);
}

fn type_keys(engine: &mut Engine, out: &mut String, keys: &str) {
    for ch in keys.chars() {
        press(engine, out, KeyInput::char(ch));
    }
}

fn backspaces(engine: &mut Engine, out: &mut String, n: usize) {
    for _ in 0..n {
        press(engine, out, KeyInput::backspace());
    }
}

/// One generated sentence: words `1..=n` and separators `1..=n` (the last one is
/// the trailing separator and may be empty).
struct Sentence {
    words: Vec<(&'static str, usize)>, // (keys, display chars)
    seps: Vec<&'static str>,
}

impl Sentence {
    fn random(rng: &mut Rng, vni: bool) -> Self {
        let n = 2 + rng.below(4);
        let mut words = Vec::with_capacity(n);
        for _ in 0..n {
            let (telex, v) = WORDS[rng.below(WORDS.len())];
            let keys = if vni { v } else { telex };
            let display = simulate(options(vni), keys).chars().count();
            words.push((keys, display));
        }
        let mut seps: Vec<&'static str> = (0..n - 1).map(|_| SEPARATORS[rng.below(SEPARATORS.len())]).collect();
        seps.push(if rng.chance(30) { SEPARATORS[rng.below(SEPARATORS.len())] } else { "" });
        Self { words, seps }
    }

    /// Keystrokes of words `1..=j` with the separators between them, and the
    /// separator after word `j` when `with_sep`.
    fn kept(&self, j: usize, with_sep: bool) -> String {
        let mut out = String::new();
        for i in 0..j {
            out.push_str(self.words[i].0);
            if i + 1 < j || with_sep {
                out.push_str(self.seps[i]);
            }
        }
        out
    }

    /// The whole typed line.
    fn typed(&self) -> String {
        let n = self.words.len();
        let mut out = String::new();
        for i in 0..n {
            out.push_str(self.words[i].0);
            out.push_str(self.seps[i]);
        }
        out
    }

    /// Backspace presses that delete everything after word `j`, counting the
    /// separator after `j` when `with_sep_deleted` (one press per character).
    fn presses_after(&self, j: usize, with_sep_deleted: bool) -> usize {
        let n = self.words.len();
        let mut presses = if with_sep_deleted { self.seps[j - 1].chars().count() } else { 0 };
        for i in j..n {
            presses += self.words[i].1 + self.seps[i].chars().count();
        }
        presses
    }
}

#[derive(Default)]
struct Tally {
    run: usize,
    skipped: usize,
    failures: Vec<String>,
}

fn run_case(sentence: &Sentence, vni: bool, j: usize, delete_sep: bool, cont: &str, tally: &mut Tally) {
    let opts = options(vni);
    let kept = sentence.kept(j, !delete_sep);
    let presses = sentence.presses_after(j, delete_sep);
    let typed = sentence.typed();
    let expected_screen = simulate(opts, &kept);
    // A Tab never reopens (owner ruling 2026-11-02), and the line is forgotten
    // at it. Deleting the separator after word `j` brings `j` back only when no
    // Tab sits among the separators from there on; otherwise the continuation is
    // a fresh word after the kept text.
    let reopens = delete_sep && sentence.seps[j - 1..].iter().all(|sep| !sep.contains('\t'));
    let expected_final = if delete_sep && !reopens {
        format!("{expected_screen}{}", simulate(opts, cont))
    } else {
        simulate(opts, &format!("{kept}{cont}"))
    };

    let mut engine = Engine::new(opts);
    let mut out = String::new();
    type_keys(&mut engine, &mut out, &typed);
    backspaces(&mut engine, &mut out, presses);
    let after_cut = out.clone();
    type_keys(&mut engine, &mut out, cont);
    tally.run += 1;
    if after_cut != expected_screen {
        tally.failures.push(format!(
            "cut: typed {typed:?} presses {presses} delete_sep {delete_sep}: screen {after_cut:?}, fresh {expected_screen:?}"
        ));
    } else if out != expected_final {
        tally.failures.push(format!(
            "continue: typed {typed:?} cut {presses} (kept {kept:?}) + {cont:?}: got {out:?}, fresh {expected_final:?}"
        ));
    }
}

#[test]
fn generated_line_reopen_cases_match_fresh_typing() {
    let mut rng = Rng(SEED);
    let mut tally = Tally::default();
    let mut generated = 0usize;
    while generated < CASES {
        generated += 1;
        let vni = generated % 2 == 0;
        let sentence = Sentence::random(&mut rng, vni);
        let n = sentence.words.len();
        let j = 1 + rng.below(n - 1);
        let tones = if vni { VNI_TONES } else { TELEX_TONES };
        let cont = if rng.chance(50) {
            let tone = tones[rng.below(tones.len())];
            if rng.chance(50) {
                let (telex, v) = WORDS[rng.below(WORDS.len())];
                format!("{tone}{}", if vni { v } else { telex })
            } else {
                tone.to_string()
            }
        } else {
            let (telex, v) = WORDS[rng.below(WORDS.len())];
            (if vni { v } else { telex }).to_string()
        };
        match rng.below(10) {
            // Cut on a boundary, deleting the separator: the word before comes back.
            0..=3 => run_case(&sentence, vni, j, true, &cont, &mut tally),
            // Cut on a boundary, keeping the separator: a fresh word follows.
            4..=7 => run_case(&sentence, vni, j, false, &cont, &mut tally),
            // Cut inside word j+1: the partial word's keystrokes are not well
            // defined, so the case is skipped rather than given a guessed oracle.
            _ => tally.skipped += 1,
        }
    }
    eprintln!(
        "line reopen generated: {} run, {} skipped (cut inside a word), {} failures",
        tally.run,
        tally.skipped,
        tally.failures.len()
    );
    assert!(
        tally.failures.is_empty(),
        "{} of {} cases failed; first ones:\n{}",
        tally.failures.len(),
        tally.run,
        tally.failures.iter().take(25).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn the_pattern_delete_into_word_n_then_tone_then_rest() {
    // Named pattern from the owner's QA: delete back into an earlier word, add a
    // tone or modifier to it, then retype the rest. Every cut depth, both methods.
    let mut tally = Tally::default();
    for vni in [false, true] {
        let keys: [&str; 4] = if vni {
            ["tie6ng1", "vie6t5", "nam", "ho"]
        } else {
            ["tieengs", "vieetj", "nam", "ho"]
        };
        let words = keys
            .iter()
            .map(|&k| (k, simulate(options(vni), k).chars().count()))
            .collect();
        let sentence = Sentence { words, seps: vec![" ", " ", " ", " "] };
        let tones: &[&str] = if vni { &["1", "6", "5"] } else { &["s", "j", "aa"] };
        for j in 1..=3 {
            for &tone in tones {
                for cont in [tone.to_string(), format!("{tone}ho")] {
                    run_case(&sentence, vni, j, true, &cont, &mut tally);
                    run_case(&sentence, vni, j, false, &cont, &mut tally);
                }
            }
        }
    }
    assert!(tally.failures.is_empty(), "pattern failures: {:#?}", tally.failures);
}
