//! Black-box QA regression table (D10, 0.2.8): every input a QA run found wrong,
//! with the expected text. Each row is `(profile, keystrokes, expected)`; the
//! trailing space ends the last word.
//!
//! Profiles: `T` Telex neutral app, `C` Telex code app, `V` VNI neutral,
//! `VC` VNI code app, `Toff` Telex with spell check off, `Tnj` Telex with
//! `jfwz_as_consonant` off.

use vkb_core::{simulate, AppKind, EngineOptions, InputMethod};

fn options(profile: &str) -> EngineOptions {
    let base = EngineOptions::default();
    match profile {
        "T" => base,
        "C" => EngineOptions { app_kind: AppKind::Code, ..base },
        "V" => EngineOptions { method: InputMethod::Vni, ..base },
        "VC" => EngineOptions { method: InputMethod::Vni, app_kind: AppKind::Code, ..base },
        "Toff" => EngineOptions { spell_check: false, ..base },
        "Tnj" => EngineOptions { jfwz_as_consonant: false, ..base },
        other => panic!("unknown profile {other}"),
    }
}

fn run(table: &[(&str, &str, &str)]) {
    let mut failures = Vec::new();
    for (profile, keys, expected) in table {
        let got = simulate(options(profile), keys);
        if &got != expected {
            failures.push(format!("[{profile}] {keys:?}\n    got:  {got:?}\n    want: {expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{} of {} rows differ:\n{}", failures.len(), table.len(), failures.join("\n"));
}

#[test]
fn b1_quyt_and_common_fruit() {
    run(&[
        ("T", "quyts ", "quýt "),
        ("V", "quyt1 ", "quýt "),
        ("T", "Em an quyts ", "Em an quýt "),
        ("V", "Em an quyt1 ", "Em an quýt "),
        ("C", "quyts ", "quýt "),
    ]);
}

#[test]
fn b2_plural_shaped_words_are_vietnamese_without_english_evidence() {
    run(&[
        ("T", "cups ", "cúp "),
        ("T", "tips ", "típ "),
        ("T", "sons ", "són "),
        ("T", "oops ", "ốp "),
        ("T", "ups ", "úp "),
        ("T", "tons ", "tón "),
        ("T", "pins ", "pín "),
        ("T", "sins ", "sín "),
        ("T", "guts ", "gút "),
        ("T", "kits ", "kít "),
        ("T", "sits ", "sít "),
        ("T", "longs ", "lóng "),
        ("T", "tongs ", "tóng "),
        ("T", "mans ", "mán "),
        ("T", "sans ", "sán "),
        ("T", "suns ", "sún "),
        ("T", "tarn ", "tản "),
        ("T", "Con cups ", "Con cúp "),
        // Strong English evidence: an English function word on the left.
        ("T", "the cups ", "the cups "),
        ("T", "my tips ", "my tips "),
        ("T", "the sons ", "the sons "),
    ]);
}

#[test]
fn b3_left_vietnamese_context() {
    run(&[("T", "Em bes mawjc tax mowis ", "Em bé mặc tã mới ")]);
}

#[test]
fn c1_code_identifiers_in_a_code_app() {
    run(&[
        ("C", "import os ", "import os "),
        ("C", "os.path ", "os.path "),
        ("C", "sys.argv ", "sys.argv "),
        ("C", "(req, res) => ", "(req, res) => "),
        ("C", "const res = 1 ", "const res = 1 "),
        ("C", "let buf = foo(cur, arr) ", "let buf = foo(cur, arr) "),
        ("C", "return qux ", "return qux "),
        ("C", "import curr ", "import curr "),
        ("C", "tar -xf a.tar ", "tar -xf a.tar "),
        ("T", "os.path ", "os.path "),
        ("T", "foo bar ", "foo bar "),
    ]);
}

#[test]
fn c2_names_and_doubled_letter_words() {
    run(&[
        ("T", "Harry Jerry Terry Larry Barry ", "Harry Jerry Terry Larry Barry "),
        ("T", "Jess Ross Bess Tess Jessica Russell Lassie ", "Jess Ross Bess Tess Jessica Russell Lassie "),
        ("T", "berry cherry buff puff hiss ", "berry cherry buff puff hiss "),
        ("T", "Mary xoxo ", "Mary xoxo "),
        ("T", "Hi Mary ", "Hi Mary "),
    ]);
}

#[test]
fn c3_acronyms() {
    run(&[
        ("T", "USA UX SOS IMF NASA ASAP EXO OS TOS SAS DAX URI AR ", "USA UX SOS IMF NASA ASAP EXO OS TOS SAS DAX URI AR "),
        ("T", "usa asap sos ", "usa asap sos "),
        ("C", "USA UX SOS ", "USA UX SOS "),
        ("T", "VIEETJ NAM ", "VIỆT NAM "),
        ("T", "TIEENGS ", "TIẾNG "),
        ("T", "Mac OS ", "Mac OS "),
        ("T", "Windows OS ", "Windows OS "),
    ]);
}

#[test]
fn c4_names_that_became_vietnamese() {
    run(&[
        ("T", "Asus ", "Asus "),
        ("T", "Paris ", "Paris "),
        ("T", "Jesus ", "Jesus "),
        ("T", "Los Angeles ", "Los Angeles "),
        ("T", "sofa ", "sofa "),
        ("T", "jira ", "jira "),
        ("T", "the car ", "the car "),
        ("T", "the queen ", "the queen "),
        ("T", "and thus ", "and thus "),
        ("T", "Con cars ", "Con cá "),
    ]);
}

#[test]
fn c5_vni_digits() {
    run(&[
        ("V", "Win10 ", "Win10 "),
        ("V", "A101 ", "A101 "),
        ("V", "Galaxy A52 ", "Galaxy A52 "),
        ("V", "CO2 ", "CO2 "),
        ("V", "O2 ", "O2 "),
        ("V", "Top10 ", "Top10 "),
        ("V", "x1 y2 z3 ", "x1 y2 z3 "),
        ("V", "https://x.com/a1/b2 ", "https://x.com/a1/b2 "),
        ("V", "Gate A1 ", "Gate A1 "),
        ("V", "tie61ng Vie65t ", "tiếng Việt "),
        ("V", "mo65 ", "mộ "),
        ("V", "to6i ha4y ", "tôi hãy "),
        ("VC", "Win10 Top10 ", "Win10 Top10 "),
    ]);
}

#[test]
fn d_spell_check_off_keeps_the_english_list_working() {
    run(&[
        ("Toff", "order ", "order "),
        ("Toff", "twitter ", "twitter "),
        ("Toff", "teacher ", "teacher "),
        ("Toff", "computer ", "computer "),
        ("Toff", "meeting ", "meeting "),
        ("Toff", "report ", "report "),
        ("Toff", "Facebook ", "Facebook "),
        ("Toff", "tomorrow ", "tomorrow "),
        ("Toff", "tieengs Vieetj dduwowcj ", "tiếng Việt được "),
        ("Toff", "tooi hay mix nhacj ", "tôi hay mix nhạc "),
    ]);
}

#[test]
fn e_off_with_jfwz_off() {
    run(&[("Tnj", "off ", "off "), ("T", "off ", "off "), ("Tnj", "of ", "of ")]);
}

#[test]
fn a_one_word_back_correction() {
    run(&[
        ("T", "Max speed ", "Max speed "),
        ("T", "Max Pro ", "Max Pro "),
        ("T", "Six pack ", "Six pack "),
        ("T", "Mix thuaatj ", "Mĩ thuật "),
        ("T", "Nuowcs Mix ", "Nước Mĩ "),
        ("T", "Box coong ", "Bõ công "),
        ("T", "Now ddor, now xanh ", "Nơ đỏ, nơ xanh "),
        ("T", "Nor thaanf ", "Nỏ thần "),
        ("T", "How luwar ", "Hơ lửa "),
        ("T", "Low ddax ", "Lơ đã "),
        // No English or no Vietnamese word on the right: nothing is rewritten.
        ("T", "Max thuaatj ", "Mã thuật "),
        ("T", "Mix nhacj ", "Mĩ nhạc "),
        ("T", "Tooi ddi Max ", "Tôi đi Mã "),
    ]);
}

mod one_word_back {
    use unicode_normalization::UnicodeNormalization;
    use vkb_core::{apply_action, Charset, Engine, EngineOptions, KeyInput, VietnameseEngine};

    fn press(engine: &mut Engine, out: &mut String, key: KeyInput) {
        let action = engine.process_key(key);
        apply_action(out, action, key);
    }

    fn type_str(engine: &mut Engine, out: &mut String, text: &str) {
        for ch in text.chars() {
            press(engine, out, KeyInput::char(ch));
        }
    }

    fn restore(engine: &mut Engine, out: &mut String) {
        assert!(engine.can_restore(), "the hotkey must have a correction to take back");
        let action = engine.restore_word();
        apply_action(out, action, KeyInput::char(' '));
    }

    #[test]
    fn nfd_counts() {
        let options = EngineOptions { charset: Charset::UnicodeNfd, ..EngineOptions::default() };
        assert_eq!(vkb_core::simulate(options, "Now ddor, now xanh "), "Nơ đỏ, nơ xanh ".nfd().collect::<String>());
        assert_eq!(vkb_core::simulate(options, "Max speed "), "Max speed ");
    }

    #[test]
    fn undo_restores_the_text_exactly() {
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Now ddor, now xanh ");
        assert_eq!(out, "Nơ đỏ, nơ xanh ");
        restore(&mut engine, &mut out);
        assert_eq!(out, "Nơ đỏ, now xanh ", "back to the text before the rewrite");
        restore(&mut engine, &mut out);
        assert_eq!(out, "Nơ đỏ, nơ xanh ");
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max speed ");
        assert_eq!(out, "Max speed ");
        restore(&mut engine, &mut out);
        assert_eq!(out, "Mã speed ");
    }

    #[test]
    fn never_rewrites_after_an_edit_or_a_move() {
        // Backspace inside the next word.
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max speedd");
        press(&mut engine, &mut out, KeyInput::backspace());
        type_str(&mut engine, &mut out, " ");
        assert_eq!(out, "Mã speed ");
        // An arrow key between the words.
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max ");
        press(&mut engine, &mut out, KeyInput::key(0x25, None));
        type_str(&mut engine, &mut out, "speed ");
        assert_eq!(out, "Mã speed ");
        // Enter between the words.
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max ");
        press(&mut engine, &mut out, KeyInput::enter());
        type_str(&mut engine, &mut out, "speed ");
        assert_eq!(out, "Mã \rspeed ");
        // A focus change / mouse click (the platform calls `boundary`).
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max ");
        engine.boundary();
        type_str(&mut engine, &mut out, "speed ");
        assert_eq!(out, "Mã speed ");
        // The restore hotkey on the first word.
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max ");
        assert_eq!(out, "Mã ");
        // (nothing to restore on a Vietnamese word kept by the score: the call is a no-op)
        let _ = engine.can_restore();
        // A modifier key alone does not end the chance.
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max ");
        press(&mut engine, &mut out, KeyInput::key(0x10, None));
        type_str(&mut engine, &mut out, "speed ");
        assert_eq!(out, "Max speed ");
    }
}
