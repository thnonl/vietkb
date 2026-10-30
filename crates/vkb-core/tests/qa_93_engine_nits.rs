//! QA independent re-verification of the two raw-zone state leaks fixed after
//! `92-verify-report.md` (§7.13 and the `set_options`/`set_macro_only` arms).
//! Test-only: no product code is touched.
//!
//! Leak A — `set_options` / `set_macro_only` used to leave `raw_pending` armed, so
//! an opener typed before a config/mode switch froze the word typed after it.
//! Leak B — a `ch: None` key (arrow, Home/End, PgUp/PgDn, Delete, F-key) used to
//! return early **without** clearing `raw_pending`, so an armed opener survived a
//! caret move inside `Engine` (the Windows hook's `reset_engine` masked it).
//!
//! Own harness: `KeyInput::char` / `KeyInput::key(vk, None)` straight into
//! `Engine::process_key`, never Dev's `raw_zone.rs` helpers. The un-toggled /
//! un-moved control row is the pre-fix output on this build, because
//! `word_key`'s consumer reads `raw_pending` and never re-reads `raw_zone`:
//! with the flag still armed the same `cof` stays literal.

use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_DELETE: u16 = 0x2E;
const VK_F1: u16 = 0x70;

fn type_str(engine: &mut Engine, out: &mut String, keys: &str) {
    for ch in keys.chars() {
        let key = KeyInput::char(ch);
        apply_action(out, engine.process_key(key), key);
    }
}

fn press(engine: &mut Engine, out: &mut String, vk: u16) {
    let key = KeyInput::key(vk, None);
    apply_action(out, engine.process_key(key), key);
}

fn armed_control() -> String {
    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    type_str(&mut engine, &mut out, "github.");
    type_str(&mut engine, &mut out, "cof");
    out
}

/// Leak A, `set_options` route: arm `github.`, switch `raw_zone` off, type `cof`.
#[test]
fn set_options_after_an_armed_opener_does_not_freeze_the_next_word() {
    // Control: no switch — the armed zone keeps `cof` literal. This is also the
    // pre-fix result of the row below: `word_key` consumes `raw_pending` and
    // does not consult `options.raw_zone`, so the flag alone decided the text.
    assert_eq!(armed_control(), "github.cof", "un-toggled control");

    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    type_str(&mut engine, &mut out, "github.");
    engine.set_options(EngineOptions {
        raw_zone: false,
        ..EngineOptions::default()
    });
    type_str(&mut engine, &mut out, "cof");
    assert_eq!(out, "github.cò");
}

/// Leak A, `set_macro_only` route: the same armed flag must not survive a mode
/// switch. Both switches are exercised because either one used to leave it set.
#[test]
fn set_macro_only_after_an_armed_opener_does_not_freeze_the_next_word() {
    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    type_str(&mut engine, &mut out, "github.");
    engine.set_macro_only(true);
    engine.set_macro_only(false);
    type_str(&mut engine, &mut out, "cof");
    assert_eq!(out, "github.cò");
}

/// Leak B: every `ch: None` command/caret key ends the armed window. Before the
/// fix each of these returned `Noop` with `raw_pending` untouched.
#[test]
fn caret_and_command_keys_end_the_armed_window() {
    let vks = [
        VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN, VK_HOME, VK_END, VK_PRIOR, VK_NEXT, VK_DELETE, VK_F1,
    ];
    for vk in vks {
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "github.");
        press(&mut engine, &mut out, vk);
        type_str(&mut engine, &mut out, "cof");
        assert_eq!(out, "github.cò", "vk {vk:#04x}");
    }
}

/// The same two routes must not lose a legitimate arm: with no switch and no
/// caret key between the opener and the word, the zone still protects it.
#[test]
fn a_legitimate_arm_survives_an_ordinary_sequence() {
    // char path, Telex
    assert_eq!(armed_control(), "github.cof");
    // VNI, path with a chained opener
    let mut engine = Engine::new(EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    });
    let mut out = String::new();
    type_str(&mut engine, &mut out, "C:\\Users\\nam");
    assert_eq!(out, "C:\\Users\\nam");
}
