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

#[test]
fn left_side_compounds_keep_the_vietnamese_word() {
    // The English-looking word after a Vietnamese word is Vietnamese in its compound
    // (owner: a word has partners on both sides; `room` is `rôm` in `nổi rôm`, `mọc rôm`, `bị rôm`).
    check(
        telex(),
        &[
            ("tooi bij nooir room ", "tôi bị nổi rôm "),
            ("tooi mojc room ", "tôi mọc rôm "),
            ("tooi bij room ", "tôi bị rôm "),
            ("tooi ddanhs vary ", "tôi đánh vảy "),
            ("tooi caoj vary ", "tôi cạo vảy "),
            ("ngawns churn ", "ngắn chủn "),
            // The right side: `vảy rồng`, `chủn ngủn`.
            ("vary roongf ", "vảy rồng "),
            ("churn ngunr ", "chủn ngủn "),
        ],
    );
}

#[test]
fn churn_is_english_alone_and_after_a_vietnamese_word() {
    // `churn` (chủn) is compound-only: `ngắn chủn`, `chủn ngủn` keep it Vietnamese.
    check(telex(), &[("churn ", "churn "), ("tooi churn ", "tôi churn ")]);
}

/// Keys (`simulate_keys`) for a text with `\x08` standing for Backspace.
fn keys_of(text: &str) -> Vec<vkb_core::KeyInput> {
    text.chars()
        .map(|ch| if ch == '\u{8}' { vkb_core::KeyInput::backspace() } else { vkb_core::KeyInput::char(ch) })
        .collect()
}

#[test]
fn backspace_over_the_separator_drops_the_back_state() {
    // `room` is restored at the space; Backspace takes the space back, so `rar` joins `room`
    // as the same word, exactly as with D10 off (main: `roomrar`).
    let on = vkb_core::simulate_keys(EngineOptions::default(), &keys_of("room \u{8}rar "));
    let off = vkb_core::simulate_keys(
        EngineOptions { auto_restore_english: false, ..EngineOptions::default() },
        &keys_of("room \u{8}rar "),
    );
    assert_eq!(off, "roomrar ", "main behaviour");
    assert_eq!(on, off, "D10 must join the word like main");
    // The Vietnamese compound with the space in place is unchanged.
    let on = vkb_core::simulate_keys(EngineOptions::default(), &keys_of("room rar "));
    assert_eq!(on, "rôm rả ");
}
