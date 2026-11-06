//! Owner bug (2026-11-06): VNI `vi22` + space shows `vi22` (restore_wrong_word);
//! Backspace back to `vi` and `2` must give `vì`, not `vi2`. The same for every
//! word and key: type a word whose last key is a tone or mark key, double that
//! key (escape), add a space, delete back to the word without its key, retype the
//! key. The oracle is fresh typing of the word. Both methods, `restore_wrong_word`
//! on and off.

use vkb_core::{apply_action, simulate, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

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

fn options(vni: bool, restore: bool) -> EngineOptions {
    EngineOptions {
        method: if vni { InputMethod::Vni } else { InputMethod::Telex },
        restore_wrong_word: restore,
        ..EngineOptions::default()
    }
}

fn press(engine: &mut Engine, out: &mut String, key: KeyInput) {
    let action = engine.process_key(key);
    apply_action(out, action, key);
}

#[test]
fn retyped_key_after_deleting_an_escaped_tail_composes() {
    let mut failures = Vec::new();
    let mut ran = 0;
    for vni in [false, true] {
        for restore in [false, true] {
            let opts = options(vni, restore);
            for &(telex, vni_keys) in WORDS {
                let keys = if vni { vni_keys } else { telex };
                let key = keys.chars().last().unwrap();
                let tone_key = if vni { key.is_ascii_digit() } else { "sfrxjwaeod".contains(key) };
                if !tone_key {
                    continue;
                }
                let base = &keys[..keys.len() - 1];
                let want = simulate(opts, keys);
                let root = simulate(opts, base);
                let mut engine = Engine::new(opts);
                let mut out = String::new();
                for ch in format!("{keys}{key} ").chars() {
                    press(&mut engine, &mut out, KeyInput::char(ch));
                }
                // Delete back to the word without its key, as a person would.
                let mut guard = 0;
                while out != root && out.chars().count() > root.chars().count() && guard < 20 {
                    press(&mut engine, &mut out, KeyInput::backspace());
                    guard += 1;
                }
                if out != root {
                    continue; // the escaped text does not start with the root
                }
                ran += 1;
                press(&mut engine, &mut out, KeyInput::char(key));
                if out != want {
                    failures.push(format!("vni={vni} restore={restore} {keys}{key}+space, BS to {root:?}, {key}: got {out:?}, want {want:?}"));
                }
            }
        }
    }
    assert!(ran >= 40, "too few cases ran: {ran}");
    assert!(failures.is_empty(), "{} of {ran} failed:
{}", failures.len(), failures.join("
"));
}
