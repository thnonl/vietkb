//! QA stream-B re-verification probe (round `92-verify-report.md`).
//!
//! Test-only. Measures the §A9-amendment / §2b behaviour on the real engine
//! through the same paths `special_cases.rs` uses, but from an independent
//! harness: it adds nothing to the shipped matrix and asserts only the
//! documented outcomes plus the numbers this report quotes.
//!
//! Run:
//!   cargo test -p vkb-core --test qa_92_verify -- --nocapture

use vkb_core::{
    apply_action, simulate, simulate_keys, Engine, EngineOptions, InputMethod, KeyInput, MacroTable,
    VietnameseEngine,
};

const VK_OEM_PERIOD: u16 = 0xBE;
const VK_OEM_2: u16 = 0xBF;
const VK_NUMPAD0: u16 = 0x60;
const VK_BACKSPACE: u16 = 0x08;
const VK_LEFT: u16 = 0x25;

fn vni() -> EngineOptions {
    EngineOptions { method: InputMethod::Vni, ..EngineOptions::default() }
}

fn telex() -> EngineOptions {
    EngineOptions { method: InputMethod::Telex, ..EngineOptions::default() }
}

fn sim(options: EngineOptions, keys: &str) -> String {
    simulate(options, keys)
}

fn sim_vk(options: EngineOptions, keys: &[(u16, Option<char>)]) -> String {
    let keys: Vec<KeyInput> = keys.iter().map(|&(vk, ch)| KeyInput::key(vk, ch)).collect();
    simulate_keys(options, &keys)
}

fn vk_for(ch: char) -> u16 {
    match ch {
        '/' => VK_OEM_2,
        '.' => VK_OEM_PERIOD,
        'a'..='z' => ch.to_ascii_uppercase() as u16,
        'A'..='Z' => ch as u16,
        '0'..='9' => ch as u16,
        _ => 0,
    }
}

fn keys_of(keys: &str) -> Vec<(u16, Option<char>)> {
    keys.chars().map(|ch| (vk_for(ch), Some(ch))).collect()
}

fn run(engine: &mut Engine, keys: &[(u16, Option<char>)]) -> String {
    let mut out = String::new();
    for &(vk, ch) in keys {
        let key = KeyInput::key(vk, ch);
        let action = engine.process_key(key);
        apply_action(&mut out, action, key);
    }
    out
}

/// Same, but models the app: when the engine answers `Noop` to a Backspace the
/// focused app still deletes one character (that is the whole reopen contract).
fn run_screen(engine: &mut Engine, keys: &[(u16, Option<char>)]) -> String {
    let mut out = String::new();
    for &(vk, ch) in keys {
        let key = KeyInput::key(vk, ch);
        let action = engine.process_key(key);
        let before = out.len();
        let noop = matches!(&action, vkb_core::EngineAction::Noop);
        apply_action(&mut out, action, key);
        if noop && vk == VK_BACKSPACE && out.len() == before {
            out.pop();
        }
    }
    out
}

/// Numpad digits after a separator: SC-63 through the real VK path.
#[test]
fn keypad_digits_after_a_separator() {
    let mut keys: Vec<(u16, Option<char>)> = keys_of("ba65t/");
    // `ta8t1` on the keypad: digits become VK_NUMPAD0+n, letters stay VK_A..Z.
    for ch in "ta8t1".chars() {
        if ch.is_ascii_digit() {
            keys.push((VK_NUMPAD0 + (ch as u16 - '0' as u16), Some(ch)));
        } else {
            keys.push((vk_for(ch), Some(ch)));
        }
    }
    let got = sim_vk(vni(), &keys);
    println!("keypad ba65t/ta8t1 -> {got}");
    assert_eq!(got, "bật/tắt");

    let mut keys: Vec<(u16, Option<char>)> = Vec::new();
    for ch in "ta8t1".chars() {
        if ch.is_ascii_digit() {
            keys.push((VK_NUMPAD0 + (ch as u16 - '0' as u16), Some(ch)));
        } else {
            keys.push((vk_for(ch), Some(ch)));
        }
    }
    let got = sim_vk(vni(), &keys);
    println!("keypad ta8t1       -> {got}");
    assert_eq!(got, "tắt");
}

/// Backspace and caret-mover interruptions around an opener run.
#[test]
fn interruptions_around_an_armed_opener() {
    let mut rows: Vec<(String, String)> = Vec::new();

    // `Tha` + `/` + BS + `s` — BS removes the opener, the word reopens.
    let mut k = keys_of("Tha");
    k.push((VK_OEM_2, Some('/')));
    k.push((VK_BACKSPACE, None));
    k.extend(keys_of("s"));
    rows.push(("Tha / BS s".into(), run_screen(&mut Engine::new(telex()), &k)));

    // Armed code opener, then BS deletes the opener, then a Vietnamese word.
    let mut k = keys_of("github");
    k.push((VK_OEM_PERIOD, Some('.')));
    k.push((VK_BACKSPACE, None));
    k.extend(keys_of("tawts"));
    rows.push(("github . BS tawts".into(), run_screen(&mut Engine::new(telex()), &k)));

    // `bật` + `.` (no arm) + BS + `cof`: no zone existed, BS reopens `bật`.
    let mut k = keys_of("baajt");
    k.push((VK_OEM_PERIOD, Some('.')));
    k.push((VK_BACKSPACE, None));
    k.extend(keys_of("cof"));
    rows.push(("bật . BS cof".into(), run_screen(&mut Engine::new(telex()), &k)));

    // Two openers, one BS: `C:\` then BS drops the `\`, `Users` composes again.
    let mut k = keys_of("C:");
    k.push((vk_for('\\'), Some('\\')));
    k.push((VK_BACKSPACE, None));
    k.extend(keys_of("Users"));
    rows.push(("C : \\ BS Users".into(), run_screen(&mut Engine::new(telex()), &k)));

    // A caret move between the opener and the letter. The pure engine has no
    // idea what a caret is; `vkb-windows::state` resets the engine on every
    // `is_caret_mover` VK before any key reaches it, so measure both.
    let mut k = keys_of("github");
    k.push((VK_OEM_PERIOD, Some('.')));
    k.push((VK_LEFT, None));
    k.extend(keys_of("cof"));
    rows.push(("github . LEFT cof (engine only)".into(), run(&mut Engine::new(telex()), &k)));

    // Same but with the hook's reset, which is what production does.
    let mut engine = Engine::new(telex());
    let mut out = String::new();
    for ch in "github".chars() {
        let key = KeyInput::char(ch);
        apply_action(&mut out, engine.process_key(key), key);
    }
    let key = KeyInput::key(VK_OEM_PERIOD, Some('.'));
    apply_action(&mut out, engine.process_key(key), key);
    let key = KeyInput::key(VK_LEFT, None);
    apply_action(&mut out, engine.process_key(key), key);
    vkb_core::VietnameseEngine::reset(&mut engine); // state.rs:349 does this
    for ch in "cof".chars() {
        let key = KeyInput::char(ch);
        apply_action(&mut out, engine.process_key(key), key);
    }
    rows.push(("github . LEFT(reset) cof (hook)".into(), out));

    println!();
    for (label, got) in &rows {
        println!("{label:<36} -> {got:?}");
    }

    // Documented outcomes only.
    assert_eq!(rows[0].1, "Thá");
    // Engine-only arrow: the `ch: None` key is a boundary, so `raw_pending` dies
    // with it and `cof` composes (spec §A9 §7). The hook-level reset below is
    // therefore belt and braces, not the only thing keeping this honest.
    assert_eq!(rows[4].1, "github.cò");
    // Hook-level reset: same text.
    assert_eq!(rows[5].1, "github.cò");
}

/// Behavioural arm table: does a Vietnamese word right of `left + opener` compose?
/// The probe word is Telex `taawts` (`tắt`); literal means the zone armed.
#[test]
fn arm_asymmetry_probe() {
    let lefts = [
        "a", "ab", "ban", "cof", "2", "github", "x", "baajt", "Baajt", "dduwowngf", "file", "me",
    ];
    let openers = ['.', '/', '?', '_', '@', ':', '=', '#', '\\'];
    println!();
    print!("{:<8}", "left");
    for o in openers {
        print!(" {o:>4}");
    }
    println!("   (keystrokes; C = right word composed, L = literal => zone armed)");
    for left in lefts {
        print!("{left:<8}");
        for o in openers {
            let got = sim(telex(), &format!("{left}{o}tawts"));
            let mark = if got.ends_with("tắt") { "C" } else { "L" };
            print!(" {mark:>4}");
        }
        println!();
    }
    println!();
    // Chain probes: `left` + `:` + `\` or `/` + word.
    for (left, mid) in [("C:", "\\"), ("C:", "/"), ("http:", "/"), ("a:", "/"), ("abc:", "//")] {
        let got = sim(telex(), &format!("{left}{mid}tawts"));
        println!("chain {left}{mid:<3} -> {got:?}");
    }
    // And the same probe for a shape that must NOT chain.
    for (label, keys) in [
        ("dot-slash", "xyz./tawts"),
        ("underscore-slash", "x_/tawts"),
        ("colon-slash after VN", "baajt:/tawts"),
    ] {
        println!("nochain {label:<22} -> {:?}", sim(telex(), keys));
    }

    // Two of the cells are the published asymmetry, pinned here so the report
    // and the engine cannot drift: `ban` + `. / ?` composes, `ban` + code
    // openers arms; `a` (valid syllable) + `/` composes; `bật` never arms.
    assert_eq!(sim(telex(), "ban.tawts"), "ban.tắt");
    assert_eq!(sim(telex(), "ban?tawts"), "ban?tắt");
    assert_eq!(sim(telex(), "ban_tawts"), "ban_tawts");
    assert_eq!(sim(telex(), "a/tawts"), "a/tắt");
    assert_eq!(sim(telex(), "ab/tawts"), "ab/tawts");
    assert_eq!(sim(telex(), "baajt_tawts"), "bật_tắt");
    assert_eq!(sim(telex(), "baajt:tawts"), "bật:tắt");
}

/// Macro paths at a boundary must be untouched by the arming change.
#[test]
fn macro_at_boundary() {
    let mut engine = Engine::new(vni());
    let mut macros = MacroTable::new();
    macros.insert("ko", "không");
    engine.set_macros(macros);

    assert_eq!(run(&mut engine, &keys_of("ko ")), "không ");
    assert_eq!(run(&mut engine, &keys_of("ba65t/ko ")), "bật/không ");
    assert_eq!(run(&mut engine, &keys_of("ko/")), "không/");
    assert_eq!(run(&mut engine, &keys_of("ba65t/ko/")), "bật/không/");
    assert_eq!(run(&mut engine, &keys_of("ko.")), "không.");
    assert_eq!(run(&mut engine, &keys_of("ba65t/ko.")), "bật/không.");

    // macro_only: nothing but macro expansion runs.
    let mut engine = Engine::new(vni());
    let mut macros = MacroTable::new();
    macros.insert("ko", "không");
    engine.set_macros(macros);
    engine.set_macro_only(true);
    assert_eq!(run(&mut engine, &keys_of("ko/")), "không/");
    assert_eq!(run(&mut engine, &keys_of("ba65t/")), "ba65t/");
}

/// New shapes the fix could have broken, pinned as measured.
#[test]
fn new_shapes_after_the_fix() {
    let cases: [(&str, EngineOptions, &str, &str); 16] = [
        ("path mid-sentence VNI", vni(), "to6i d9ang o73 C:\\Users\\nam", "tôi đang ở C:\\Users\\nam"),
        ("path mid-sentence Telex", telex(), "tooi ddang owr C:\\Users\\nam", "tôi đang ở C:\\Users\\nam"),
        ("VN word then path", telex(), "baajt.C:\\Users\\nam", "bật.C:\\Users\\nam"),
        ("bật.C:\\x", telex(), "baajt.C:\\x", "bật.C:\\x"),
        ("a.b/c", telex(), "a.b/c", "a.b/c"),
        ("e.g./i.e.", telex(), "e.g./i.e.", "e.g./i.e."),
        ("v1.2.3/tag", vni(), "v1.2.3/tag", "v1.2.3/tag"),
        ("100%/50%", vni(), "100%/50%", "100%/50%"),
        ("#hashtag/tieesng (L2)", telex(), "#hashtag/tieesng", "#hashtag/tieesng"),
        ("abc/d9uo7ng2 (L2)", vni(), "abc/d9uo7ng2", "abc/d9uo7ng2"),
        ("bật https://x", telex(), "baajt https://x", "bật https://x"),
        ("bật:https://x", telex(), "baajt:https://x", "bật:https://x"),
        ("bật:/tắt (L5 near miss)", vni(), "ba65t:/ta8t1", "bật:/tắt"),
        ("cof:/tiếng (L5)", telex(), "cof:/tieesng", "cò:/tiếng"),
        ("x/tắt (single letter)", vni(), "x/ta8t1", "x/tắt"),
        ("km/h", vni(), "km/h", "km/h"),
    ];
    println!();
    for (label, opts, keys, expected) in cases {
        let got = sim(opts, keys);
        let mark = if got == expected { "PASS" } else { "FAIL" };
        println!("{mark} {label:<28} {keys:<26} -> {got:?}");
        assert_eq!(got, expected, "{label}");
    }
}
