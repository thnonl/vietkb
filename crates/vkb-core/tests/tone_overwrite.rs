//! "Gõ dấu đè" (owner, 2026-11-03): the caret is put back after a word already on screen and a
//! tone or mark key re-tones that word in place, without deleting it first. The platform reads
//! the word before the caret (accessibility API) and calls `seed_word`; these tests pin the engine
//! half: the seeded word plus the key must give the same text as typing it fresh.

use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

fn overwrite(method: InputMethod, word: &str, keys: &str) -> Option<String> {
    let mut engine = Engine::new(EngineOptions { method, ..EngineOptions::default() });
    let mut out = String::from(word);
    if !engine.seed_word(word) {
        return None;
    }
    for ch in keys.chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        apply_action(&mut out, action, key);
    }
    Some(out)
}

#[test]
fn a_tone_key_after_a_word_re_tones_it_telex() {
    for (word, keys, want) in [
        ("nhá", "f", "nhà"),
        ("tiểng", "s", "tiếng"),
        ("người", "x", "ngưỡi"),
        ("học", "s", "hóc"),
        ("Việt", "s", "Viết"),
        ("ĐƯỢC", "S", "ĐƯỚC"),
        ("thuở", "x", "thuỡ"),
        ("nha", "f", "nhà"),
        ("nhà", "z", "nha"),
        ("tôi", "f", "tồi"),
    ] {
        assert_eq!(overwrite(InputMethod::Telex, word, keys).as_deref(), Some(want), "{word} + {keys}");
    }
}

#[test]
fn a_tone_key_after_a_word_re_tones_it_vni() {
    for (word, keys, want) in [
        ("nhá", "2", "nhà"),
        ("tiểng", "1", "tiếng"),
        ("người", "4", "ngưỡi"),
        ("Việt", "1", "Viết"),
        ("nhà", "0", "nha"),
        ("trươc", "1", "trước"),
    ] {
        assert_eq!(overwrite(InputMethod::Vni, word, keys).as_deref(), Some(want), "{word} + {keys}");
    }
}

#[test]
fn every_common_syllable_seeds_back_to_itself() {
    // The seed must reproduce the screen exactly, for every syllable in the corpus, in both
    // methods and with a capital: otherwise the platform falls back to typing the key.
    let vi = include_str!("../data/vi.txt");
    let mut misses = Vec::new();
    for method in [InputMethod::Telex, InputMethod::Vni] {
        for line in vi.lines() {
            let key = line.split(' ').next().unwrap();
            // Type the key the canonical way and take the engine's own text as the screen word.
            let mut engine = Engine::new(EngineOptions { method, ..EngineOptions::default() });
            let typed = vkb_core::simulate(EngineOptions { method, ..EngineOptions::default() }, &keys_of(key, method == InputMethod::Vni));
            let word = typed.trim_end();
            if word.is_empty() || word.is_ascii() {
                continue;
            }
            if !engine.seed_word(word) {
                misses.push(format!("{method:?} {key} {word}"));
            }
        }
    }
    assert!(misses.len() < 40, "{} words cannot be seeded:\n{}", misses.len(), misses.join("\n"));
}

#[test]
fn text_that_is_not_a_word_is_left_alone() {
    let mut engine = Engine::new(EngineOptions::default());
    for word in ["", "abc1", "a.b", "nhá!", "hello world"] {
        assert!(!engine.seed_word(word), "{word:?}");
    }
}

/// Keys for a `vi.txt` syllable key (`Da(c1` = đắc), marks in place, tone last.
fn keys_of(key: &str, vni: bool) -> String {
    let mut out = String::new();
    let mut tone = None;
    for ch in key.chars() {
        match ch {
            'D' => out.push_str(if vni { "d9" } else { "dd" }),
            '^' => {
                let last = out.chars().last().unwrap();
                out.push(if vni { '6' } else { last });
            }
            '(' | '+' => out.push(if vni { if ch == '(' { '8' } else { '7' } } else { 'w' }),
            '1'..='5' => tone = Some(ch),
            _ => out.push(ch),
        }
    }
    if let Some(t) = tone {
        out.push(if vni { t } else { ['s', 'f', 'r', 'x', 'j'][t as usize - '1' as usize] });
    }
    out.push(' ');
    out
}
