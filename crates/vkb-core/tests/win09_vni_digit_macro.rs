//! D-WIN-09: VNI macro whose trigger ends in a tone digit (`qa1` -> "QA ONE").
use vkb_core::{apply_action, Engine, EngineAction, EngineOptions, InputMethod, KeyInput, MacroTable, VietnameseEngine};

fn options(method: InputMethod, case_follow: bool) -> EngineOptions {
    EngineOptions { method, macro_enabled: true, macro_case_follow: case_follow, ..EngineOptions::default() }
}

fn engine(method: InputMethod, case_follow: bool) -> Engine {
    let mut engine = Engine::new(options(method, case_follow));
    engine.set_macros(MacroTable::from_pairs([("qa1", "QA ONE")]));
    engine
}

fn type_str(engine: &mut Engine, screen: &mut String, input: &str) {
    for ch in input.chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        apply_action(screen, action, key);
    }
}

fn typed(input: &str) -> String {
    let mut e = engine(InputMethod::Vni, true);
    let mut s = String::new();
    type_str(&mut e, &mut s, input);
    s
}

#[test]
fn vni_qa1_space_expands_in_every_case_variant() {
    for input in ["qa1 ", "Qa1 ", "QA1 ", "qA1 "] {
        assert_eq!(typed(input), "QA ONE ", "{input:?}");
    }
}

#[test]
fn vni_qa1_with_case_follow_off_is_the_same() {
    let mut e = engine(InputMethod::Vni, false);
    let mut s = String::new();
    type_str(&mut e, &mut s, "qa1 ");
    assert_eq!(s, "QA ONE ");
}

#[test]
fn repeated_runs_on_one_engine_are_identical() {
    let mut e = engine(InputMethod::Vni, true);
    let mut s = String::new();
    for _ in 0..30 {
        type_str(&mut e, &mut s, "qa1 ");
    }
    assert_eq!(s, "QA ONE ".repeat(30));
}

#[test]
fn first_run_after_a_method_switch() {
    for first in [InputMethod::Telex, InputMethod::Vni] {
        let mut e = engine(first, true);
        let mut s = String::new();
        type_str(&mut e, &mut s, "abc ");
        e.set_options(options(InputMethod::Vni, true));
        e.set_macros(MacroTable::from_pairs([("qa1", "QA ONE")]));
        s.clear();
        type_str(&mut e, &mut s, "qa1 ");
        assert_eq!(s, "QA ONE ");
    }
}

#[test]
fn switch_in_the_middle_of_a_word_then_type() {
    let mut e = engine(InputMethod::Telex, true);
    let mut s = String::new();
    type_str(&mut e, &mut s, "qa");
    e.set_options(options(InputMethod::Vni, true));
    s.clear();
    type_str(&mut e, &mut s, "qa1 ");
    assert_eq!(s, "QA ONE ");
}

#[test]
fn replace_deletes_exactly_what_is_on_screen() {
    let mut e = engine(InputMethod::Vni, true);
    let mut shown = 0usize;
    for ch in "qa1 ".chars() {
        match e.process_key(KeyInput::char(ch)) {
            EngineAction::Replace { delete_units, insert } => {
                assert!(delete_units as usize <= shown, "deletes {delete_units} of {shown}");
                shown = shown - delete_units as usize + insert.encode_utf16().count();
            }
            EngineAction::Noop => shown += 1,
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(shown, "QA ONE ".len());
}
