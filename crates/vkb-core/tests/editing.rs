//! Backspace over the raw keystroke history, restore-word, boundaries,
//! auto-capitalisation and the word cap.

use vkb_core::{
    apply_action, simulate, simulate_keys, Engine, EngineOptions, InputMethod, KeyInput, MacroTable,
    VietnameseEngine, MAX_WORD,
};

fn keys(input: &str) -> Vec<KeyInput> {
    input.chars().map(KeyInput::char).collect()
}

fn run(options: EngineOptions, all: &[KeyInput]) -> String {
    let mut engine = Engine::new(options);
    let mut out = String::new();
    for key in all {
        let action = engine.process_key(*key);
        apply_action(&mut out, action, *key);
    }
    out
}

#[test]
fn backspace_recomputes_from_the_raw_keystrokes() {
    let mut all = keys("tieengs");
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "tiêng");

    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "tiên");

    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "tiê");
}

#[test]
fn backspace_keeps_the_original_case() {
    let mut all = keys("Thuongwf");
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "Thương");
}

#[test]
fn backspace_after_a_tone_restores_the_previous_word() {
    let mut all = keys("Hoas");
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "Hoa");
}

#[test]
fn backspace_undoes_vni_modifiers() {
    let vni = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    let mut all = keys("To6i1");
    all.push(KeyInput::backspace());
    assert_eq!(run(vni, &all), "Tôi");

    let mut all = keys("nghieng2");
    all.push(KeyInput::backspace());
    assert_eq!(run(vni, &all), "nghieng");
}

#[test]
fn backspace_to_empty_leaves_nothing_behind() {
    let mut all = keys("as");
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "a");
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "");
    // With no word of ours, backspace reaches the application.
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "");
}

#[test]
fn backspace_with_no_word_is_a_noop() {
    let mut engine = Engine::new(EngineOptions::default());
    assert!(engine.process_key(KeyInput::backspace()).is_noop());
    assert!(engine.buffer().is_empty());
}

#[test]
fn restore_word_puts_the_raw_keystrokes_back() {
    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    for key in keys("tieengs") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "tiếng");
    apply_action(&mut out, engine.restore_word(), KeyInput::char(' '));
    assert_eq!(out, "tieengs");
    // The word stays literal after the restore.
    for key in keys("s") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "tieengss");
}

#[test]
fn restore_word_on_an_untransformed_word_changes_nothing() {
    use vkb_core::EngineAction;
    let mut engine = Engine::new(EngineOptions::default());
    for key in keys("abc") {
        engine.process_key(key);
    }
    let action = engine.restore_word();
    assert!(matches!(
        action,
        EngineAction::Noop | EngineAction::Swallow
    ));
    assert_eq!(engine.current_text(), "abc");
}

#[test]
fn boundary_flushes_the_word() {
    let mut engine = Engine::new(EngineOptions::default());
    for key in keys("tieengs") {
        engine.process_key(key);
    }
    engine.boundary();
    assert!(engine.buffer().is_empty());
    // The next word starts fresh: a stray tone key is literal.
    let mut out = String::new();
    for key in keys("s") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "s");
}

#[test]
fn word_cap_commit_and_restart() {
    let input: String = "bn".repeat(MAX_WORD);
    assert_eq!(simulate(EngineOptions::default(), &input), input);
}

#[test]
fn auto_capitalize_after_a_sentence_end() {
    let options = EngineOptions {
        auto_capitalize: true,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, "tieengs. vieetj"), "tiếng. Việt");
    assert_eq!(simulate(options, "hoaf? hoaf"), "hòa? Hòa");
    assert_eq!(simulate(options, "hoaf hoaf"), "hòa hòa");
}

#[test]
fn switching_method_resets_the_word() {
    let mut engine = Engine::new(EngineOptions::default());
    for key in keys("tieengs") {
        engine.process_key(key);
    }
    engine.set_options(EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    });
    assert!(engine.buffer().is_empty());
}

#[test]
fn macros_expand_at_the_boundary_only() {
    let mut engine = Engine::new(EngineOptions::default());
    engine.set_macros(MacroTable::from_pairs([
        ("btw", "by the way"),
        ("vn", "Việt Nam"),
    ]));
    let mut out = String::new();
    for key in keys("btw ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "by the way ");

    let mut out = String::new();
    for key in keys("vn ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "Việt Nam ");

    // Capitalised trigger → capitalised expansion.
    let mut out = String::new();
    for key in keys("Btw ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "By the way ");

    // Not a boundary, not a macro hit.
    let mut out = String::new();
    for key in keys("btwx") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "btwx");
}

#[test]
fn macros_can_be_disabled() {
    let options = EngineOptions {
        macro_enabled: false,
        ..EngineOptions::default()
    };
    let mut engine = Engine::new(options);
    engine.set_macros(MacroTable::from_pairs([("btw", "by the way")]));
    let mut out = String::new();
    for key in keys("btw ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "btw ");
}

#[test]
fn macro_expansion_types_the_boundary_itself() {
    let mut engine = Engine::new(EngineOptions::default());
    engine.upsert_macro("vn", "Việt Nam");
    let mut out = String::new();
    for key in keys("vn.") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "Việt Nam.");
}

#[test]
fn simulate_keys_matches_simulate_on_plain_text() {
    let text = "tieengs Vieetj";
    assert_eq!(
        simulate_keys(EngineOptions::default(), &keys(text)),
        simulate(EngineOptions::default(), text)
    );
}
