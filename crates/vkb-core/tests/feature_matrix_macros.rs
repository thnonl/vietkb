//! Macro feature matrix across input methods.
use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, MacroTable, VietnameseEngine, MAX_WORD};

fn keys(input: &str) -> Vec<KeyInput> { input.chars().map(KeyInput::char).collect() }
fn run(method: InputMethod, trigger: &str, expansion: &str, boundary: KeyInput, enabled: bool) -> String {
    let mut engine = Engine::new(EngineOptions { method, macro_enabled: enabled, ..EngineOptions::default() });
    engine.set_macros(MacroTable::from_pairs([(trigger, expansion)]));
    let mut out = String::new();
    for key in keys(trigger).into_iter().chain([boundary]) { apply_action(&mut out, engine.process_key(key), key); }
    out
}

#[test]
fn macros_expand_at_space_and_enter_for_both_methods() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::char(' '), true), "Việt Nam ");
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::enter(), true), "Việt Nam\r");
    }
}

#[test]
fn digit_trigger_expands_for_vni() {
    assert_eq!(run(InputMethod::Vni, "vn1", "Vietnam", KeyInput::char(' '), true), "Vietnam ");
}

#[test]
fn disabled_macros_do_not_expand() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::char(' '), false), "vn ");
    }
}

#[test]
fn macro_case_follow_flag_behavior() {
    // EngineOptions has no macro_case_follow field yet; current hard-coded behavior capitalizes.
    for method in [InputMethod::Telex, InputMethod::Vni] {
        assert_eq!(run(method, "btw", "by the way", KeyInput::char(' '), true), "by the way ");
        assert_eq!(run(method, "Btw", "by the way", KeyInput::char(' '), true), "By the way ");
    }
}

#[test]
#[ignore = "engine has no off-state expansion entry point"]
fn macro_when_disabled_expands_while_vietnamese_is_off() {
    // Engine exposes no enabled=false entry point.
}

#[test]
fn case_behavior_for_both_methods() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        assert_eq!(run(method, "VN", "Việt Nam", KeyInput::char(' '), true), "Việt Nam ");
        assert_eq!(run(method, "vn", "Việt Nam", KeyInput::char(' '), true), "Việt Nam ");
    }
}

#[test]
fn trigger_at_max_word_expands_but_one_longer_is_literal() {
    // `a` cannot be used: the D3 escape rule collapses runs like `aaa` -> `aa`.
    let at_limit = "b".repeat(MAX_WORD);
    assert_eq!(run(InputMethod::Telex, &at_limit, "hit", KeyInput::char(' '), true), "hit ");
    let over_limit = "b".repeat(MAX_WORD + 1);
    assert_eq!(run(InputMethod::Telex, &over_limit, "hit", KeyInput::char(' '), true), format!("{over_limit} "));
}
