//! Macro-only mode: with Vietnamese switched off, gõ tắt still expands while no
//! Vietnamese transform may apply (spec §B: `macro_when_disabled`).

use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, MacroTable, VietnameseEngine};

fn engine_with(method: InputMethod, macro_only: bool) -> Engine {
    let mut engine = Engine::new(EngineOptions {
        method,
        ..EngineOptions::default()
    });
    engine.set_macros(MacroTable::from_pairs([
        ("vn", "Việt Nam"),
        ("btw", "by the way"),
    ]));
    engine.set_macro_only(macro_only);
    engine
}

fn type_keys(engine: &mut Engine, keys: &[KeyInput]) -> String {
    let mut out = String::new();
    for &key in keys {
        apply_action(&mut out, engine.process_key(key), key);
    }
    out
}

fn chars(text: &str) -> Vec<KeyInput> {
    text.chars().map(KeyInput::char).collect()
}

#[test]
fn macro_only_expands_telex_and_vni_triggers() {
    let mut telex = engine_with(InputMethod::Telex, true);
    assert_eq!(type_keys(&mut telex, &chars("vn ")), "Việt Nam ");

    let mut vni = engine_with(InputMethod::Vni, true);
    assert_eq!(type_keys(&mut vni, &chars("btw ")), "by the way ");
}

#[test]
fn macro_only_leaves_ordinary_words_raw() {
    let mut engine = engine_with(InputMethod::Telex, true);
    assert_eq!(type_keys(&mut engine, &chars("tieengs ")), "tieengs ");
}

#[test]
fn without_macro_only_normal_typing_still_composes() {
    // The engine itself has no "disabled" state: the backend simply does not
    // call it. With `macro_only` off, ordinary Telex composition applies.
    let mut engine = engine_with(InputMethod::Telex, false);
    assert_eq!(type_keys(&mut engine, &chars("tieengs ")), "tiếng ");
}

#[test]
fn macro_only_enter_expands_then_passes_enter() {
    let mut engine = engine_with(InputMethod::Telex, true);
    let mut keys = chars("vn");
    keys.push(KeyInput::enter());
    assert_eq!(type_keys(&mut engine, &keys), "Việt Nam\r");
}

#[test]
fn macro_only_never_claims_text_is_on_screen() {
    // The platform trims a rewrite against `on_screen()` only when the rewrite
    // deletes exactly that text. Macro-only mode deletes the trigger the app
    // typed while it leaves `on_screen()` empty, so no trim can ever apply.
    let mut engine = engine_with(InputMethod::Telex, true);
    for key in chars("vn") {
        engine.process_key(key);
        assert!(engine.on_screen().is_empty(), "after {key:?}");
    }
    let action = engine.process_key(KeyInput::char(' '));
    assert!(matches!(action, vkb_core::EngineAction::Replace { delete_units: 2, .. }));
    assert!(engine.on_screen().is_empty());
}
