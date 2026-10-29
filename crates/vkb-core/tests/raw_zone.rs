//! Raw-zone protection (spec §A9): punctuation opens a literal word only when a
//! letter follows immediately, never after whitespace, an ellipsis or a digit-dot.

use vkb_core::{apply_action, Engine, EngineOptions, KeyInput, VietnameseEngine};

fn simulate(options: EngineOptions, input: &str) -> String {
    let mut engine = Engine::new(options);
    let mut out = String::new();
    for ch in input.chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        apply_action(&mut out, action, key);
    }
    out
}

#[test]
fn raw_zone_keeps_paths_and_emails_literal() {
    let options = EngineOptions::default();
    assert_eq!(simulate(options, "github.cof"), "github.cof");
    assert_eq!(simulate(options, "x=cof"), "x=cof");
    assert_eq!(simulate(options, "me@cof"), "me@cof");
}

#[test]
fn raw_zone_does_not_fire_after_whitespace() {
    assert_eq!(simulate(EngineOptions::default(), "a. tieengs"), "a. tiếng");
}

#[test]
fn ellipsis_and_digit_dot_do_not_arm_the_raw_zone() {
    assert_eq!(simulate(EngineOptions::default(), "a...cof"), "a...cò");
    assert_eq!(simulate(EngineOptions::default(), "3.cof"), "3.cò");
}

#[test]
fn raw_zone_can_be_disabled() {
    let options = EngineOptions {
        raw_zone: false,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, "github.cof"), "github.cò");
}

#[test]
fn backspace_after_an_opener_restores_the_word() {
    // The opener arms the raw zone; Backspace removes the opener itself, so the
    // next letter must compose again (`28-engine-rulings.md` B4/D8).
    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    for ch in "Tha".chars() {
        let key = KeyInput::char(ch);
        apply_action(&mut out, engine.process_key(key), key);
    }
    let space = KeyInput::char(' ');
    apply_action(&mut out, engine.process_key(space), space);
    let bs = KeyInput::backspace();
    apply_action(&mut out, engine.process_key(bs), bs);
    let s = KeyInput::char('s');
    apply_action(&mut out, engine.process_key(s), s);
    assert_eq!(out, "Thá");
}
