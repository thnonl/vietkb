//! QA measurement harness for the special-case typing audit
//! (round plan `91-round-plan-sound-cases.md`, stream B1; artifact
//! `docs/evkey-clone-work/51-special-cases.md`).
//!
//! Two kinds of test live here:
//!
//! * `#[test] fn matrix()` — prints every measured case as
//!   `id | method | keys | expected | actual | PASS/FAIL`. Run it with
//!   `cargo test -p vkb-core --test special_cases matrix -- --nocapture`
//!   to regenerate the artifact table.
//! * the D-SC repros and the §A9-amendment tests. The five D-SC repros were
//!   `#[ignore]`d while the defects were open; since the amendment (2026-10-30)
//!   they are ordinary regression tests, so no `#[ignore]` is left here and
//!   `matrix()` itself asserts 0 failures.
//!
//! Test-only file: no product code is touched.

use vkb_core::{
    apply_action, simulate, simulate_keys, Engine, EngineOptions, InputMethod, KeyInput, MacroTable,
    VietnameseEngine,
};

const VK_OEM_PERIOD: u16 = 0xBE;
const VK_OEM_2: u16 = 0xBF; // US layout: `/` unshifted, `?` shifted
const VK_NUMPAD0: u16 = 0x60;
const VK_BACKSPACE: u16 = 0x08;

fn vni() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    }
}

fn telex() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Telex,
        ..EngineOptions::default()
    }
}

/// `simulate` over a raw keystroke string (every keystroke is `KeyInput::char`).
fn sim(options: EngineOptions, keys: &str) -> String {
    simulate(options, keys)
}

/// `simulate_keys` with real virtual-key codes, as the Windows hook delivers them.
fn sim_vk(options: EngineOptions, keys: &[(u16, Option<char>)]) -> String {
    let keys: Vec<KeyInput> = keys.iter().map(|&(vk, ch)| KeyInput::key(vk, ch)).collect();
    simulate_keys(options, &keys)
}

/// Letters/digits as `VK_A`..`VK_Z`, `VK_0`..`VK_9`, plus the OEM keys used here.
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

/// Drive an `Engine` directly so macro/macro-only state can be installed.
fn run(engine: &mut Engine, keys: &[(u16, Option<char>)]) -> String {
    let mut out = String::new();
    for &(vk, ch) in keys {
        let key = KeyInput::key(vk, ch);
        let action = engine.process_key(key);
        apply_action(&mut out, action, key);
    }
    out
}

struct Row {
    id: &'static str,
    method: &'static str,
    keys: String,
    expected: String,
    actual: String,
}

fn add(r: &mut Vec<Row>, id: &'static str, method: &'static str, keys: String, expected: &str, actual: String) {
    r.push(Row {
        id,
        method,
        keys,
        expected: expected.to_owned(),
        actual,
    });
}

/// A case measured through `simulate` (char path).
fn case(
    r: &mut Vec<Row>,
    id: &'static str,
    method: &'static str,
    options: EngineOptions,
    keys: &str,
    expected: &str,
) {
    let actual = sim(options, keys);
    add(r, id, method, keys.to_owned(), expected, actual);
}

fn rows() -> Vec<Row> {
    let mut r: Vec<Row> = Vec::new();

    // ---- separators between two VNI words: `bật<sep>tắt` ----------------
    let seps: [(&'static str, char); 27] = [
        ("SC-01", ' '),
        ("SC-02", '/'),
        ("SC-03", '.'),
        ("SC-04", ','),
        ("SC-05", '-'),
        ("SC-06", '_'),
        ("SC-07", ':'),
        ("SC-08", '@'),
        ("SC-09", ';'),
        ("SC-10", '('),
        ("SC-11", ')'),
        ("SC-12", '['),
        ("SC-13", ']'),
        ("SC-14", '"'),
        ("SC-15", '\''),
        ("SC-16", '+'),
        ("SC-17", '='),
        ("SC-18", '*'),
        ("SC-19", '&'),
        ("SC-20", '%'),
        ("SC-21", '#'),
        ("SC-22", '!'),
        ("SC-23", '?'),
        ("SC-24", '~'),
        ("SC-25", '`'),
        ("SC-26", '|'),
        ("SC-27", '\\'),
    ];
    for (id, sep) in seps {
        let keys = format!("ba65t{sep}ta8t1");
        let expected = format!("bật{sep}tắt");
        let actual = sim(vni(), &keys);
        add(&mut r, id, "VNI", keys, &expected, actual);
    }

    // ---- slash cases ---------------------------------------------------
    case(&mut r, "SC-28", "VNI", vni(), "ba65t/ta8t1", "bật/tắt");
    // `tawts` is Telex for `tắt`; the audit used `taats`, which spells `tất`
    // (`aa` = `â`). The row tests Telex parity, so the keystrokes are corrected
    // here and `taats` is pinned in `d_sc_01_slash_between_two_vni_words`.
    case(&mut r, "SC-29", "Telex", telex(), "baajt/tawts", "bật/tắt");
    case(&mut r, "SC-30", "VNI", vni(), "a/b", "a/b");
    case(&mut r, "SC-31", "VNI", vni(), "/ta8t1", "/tắt");
    case(&mut r, "SC-32", "VNI", vni(), "ba65t//ta8t1", "bật//tắt");
    case(&mut r, "SC-33", "VNI", vni(), "ba65t /ta8t1", "bật /tắt");
    case(&mut r, "SC-34", "VNI", vni(), "ba65t/ ta8t1", "bật/ tắt");
    case(&mut r, "SC-35", "VNI", vni(), "http://a.b/c", "http://a.b/c");
    case(&mut r, "SC-36", "VNI", vni(), "a@b.com", "a@b.com");
    case(&mut r, "SC-37", "VNI", vni(), "github.cof", "github.cof");
    case(&mut r, "SC-38", "Telex", telex(), "a. tieengs", "a. tiếng");
    case(&mut r, "SC-39", "Telex", telex(), "3.cof", "3.cò");
    case(&mut r, "SC-40", "VNI", vni(), "-ta8t1", "-tắt");
    case(&mut r, "SC-41", "VNI", vni(), "--opt", "--opt");
    case(&mut r, "SC-42", "VNI", vni(), "file_name", "file_name");
    case(&mut r, "SC-43", "VNI", vni(), "2/3", "2/3");
    case(&mut r, "SC-44", "VNI", vni(), "1.5", "1.5");
    case(&mut r, "SC-45", "VNI", vni(), "%20", "%20");

    // ---- raw_zone = false: the isolation experiment ---------------------
    let no_raw = EngineOptions {
        raw_zone: false,
        ..vni()
    };
    case(&mut r, "SC-46", "VNI", no_raw, "ba65t/ta8t1", "bật/tắt");
    case(&mut r, "SC-47", "VNI", no_raw, "ba65t.ta8t1", "bật.tắt");
    case(&mut r, "SC-48", "VNI", no_raw, "ba65t_ta8t1", "bật_tắt");

    // ---- digits with no target stay literal, digits with a target do not --
    case(&mut r, "SC-49", "VNI", vni(), "t8", "t8");
    case(&mut r, "SC-50", "VNI", vni(), "ba65t t8", "bật t8");
    case(&mut r, "SC-51", "VNI", vni(), "t8 t8", "t8 t8");
    case(&mut r, "SC-52", "VNI", vni(), "ba65t.t8", "bật.t8");
    case(&mut r, "SC-53", "VNI", vni(), "a8", "ă");
    case(&mut r, "SC-54", "VNI", vni(), "ba65t/a8", "bật/ă");
    case(&mut r, "SC-55", "VNI", vni(), "a61", "ấ");
    case(&mut r, "SC-56", "VNI", vni(), "ba65t/a61", "bật/ấ");
    case(&mut r, "SC-57", "VNI", vni(), "d9", "đ");
    case(&mut r, "SC-58", "VNI", vni(), "ba65t/d9", "bật/đ");
    case(&mut r, "SC-59", "VNI", vni(), "o7", "ơ");
    case(&mut r, "SC-60", "VNI", vni(), "ba65t/o7", "bật/ơ");
    case(&mut r, "SC-61", "VNI", vni(), "u7", "ư");
    case(&mut r, "SC-62", "VNI", vni(), "ba65t/u7", "bật/ư");

    // ---- keypad digits and shifted digits (vk path) --------------------
    fn keypad(s: &str) -> Vec<(u16, Option<char>)> {
        s.chars()
            .map(|ch| {
                if ch.is_ascii_digit() {
                    (VK_NUMPAD0 + ch as u16 - '0' as u16, Some(ch))
                } else {
                    (vk_for(ch), Some(ch))
                }
            })
            .collect()
    }
    let actual = sim_vk(vni(), &keypad("ba65t/ta8t1"));
    add(&mut r, "SC-63", "VNI+keypad", "ba65t/ta8t1 (numpad digits)".into(), "bật/tắt", actual);
    let actual = sim_vk(vni(), &keypad("ta8t1"));
    add(&mut r, "SC-64", "VNI+keypad", "ta8t1 (numpad digits)".into(), "tắt", actual);

    let actual = sim_vk(vni(), &keys_of("ba65t!ta8t1"));
    add(&mut r, "SC-65", "VNI+VK", "ba65t!ta8t1".into(), "bật!tắt", actual);
    let shifted = [
        (b'b' as u16, Some('b')),
        (b'a' as u16, Some('a')),
        (b'6' as u16, Some('6')),
        (b'5' as u16, Some('5')),
        (b't' as u16, Some('t')),
        (0x31u16, Some('!')),
        (b't' as u16, Some('t')),
        (b'a' as u16, Some('a')),
        (b'8' as u16, Some('8')),
        (b't' as u16, Some('t')),
        (b'1' as u16, Some('1')),
    ];
    let actual = sim_vk(vni(), &shifted);
    add(&mut r, "SC-66", "VNI+Shift+1", "ba65t!ta8t1 (vk 0x31 + `!`)".into(), "bật!tắt", actual);

    // ---- reopen after a separator (the class-1 rule) -------------------
    let mut keys = keys_of("Tha");
    keys.push((VK_OEM_2, Some('/')));
    keys.push((VK_BACKSPACE, None));
    keys.extend(keys_of("s"));
    add(&mut r, "SC-67", "Telex", "Tha/ + BS + s".into(), "Thá", run(&mut Engine::new(telex()), &keys));

    // ---- macros --------------------------------------------------------
    let mut engine = Engine::new(vni());
    engine.set_macros(MacroTable::from_pairs([("ko", "không")]));
    let actual = run(&mut engine, &keys_of("ko "));
    add(&mut r, "SC-68", "VNI+macro", "ko + space".into(), "không ", actual);

    let mut engine = Engine::new(vni());
    engine.set_macros(MacroTable::from_pairs([("ko", "không")]));
    let actual = run(&mut engine, &keys_of("ba65t/ko "));
    add(&mut r, "SC-69", "VNI+macro", "ba65t/ko + space".into(), "bật/không ", actual);

    let mut engine = Engine::new(vni());
    engine.set_macros(MacroTable::from_pairs([("bt/", "bật tắt")]));
    let actual = run(&mut engine, &keys_of("bt/ "));
    add(
        &mut r,
        "SC-70",
        "VNI+macro",
        "bt/ + space (trigger contains `/`)".into(),
        "bt/ ",
        actual,
    );

    let mut engine = Engine::new(vni());
    engine.set_macros(MacroTable::from_pairs([("ko", "không")]));
    let actual = run(&mut engine, &keys_of("ko/"));
    add(&mut r, "SC-76", "VNI+macro", "ko/ (macro committed by `/`)".into(), "không/", actual);

    let mut engine = Engine::new(vni());
    engine.set_macros(MacroTable::from_pairs([("ko", "không")]));
    let actual = run(&mut engine, &keys_of("ba65t/ko/"));
    add(&mut r, "SC-77", "VNI+macro", "ba65t/ko/".into(), "bật/không/", actual);

    let mut engine = Engine::new(vni());
    engine.set_macro_only(true);
    engine.set_macros(MacroTable::from_pairs([("ko", "không")]));
    let actual = run(&mut engine, &keys_of("ko/"));
    add(&mut r, "SC-71", "VNI+macro_only", "ko/ (macro only)".into(), "không/", actual);

    // ---- misc finds ----------------------------------------------------
    // Backspace deletes one displayed character of the now-composing word
    // (`tắt` → `tắ`), per B1 — not one raw keystroke. The old expectation
    // (`bật/ta8t`) pinned the frozen-literal defect.
    let mut keys = keys_of("ba65t/ta8t1");
    keys.push((VK_BACKSPACE, None));
    add(
        &mut r,
        "SC-72",
        "VNI",
        "ba65t/ta8t1 + BS".into(),
        "bật/tắ",
        run(&mut Engine::new(vni()), &keys),
    );

    let mut engine = Engine::new(vni());
    let actual = run(&mut engine, &keys_of("ba65t/ta8t1/ba65t"));
    add(&mut r, "SC-73", "VNI", "ba65t/ta8t1/ba65t".into(), "bật/tắt/bật", actual);

    let mut engine = Engine::new(vni());
    let mut k = keys_of("ba65t/ta8t1");
    k.push((0x0D, Some('\r')));
    k.extend(keys_of("ta8t1"));
    let actual = run(&mut engine, &k);
    add(&mut r, "SC-74", "VNI", "ba65t/ta8t1 + Enter + ta8t1".into(), "bật/tắt\rtắt", actual);

    case(&mut r, "SC-75", "VNI", vni(), "Ba65t/Ta8t1", "Bật/Tắt");
    case(&mut r, "SC-78", "VNI", vni(), "ba65t$ta8t1", "bật$tắt");

    r
}

#[test]
fn matrix() {
    let rows = rows();
    println!();
    println!("id | method | keys | expected | actual | verdict");
    let (mut pass, mut fail) = (0usize, 0usize);
    let mut failed: Vec<&str> = Vec::new();
    for row in &rows {
        let ok = row.expected == row.actual;
        if ok {
            pass += 1;
        } else {
            fail += 1;
            failed.push(row.id);
        }
        println!(
            "{} | {} | `{}` | `{}` | `{}` | {}",
            row.id,
            row.method,
            row.keys,
            row.expected,
            row.actual,
            if ok { "PASS" } else { "FAIL" }
        );
    }
    println!("-- total {} · pass {} · fail {}", rows.len(), pass, fail);
    println!("-- failed: {}", failed.join(", "));
    assert!(!rows.is_empty());
    // Every row is a declared outcome: the matrix is a regression gate, not just
    // a report. The 22 FAILs of `51-special-cases.md` must stay at 0.
    assert_eq!(fail, 0, "failing rows: {}", failed.join(", "));
}

/// Action-level trace for one key sequence (`--nocapture` prints it), used to
/// locate the layer that emits a wrong action.
fn trace(options: EngineOptions, keys: &[(u16, Option<char>)], label: &str) {
    let mut engine = Engine::new(options);
    let mut out = String::new();
    println!("-- trace {label}");
    for &(vk, ch) in keys {
        let key = KeyInput::key(vk, ch);
        let action = engine.process_key(key);
        println!("   {:?} -> {:?}", key.ch, action);
        apply_action(&mut out, action, key);
        println!("      screen {:?}", out);
    }
}

#[test]
fn trace_backspace_cases() {
    let mut keys = keys_of("ba65t/ta8t1");
    keys.push((VK_BACKSPACE, None));
    trace(vni(), &keys, "ba65t/ta8t1 + BS");

    let mut keys = keys_of("Tha");
    keys.push((VK_OEM_2, Some('/')));
    keys.push((VK_BACKSPACE, None));
    keys.extend(keys_of("s"));
    trace(telex(), &keys, "Tha/ + BS + s");

    // The traces above are the diagnostic; these two are the contract.
    let mut keys = keys_of("ba65t/ta8t1");
    keys.push((VK_BACKSPACE, None));
    assert_eq!(run(&mut Engine::new(vni()), &keys), "bật/tắ");
    let mut keys = keys_of("Tha");
    keys.push((VK_OEM_2, Some('/')));
    keys.push((VK_BACKSPACE, None));
    keys.extend(keys_of("s"));
    assert_eq!(run(&mut Engine::new(telex()), &keys), "Thá");
}

// ---------------------------------------------------------------------------
// The four D-SC repros — regression tests since the §A9 amendment (2026-10-30).
// ---------------------------------------------------------------------------

#[test]
fn d_sc_01_slash_between_two_vni_words() {
    // The owner report, verbatim.
    assert_eq!(sim(vni(), "ba65t/ta8t1"), "bật/tắt");
    // Same in Telex: the letters compose again. `tawts` is Telex for `tắt`, and
    // `taats` spells `tất` (`aa` = `â`), so both are pinned.
    assert_eq!(sim(telex(), "baajt/tawts"), "bật/tắt");
    assert_eq!(sim(telex(), "baajt/taats"), "bật/tất");
    // Every VNI transform is lost, not just the tone.
    assert_eq!(sim(vni(), "ba65t/a8"), "bật/ă");
    assert_eq!(sim(vni(), "ba65t/a61"), "bật/ấ");
    assert_eq!(sim(vni(), "ba65t/d9"), "bật/đ");
    assert_eq!(sim(vni(), "ba65t/o7"), "bật/ơ");
    assert_eq!(sim(vni(), "ba65t/u7"), "bật/ư");
    // Siblings: leading `/`, `//`, space before `/`, a chain, uppercase.
    assert_eq!(sim(vni(), "/ta8t1"), "/tắt");
    assert_eq!(sim(vni(), "ba65t//ta8t1"), "bật//tắt");
    assert_eq!(sim(vni(), "ba65t /ta8t1"), "bật /tắt");
    assert_eq!(sim(vni(), "ba65t/ta8t1/ba65t"), "bật/tắt/bật");
    assert_eq!(sim(vni(), "Ba65t/Ta8t1"), "Bật/Tắt");
    // Keypad digits behave identically (same engine path, not a hook issue).
    assert_eq!(sim_vk(vni(), &keys_of("ba65t/ta8t1")), "bật/tắt");
}

#[test]
fn d_sc_02_dot_and_question_between_words() {
    assert_eq!(sim(vni(), "ba65t.ta8t1"), "bật.tắt");
    assert_eq!(sim(vni(), "ba65t?ta8t1"), "bật?tắt");
}

#[test]
fn d_sc_03_underscore_at_colon_between_words() {
    assert_eq!(sim(vni(), "ba65t_ta8t1"), "bật_tắt");
    assert_eq!(sim(vni(), "ba65t@ta8t1"), "bật@tắt");
    assert_eq!(sim(vni(), "ba65t:ta8t1"), "bật:tắt");
}

#[test]
fn d_sc_04_code_punctuation_openers() {
    assert_eq!(sim(vni(), "ba65t=ta8t1"), "bật=tắt");
    assert_eq!(sim(vni(), "ba65t#ta8t1"), "bật#tắt");
    assert_eq!(sim(vni(), "ba65t\\ta8t1"), "bật\\tắt");
}

// ---------------------------------------------------------------------------
// The amendment's own contract (spec §A9 amendment 2026-10-30 §4 and §6).
// ---------------------------------------------------------------------------

/// A separator between two Vietnamese words never disables Vietnamese, in both
/// methods, for every separator QA measured and for all five VNI triggers.
#[test]
fn separators_keep_the_next_word_vietnamese() {
    for separator in ['.', '/', '_', '@', ':'] {
        assert_eq!(
            sim(vni(), &format!("ba65t{separator}ta8t1")),
            format!("bật{separator}tắt"),
            "VNI {separator}"
        );
        assert_eq!(
            sim(telex(), &format!("baajt{separator}tawts")),
            format!("bật{separator}tắt"),
            "Telex {separator}"
        );
        // The five VNI modifier/tone triggers: `a8 a61 d9 o7 u7`.
        for (keys, expected) in [
            ("a8", "ă"),
            ("a61", "ấ"),
            ("d9", "đ"),
            ("o7", "ơ"),
            ("u7", "ư"),
        ] {
            assert_eq!(
                sim(vni(), &format!("ba65t{separator}{keys}")),
                format!("bật{separator}{expected}"),
                "VNI trigger {keys} after {separator}"
            );
        }
    }
}

/// The declared borderline outcomes (`20-spec.md` §A9 amendment §6 and §9),
/// including the two declared losses L1 and L2. Pinned as measured, so a later
/// change that silently widens or narrows the raw zone trips here.
#[test]
fn amendment_2026_10_30_borderlines() {
    // Vietnamese on the left, URL on the right: the `/` does not arm (`bật` is
    // not plain ASCII), the `:` after `http` does (`http` is), and the two `/`
    // after it see an empty left token.
    assert_eq!(sim(telex(), "baajt/http://x"), "bật/http://x");

    // Declared loss L2: a plain-ASCII, invalid-syllable left token suppresses a
    // Vietnamese right word, in both segment shapes.
    assert_eq!(sim(vni(), "abc/d9uo7ng2"), "abc/d9uo7ng2");
    assert_eq!(sim(telex(), "#hashtag/tieesng"), "#hashtag/tieesng");

    // Declared loss L1 (`20-spec.md` §9): a second opener outside the chain set
    // `{':', '\\', '/'}` has an empty left token, so it arms nothing and the word
    // after it composes again — the price of `bật./tắt` and `bật//tắt`.
    assert_eq!(sim(telex(), "x=@cof"), "x=@cò");
    assert_eq!(sim(telex(), "a./cof"), "a./cò");

    // A Windows path survives in VNI: `C` arms the `:`, and `Users` is plain
    // ASCII with no VNI trigger inside it.
    assert_eq!(sim(vni(), "C:\\Users\\nam"), "C:\\Users\\nam");
    // Telex reproduces the same row since §2b: the `\` immediately after `:` is a
    // chained opener (word empty, `raw_pending` still set from `:`, last emitted
    // `:`), so it arms too and `Users` stays literal (D-SC-05).
    assert_eq!(sim(telex(), "C:\\Users\\nam"), "C:\\Users\\nam");

    // Two ASCII single letters either side: `a` is a valid syllable (so the `/`
    // after it does not arm), `b` is not (so its `/` does) — same text either way.
    assert_eq!(sim(vni(), "a/b/c"), "a/b/c");
    // `%` is not an opener, and `10`/`50` are digit-leading (`non_viet`).
    assert_eq!(sim(vni(), "100%/50%"), "100%/50%");
    // `e` is a valid syllable, `g` is not, `i` is — no transform trigger in any
    // segment, so the text is identical whichever way the flags fell.
    assert_eq!(sim(telex(), "e.g./i.e."), "e.g./i.e.");
    // Both dots are digit-dots; the `/` after `3` arms (plain ASCII, not a syllable).
    assert_eq!(sim(vni(), "v1.2.3/tag"), "v1.2.3/tag");
}

// ---------------------------------------------------------------------------
// §2b chained openers (D-SC-05).
// ---------------------------------------------------------------------------

/// An opener that arrives right after another *armed* opener, with an empty left
/// token, continues a path instead of ending one (`20-spec.md` §A9 amendment §2b).
/// The negative controls at the end are the point of the test: the chain fires for
/// `\` and `/` after `: \ /` only.
#[test]
fn chained_openers_extend_a_path() {
    // Must stay literal — the four §6 chained rows, in both methods.
    for (name, options) in [("telex", telex()), ("vni", vni())] {
        for (keys, expected) in [
            ("C:\\Users\\nam", "C:\\Users\\nam"),
            ("C:/Users/nam", "C:/Users/nam"),
            ("C:\\\\server\\share", "C:\\\\server\\share"),
            ("https://x/y/z", "https://x/y/z"),
            ("http://a.b/c", "http://a.b/c"),
        ] {
            assert_eq!(sim(options, keys), expected, "{name} {keys:?}");
        }
    }

    // Must stay composing — the chain must not swallow an ordinary word, and a
    // word that was never armed cannot start one.
    assert_eq!(sim(telex(), "baajt/taawts"), "bật/tắt");
    assert_eq!(sim(telex(), "baajt//taawts"), "bật//tắt");
    assert_eq!(sim(telex(), "baajt /taawts"), "bật /tắt");
    assert_eq!(sim(telex(), "/taawts"), "/tắt");
    assert_eq!(sim(telex(), "a/b"), "a/b");
    assert_eq!(sim(vni(), "2/3"), "2/3");
    assert_eq!(sim(telex(), "baajt.http"), "bật.http");
    assert_eq!(sim(telex(), "baajt./taawts"), "bật./tắt");
    assert_eq!(sim(telex(), "baajt:_/taawts"), "bật:_/tắt");

    // The negative control for §2b: after `.` the last emitted char is not an
    // opener char, and after `_`/`:` the `_` is outside the chain set and `bật`
    // never armed at `:` — so the following `/` arms nothing and `tiếng` composes.
    assert_eq!(sim(telex(), "xyz./tieesng"), "xyz./tiếng");
    assert_eq!(sim(telex(), "baajt:_/tieesng"), "bật:_/tiếng");
}

// ---------------------------------------------------------------------------
// Guards: must stay green before and after the fix.
// ---------------------------------------------------------------------------

/// The hook and the CLI must not diverge: `KeyInput::char` (vk 0) and a real
/// virtual-key code with the layout-resolved character produce the same text.
#[test]
fn vk_path_matches_char_path() {
    for keys in [
        "ba65t ta8t1",
        "ba65t/ta8t1",
        "ba65t.ta8t1",
        "ba65t!ta8t1",
        "ba65t/ko",
        "http://a.b/c",
    ] {
        let chars = sim(vni(), keys);
        let vk = sim_vk(vni(), &keys_of(keys));
        assert_eq!(chars, vk, "char and vk paths diverged for {keys:?}");
    }
}

/// URL/email/code protection is the *reason* the raw zone exists: whatever the
/// fix does to `bật/tắt`, these must keep passing (`tests/raw_zone.rs` pins the
/// same behaviour).
#[test]
fn url_email_and_code_shapes_stay_literal() {
    assert_eq!(sim(vni(), "http://a.b/c"), "http://a.b/c");
    assert_eq!(sim(vni(), "a@b.com"), "a@b.com");
    assert_eq!(sim(telex(), "github.cof"), "github.cof");
    assert_eq!(sim(vni(), "file_name"), "file_name");
    assert_eq!(sim(telex(), "a. tieengs"), "a. tiếng");
    assert_eq!(sim(telex(), "3.cof"), "3.cò");
    assert_eq!(sim(vni(), "2/3"), "2/3");
    assert_eq!(sim(vni(), "1.5"), "1.5");
    assert_eq!(sim(vni(), "%20"), "%20");
    assert_eq!(sim(vni(), "--opt"), "--opt");
}
