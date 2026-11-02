//! D10 (0.2.9): the stand-alone rule for listed English words is decided by the composed
//! syllable. A listed word whose composition is not a real Vietnamese syllable (the reviewed
//! list in `tests/fixtures/d10_not_vietnamese.txt`) is English at every SCOWL level; a real
//! syllable (`data/real_syllables.rs`, or attested in `vi.txt`) stays Vietnamese.
//! Every case types raw keystrokes through `simulate`.

use vkb_core::{simulate, EngineOptions, InputMethod};

fn telex() -> EngineOptions {
    EngineOptions::default()
}

fn vni() -> EngineOptions {
    EngineOptions { method: InputMethod::Vni, ..EngineOptions::default() }
}

fn check(options: EngineOptions, cases: &[(&str, &str)]) {
    let mut failures = Vec::new();
    for (keys, expected) in cases {
        let got = simulate(options, keys);
        if &got != expected {
            failures.push(format!("keys {keys:?}: got {got:?}, want {expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn junk_compositions_of_listed_words_stay_english_alone_and_after_a_vietnamese_word() {
    // Owner list (2026-11-02): rose->róe, buses->búe, dyes->dýe, rooms->rốm, doors->dố,
    // trees->trế, moons->mốn, soons->sốn, sees->sế, mores->móe, sixes->síe, mixer->mỉe,
    // boxer->bỏe, cursors->cúo, bier->bỉe, corer->cỏe.
    let words = [
        "rose", "buses", "dyes", "rooms", "doors", "trees", "moons", "soons", "sees", "mores", "sixes", "mixer",
        "boxer", "cursors", "bier", "corer", "room", "soon", "door", "more", "tree", "showroom",
    ];
    let mut alone = Vec::new();
    let mut after = Vec::new();
    for w in words {
        alone.push((format!("{w} "), format!("{w} ")));
        after.push((format!("tooi {w} "), format!("tôi {w} ")));
    }
    let alone: Vec<(&str, &str)> = alone.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let after: Vec<(&str, &str)> = after.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    check(telex(), &alone);
    check(telex(), &after);
}

#[test]
fn real_syllables_still_compose_alone_and_after_a_vietnamese_word() {
    // Real Vietnamese readings of listed English words stay Vietnamese.
    check(
        telex(),
        &[
            ("sons ", "són "),
            ("suns ", "sún "),
            ("lair ", "lải "),
            ("chow ", "chơ "),
            ("mar ", "mả "),
            ("gas ", "gá "),
            ("max ", "mã "),
            ("box ", "bõ "),
            ("mix ", "mĩ "),
            ("six ", "sĩ "),
            ("tax ", "tã "),
            ("moon ", "môn "),
            ("tooi sons ", "tôi són "),
            ("tooi lair ", "tôi lải "),
            ("tooi chow ", "tôi chơ "),
            ("tooi mar ", "tôi mả "),
            ("tooi tax ", "tôi tã "),
            ("tooi moon ", "tôi môn "),
        ],
    );
}

#[test]
fn compounds_survive_and_their_parts_stay_english_alone() {
    check(
        telex(),
        &[
            // Compounds keep their Vietnamese reading through the pair and one-word-back rules.
            ("room rar ", "rôm rả "),
            ("room sayr ", "rôm sảy "),
            ("cas tree ", "cá trê "),
            // Alone (or after English) the same keys stay English.
            ("tree ", "tree "),
            ("turn ", "turn "),
            ("vary ", "vary "),
        ],
    );
}

#[test]
fn vni_equivalents_of_the_same_rules() {
    // VNI: a listed English word typed plainly is unchanged; real syllables still compose.
    check(
        vni(),
        &[
            ("rose ", "rose "),
            ("rooms ", "rooms "),
            ("doors ", "doors "),
            ("mixer ", "mixer "),
            ("so1n ", "són "),
            ("to6i so1n ", "tôi són "),
            ("ma3 ", "mả "),
        ],
    );
}
