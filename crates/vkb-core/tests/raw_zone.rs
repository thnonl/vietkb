//! Raw-zone protection (spec §A9 + amendment 2026-10-30): an opener arms only
//! when the token on its left is non-empty, plainly ASCII (no composed Vietnamese
//! letter, no tone) and — for the three characters a Vietnamese typist also puts
//! between two words (`. / ?`) — not a valid A9 syllable. It never arms after
//! whitespace, after an ellipsis, after a digit-dot, or as a second punctuation
//! outside the chain set `{':', '\\', '/'}` (declared loss L1, `20-spec.md` §9);
//! inside that set a chained opener continues a path (`C:\`, `C:/`, `http://`).

use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

const VK_LEFT: u16 = 0x25;
const VK_HOME: u16 = 0x24;

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

fn vni() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    }
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

/// §8 of the amendment: the arm decision needs a code-shaped left token — a
/// non-empty, plain-ASCII one that (for `. / ?` only) is not a valid A9 syllable.
/// Rows §4 must compose; rows §5 must stay literal.
#[test]
fn arming_needs_a_code_shaped_left_token() {
    // §4 — the left token is Vietnamese (or empty), so nothing arms.
    for sep in ['.', '/', '?', '_', '@', ':', '=', '#'] {
        assert_eq!(
            simulate(vni(), &format!("ba65t{sep}ta8t1")),
            format!("bật{sep}tắt"),
            "VNI {sep}"
        );
    }
    assert_eq!(simulate(vni(), "ba65t\\ta8t1"), "bật\\tắt");
    assert_eq!(simulate(vni(), "Ba65t/Ta8t1"), "Bật/Tắt");
    // Empty left token: leading opener, after whitespace, after another opener.
    assert_eq!(simulate(vni(), "/ta8t1"), "/tắt");
    assert_eq!(simulate(vni(), "ba65t /ta8t1"), "bật /tắt");
    assert_eq!(simulate(vni(), "ba65t//ta8t1"), "bật//tắt");

    // §5 — plain-ASCII, non-syllable left token: the zone arms.
    assert_eq!(simulate(EngineOptions::default(), "github.cof"), "github.cof");
    assert_eq!(simulate(EngineOptions::default(), "x=cof"), "x=cof");
    assert_eq!(simulate(EngineOptions::default(), "me@cof"), "me@cof");
    assert_eq!(simulate(EngineOptions::default(), "file_name"), "file_name");
    assert_eq!(simulate(vni(), "a@b.com"), "a@b.com");
    assert_eq!(simulate(vni(), "http://a.b/c"), "http://a.b/c");
    // Digits are not a syllable, so `/` arms where a digit-dot does not.
    assert_eq!(simulate(vni(), "2/3"), "2/3");
    assert_eq!(simulate(EngineOptions::default(), "3.cof"), "3.cò");
    // `%` and `-` are not openers at all.
    assert_eq!(simulate(vni(), "%20"), "%20");
    assert_eq!(simulate(vni(), "--opt"), "--opt");
    // Ellipsis: the 2nd and 3rd dot see a `.` on their left.
    assert_eq!(simulate(EngineOptions::default(), "a...cof"), "a...cò");
}

/// §7 of the amendment: `arm` is recomputed by **every** boundary key, so it never
/// survives as text protection.
#[test]
fn raw_zone_is_recomputed_per_opener() {
    // The first opener arms: `x` is plain ASCII and `=` needs no syllable test.
    assert_eq!(simulate(EngineOptions::default(), "x=cof"), "x=cof");
    // The second punctuation arrives with an empty left token, so it arms
    // nothing and the word after it composes again.
    // Declared loss, see 20-spec §9 L1 — the price of `bật/tắt` and `bật//tắt`.
    assert_eq!(simulate(EngineOptions::default(), "x=@cof"), "x=@cò");
    assert_eq!(simulate(EngineOptions::default(), "a./cof"), "a./cò");
    // The shapes the empty-left rule exists for: `//` and `/ ` stay composing.
    assert_eq!(simulate(vni(), "ba65t//ta8t1"), "bật//tắt");
    assert_eq!(simulate(vni(), "ba65t /ta8t1"), "bật /tắt");
    // A chained opener inside `{':', '\\', '/'}` is the one exception (§2b).
    assert_eq!(
        simulate(EngineOptions::default(), "C:\\Users\\nam"),
        "C:\\Users\\nam"
    );
}

/// Regression: `set_options` is a boundary for the armed flag, so flipping
/// `raw_zone` off after an armed opener must not freeze the next word.
#[test]
fn set_options_clears_the_armed_raw_zone() {
    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    for ch in "github.".chars() {
        let key = KeyInput::char(ch);
        apply_action(&mut out, engine.process_key(key), key);
    }
    engine.set_options(EngineOptions {
        raw_zone: false,
        ..EngineOptions::default()
    });
    for ch in "cof".chars() {
        let key = KeyInput::char(ch);
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "github.cò");
}

/// Regression: a caret/function key (arrows, Home/End, PgUp/PgDn, Delete, F-keys)
/// has no character and no word role, so it ends the armed window too.
#[test]
fn caret_keys_clear_the_armed_raw_zone() {
    for vk in [VK_LEFT, VK_HOME] {
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        for ch in "github.".chars() {
            let key = KeyInput::char(ch);
            apply_action(&mut out, engine.process_key(key), key);
        }
        let key = KeyInput::key(vk, None);
        apply_action(&mut out, engine.process_key(key), key);
        for ch in "cof".chars() {
            let key = KeyInput::char(ch);
            apply_action(&mut out, engine.process_key(key), key);
        }
        assert_eq!(out, "github.cò", "vk {vk:#04x}");
    }
}
