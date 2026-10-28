//! Tone placement: modern vs old style, `qu`/`gi`, `ươ`, `oai`, `uyê`,
//! two-vowel nuclei, and the tone moving when the syllable grows.

use vkb_core::{simulate, EngineOptions};

fn modern() -> EngineOptions {
    EngineOptions::default()
}

fn old() -> EngineOptions {
    EngineOptions {
        modern_tone: false,
        ..EngineOptions::default()
    }
}

fn check(options: EngineOptions, cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_eq!(simulate(options, input), *expected, "input = {input:?}");
    }
}

#[test]
fn single_vowel_takes_the_tone() {
    check(
        modern(),
        &[("as", "á"), ("es", "é"), ("is", "í"), ("os", "ó"), ("us", "ú"), ("ys", "ý")],
    );
}

#[test]
fn modified_vowel_wins_over_the_open_pair_rule() {
    check(
        modern(),
        &[
            ("thuyeenf", "thuyền"),
            ("nguyeenx", "nguyễn"),
            ("quyeets", "quyết"),
            ("chuyeenf", "chuyền"),
            ("nguowif", "người"),
            ("tuowngr", "tưởng"),
            ("dduwowcj", "được"),
            ("muaf", "mùa"),
        ],
    );
}

#[test]
fn three_vowel_nuclei_take_the_middle_vowel() {
    check(
        modern(),
        &[
            ("khoais", "khoái"),
            ("xoays", "xoáy"),
            ("khuyur", "khuỷu"),
            ("ngoeos", "ngoéo"),
            ("quaays", "quấy"),
        ],
    );
}

#[test]
fn uy_with_a_coda_takes_the_y() {
    check(
        modern(),
        &[
            ("nguyts", "nguýt"),
            ("uynhf", "uỳnh"),
            ("uychs", "uých"),
            ("uychj", "uỵch"),
            ("uynhs", "uýnh"),
            ("huynhf", "huỳnh"),
        ],
    );
}

#[test]
fn oa_and_oe_follow_the_coda_rule() {
    check(
        modern(),
        &[
            ("hoaf", "hòa"),
            ("hoafn", "hoàn"),
            ("toans", "toán"),
            ("khoef", "khòe"),
            ("khoer", "khỏe"),
            ("hoef", "hòe"),
        ],
    );
}

#[test]
fn old_style_moves_the_open_pair_tone() {
    check(
        old(),
        &[
            ("hoaf", "hoà"),
            ("hoef", "hoè"),
            ("khoef", "khoè"),
            ("khoer", "khoẻ"),
            ("thuyf", "thuỳ"),
            ("hoafn", "hoàn"),
            ("toans", "toán"),
        ],
    );
}

#[test]
fn two_vowel_nuclei_take_the_first_vowel() {
    check(
        modern(),
        &[
            ("mais", "mái"),
            ("meos", "méo"),
            ("baif", "bài"),
            ("cuar", "của"),
            ("tuys", "túy"),
            ("thuys", "thúy"),
            ("muoois", "muối"),
        ],
    );
}

#[test]
fn qu_and_gi_are_onsets() {
    check(
        modern(),
        &[
            ("quas", "quá"),
            ("quys", "quý"),
            ("quees", "quế"),
            ("quyeenr", "quyển"),
            ("giaf", "già"),
            ("gif", "gì"),
            ("giuwx", "giữ"),
            ("giuwowngf", "giường"),
        ],
    );
}

#[test]
fn tone_moves_when_the_syllable_grows() {
    check(
        modern(),
        &[
            ("hoafn", "hoàn"),
            ("hoanf", "hoàn"),
            ("khoarn", "khoản"),
            ("toans", "toán"),
            ("toasn", "toán"),
        ],
    );
}

#[test]
fn tone_survives_further_letters() {
    check(
        modern(),
        &[
            ("thasi", "thái"),
            ("tieengs", "tiếng"),
            ("vieetj", "việt"),
            ("hoafng", "hoàng"),
        ],
    );
}

#[test]
fn candidates_never_lose_the_tone_when_a_vowel_is_added() {
    // `hoà` typed before the final vowel: the tone must land on `a`.
    check(modern(), &[("hoas", "hóa"), ("khoas", "khóa")]);
}
