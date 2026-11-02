//! Generated QA: every syllable in `data/vi.txt` (4 048 common Vietnamese
//! syllables), typed the ways people type it, in Telex and VNI, with the
//! default options (D10 on).
//!
//! Oracle: the canonical typing (marks where they belong, tone last) must give
//! the syllable's letters, marks and tone (placement is pinned elsewhere, in
//! `tone_placement.rs`); every other way of typing it must then give exactly
//! the canonical output: capitals, the tone typed early, a tone changed, a tone
//! removed and typed again, a tone typed and removed (`To6i40` = `Tôi`), marks
//! typed after the word, and the word inside a sentence.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use vkb_core::{simulate, Charset, EngineOptions, InputMethod};

const VI: &str = include_str!("../data/vi.txt");

/// Keys left out, each for a reason:
/// - corpus typos, not words: `suơng`, `tuơng`, `rỡa`, `qia`;
/// - `o` alone: Telex `of`/`oj` stay as typed (EVKey compatibility, `COMPAT_RESTORE_RAW`).
const KNOWN: &[&str] = &["suo+ng", "tuo+ng", "ro+a4", "qia", "o", "o1", "o2", "o5", "o+", "o+2", "o+5", "o+m2", "o^2", "o^m2", "o^n2", "o^p5", "oa2", "om2"];

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    None,
    Hat,
    Breve,
    Horn,
    Bar,
}

struct Syllable {
    key: String,
    letters: Vec<(char, Mark)>,
    tone: usize,
}

fn parse(key: &str) -> Option<Syllable> {
    let mut letters: Vec<(char, Mark)> = Vec::new();
    let mut tone = 0;
    for ch in key.chars() {
        match ch {
            'D' => letters.push(('d', Mark::Bar)),
            '^' => letters.last_mut()?.1 = Mark::Hat,
            '(' => letters.last_mut()?.1 = Mark::Breve,
            '+' => letters.last_mut()?.1 = Mark::Horn,
            '1'..='5' => tone = ch as usize - '0' as usize,
            'a'..='z' => letters.push((ch, Mark::None)),
            _ => return None,
        }
    }
    Some(Syllable { key: key.to_string(), letters, tone })
}

/// Corpus noise that breaks the c/k, g/gh, ng/ngh spelling rules (`ceo`, `ko`,
/// `ngieng`): the spell check rejects these on purpose.
fn odd_spelling(s: &Syllable) -> bool {
    let plain: String = s.letters.iter().map(|&(c, _)| c).collect();
    let front = |rest: &str| rest.starts_with(['e', 'i', 'y']);
    if let Some(rest) = plain.strip_prefix("ngh").or_else(|| plain.strip_prefix("gh")) {
        return !front(rest);
    }
    if let Some(rest) = plain.strip_prefix("ng") {
        return front(rest);
    }
    if let Some(rest) = plain.strip_prefix('k') {
        return !front(rest);
    }
    if plain.starts_with("ch") {
        return false;
    }
    if let Some(rest) = plain.strip_prefix('c') {
        return front(rest);
    }
    if plain.starts_with("gi") {
        return false;
    }
    plain.strip_prefix('g').is_some_and(|rest| rest.starts_with('e'))
}

fn is_vowel(ch: char) -> bool {
    matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

fn tone_key(method: InputMethod, tone: usize) -> char {
    match method {
        InputMethod::Telex => ['z', 's', 'f', 'r', 'x', 'j'][tone],
        _ => ['0', '1', '2', '3', '4', '5'][tone],
    }
}

fn mark_keys(method: InputMethod, ch: char, mark: Mark) -> &'static str {
    match (method, mark) {
        (_, Mark::None) => "",
        (InputMethod::Telex, Mark::Hat) => match ch {
            'a' => "a",
            'e' => "e",
            _ => "o",
        },
        (InputMethod::Telex, Mark::Breve | Mark::Horn) => "w",
        (InputMethod::Telex, Mark::Bar) => "d",
        (_, Mark::Hat) => "6",
        (_, Mark::Breve) => "8",
        (_, Mark::Horn) => "7",
        (_, Mark::Bar) => "9",
    }
}

/// Letters with their marks typed in place.
fn letters_inline(method: InputMethod, s: &Syllable) -> String {
    let mut out = String::new();
    for &(ch, mark) in &s.letters {
        out.push(ch);
        out.push_str(mark_keys(method, ch, mark));
    }
    out
}

/// Expected NFD text without the tone, lowercase.
fn toneless_nfd(s: &Syllable) -> String {
    let mut out = String::new();
    for &(ch, mark) in &s.letters {
        match mark {
            Mark::Bar => out.push('đ'),
            Mark::None => out.push(ch),
            Mark::Hat => {
                out.push(ch);
                out.push('\u{302}');
            }
            Mark::Breve => {
                out.push(ch);
                out.push('\u{306}');
            }
            Mark::Horn => {
                out.push(ch);
                out.push('\u{31b}');
            }
        }
    }
    out
}

const TONE_MARKS: [char; 5] = ['\u{301}', '\u{300}', '\u{309}', '\u{303}', '\u{323}'];

/// `None` when `out` has the syllable's letters, marks and tone; otherwise why not.
fn oracle(s: &Syllable, out: &str) -> Option<String> {
    let nfd = String::from_utf8(Charset::UnicodeNfd.encode(&out.to_lowercase())).ok()?;
    let mut tones = Vec::new();
    let rest: String = nfd
        .chars()
        .filter(|c| {
            let t = TONE_MARKS.iter().position(|m| m == c);
            if let Some(t) = t {
                tones.push(t + 1);
            }
            t.is_none()
        })
        .collect();
    let want_tones: Vec<usize> = if s.tone == 0 { vec![] } else { vec![s.tone] };
    if rest != toneless_nfd(s) || tones != want_tones {
        return Some(format!("letters/tone wrong: {out:?}"));
    }
    None
}

fn capitalise(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn options(method: InputMethod) -> EngineOptions {
    EngineOptions { method, ..EngineOptions::default() }
}

/// The ways of typing `s` (category, keys typed, expected output given the
/// canonical output `canon`).
fn variants(method: InputMethod, s: &Syllable, canon: &str) -> Vec<(&'static str, String, String)> {
    let inline = letters_inline(method, s);
    let tone = if s.tone == 0 { String::new() } else { tone_key(method, s.tone).to_string() };
    let remove = tone_key(method, 0);
    let mut v = Vec::new();
    let caps = capitalise(canon);

    v.push(("caps-first", capitalise(&format!("{inline}{tone}")), caps.clone()));
    v.push(("all-caps", format!("{inline}{tone}").to_uppercase(), canon.to_uppercase()));

    // The tone typed right after the first vowel, and after every later key.
    if s.tone != 0 {
        let chars: Vec<char> = inline.chars().collect();
        if let Some(first_vowel) = chars.iter().position(|&c| is_vowel(c)) {
            for at in first_vowel + 1..chars.len() {
                let mut keys: String = chars[..at].iter().collect();
                keys.push_str(&tone);
                keys.extend(&chars[at..]);
                v.push(("tone-early", keys, canon.to_string()));
            }
        }
        // A different tone first (one the syllable can take), then the right one.
        let stop = matches!(s.letters.last(), Some(('c' | 'p' | 't', _)))
            || s.letters.ends_with(&[('c', Mark::None), ('h', Mark::None)]);
        let other = match (stop, s.tone) {
            (true, 1) => 5,
            (true, _) => 1,
            (false, 1) => 2,
            (false, _) => 1,
        };
        v.push((
            "tone-change",
            format!("{inline}{}{tone}", tone_key(method, other)),
            canon.to_string(),
        ));
        v.push((
            "tone-change-caps",
            capitalise(&format!("{inline}{}{tone}", tone_key(method, other))),
            caps.clone(),
        ));
        // Tone removed, then typed again.
        v.push(("tone-remove-retype", format!("{inline}{tone}{remove}{tone}"), canon.to_string()));
    } else {
        // A tone typed by mistake, then removed (`To6i40` = `Tôi`).
        for t in 1..=5 {
            let keys = format!("{inline}{}{remove}", tone_key(method, t));
            v.push(("tone-then-remove", keys.clone(), canon.to_string()));
            v.push(("tone-then-remove-caps", capitalise(&keys), caps.clone()));
        }
    }

    // Marks typed after the whole word (VNI style `toi6`, Telex `tuongw`).
    let has_mark = s.letters.iter().any(|&(_, m)| m != Mark::None);
    if has_mark {
        let plain: String = s.letters.iter().map(|&(c, _)| c).collect();
        let marks: String = match method {
            InputMethod::Telex => {
                // Only the `w` marks are typed late in Telex; the doubles stay in place.
                if s.letters.iter().any(|&(_, m)| matches!(m, Mark::Hat | Mark::Bar)) {
                    String::new()
                } else {
                    "w".to_string()
                }
            }
            _ => {
                let mut m = String::new();
                for &(c, mark) in &s.letters {
                    let k = mark_keys(method, c, mark);
                    if !k.is_empty() && !m.contains(k) {
                        m.push_str(k);
                    }
                }
                m
            }
        };
        if !marks.is_empty() {
            v.push(("marks-last", format!("{plain}{marks}{tone}"), canon.to_string()));
            v.push(("marks-last-tone-first", format!("{plain}{tone}{marks}"), canon.to_string()));
        }
    }

    // Inside a sentence.
    let (toi, toi_out) = match method {
        InputMethod::Telex => ("tooi", "tôi"),
        _ => ("to6i", "tôi"),
    };
    let word = format!("{inline}{tone}");
    v.push(("after-toi", format!("{toi} {word} "), format!("{toi_out} {canon} ")));
    v.push(("sentence-start", format!("{toi}. {} ", capitalise(&word)), format!("{toi_out}. {caps} ")));
    v.push(("before-comma", format!("{word}, {toi} "), format!("{canon}, {toi_out} ")));
    v.push(("before-period", format!("{word}."), format!("{canon}.")));
    v
}

#[test]
fn every_common_syllable_types_every_way() {
    let mut failures: BTreeMap<(String, &'static str), Vec<String>> = BTreeMap::new();
    let mut total = 0usize;
    let mut canon_bad = 0usize;
    for method in [InputMethod::Telex, InputMethod::Vni] {
        let name = if method == InputMethod::Telex { "Telex" } else { "VNI" };
        for line in VI.lines() {
            let Some(key) = line.split_whitespace().next() else { continue };
            let Some(s) = parse(key) else { continue };
            if odd_spelling(&s) || KNOWN.contains(&key) {
                continue;
            }
            let inline = letters_inline(method, &s);
            let tone = if s.tone == 0 { String::new() } else { tone_key(method, s.tone).to_string() };
            let canon_keys = format!("{inline}{tone} ");
            let canon_out = simulate(options(method), &canon_keys);
            let canon = canon_out.strip_suffix(' ').unwrap_or(&canon_out).to_string();
            total += 1;
            let canon_english = canon_out == canon_keys
                && vkb_core::lexicon::english_word(canon_keys.trim_end().as_bytes()).is_some();
            if canon_english {
                // D10 by design (`is`, `ces`): the canonical keys are a listed English word.
                continue;
            }
            if let Some(why) = oracle(&s, &canon) {
                canon_bad += 1;
                failures
                    .entry((name.to_string(), "canonical"))
                    .or_default()
                    .push(format!("{} keys {canon_keys:?}: {why}", s.key));
                continue;
            }
            for (category, keys, want) in variants(method, &s, &canon) {
                total += 1;
                // A bare word gets a space to end it, like the canonical run.
                let (keys, want) = if keys.ends_with([' ', '.']) {
                    (keys, want)
                } else {
                    (format!("{keys} "), format!("{want} "))
                };
                let got = simulate(options(method), &keys);
                // D10 by design: the keys spell a listed English word and went back to them.
                let word = keys.trim_end_matches([' ', '.']);
                let last = word.rsplit(' ').next().unwrap_or(word);
                let english = (got == keys || got.ends_with(&format!(" {last} ")))
                    && ((word.bytes().all(|b| b.is_ascii_alphabetic())
                        && vkb_core::lexicon::english_word(word.to_ascii_lowercase().as_bytes()).is_some())
                        || vkb_core::lexicon::is_acronym(last.as_bytes()));
                if got != want && !english {
                    failures
                        .entry((name.to_string(), category))
                        .or_default()
                        .push(format!("{keys:?} got {got:?} want {want:?}"));
                }
            }
        }
    }
    let mut report = String::new();
    let failed: usize = failures.values().map(Vec::len).sum();
    let _ = writeln!(report, "syllable sweep: {total} typings, {failed} wrong ({canon_bad} canonical)");
    for ((method, category), list) in &failures {
        let _ = writeln!(report, "\n## {method} {category}: {} wrong", list.len());
        for item in list {
            let _ = writeln!(report, "  {item}");
        }
    }
    let path = std::env::var("SWEEP_REPORT").unwrap_or_default();
    if !path.is_empty() {
        std::fs::write(&path, &report).expect("write report");
    }
    let summary: String = report.lines().filter(|l| !l.starts_with("  ")).collect::<Vec<_>>().join("\n");
    println!("{summary}");
    assert_eq!(failed, 0, "{summary}");
}
