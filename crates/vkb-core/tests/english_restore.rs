//! D10 (0.2.8): `Tự nhận biết tiếng Anh và code`.
//!
//! Every case types raw keystrokes through the engine the way a backend would
//! (`simulate`) and compares the text on screen. A trailing space ends the last
//! word, because the decision is taken at the word break.

use vkb_core::{
    simulate, simulate_keys, AppKind, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine,
};

fn telex() -> EngineOptions {
    EngineOptions::default()
}

fn code_app() -> EngineOptions {
    EngineOptions { app_kind: AppKind::Code, ..telex() }
}

fn vni() -> EngineOptions {
    EngineOptions { method: InputMethod::Vni, ..telex() }
}

fn off() -> EngineOptions {
    EngineOptions { auto_restore_english: false, ..telex() }
}

fn check(options: EngineOptions, cases: &[(&str, &str)]) {
    let mut failures = Vec::new();
    for (keys, expected) in cases {
        let got = simulate(options, keys);
        if &got != expected {
            failures.push(format!("keys {keys:?}: got {got:?}, want {expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn english_words_come_back_as_typed() {
    // 0.2.7 showed ủe úe wể hể for these.
    check(
        telex(),
        &[
            ("user ", "user "),
            ("use ", "use "),
            ("were ", "were "),
            ("here ", "hể "),
            // `has` is a Vietnamese `há` that a line start keeps; after English it is `has`.
            ("has ", "há "),
            ("tax ", "tã "),
            ("see ", "sê "),
            ("through ", "through "),
            ("USER ", "USER "),
            ("User ", "User "),
            ("user, ", "user, "),
            ("user. ", "user. "),
            ("user\r", "user\r"),
        ],
    );
}

#[test]
fn already_raw_words_stay_raw() {
    check(
        telex(),
        &[("set test case window google install string class import export ", "set test case window google install string class import export ")],
    );
}

#[test]
fn start_of_sentence_common_vietnamese_wins() {
    check(
        telex(),
        &[
            ("cos ", "có "),
            ("nos ", "nó "),
            ("there ", "thể "),
            ("teen ", "tên "),
            ("max ", "mã "),
            ("sex ", "sẽ "),
        ],
    );
}

#[test]
fn start_of_sentence_rare_vietnamese_loses_to_common_english() {
    // Owner amendment 2: "if" at the start is `if`, not `ì`.
    // Only a keyword whose Vietnamese twin is rare stays English at a line start.
    check(telex(), &[("if ", "if "), ("is ", "í "), ("or ", "ỏ "), ("of ", "of "), ("us ", "ú ")]);
    check(code_app(), &[("if ", "if "), ("is ", "í ")]);
}

#[test]
fn context_decides_the_ambiguous_words() {
    check(
        telex(),
        &[
            // The owner's examples.
            ("tooi hay mix nhacj chung vowis tieengs oonf ", "tôi hay mix nhạc chung với tiếng ồn "),
            ("tuooir teen ", "tuổi teen "),
            // Vietnamese contexts keep the Vietnamese word.
            ("tooi cos ", "tôi có "),
            ("nhaf cos ", "nhà có "),
            ("ddawjt teen cho con ", "đặt tên cho con "),
            ("teen tooi laf Nam ", "tên tôi là Nam "),
            ("Vieetj Nam laf nos ", "Việt Nam là nó "),
            // After English the English word wins.
            ("I like mix ", "I like mix "),
            // After an English function word the programming word wins over `có`.
            ("the cos ", "the có "),
            ("user max ", "user max "),
            // Context is cut by punctuation: after a comma it is a fresh start,
            // and a common English word beats the rare `mĩ`.
            ("tooi hay, mix ", "tôi hay, mĩ "),
        ],
    );
}

#[test]
fn double_letter_english_words() {
    // ee/oo/aa/dd transforms, not tone keys: English in an English context,
    // Vietnamese against common Vietnamese uses.
    check(
        telex(),
        &[
            ("I see ", "I see "),
            // (`the moon` is `the môn` now: moon has a common Vietnamese reading.)
            ("tuooir teen ", "tuổi teen "),
            ("tooi hay teen ", "tôi hay tên "),
            ("hocj mon ", "học mon "),
            ("tên ", "tên "),
            ("teen tooi ", "tên tôi "),
            ("mooin ", "mooin "),
        ],
    );
    // Words whose Telex form is not a Vietnamese syllable are raw already.
    check(
        telex(),
        // (`door` after English would be `dổ` now: a doubtful call is Vietnamese.)
        &[("good book food cool pool need feed keep ", "good book food cool pool need feed keep ")],
    );
}

#[test]
fn code_apps_lean_english_but_vietnamese_comments_work() {
    check(
        code_app(),
        &[
            ("user ", "user "),
            // A protected Vietnamese word at a line start stays Vietnamese in a code app.
            ("cos ", "có "),
            ("Cos bajn hoir ", "Có bạn hỏi "),
            // The app alone never flips a plausible Vietnamese word.
            ("max ", "mã "),
            ("mix ", "mĩ "),
            // `thể` is a common Vietnamese word: only a programming word turns it into code.
            ("there ", "thể "),
            // Vietnamese line.
            ("tieengs Vieetj ", "tiếng Việt "),
            ("tooi cos nhaf ", "tôi có nhà "),
            ("nhaf cos ", "nhà có "),
            ("nhaf teen ", "nhà tên "),
            ("Vieetj Nam laf nos ", "Việt Nam là nó "),
        ],
    );
}

#[test]
fn code_shapes_keep_the_typed_keys() {
    check(
        telex(),
        &[
            ("user_id ", "user_id "),
            ("as_str ", "as_str "),
            ("is_empty ", "is_empty "),
            ("my_user_name ", "my_user_name "),
            ("getValue ", "getValue "),
            ("HttpClient ", "HttpClient "),
            ("useEffect ", "useEffect "),
            ("setUserName ", "setUserName "),
            ("iPhone ", "iPhone "),
            ("foo.bar ", "foo.bar "),
            ("foo.bar.baz ", "foo.bar.baz "),
            ("std::io ", "std::io "),
            ("a->b ", "a->b "),
            ("self.fieldd ", "self.fieldd "),
            ("item2 ", "item2 "),
            ("utf8 ", "utf8 "),
            ("user@mail.com ", "user@mail.com "),
        ],
    );
}

#[test]
fn sentence_punctuation_stays_vietnamese() {
    check(
        telex(),
        &[
            ("cos. ", "có. "),
            ("Vieetj. ", "Việt. "),
            ("Vieetj Nam. Tooi ", "Việt Nam. Tôi "),
            ("cos, Vieetj? ", "có, Việt? "),
            ("kinh-doanh ", "kinh-doanh "),
            ("baajt/tawst ", "bật/tắt "),
            ("tieengs: ", "tiếng: "),
            ("khoong! ", "không! "),
            ("3.5 ", "3.5 "),
            ("coos... ", "cố... "),
        ],
    );
}

#[test]
fn vietnamese_followed_directly_by_a_letter_after_a_dot_is_left_alone() {
    // "có.Việt": a common Vietnamese word is never code.
    check(telex(), &[("cos.Vieetj ", "có.Việt ")]);
}

#[test]
fn vni_rules() {
    check(
        vni(),
        &[
            // Digits are tone keys in VNI; they never make a word code.
            ("tie61ng Vie65t ", "tiếng Việt "),
            ("to6i ha4y ", "tôi hãy "),
            ("co1 ", "có "),
            // English letters are never altered in VNI, so there is nothing to restore.
            ("user use here ", "user use here "),
            ("user_id ", "user_id "),
            ("getValue ", "getValue "),
            ("foo.bar ", "foo.bar "),
            ("item2 ", "item2 "),
            ("utf8 ", "utf8 "),
            ("a->b ", "a->b "),
        ],
    );
}

#[test]
fn option_off_is_exactly_0_2_7() {
    check(
        off(),
        &[
            ("user ", "ủe "),
            ("use ", "úe "),
            ("were ", "wể "),
            ("here ", "hể "),
            ("there ", "thể "),
            ("mix ", "mĩ "),
            ("user_id ", "ủe_id "),
            ("tooi hay mix ", "tôi hay mĩ "),
            ("tieengs Vieetj ", "tiếng Việt "),
        ],
    );
}

#[test]
fn works_with_spell_check_off() {
    let options = EngineOptions { spell_check: false, ..telex() };
    check(options, &[("user ", "user "), ("were ", "were "), ("cos ", "có "), ("tooi hay mix ", "tôi hay mix ")]);
}

#[test]
fn works_with_restore_wrong_word_on() {
    let options = EngineOptions { restore_wrong_word: true, ..telex() };
    check(options, &[("user ", "user "), ("tieengsk ", "tieengsk "), ("tieengs ", "tiếng "), ("tooi hay mix ", "tôi hay mix ")]);
}

#[test]
fn mid_word_switch_to_raw() {
    let mut engine = Engine::new(telex());
    let mut shown = String::new();
    let mut after = Vec::new();
    for ch in "order".chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        vkb_core::apply_action(&mut shown, action, key);
        after.push(shown.clone());
    }
    // `or` is still a Vietnamese syllable (`ỏ`); `ord` cannot be one.
    assert_eq!(after, ["o", "ỏ", "ord", "orde", "order"]);
    check(
        telex(),
        &[
            ("order ", "order "),
            ("why ", "why "),
            ("twitter ", "twitter "),
            ("what ", "what "),
            ("where ", "where "),
            ("with ", "with "),
            ("world ", "world "),
            ("swift ", "swift "),
            ("strong ", "strong "),
            ("script ", "script "),
            ("schema ", "schema "),
            ("through ", "through "),
            ("export ", "export "),
            ("query ", "query "),
        ],
    );
    let jfwz_off = EngineOptions { jfwz_as_consonant: false, ..telex() };
    check(jfwz_off, &[("swift ", "swift "), ("twitter ", "twitter "), ("why ", "why ")]);
}

#[test]
fn mid_word_never_fires_on_vietnamese() {
    check(
        telex(),
        &[
            ("nghi ", "nghi "),
            ("khuy ", "khuy "),
            ("uow ", "ươ "),
            ("truowngf ", "trường "),
            ("dd ", "đ "),
            ("Vieetj ", "Việt "),
            ("tieengs ", "tiếng "),
            ("muoons ", "muốn "),
            ("nguowif ", "người "),
            ("quaanf ", "quần "),
            ("giuwax ", "giữa "),
            // The tone key may come before the final consonant (free tone).
            ("tieesng ", "tiếng "),
            ("hoafn ", "hoàn "),
        ],
    );
}

/// Every real syllable the corpora saw, typed in Telex and VNI with the tone at
/// the end and right after the nucleus, must come out the same as with the
/// feature off: the prefix-feasibility check never trips on Vietnamese.
#[test]
fn feasibility_never_trips_on_a_real_syllable() {
    let vi = include_str!("../data/vi.txt");
    let mut checked = 0usize;
    for line in vi.lines() {
        let key = line.split(' ').next().unwrap();
        for (method, variants) in [(InputMethod::Telex, telex_variants(key)), (InputMethod::Vni, vni_variants(key))] {
            for typed in variants {
                let on = EngineOptions { method, ..telex() };
                let base = EngineOptions { method, auto_restore_english: false, ..telex() };
                let (with, without) = (simulate(on, &typed), simulate(base, &typed));
                assert_eq!(with, without, "{method:?} {key} typed {typed:?}");
                assert!(!with.is_empty());
                checked += 1;
            }
        }
    }
    assert!(checked > 8_000, "only {checked} typing paths");
}

/// Letters of a syllable key: `(base, mark)`; tone digit separate.
fn parse_key(key: &str) -> (Vec<(char, char)>, Option<char>) {
    let mut letters: Vec<(char, char)> = Vec::new();
    let mut tone = None;
    for ch in key.chars() {
        match ch {
            '^' | '(' | '+' => letters.last_mut().unwrap().1 = ch,
            '1'..='5' => tone = Some(ch),
            'D' => letters.push(('d', '-')),
            _ => letters.push((ch, ' ')),
        }
    }
    (letters, tone)
}

fn last_vowel(letters: &[(char, char)]) -> usize {
    letters.iter().rposition(|(c, _)| "aeiouy".contains(*c)).map_or(letters.len(), |i| i + 1)
}

fn telex_variants(key: &str) -> Vec<String> {
    let (letters, tone) = parse_key(key);
    let tone_key = tone.map(|t| ['s', 'f', 'r', 'x', 'j'][t as usize - b'1' as usize]);
    let mut body = String::new();
    let mut at_vowel_end = 0usize;
    for (i, (c, m)) in letters.iter().enumerate() {
        match m {
            '^' => {
                body.push(*c);
                body.push(*c);
            }
            '(' => body.push_str("aw"),
            '+' => {
                body.push(*c);
                body.push('w');
            }
            '-' => body.push_str("dd"),
            _ => body.push(*c),
        }
        if i + 1 == last_vowel(&letters) {
            at_vowel_end = body.len();
        }
    }
    let mut out = Vec::new();
    match tone_key {
        Some(t) => {
            out.push(format!("{body}{t}"));
            let mut mid = body.clone();
            mid.insert(at_vowel_end, t);
            out.push(mid);
        }
        None => out.push(body),
    }
    out
}

fn vni_variants(key: &str) -> Vec<String> {
    let (letters, tone) = parse_key(key);
    let mut body = String::new();
    let mut at_vowel_end = 0usize;
    for (i, (c, m)) in letters.iter().enumerate() {
        body.push(*c);
        match m {
            '^' => body.push('6'),
            '(' => body.push('8'),
            '+' => body.push('7'),
            '-' => body.push('9'),
            _ => {}
        }
        if i + 1 == last_vowel(&letters) {
            at_vowel_end = body.len();
        }
    }
    let mut out = Vec::new();
    match tone {
        Some(t) => {
            out.push(format!("{body}{t}"));
            let mut mid = body.clone();
            mid.insert(at_vowel_end, t);
            out.push(mid);
        }
        None => out.push(body),
    }
    out
}

#[test]
fn history_is_cut_by_enter_and_focus_changes() {
    // Enter ends the sentence: `mix` at the start of the next line has no left
    // context, so the common Vietnamese reading of `if`-like rare words does not
    // matter but a common one (`cos`) wins.
    let keys: Vec<KeyInput> = "tooi hay\rcos ".chars().map(|c| if c == '\r' { KeyInput::enter() } else { KeyInput::char(c) }).collect();
    assert_eq!(simulate_keys(telex(), &keys), "tôi hay\rcó ");
    let mut engine = Engine::new(telex());
    let mut shown = String::new();
    for ch in "user ".chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        vkb_core::apply_action(&mut shown, action, key);
    }
    engine.boundary();
    for ch in "cos ".chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        vkb_core::apply_action(&mut shown, action, key);
    }
    assert_eq!(shown, "user có ");
}

#[test]
fn common_vietnamese_words_after_a_vietnamese_word_never_flip() {
    // Review blocker: a missing word pair is neutral, a protected word stays.
    for options in [telex(), code_app()] {
        check(
            options,
            &[
                ("Minhf teen laf Lan ", "Mình tên là Lan "),
                ("Em teen laf Lan ", "Em tên là Lan "),
                ("Con teen laf Lan ", "Con tên là Lan "),
                ("Chij teen laf Lan ", "Chị tên là Lan "),
                ("Ddang teen ", "Đang tên "),
                ("Con meof nayf ddang lens ddi ", "Con mèo này đang lén đi "),
                ("Minhf lens nhinf ", "Mình lén nhìn "),
                ("Em cos ", "Em có "),
                ("Con nos ", "Con nó "),
                ("Chij cos nhaf ", "Chị có nhà "),
                ("Anh nos ", "Anh nó "),
                ("nhaf cos ", "nhà có "),
            ],
        );
    }
    // The owner's examples still hold: they are English collocations.
    check(telex(), &[("tuooir teen ", "tuổi teen "), ("tooi hay mix ", "tôi hay mix ")]);
}

#[test]
fn rare_vietnamese_words_keep_their_marks() {
    // Review: the `.` lookahead, digits, `_` and `@` gate on the spelling, not on a corpus count.
    check(
        telex(),
        &[
            ("Con ngoongx.Con vitj ", "Con ngỗng.Con vịt "),
            ("Hocj sinh goir.Cos ", "Học sinh gỏi.Có "),
            ("Phuwowngf1 Quaanj3 ", "Phường1 Quận3 "),
            ("Ddaau, ngooix.Banj ", "Đâu, ngỗi.Bạn "),
            ("ngoongx_to ", "ngỗng_to "),
        ],
    );
    // Code still wins where the word cannot be Vietnamese.
    check(telex(), &[("foo.bar ", "foo.bar "), ("utf8 ", "utf8 "), ("foo_bar ", "foo_bar ")]);
}

#[test]
fn english_words_with_a_doubled_tone_key_are_restored() {
    for word in [
        "pass", "password", "message", "issue", "error", "sorry", "less", "miss", "boss", "possible", "effect", "coffee",
        "process", "access", "address", "success", "class", "mass", "guess", "press",
    ] {
        assert_eq!(simulate(telex(), &format!("{word} ")), format!("{word} "), "{word}");
        assert_eq!(simulate(telex(), &format!("tooi hay {word} ")), format!("tôi hay {word} "), "{word} in context");
    }
    // Mid-word: restored as soon as the keys are an English word.
    let mut engine = Engine::new(telex());
    let mut shown = String::new();
    feed(&mut engine, &mut shown, "pass");
    assert_eq!(shown, "pass");
}

#[test]
fn the_telex_undo_idiom_still_types_a_literal() {
    // `ass` types `as`, `lass` types `las`: the user means the literal.
    check(
        telex(),
        &[("ass ", "as "), ("lass ", "las "), ("iss ", "is "), ("uss ", "us "), ("tooi lass ", "tôi las ")],
    );
}

#[test]
fn paths_and_camel_case_with_vietnamese_parts() {
    check(
        telex(),
        &[
            ("cd /usr/bin ", "cd /usr/bin "),
            ("ls ~/docs/src ", "ls ~/docs/src "),
            ("baajt/tawst ", "bật/tắt "),
            ("VieetjNam ", "ViệtNam "),
            ("getValue ", "getValue "),
            ("iPhone ", "iPhone "),
            ("useEffect ", "useEffect "),
        ],
    );
}

fn feed(engine: &mut Engine, shown: &mut String, keys: &str) {
    for ch in keys.chars() {
        let key = KeyInput::char(ch);
        let action = engine.process_key(key);
        vkb_core::apply_action(shown, action, key);
    }
}

fn press_restore(engine: &mut Engine, shown: &mut String) -> bool {
    if !engine.can_restore() {
        return false;
    }
    let action = engine.restore_word();
    vkb_core::apply_action(shown, action, KeyInput::char(' '));
    true
}

#[test]
fn a_wrong_auto_restore_can_be_taken_back_and_put_again() {
    // At the word break: `mix ` stays English; the hotkey brings back `mĩ `, again -> `mix `.
    let mut engine = Engine::new(telex());
    let mut shown = String::new();
    feed(&mut engine, &mut shown, "tooi hay mix ");
    assert_eq!(shown, "tôi hay mix ");
    assert!(press_restore(&mut engine, &mut shown));
    assert_eq!(shown, "tôi hay mĩ ");
    assert!(press_restore(&mut engine, &mut shown));
    assert_eq!(shown, "tôi hay mix ");
    // The next typed key ends the chance.
    feed(&mut engine, &mut shown, "n");
    assert!(!engine.can_restore(), "a typed key ends the chance to take a correction back");
    // Mid-word: `pass` restored on the fourth key, the hotkey takes it back.
    let mut engine = Engine::new(telex());
    let mut shown = String::new();
    feed(&mut engine, &mut shown, "pass");
    assert_eq!(shown, "pass");
    assert!(press_restore(&mut engine, &mut shown));
    assert_eq!(shown, "pas");
    feed(&mut engine, &mut shown, " ");
    assert_eq!(shown, "pas ", "taken back: the word is left alone");
}

#[test]
fn restore_hotkey_without_a_correction_is_unchanged() {
    let mut engine = Engine::new(telex());
    let mut shown = String::new();
    feed(&mut engine, &mut shown, "tieengs ");
    assert!(!engine.can_restore(), "nothing to take back after a normal word break");
}

#[test]
fn code_shapes_keep_programming_words_in_a_code_app() {
    // `cos` is Vietnamese `có` at a line start, but the shape makes it code.
    check(
        code_app(),
        &[
            ("cos(x) ", "cos(x) "),
            ("Math.cos(x) ", "Math.cos(x) "),
            ("x = cos ", "x = cos "),
            ("y = max ", "y = max "),
            ("foo(cos ", "foo(cos "),
        ],
    );
    // The same Vietnamese words after a Vietnamese word, or alone, stay Vietnamese.
    check(code_app(), &[("nhaf cos ", "nhà có "), ("Cos nhaf ", "Có nhà "), ("tooi cos ", "tôi có ")]);
}

#[test]
fn product_names_with_an_inner_capital_stay_as_typed() {
    // Review blocker: `iOS` was `iÓ` (the S acted as the sắc key).
    check(
        telex(),
        &[
            ("iOS ", "iOS "),
            ("iPadOS ", "iPadOS "),
            ("macOS ", "macOS "),
            ("tvOS ", "tvOS "),
            ("watchOS ", "watchOS "),
            ("eBay ", "eBay "),
            ("iCloud ", "iCloud "),
            ("iMac ", "iMac "),
            ("eKYC ", "eKYC "),
            ("Ddi iOS 17 nhes ", "Đi iOS 17 nhé "),
            ("ddt iOS ", "ddt iOS "),
            ("VieetjNam ", "ViệtNam "),
            // After a split a capital is a letter, never a key.
            ("VieetjNamS ", "ViệtNamS "),
            // Shift+X as the tone key is 0.2.7 behaviour and stays.
            ("NguyeenX ", "Nguyễn "),
        ],
    );
    check(code_app(), &[("iOS ", "iOS "), ("macOS ", "macOS "), ("NguyeenX ", "Nguyễn ")]);
}

#[test]
fn slash_lists_are_not_paths() {
    check(
        telex(),
        &[
            ("xanh/ddor/tims/vangf ", "xanh/đỏ/tím/vàng "),
            ("ddaajm/nhatj ", "đậm/nhạt "),
            ("gaoj/luas ", "gạo/lúa "),
            ("ddieeuj/kieenj ", "điệu/kiện "),
            ("anh/chij ", "anh/chị "),
            // Real paths keep the typed keys.
            ("cd /usr/bin ", "cd /usr/bin "),
            ("~/Downloads ", "~/Downloads "),
            ("C:\\Users ", "C:\\Users "),
            ("./build.sh ", "./build.sh "),
            ("ls /usr/local/bin ", "ls /usr/local/bin "),
        ],
    );
}

#[test]
fn more_doubled_key_words() {
    for word in ["loss", "toss", "moss", "assess", "pass"] {
        assert_eq!(simulate(telex(), &format!("{word} ")), format!("{word} "), "{word}");
    }
    check(telex(), &[("lass ", "las "), ("bass ", "bas ")]);
}

#[test]
fn bang_is_sentence_punctuation_but_not_equals() {
    check(telex(), &[("Hay!Cos ", "Hay!Có "), ("wow! cos ", "wow! có ")]);
    check(code_app(), &[("x != cos ", "x != cos "), ("Hay! Cos ", "Hay! Có ")]);
}

#[test]
fn neutral_line_start_keeps_ordinary_vietnamese() {
    check(
        telex(),
        &[
            ("Chaos ", "Cháo "),
            ("Lens ", "Lén "),
            ("Hair ", "Hải "),
            ("Seen ", "Sên "),
            ("Chars ", "Chá "),
            ("If ", "If "),
            ("if ", "if "),
            // `Mix` and `Max` read as the Vietnamese names `Mĩ` and `Mã` at a line start;
            // `hay mix` and the code apps still give `mix` and `max`.
            ("Mix ", "Mĩ "),
            ("Max ", "Mã "),
        ],
    );
    check(code_app(), &[("Max ", "Mã "), ("if ", "if ")]);
}
