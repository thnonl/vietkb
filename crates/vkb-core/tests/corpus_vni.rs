//! VNI corpus.
//!
//! `6` circumflex, `7` horn, `8` breve, `9` đ, `0` strip, `1..5` tones. The
//! reference implementation routed `8` through the Telex `w` rule (so
//! `thuong82` produced `thường`); VNI documents `8` as the breve, so the
//! faithful spelling is `thu7o7ng2`. See `70-impl-notes-engine.md`.

use vkb_core::{simulate, EngineOptions, InputMethod};

fn vni() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    }
}

fn check(options: EngineOptions, cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_eq!(simulate(options, input), *expected, "input = {input:?}");
    }
}

#[test]
fn basic_words() {
    check(
        vni(),
        &[
            ("Tie6ng1 Vie6t5", "Tiếng Việt"),
            ("d9u7o7ng2", "đường"),
            ("hoa2", "hòa"),
            ("thu7o7ng2", "thường"),
            ("Thu7o7ng2", "Thường"),
            ("TU7O7NG2", "TƯỜNG"),
            ("VIE6T5", "VIỆT"),
        ],
    );
}

#[test]
fn modifier_keys() {
    check(
        vni(),
        &[
            ("a6", "â"),
            ("e6", "ê"),
            ("o6", "ô"),
            ("a8", "ă"),
            ("o7", "ơ"),
            ("u7", "ư"),
            ("d9", "đ"),
        ],
    );
}

#[test]
fn tone_keys() {
    check(
        vni(),
        &[
            ("a1", "á"),
            ("a2", "à"),
            ("a3", "ả"),
            ("a4", "ã"),
            ("a5", "ạ"),
            ("u7o7ng1", "ướng"),
        ],
    );
}

#[test]
fn zero_strips_the_previous_tone() {
    check(
        vni(),
        &[
            ("nghie6ng0", "nghieng"),
            ("di90", "di"),
            ("ta8t10", "tat"),
        ],
    );
}

#[test]
fn repeated_tone_key_becomes_literal() {
    check(vni(), &[("a11", "a1")]);
}

#[test]
fn repeated_tone_changes_keep_the_word_stable() {
    check(vni(), &[("thai1212", "thài")]);
}

#[test]
fn tone_positions_match_the_telex_rules() {
    check(
        vni(),
        &[
            ("khuyu3", "khuỷu"),
            ("nguyt1", "nguýt"),
            ("uynh2", "uỳnh"),
            ("uych1", "uých"),
            ("uych5", "uỵch"),
            ("uynh1", "uýnh"),
            ("hoan2", "hoàn"),
            ("toan1", "toán"),
        ],
    );
}

#[test]
fn numeric_words_are_not_transformed() {
    check(
        vni(),
        &[
            ("123 1thuong82 2Tie6ng1", "123 1thuong82 2Tie6ng1"),
            ("2026", "2026"),
        ],
    );
}

#[test]
fn digit_suffix_restores_e_mail_shaped_words() {
    check(vni(), &[("pndquyen203@gmail.com", "pndquyen203@gmail.com")]);
}

#[test]
fn vni_letters_are_not_telex_triggers() {
    // `w` and `z` are plain letters for VNI; `dd` is not `đ`.
    check(vni(), &[("www", "www"), ("dd", "dd"), ("z", "z")]);
}

#[test]
fn full_paragraph_of_words() {
    let cases = [
        ("Co6ng", "Công"),
        ("nghe65", "nghệ"),
        ("tri1", "trí"),
        ("tue65", "tuệ"),
        ("nha6n", "nhân"),
        ("tao5", "tạo"),
        ("d9ang", "đang"),
        ("thay", "thay"),
        ("d9o6i3", "đổi"),
        ("sa6u", "sâu"),
        ("sa8c1", "sắc"),
        ("cach1", "cách"),
        ("chu1ng", "chúng"),
        ("ta", "ta"),
        ("so6ng1", "sống"),
        ("va2", "và"),
        ("lam2", "làm"),
        ("vie65c", "việc"),
        ("mo6i4", "mỗi"),
        ("ngay2", "ngày"),
    ];
    for (input, expected) in cases {
        assert_eq!(simulate(vni(), input), expected, "input = {input:?}");
    }
}
