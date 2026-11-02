//! D10 (0.2.9) regression: the one-word-back rewrite and the English pair rule must not
//! turn a common Vietnamese reading into English because an English pair happens to match
//! (`tôi sẽ in` typed as `tooi sex in`, `tôi cả the` typed as `tooi car the`).
//! Every case types raw keystrokes through `simulate` and compares the text on screen.

use vkb_core::{simulate, EngineOptions};

fn check(cases: &[(&str, &str)]) {
    let mut failures = Vec::new();
    for (keys, expected) in cases {
        let got = simulate(EngineOptions::default(), keys);
        if &got != expected {
            failures.push(format!("keys {keys:?}: got {got:?}, want {expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn common_vietnamese_reading_survives_an_english_pair() {
    // `sẽ` (se4) is common; `sex in` is an English pair. The Vietnamese reading wins.
    check(&[
        ("tooi sex in ", "tôi sẽ in "),
        ("tooi sex on ", "tôi sẽ on "),
        ("tooi sex at ", "tôi sẽ at "),
        ("tooi sex of ", "tôi sẽ of "),
        ("tooi sex it ", "tôi sẽ it "),
        ("tooi sex is ", "tôi sẽ is "),
        ("tooi sex and ", "tôi sẽ and "),
        ("tooi sex for ", "tôi sẽ for "),
        ("tooi sex with ", "tôi sẽ with "),
        ("tooi sex the ", "tôi sẽ the "),
        ("tooi sex you ", "tôi sẽ you "),
        ("caau nay sex in ", "câu nay sẽ in "),
        ("mai sex in ", "mai sẽ in "),
        ("ngay mai sex in ", "ngay mai sẽ in "),
    ]);
    // `cả` (car), `bên` (been): common readings followed by function words.
    check(&[
        ("tooi car in ", "tôi cả in "),
        ("tooi car the ", "tôi cả the "),
        ("tooi been in ", "tôi bên in "),
        ("tooi been the ", "tôi bên the "),
    ]);
}

#[test]
fn rare_reading_still_gives_the_english_pair() {
    // `see` (sê, count 2) and the English pairs stay English.
    check(&[
        ("see you soon ", "see you soon "),
        ("meeting room ", "meeting room "),
        ("the room is ready ", "the room is ready "),
        ("check mail box ", "check mail box "),
        ("tooi see it ", "tôi see it "),
        ("tooi seen it ", "tôi seen it "),
        ("tooi down in ", "tôi down in "),
    ]);
}

#[test]
fn explicit_back_pairs_are_trusted_even_with_a_common_reading() {
    // `max` (mã, count 31) and `six` (sĩ) are on the explicit list; `max speed` is English.
    check(&[
        ("Max speed ", "Max speed "),
        ("tooi max speed ", "tôi max speed "),
        ("Box công ", "Bõ công "),
    ]);
}
