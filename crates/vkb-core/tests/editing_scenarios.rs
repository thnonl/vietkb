//! Data-driven sweep of real-typing EDITING patterns (Backspace, retype,
//! reopen across separators, typos, prefixes, caps) for Telex and VNI, with
//! spell_check and restore_wrong_word on/off.
//!
//! Script syntax: every char is a key press, `BS` (U+232B) is Backspace.
//! Oracles are mostly relative: "fresh typing of the final word" (a brand new
//! engine fed only the word's keystrokes). Failures are collected and asserted
//! at the end of each generator test so one run lists everything. The full
//! failure list is written to `target/editing_scenarios_<name>.txt`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

const BS: char = '\u{232B}';

#[derive(Clone, Copy)]
struct Cfg {
    vni: bool,
    spell: bool,
    restore: bool,
}

impl Cfg {
    fn all() -> Vec<Cfg> {
        let mut v = Vec::new();
        for vni in [false, true] {
            for spell in [true, false] {
                for restore in [false, true] {
                    v.push(Cfg { vni, spell, restore });
                }
            }
        }
        v
    }
    fn options(&self) -> EngineOptions {
        EngineOptions {
            method: if self.vni { InputMethod::Vni } else { InputMethod::Telex },
            spell_check: self.spell,
            restore_wrong_word: self.restore,
            ..EngineOptions::default()
        }
    }
    fn method(&self) -> &'static str {
        if self.vni {
            "VNI"
        } else {
            "Telex"
        }
    }
    fn label(&self) -> String {
        format!(
            "{} spell={} restore={}",
            self.method(),
            if self.spell { "on" } else { "off" },
            if self.restore { "on" } else { "off" }
        )
    }
}

struct Run {
    out: String,
    panicked: bool,
    bs_violation: Option<String>,
}

fn run(cfg: Cfg, script: &str) -> Run {
    let r = catch_unwind(AssertUnwindSafe(|| {
        let mut e = Engine::new(cfg.options());
        let mut out = String::new();
        let mut viol = None;
        for (i, ch) in script.chars().enumerate() {
            let key = if ch == BS { KeyInput::backspace() } else { KeyInput::char(ch) };
            let before = out.clone();
            let a = e.process_key(key);
            apply_action(&mut out, a, key);
            if ch == BS && viol.is_none() {
                let mut exp = before.clone();
                exp.pop();
                if out != exp {
                    viol = Some(format!("step {i}: screen {before:?} -> {out:?} (expected {exp:?})"));
                }
            }
        }
        (out, viol)
    }));
    match r {
        Ok((out, bs_violation)) => Run { out, panicked: false, bs_violation },
        Err(_) => Run { out: String::new(), panicked: true, bs_violation: None },
    }
}

fn fresh(cfg: Cfg, keys: &str) -> String {
    run(cfg, keys).out
}

fn show(s: &str) -> String {
    s.replace(' ', "\u{2423}")
}

struct Failure {
    scenario: String,
    kind: &'static str,
    cfg: Cfg,
    script: String,
    expected: String,
    actual: String,
}

#[derive(Default)]
struct Sweep {
    total: usize,
    failures: Vec<Failure>,
}

impl Sweep {
    /// Run `script`; `expected` None = invariants only (no panic, BS = pop one char).
    fn check(&mut self, scenario: &str, cfg: Cfg, script: &str, expected: Option<&str>) {
        self.total += 1;
        let r = run(cfg, script);
        if r.panicked {
            self.fail(scenario, "panic", cfg, script, "no panic", "PANIC");
            return;
        }
        if let Some(v) = &r.bs_violation {
            self.fail(scenario, "bs-not-pop", cfg, script, "BS deletes exactly one char", v);
        }
        if let Some(e) = expected {
            if r.out != e {
                self.fail(scenario, "mismatch", cfg, script, e, &r.out);
            }
        }
    }
    fn fail(&mut self, scenario: &str, kind: &'static str, cfg: Cfg, script: &str, e: &str, a: &str) {
        self.failures.push(Failure {
            scenario: scenario.into(),
            kind,
            cfg,
            script: script.into(),
            expected: e.into(),
            actual: a.into(),
        });
    }
}

fn finish(name: &str, sweep: Sweep) {
    let mut groups: BTreeMap<(String, &'static str, &'static str), Vec<&Failure>> = BTreeMap::new();
    for f in &sweep.failures {
        groups.entry((f.scenario.clone(), f.cfg.method(), f.kind)).or_default().push(f);
    }
    let mut report = String::new();
    let _ = writeln!(
        report,
        "=== {name}: {} scenarios run, {} failing ===",
        sweep.total,
        sweep.failures.len()
    );
    for ((scen, method, kind), mut fs) in groups {
        fs.sort_by_key(|f| f.script.chars().count());
        let _ = writeln!(report, "-- [{scen}] {method} {kind}: {} failures; shortest repros:", fs.len());
        let mut seen = std::collections::HashSet::new();
        let mut shown = 0;
        for f in fs {
            if !seen.insert((f.expected.clone(), f.actual.clone())) {
                continue;
            }
            let _ = writeln!(
                report,
                "     keys={} | {} | expected={} | actual={}",
                show(&f.script),
                f.cfg.label(),
                show(&f.expected),
                show(&f.actual)
            );
            shown += 1;
            if shown >= 4 {
                break;
            }
        }
    }
    println!("{report}");
    let mut full = String::new();
    for f in &sweep.failures {
        let _ = writeln!(
            full,
            "[{}] {} {} keys={} expected={} actual={}",
            f.scenario,
            f.kind,
            f.cfg.label(),
            show(&f.script),
            show(&f.expected),
            show(&f.actual)
        );
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join(format!("editing_scenarios_{name}.txt")), full);
    assert!(
        sweep.failures.is_empty(),
        "{name}: {} of {} scenarios failed (see report above / target/editing_scenarios_{name}.txt)",
        sweep.failures.len(),
        sweep.total
    );
}

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

/// (visible, telex keys, vni keys)
const VN_WORDS: &[(&str, &str, &str)] = &[
    ("được", "dduwowcj", "d9u7o7c5"),
    ("người", "nguwowif", "ngu7o7i2"),
    ("không", "khoong", "kho6ng"),
    ("tiếng", "tieengs", "tie6ng1"),
    ("việt", "vieetj", "vie6t5"),
    ("những", "nhuwngx", "nhu7ng4"),
    ("thì", "thif", "thi2"),
    ("quá", "quas", "qua1"),
    ("hòa", "hoaf", "hoa2"),
    ("khuya", "khuya", "khuya"),
    ("gì", "gif", "gi2"),
    ("đêm", "ddeem", "d9e6m"),
    ("đêi", "ddeei", "d9e6i"),
    ("nội", "nooji", "no6i5"),
    ("ông", "oong", "o6ng"),
    ("bạn", "banj", "ban5"),
    ("đẹp", "ddepj", "d9ep5"),
    ("đẹp", "ddejp", "d9e5p"),
    ("nhạt", "nhajt", "nha5t"),
    ("xin", "xin", "xin"),
    ("chào", "chaof", "chao2"),
    ("cảm", "camr", "cam3"),
    ("ơn", "own", "o7n"),
    ("mưa", "muwa", "mu7a"),
    ("trường", "truwowngf", "tru7o7ng2"),
    ("sáng", "sangs", "sang1"),
    ("ngày", "ngayf", "ngay2"),
    ("tuần", "tuaanf", "tua6n2"),
    ("yêu", "yeeu", "ye6u"),
    ("muốn", "muoons", "muo6n1"),
    ("nước", "nuwowcs", "nu7o7c1"),
    ("biển", "bieenr", "bie6n3"),
    ("thương", "thuwowng", "thu7o7ng"),
    ("quyết", "quyeets", "quye6t1"),
    ("giúp", "giups", "giup1"),
    ("học", "hojc", "hoc5"),
    ("học", "hocj", "hoc5"),
    ("hôm", "hoom", "ho6m"),
    ("nhà", "nhaf", "nha2"),
    ("đi", "ddi", "d9i"),
    ("là", "laf", "la2"),
    ("rồi", "roofi", "ro6i2"),
    ("ăn", "awn", "a8n"),
    ("lắm", "lawms", "la8m1"),
];

/// Non-Vietnamese / code-ish tokens (same keys for both methods).
const OTHER: &[&str] = &[
    "class", "file", "test", "will", "google", "access", "http", "window", "flower", "show",
    "press", "school", "world", "github", "code", "rust", "java", "json", "fix", "print",
    "address", "user", "pass", "wifi", "zoom", "dog", "data", "desk", "docs", "dev", "debug",
    "default", "deploy", "abc", "xyz", "C:\\", "a1", "2x", "x2", "hello", "win10", "abc123", "ok2", "v2", "c++", "e-mail", "P2P", "3d", "dd9", "d9", "tex9",
];

#[derive(Clone)]
struct Tok {
    vis: String,
    keys: String,
}

fn cap1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn variants(w: (&str, &str, &str), vni: bool) -> Vec<Tok> {
    let keys = if vni { w.2 } else { w.1 };
    let mut v = vec![Tok { vis: w.0.to_string(), keys: keys.to_string() }];
    for t in [
        Tok { vis: cap1(w.0), keys: cap1(keys) },
        Tok { vis: w.0.to_uppercase(), keys: keys.to_uppercase() },
    ] {
        if !v.iter().any(|x| x.keys == t.keys) {
            v.push(t);
        }
    }
    v
}

fn vn_tokens(cfg: Cfg) -> Vec<Tok> {
    VN_WORDS.iter().flat_map(|w| variants(*w, cfg.vni)).collect()
}

/// Vietnamese tokens whose fresh typing yields exactly the expected text.
fn good_vn(cfg: Cfg) -> Vec<Tok> {
    vn_tokens(cfg).into_iter().filter(|t| fresh(cfg, &t.keys) == t.vis).collect()
}

fn other_tokens() -> Vec<Tok> {
    OTHER.iter().map(|s| Tok { vis: String::new(), keys: s.to_string() }).collect()
}

fn all_tokens(cfg: Cfg) -> Vec<Tok> {
    let mut v = good_vn(cfg);
    v.extend(other_tokens());
    v
}

fn bs(n: usize) -> String {
    std::iter::repeat(BS).take(n).collect()
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

fn split_at_char(s: &str, i: usize) -> (&str, &str) {
    let idx = s.char_indices().nth(i).map(|(b, _)| b).unwrap_or(s.len());
    s.split_at(idx)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Word table itself must type correctly fresh: otherwise the table or the
/// engine is wrong. Words failing here are excluded from the other sweeps.
#[test]
fn sanity_fresh_typing_of_word_table() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        for t in vn_tokens(cfg) {
            sw.check("fresh-word", cfg, &t.keys, Some(&t.vis));
        }
    }
    finish("sanity", sw);
}

/// 1. type X, Backspace until empty (optionally through a space), type Y == fresh(Y).
#[test]
fn s1_retype_after_full_delete() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        let xs = all_tokens(cfg);
        let ys: Vec<Tok> = good_vn(cfg)
            .into_iter()
            .enumerate()
            .filter(|(i, _)| i % 3 == 0)
            .map(|(_, t)| t)
            .collect();
        for x in &xs {
            let shown = fresh(cfg, &x.keys);
            for y in &ys {
                let exp = fresh(cfg, &y.keys);
                let s = format!("{}{}{}", x.keys, bs(chars(&shown)), y.keys);
                sw.check("retype-after-delete", cfg, &s, Some(&exp));
                let s = format!("{} {}{}", x.keys, bs(chars(&shown) + 1), y.keys);
                sw.check("retype-after-delete-through-space", cfg, &s, Some(&exp));
            }
        }
    }
    finish("s1", sw);
}

fn raw_of_screen(cfg: Cfg, p: &str) -> Option<String> {
    let mut raw = String::new();
    for c in p.chars() {
        match c {
            'đ' => raw.push_str(if cfg.vni { "d9" } else { "dd" }),
            'Đ' => raw.push_str(if cfg.vni { "D9" } else { "DD" }),
            c if c.is_ascii_alphabetic() && !"aeiouyAEIOUY".contains(c) => raw.push(c),
            _ => return None,
        }
    }
    Some(raw)
}

/// 2. type X (optionally + space), Backspace k times, then complete a Vietnamese
/// word W from the surviving consonant-only prefix == fresh(W).
#[test]
fn s2_delete_to_prefix_then_continue() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        let words = good_vn(cfg);
        for x in all_tokens(cfg) {
            for with_space in [false, true] {
                let base = if with_space { format!("{} ", x.keys) } else { x.keys.clone() };
                let screen = run(cfg, &base).out;
                for k in 1..=chars(&screen) {
                    let p = run(cfg, &format!("{base}{}", bs(k))).out;
                    let Some(raw) = raw_of_screen(cfg, &p) else { continue };
                    if raw.is_empty() {
                        continue;
                    }
                    for w in &words {
                        if w.keys.starts_with(&raw) && w.keys.len() > raw.len() {
                            let rest = &w.keys[raw.len()..];
                            let s = format!("{base}{}{rest}", bs(k));
                            let name = if with_space { "prefix-continue-via-space" } else { "prefix-continue" };
                            sw.check(name, cfg, &s, Some(&w.vis));
                        }
                    }
                }
            }
        }
    }
    finish("s2", sw);
}

/// Owner-reported bug: non-VN word, (space,) Backspace to its first consonant
/// (or everything), then `9e6i` / `d9e6i` must give `đêi` (Telex: `ddeei`).
#[test]
fn s2b_owner_vni_9e6i_after_deleted_word() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        let (tail_full, tail_after_d) = if cfg.vni { ("d9e6i", "9e6i") } else { ("ddeei", "deei") };
        for x in other_tokens() {
            let shown = fresh(cfg, &x.keys);
            let n = chars(&shown);
            for with_space in [false, true] {
                let base = if with_space { format!("{} ", x.keys) } else { x.keys.clone() };
                let total = n + usize::from(with_space);
                let exp = fresh(cfg, tail_full);
                sw.check("owner-9e6i-full-delete", cfg, &format!("{base}{}{tail_full}", bs(total)), Some(&exp));
                if shown.starts_with('d') && total > 1 {
                    sw.check(
                        "owner-9e6i-to-first-d",
                        cfg,
                        &format!("{base}{}{tail_after_d}", bs(total - 1)),
                        Some(&exp),
                    );
                }
            }
        }
    }
    finish("s2b", sw);
}

/// 3 + 5. Wrong/incomplete word committed with a separator, Backspace the
/// separator, type the remaining keys: == fresh(whole keys). Every split point.
fn reopen_generic(name: &str, sep: char, tag: &str) {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        for t in all_tokens(cfg) {
            let exp = fresh(cfg, &t.keys);
            for i in 1..=chars(&t.keys) {
                let (a, b) = split_at_char(&t.keys, i);
                let s = format!("{a}{sep}{BS}{b}");
                sw.check(tag, cfg, &s, Some(&exp));
                let s = format!("{a}{sep}{BS}{sep}{BS}{b}");
                sw.check(tag, cfg, &s, Some(&exp));
            }
        }
    }
    finish(name, sw);
}

#[test]
fn s5_reopen_across_space() {
    reopen_generic("s5_space", ' ', "reopen-space");
}

/// Spec for `,` and `.` reopening is less explicit than for space; reported
/// separately in case the expected value is a spec question.
#[test]
fn s5b_reopen_across_comma() {
    reopen_generic("s5b_comma", ',', "reopen-comma");
}

#[test]
fn s5c_reopen_across_period() {
    reopen_generic("s5c_period", '.', "reopen-period");
}

/// 5. `a b`, Backspace through b and the space, then type c.
#[test]
fn s5d_two_words_backspace_through_separator() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        let words = good_vn(cfg);
        for (n, a) in words.iter().enumerate().step_by(2) {
            let b = &words[(n * 7 + 3) % words.len()];
            let c = &words[(n * 11 + 5) % words.len()];
            let base = format!("{} {}", a.keys, b.keys);
            let lb = chars(&b.vis);
            // delete all of b: `a c` ; delete b and the space: `ac` glued
            sw.check(
                "two-words-delete-b",
                cfg,
                &format!("{base}{}{}", bs(lb), c.keys),
                Some(&format!("{} {}", a.vis, c.vis)),
            );
            // Backspacing the space too reopens `a`, so what `c` becomes is
            // spec-dependent: invariants only (no panic, BS pops one char).
            sw.check("two-words-delete-b-and-space", cfg, &format!("{base}{}{}", bs(lb + 1), c.keys), None);
        }
    }
    finish("s5d", sw);
}

/// 4. Symbol before a word, and a mistyped digit/symbol deleted before a word.
#[test]
fn s4_symbol_before_word() {
    let mut sw = Sweep::default();
    let symbols = ["(", "\"", "[", "@", "#", "-", "$"];
    let mistakes = ["(", "\"", "1", "2", "3", "7", "9", "12", "2024", "C:\\", "a1"];
    for cfg in Cfg::all() {
        for t in good_vn(cfg) {
            for pre in symbols {
                let exp = format!("{pre}{}", t.vis);
                sw.check("symbol-prefix", cfg, &format!("{pre}{}", t.keys), Some(&exp));
                // prefix, space, Backspace the space, then the word glued
                sw.check("prefix-space-bs-word", cfg, &format!("{pre} {BS}{}", t.keys), Some(&exp));
            }
            for pre in mistakes {
                // typed by mistake, deleted, then the word
                sw.check(
                    "mistake-prefix-deleted",
                    cfg,
                    &format!("{pre}{}{}", bs(chars(pre)), t.keys),
                    Some(&t.vis),
                );
            }
        }
    }
    finish("s4", sw);
}

/// UNCLEAR spec: A9 says a word that BEGINS with a digit is `non_viet`
/// (literal), and `C:\` arms the raw zone by design. The task text suggests
/// `1duowcj` / `2d9e6i` should still compose; listed separately, not guessed.
#[test]
fn u1_digit_glued_and_codezone_prefix_unclear() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        for t in good_vn(cfg) {
            for pre in ["1", "2", "3", "7", "9", "12", "2024", "C:\\"] {
                let exp = format!("{pre}{}", t.vis);
                sw.check("UNCLEAR-glued-prefix", cfg, &format!("{pre}{}", t.keys), Some(&exp));
            }
            for pre in ["1", "2", "9", "12"] {
                let exp = format!("{pre}{}", t.vis);
                sw.check("UNCLEAR-digit-space-bs-word", cfg, &format!("{pre} {BS}{}", t.keys), Some(&exp));
            }
        }
        if !cfg.vni {
            sw.check("UNCLEAR-a1-bs-tone", cfg, &format!("a1{BS}s"), Some("á"));
            sw.check("UNCLEAR-a1-bs-tone", cfg, &format!("x2{BS}{BS}as"), Some("á"));
        }
    }
    finish("u1", sw);
}

/// 6. Typo letter then Backspace at every position, then the real keys.
#[test]
fn s6_typo_then_backspace_every_position() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        for t in all_tokens(cfg) {
            let exp = fresh(cfg, &t.keys);
            for i in 0..=chars(&t.keys) {
                let (a, b) = split_at_char(&t.keys, i);
                for typo in ['k', 'b', 'v'] {
                    sw.check("typo-bs", cfg, &format!("{a}{typo}{BS}{b}"), Some(&exp));
                    sw.check("typo2-bs", cfg, &format!("{a}{typo}{typo}{BS}{BS}{b}"), Some(&exp));
                }
            }
        }
    }
    finish("s6", sw);
}

fn tone_keys(cfg: Cfg) -> &'static [char] {
    if cfg.vni {
        &['1', '2', '3', '4', '5']
    } else {
        &['s', 'f', 'r', 'x', 'j']
    }
}

/// 6. Tone twice (toggle off), Backspace, then another tone == fresh(base + tone2).
#[test]
fn u2_double_tone_toggle_then_bs_then_other_tone_unclear() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        for t in good_vn(cfg) {
            let last = t.keys.chars().last().unwrap();
            let lower = last.to_ascii_lowercase();
            if !tone_keys(cfg).contains(&lower) {
                continue;
            }
            let base: String = t.keys.chars().take(chars(&t.keys) - 1).collect();
            for &t2 in tone_keys(cfg) {
                if t2 == lower {
                    continue;
                }
                let t2 = if last.is_ascii_uppercase() { t2.to_ascii_uppercase() } else { t2 };
                let exp = fresh(cfg, &format!("{base}{t2}"));
                sw.check("UNCLEAR-tone-toggle-bs-other-tone", cfg, &format!("{base}{last}{last}{BS}{t2}"), Some(&exp));
            }
        }
    }
    finish("u2", sw);
}

/// 6. Extra tone key mid-word then Backspace. What the screen should be is
/// spec-unclear (BS pops one char of the already-toned word), so only the
/// invariants are checked (no panic, BS pops exactly one char).
#[test]
fn s6c_extra_tone_then_bs_invariants() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        for t in good_vn(cfg) {
            for i in 1..=chars(&t.keys) {
                let (a, b) = split_at_char(&t.keys, i);
                for &tone in tone_keys(cfg) {
                    sw.check("extra-tone-bs", cfg, &format!("{a}{tone}{BS}{b}"), None);
                }
            }
        }
    }
    finish("s6c", sw);
}

/// 7. Caps are covered by Title/UPPER variants in every generator; this adds
/// the owner's `Tha` example directly.
#[test]
fn s7_caps_reopen_examples() {
    let mut sw = Sweep::default();
    for cfg in Cfg::all() {
        let tone = if cfg.vni { "1" } else { "s" };
        sw.check("caps-Tha-space-bs-tone", cfg, &format!("Tha {BS}{tone}"), Some("Thá"));
        sw.check("caps-THA-space-bs-tone", cfg, &format!("THA {BS}{}", tone.to_uppercase()), Some("THÁ"));
        sw.check("caps-tha-space-bs-tone", cfg, &format!("tha {BS}{tone}"), Some("thá"));
    }
    finish("s7", sw);
}
