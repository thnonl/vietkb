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
        ("C", "const res = 1 ", "const res = 1 "),
        ("C", "return qux ", "return qux "),
        ("C", "import curr ", "import curr "),
        ("T", "os.path ", "os.path "),
        ("T", "foo ", "foo "),
    ]);
}

#[test]
fn c2_names_and_doubled_letter_words() {
    run(&[
        ("T", "Harry Jerry Terry Larry Barry ", "Harry Jerry Terry Larry Barry "),
        ("T", "Jess Ross Bess Tess Jessica Russell Lassie ", "Jess Ross Bess Tess Jessica Russell Lassie "),
        ("T", "berry cherry buff puff hiss ", "berry cherry buff puff hiss "),
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
        ("T", "the car ", "the cả "),
        ("T", "and thus ", "and thú "),
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
        ("V", "x1 y2 z3 ", "x1 ỳ z3 "),
        ("V", "https://x.com/a1/b2 ", "https://x.com/a1/b2 "),
        ("V", "Gate A1 ", "Gate Á "),
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
    run(&[("Tnj", "off ", "off "), ("T", "off ", "off "), ("Tnj", "of ", "ò ")]);
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


/// Same text as with the feature off (0.2.7): the doubt-means-Vietnamese check.
fn same_as_off(profile: &str, keys: &str) -> Result<(), String> {
    let on = simulate(options(profile), keys);
    let off = simulate(EngineOptions { auto_restore_english: false, ..options(profile) }, keys);
    if on == off {
        Ok(())
    } else {
        Err(format!("[{profile}] {keys:?}\n    on:  {on:?}\n    off: {off:?}"))
    }
}

fn all_same_as_off(profiles: &[&str], keys: &[String]) {
    let mut failures = Vec::new();
    for profile in profiles {
        for k in keys {
            if let Err(e) = same_as_off(profile, k) {
                failures.push(e);
            }
        }
    }
    assert!(failures.is_empty(), "{} differ from feature-off:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn round3_line_start_followed_by_english_stays_vietnamese() {
    let mut keys: Vec<String> = Vec::new();
    let phrases = [
        "Bans online luoon", "Hats live", "Mas shop", "Cats store", "Bans shop quaanf aos", "OW app", "Tows meeting",
        "Boos thank you", "Lee thank you", "Queen thank you",
    ];
    let mut shop: Vec<String> = ["bow", "cow", "how", "now", "mow", "sow", "tow", "low", "row", "vow"]
        .iter()
        .chain(["sir", "boost", "box", "bee", "been", "chef", "gas", "has", "bis", "hoof", "bar"].iter())
        .map(|w| format!("{w} shop"))
        .collect();
    shop.extend(phrases.iter().map(|s| s.to_string()));
    for prefix in ["", "Xong roofi. ", "1. ", "- "] {
        for phrase in &shop {
            keys.push(format!("{prefix}{phrase} "));
        }
    }
    all_same_as_off(&["T", "C"], &keys);
    run(&[("T", "Bans online luoon ", "Bán online luôn "), ("T", "OW app ", "Ơ app "), ("C", "Mas shop ", "Má shop ")]);
}

#[test]
fn round3_function_word_left_keeps_plausible_vietnamese() {
    let keys: Vec<String> = [
        "I love you mas ddi ", "thank you car nhaf ", "check in cos khoong ", "I will cos khoong ",
        "thank you Cos 3 nguwowif ",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    all_same_as_off(&["T", "C"], &keys);
    run(&[("T", "I love you mas ddi ", "I love you má đi "), ("T", "thank you car nhaf ", "thank you cả nhà ")]);
}

#[test]
fn round3_all_caps_interjections() {
    run(&[
        ("T", "I love you AS! DDau quas ", "I love you Á! Đau quá "),
        ("T", "no problem AS! DDau quas ", "no problem Á! Đau quá "),
        ("T", "pull request AS! DDau quas ", "pull request Á! Đau quá "),
    ]);
}

#[test]
fn round3_code_app_operators() {
    run(&[
        ("C", "x = Cos x ", "x = Có x "),
        ("C", "x = cos ", "x = cos "),
        ("C", "x != cos ", "x != cos "),
        ("C", "x == cos ", "x == cos "),
        ("C", "x | cos ", "x | cos "),
        ("C", "x && cos ", "x && cos "),
        ("C", "x + cos ", "x + cos "),
        ("C", "x * cos ", "x * cos "),
        ("C", "x < cos ", "x < cos "),
        ("C", "x > cos ", "x > cos "),
        ("C", "f(cos ", "f(cos "),
        ("C", "x[cos ", "x[cos "),
        ("C", "x{cos ", "x{cos "),
        ("C", "x != Cos ", "x != Có "),
        ("C", "f(Cos ", "f(Có "),
    ]);
}

#[test]
fn round3_separators_and_comment_markers() {
    run(&[
        ("T", "//Nguyeexn x ", "//Nguyễn x "),
        ("T", "/Uwf x ", "/Ừ x "),
        ("T", "\\Uwf x ", "\\Ừ x "),
        ("T", "~Uwf x ", "~Ừ x "),
        ("C", "//Nguyeexn x ", "//Nguyễn x "),
        ("T", "cd /usr/bin ", "cd /usr/bin "),
        ("T", "~/Downloads ", "~/Downloads "),
    ]);
}

#[test]
fn round3_telex_leftovers() {
    run(&[("T", "dD x ", "đ x "), ("T", "ar arr arrr x ", "ả ar arr x ")]);
    all_same_as_off(&["T", "C"], &["dD x ".to_string(), "ar arr arrr x ".to_string()]);
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
        assert_eq!(vkb_core::simulate(options, "Moon toans, moon vawn "), "Môn toán, môn văn ".nfd().collect::<String>());
        assert_eq!(vkb_core::simulate(options, "Max speed "), "Max speed ");
    }

    #[test]
    fn undo_restores_the_text_exactly() {
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max speed ");
        assert_eq!(out, "Max speed ");
        restore(&mut engine, &mut out);
        assert_eq!(out, "Mã speed ", "back to the text before the rewrite");
        restore(&mut engine, &mut out);
        assert_eq!(out, "Max speed ");
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
        // The restore hotkey on the first word cancels the chance too.
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max ");
        assert_eq!(out, "Mã ");
        let action = engine.restore_word();
        apply_action(&mut out, action, KeyInput::key(0, None));
        type_str(&mut engine, &mut out, "speed ");
        assert_eq!(out, "Mã speed ");
        // A modifier key alone does not end the chance.
        let mut engine = Engine::new(EngineOptions::default());
        let mut out = String::new();
        type_str(&mut engine, &mut out, "Max ");
        press(&mut engine, &mut out, KeyInput::key(0x10, None));
        type_str(&mut engine, &mut out, "speed ");
        assert_eq!(out, "Max speed ");
    }
}

#[test]
fn round2_vietnamese_regressions() {
    run(&[
        // môn, not moon
        ("T", "Booj moon mix thuaatj ", "Bộ môn mĩ thuật "),
        ("T", "Moon toans, moon vawn, moon lys ", "Môn toán, môn văn, môn lý "),
        // hơ lơ chơ nơ tơ bơ
        ("T", "Gios thooir how how ", "Gió thổi hơ hơ "),
        ("T", "Hoj how luwar ", "Họ hơ lửa "),
        ("T", "low mow, low laf ", "lơ mơ, lơ là "),
        ("T", "thawts now ddor ", "thắt nơ đỏ "),
        ("T", "chow vow ", "chơ vơ "),
        ("T", "Tow luaj ", "Tơ lụa "),
        // bú, mố
        ("T", "bus ", "bú "),
        ("T", "moos ", "mố "),
        // lải láp chát
        ("T", "lair nhair ", "lải nhải "),
        ("T", "laps xe ", "láp xe "),
        ("T", "chats chuas ", "chát chúa "),
        // tã
        ("T", "Tax lots, tax quaanf, tax dans ", "Tã lót, tã quần, tã dán "),
        ("T", "Mej mua tax lots ", "Mẹ mua tã lót "),
        // á, thê, sê
        ("T", "Oof, af, as ", "Ồ, à, á "),
        ("T", "Baf owi, as, con sowj ", "Bà ơi, á, con sợ "),
        ("T", "thee luwowng ", "thê lương "),
        ("T", "see ", "sê "),
        // the same in a code app: prose types like a neutral app
        ("C", "Max soos thuees ", "Mã số thuế "),
        ("C", "Mar toor ", "Mả tổ "),
        ("C", "Hair sanr ", "Hải sản "),
        ("C", "Tax giaays ", "Tã giấy "),
        // wrong previous-word fixes
        ("T", "Anh aays mix mieeuf quas ", "Anh ấy mĩ miều quá "),
        ("T", "mix ngheej, mix vieenj, mix camr ", "mĩ nghệ, mĩ viện, mĩ cảm "),
        ("T", "Mix nuwx, mix thuaatj, mix phaamr, mix mieeuf ", "Mĩ nữ, mĩ thuật, mĩ phẩm, mĩ miều "),
        ("C", "Mix thuaatj laf moon hocj ", "Mĩ thuật là môn học "),
        // VNI one-letter words
        ("V", "Y1 kie6n1 cua3 anh ", "Ý kiến của anh "),
        ("V", "E6, d9i d9a6u d9a6y1 ", "Ê, đi đâu đấy "),
        ("V", "O7 ki2a ", "Ơ kìa "),
        ("V", "U3 a6m1 cho be1 ", "Ủ ấm cho bé "),
        ("V", "y3 la5i ", "ỷ lại "),
        ("V", "y1 e6 o7 u3 y3 ", "ý ê ơ ủ ỷ "),
        // spell check off
        ("Toff", "Seeps baor mootj vieecj toots ", "Sếp bảo một việc tốt "),
        ("Toff", "bootj mif ", "bột mì "),
        ("Toff", "Muwowif Mootj ", "Mười Một "),
    ]);
}

#[test]
fn owner_examples_still_hold() {
    run(&[
        ("T", "tooi hay mix nhacj ", "tôi hay mix nhạc "),
        ("T", "tuooir teen ", "tuổi teen "),
        ("T", "if ", "if "),
        ("C", "if ", "if "),
        ("T", "order ", "order "),
        ("T", "why ", "why "),
        ("T", "twitter ", "twitter "),
        ("T", "user_id ", "user_id "),
        ("T", "getValue ", "getValue "),
        ("T", "iOS ", "iOS "),
        ("T", "I like mix ", "I like mix "),
        ("T", "Max speed ", "Max speed "),
        ("T", "Six pack ", "Six pack "),
        ("T", "user were ", "user were "),
    ]);
}

#[test]
fn one_word_back_never_touches_common_vietnamese() {
    // Review: the left word must be an English base form whose Vietnamese reading is not common.
    let pairs = [
        ("Teen file ", "Tên file "),
        ("Sangs team ", "Sáng team "),
        ("Been server ", "Bên server "),
        ("Toots API ", "Tốt API "),
        ("Boos API ", "Bố API "),
        ("Teen file laf gif? ", "Tên file là gì? "),
        ("Sangs team oi ", "Sáng team oi "),
        ("Toots code quas ", "Tốt code quá "),
        ("Been server ddang chayj ", "Bên server đang chạy "),
        ("Boos API ddax xong ", "Bố API đã xong "),
    ];
    let mut table: Vec<(&str, String, String)> = Vec::new();
    for (keys, expected) in pairs {
        table.push(("T", keys.to_string(), expected.to_string()));
        table.push(("C", keys.to_string(), expected.to_string()));
        table.push(("T", format!("Xong roofi. {keys}"), format!("Xong rồi. {expected}")));
        table.push(("T", format!("Cos bug. {keys}"), format!("Có bug. {expected}")));
    }
    let rows: Vec<(&str, &str, &str)> = table.iter().map(|(p, k, e)| (*p, k.as_str(), e.as_str())).collect();
    run(&rows);
    // The two that must still work, and after a full stop.
    run(&[
        ("T", "Max speed ", "Max speed "),
        ("T", "Max Pro ", "Max Pro "),
        ("T", "Xong roofi. Max speed ", "Xong rồi. Max speed "),
        ("T", "Six pack ", "Six pack "),
        ("T", "Six thing ", "Sĩ thing "),
        ("T", "Max thing ", "Mã thing "),
        ("T", "Max level ", "Mã level "),
        ("T", "Six sigma ", "Sĩ sigma "),
    ]);
}

#[test]
fn vni_tone_changes_are_not_code() {
    run(&[
        ("V", "ba12 ", "bà "),
        ("V", "ca13 ", "cả "),
        ("V", "a12 ", "à "),
        ("V", "ho21 ", "hó "),
        ("V", "to6i12 ", "tồi "),
        ("V", "ngu7o7i21 ", "ngưới "),
        ("V", "Win10 ", "Win10 "),
        ("V", "A52 ", "A52 "),
        ("V", "A101 ", "A101 "),
        ("V", "Top10 ", "Top10 "),
    ]);
}

#[test]
fn owner_rule_no_vietnamese_word_starts_with_f_j_z_w() {
    // Raw from the first key, at a line start and mid-sentence, with spell check on
    // and off, in neutral and code apps.
    let words = [
        "Fix", "fix", "Firebase", "format", "false", "function", "file", "for", "from", "Zalo", "zoom", "zero", "zip",
        "json", "java", "jira", "just", "join", "Facebook", "Java", "JSON", "Jira", "Zoom",
    ];
    let mut table: Vec<(&str, String, String)> = Vec::new();
    for word in words {
        for profile in ["T", "C", "Toff"] {
            table.push((profile, format!("{word} "), format!("{word} ")));
            table.push((profile, format!("Xong roofi. {word} "), format!("Xong rồi. {word} ")));
            table.push((profile, format!("tooi {word} "), format!("tôi {word} ")));
            table.push((profile, format!("{word}. "), format!("{word}. ")));
        }
    }
    table.push(("T", "Fix looix ".to_string(), "Fix lỗi ".to_string()));
    table.push(("T", "fix bug ".to_string(), "fix bug ".to_string()));
    table.push(("C", "Fix looix ".to_string(), "Fix lỗi ".to_string()));
    let rows: Vec<(&str, &str, &str)> = table.iter().map(|(p, k, e)| (*p, k.as_str(), e.as_str())).collect();
    run(&rows);
    // With `f j w z` not consonants f, j and z are still never Vietnamese.
    for word in ["Fix", "format", "false", "function", "file", "for", "from", "Zalo", "zoom", "zero", "zip", "json", "java", "jira", "just", "join"] {
        run(&[("Tnj", &format!("{word} "), &format!("{word} ")), ("Tnj", &format!("tooi {word} "), &format!("tôi {word} "))]);
    }
}

#[test]
fn owner_rule_w_stays_with_the_normal_rules() {
    // `w` is not in the rule: Telex `w` is ư. A leading w is a consonant by default
    // (0.2.7), ư after a vowel or with the option off; nothing here changes that.
    let keys: Vec<String> = [
        "wa ", "wowc ", "wf ", "wuwng ", "wung ", "uwng ", "uwa ", "uwowcs ", "uwngs ", "Wa ", "tooi wf ",
        "Xong roofi. wowc ",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    all_same_as_off(&["T", "C", "Tnj", "Toff"], &keys);
    run(&[
        // ưa ước ứng ưng
        ("T", "uwa ", "ưa "),
        ("T", "uwowcs ", "ước "),
        ("T", "uwngs ", "ứng "),
        ("T", "uwng ", "ưng "),
        // ừ typed with a leading w needs the option off (0.2.7)
        ("Tnj", "wf ", "ừ "),
        ("Tnj", "uwng ", "ưng "),
        // English that cannot be Vietnamese stays raw
        // `Wơ` cannot be Vietnamese (w onset): the English word wins.
        ("T", "Wow ", "Wow "),
        ("T", "Wifi ", "Wifi "),
        ("T", "web ", "web "),
        ("T", "while ", "while "),
        ("T", "why ", "why "),
        ("T", "with ", "with "),
        ("T", "window ", "window "),
        ("T", "tooi dungf web ", "tôi dùng web "),
        ("Toff", "web ", "web "),
        ("Toff", "why ", "why "),
    ]);
}
