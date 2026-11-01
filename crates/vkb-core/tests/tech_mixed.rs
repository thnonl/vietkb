//! Long Vietnamese technology passage full of English loanwords, quoted English
//! sentences and quoted code (owner case: type it, and check nothing leaks).
//!
//! Fixtures in `tests/data/`:
//! - `tech_mixed_vi.src.txt`      the text the user means, one paragraph per line (expected output)
//! - `tech_mixed_vi.txt`          the same as Telex keystrokes (`scripts/gen_tech_fixture.py`)
//! - `tech_mixed_vi_vni.txt`      the same as VNI keystrokes
//!
//! Each paragraph is typed and ended with Enter. `PINNED` lists the few places
//! where the ruling is ambiguous and the engine's output is pinned instead (none now).

use vkb_core::{apply_action, AppKind, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

pub const SRC: &str = include_str!("data/tech_mixed_vi.src.txt");
pub const TELEX: &str = include_str!("data/tech_mixed_vi.txt");
pub const VNI: &str = include_str!("data/tech_mixed_vi_vni.txt");

/// `(profile, paragraph index, text the engine produces instead of the source, why)`
/// for spots where the correct output is ambiguous. None at present: every
/// profile matches the source text.
pub const PINNED: &[(&str, usize, &str, &str)] = &[];

fn options(method: InputMethod, app: AppKind) -> EngineOptions {
    EngineOptions { method, app_kind: app, ..EngineOptions::default() }
}

/// Type every paragraph of `keys` (one per line) and end it with Enter. Returns
/// the text on screen, one string per paragraph.
pub fn type_passage(engine: &mut Engine, keys: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in keys.lines() {
        let mut screen = String::new();
        for ch in line.chars() {
            let key = KeyInput::char(ch);
            let action = engine.process_key(key);
            apply_action(&mut screen, action, key);
        }
        let key = KeyInput::enter();
        let action = engine.process_key(key);
        apply_action(&mut screen, action, key);
        out.push(screen.trim_end_matches(['\r', '\n']).to_string());
    }
    out
}

fn expected() -> Vec<String> {
    SRC.lines().filter(|l| !l.trim().is_empty()).map(str::to_string).collect()
}

fn check(label: &str, method: InputMethod, app: AppKind, keys: &str) {
    let mut engine = Engine::new(options(method, app));
    let got = type_passage(&mut engine, keys);
    let want = expected();
    assert_eq!(got.len(), want.len(), "{label}: paragraph count");
    let mut diffs = Vec::new();
    for (i, (g, w)) in got.iter().zip(&want).enumerate() {
        let pinned = PINNED.iter().find(|(l, idx, _, _)| *l == label && *idx == i);
        let w = pinned.map_or(w.as_str(), |(_, _, text, _)| *text);
        if g != w {
            diffs.push(format!("[{label}] paragraph {i}\n  got:  {g}\n  want: {w}"));
        }
    }
    assert!(diffs.is_empty(), "{} paragraph(s) differ:\n{}", diffs.len(), diffs.join("\n"));
}

#[test]
fn telex_neutral_app() {
    check("telex-neutral", InputMethod::Telex, AppKind::Neutral, TELEX);
}

#[test]
fn telex_code_app() {
    check("telex-code", InputMethod::Telex, AppKind::Code, TELEX);
}

#[test]
fn vni_neutral_app() {
    check("vni-neutral", InputMethod::Vni, AppKind::Neutral, VNI);
}

#[test]
fn vni_code_app() {
    check("vni-code", InputMethod::Vni, AppKind::Code, VNI);
}

#[test]
fn the_passage_is_long_enough() {
    let words: usize = SRC.lines().map(|l| l.split_whitespace().count()).sum();
    assert!(words >= 2500, "{words} words");
    assert_eq!(TELEX.lines().count(), SRC.lines().filter(|l| !l.trim().is_empty()).count());
    assert_eq!(VNI.lines().count(), TELEX.lines().count());
}
