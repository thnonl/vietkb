//! Macro feature matrix across input methods.
use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, MacroTable, VietnameseEngine, MAX_WORD};

fn keys(input: &str) -> Vec<KeyInput> { input.chars().map(KeyInput::char).collect() }
fn run(method: InputMethod, trigger: &str, expansion: &str, boundary: KeyInput, enabled: bool, case_follow: bool) -> String {
    let mut engine = Engine::new(EngineOptions { method, macro_enabled: enabled, macro_case_follow: case_follow, ..EngineOptions::default() });
    engine.set_macros(MacroTable::from_pairs([(trigger, expansion)]));
    let mut out = String::new();
    for key in keys(trigger).into_iter().chain([boundary]) { apply_action(&mut out, engine.process_key(key), key); }
    out
}

#[test]
fn macros_expand_at_space_and_enter_for_both_methods() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::char(' '), true, true), "Việt Nam ");
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::enter(), true, true), "Việt Nam\r");
    }
}

#[test]
fn digit_trigger_expands_for_vni() {
    assert_eq!(run(InputMethod::Vni, "vn1", "Vietnam", KeyInput::char(' '), true, true), "Vietnam ");
}

#[test]
fn disabled_macros_do_not_expand() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::char(' '), false, true), "vn ");
    }
}

#[test]
fn macro_case_follow_on_telex() {
    assert_eq!(run(InputMethod::Telex, "Btw", "by the way", KeyInput::char(' '), true, true), "By the way ");
}

#[test]
fn macro_case_follow_off_telex() {
    assert_eq!(run(InputMethod::Telex, "Btw", "by the way", KeyInput::char(' '), true, false), "by the way ");
}

#[test]
fn macro_case_follow_on_vni() {
    assert_eq!(run(InputMethod::Vni, "Btw", "by the way", KeyInput::char(' '), true, true), "By the way ");
}

#[test]
fn macro_case_follow_off_vni() {
    assert_eq!(run(InputMethod::Vni, "Btw", "by the way", KeyInput::char(' '), true, false), "by the way ");
}

#[test]
fn case_behavior_for_both_methods() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        assert_eq!(run(method, "VN", "Việt Nam", KeyInput::char(' '), true, true), "Việt Nam ");
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::char(' '), true, true), "Việt Nam ");
    }
}

#[test]
fn trigger_at_max_word_expands_but_one_longer_is_literal() {
    // `a` cannot be used: the D3 escape rule collapses runs like `aaa` -> `aa`.
    let at_limit = "b".repeat(MAX_WORD);
    assert_eq!(run(InputMethod::Telex, &at_limit, "hit", KeyInput::char(' '), true, true), "hit ");
    let over_limit = "b".repeat(MAX_WORD + 1);
    assert_eq!(run(InputMethod::Telex, &over_limit, "hit", KeyInput::char(' '), true, true), format!("{over_limit} "));
}
