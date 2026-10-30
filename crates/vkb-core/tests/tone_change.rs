//! Tone change and partial deletion (`docs/evkey-clone-work/57-tone-change-cases.md`,
//! defects `D-TN-01`, `D-TN-06`, `D-TN-07`, `D-TN-08` in `60-defects.md`).
//!
//! Family A: the word is already on screen and the user presses a tone key again.
//! Family B: the user deletes one composed character and types again; the
//! surviving text must keep composing.

use vkb_core::{
    apply_action, simulate, Charset, Engine, EngineOptions, InputMethod, KeyInput,
    VietnameseEngine,
};

fn telex() -> EngineOptions {
    EngineOptions::default()
}

fn telex_nfd() -> EngineOptions {
    EngineOptions {
        charset: Charset::UnicodeNfd,
        ..EngineOptions::default()
    }
}

fn vni() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    }
}

/// One engine plus the text a backend would have typed, so a test can assert
/// what the user sees after every single key. `<BS>` in `text`/`key` is a real
/// Backspace (`KeyInput::backspace()`, vk `0x08`).
struct Session {
    engine: Engine,
    out: String,
}

impl Session {
    fn new(options: EngineOptions) -> Self {
        Self {
            engine: Engine::new(options),
            out: String::new(),
        }
    }

    fn key(&mut self, key: KeyInput) -> &str {
        let action = self.engine.process_key(key);
        apply_action(&mut self.out, action, key);
        &self.out
    }

    fn text(&mut self, input: &str) -> &str {
        for ch in input.chars() {
            self.key(KeyInput::char(ch));
        }
        &self.out
    }

    fn backspace(&mut self) -> &str {
        self.key(KeyInput::backspace())
    }
}

/// `text` rendered the way `unicode_nfd` emits it (the QA matrix compares its
/// NFD rows after NFC, so an NFD assertion is against the NFC string's NFD form).
fn nfd(text: &str) -> String {
    String::from_utf8(Charset::UnicodeNfd.encode(text)).expect("NFD output is UTF-8")
}

fn typed(options: EngineOptions, keys: &str) -> String {
    let mut session = Session::new(options);
    let mut command = String::new();
    for ch in keys.chars() {
        if ch == '\u{8}' {
            if !command.is_empty() {
                session.text(&command);
                command.clear();
            }
            session.backspace();
        } else {
            command.push(ch);
        }
    }
    if !command.is_empty() {
        session.text(&command);
    }
    session.out
}

// ---------------------------------------------------------------------------
// D-TN-01 — the tone key is already in force and a letter sits between the two
// presses. EVKey: the tone is unchanged, the marks stay.
// ---------------------------------------------------------------------------

#[test]
fn d_tn_01_same_tone_key_with_a_coda_in_between_keeps_the_tone() {
    assert_eq!(typed(telex(), "baajtj"), "bật");
    assert_eq!(typed(telex_nfd(), "baajtj"), nfd("bật"));
    assert_eq!(typed(vni(), "ba65t5"), "bật");
    // Reopen after a separator: same rule (B-54).
    assert_eq!(typed(telex(), "baajt \u{8}j"), "bật");
}

#[test]
fn d_tn_01_adjacent_repeat_is_still_the_documented_escape() {
    // The rule narrowed, not removed: `bass` / `aass` are pinned by E-TE-002/012.
    assert_eq!(simulate(telex(), "bass"), "bas");
    assert_eq!(simulate(telex(), "aass"), "aas");
    assert_eq!(simulate(vni(), "a11"), "a1");
}

#[test]
fn d_tn_01_same_tone_with_a_coda_mid_word_keeps_the_marks() {
    // `hoangfj`/`hoangff` stay separated: a different tone replaces, the same
    // adjacent tone escapes.
    assert_eq!(simulate(telex(), "hoangfj"), "hoạng");
    assert_eq!(simulate(telex(), "hoangff"), "hoangf");
}

// ---------------------------------------------------------------------------
// D-TN-06 — after a Backspace whose display cannot be re-composed from the
// surviving keys the word freezes as literal characters. The next tone key must
// instead re-run A4 on the surviving word.
// ---------------------------------------------------------------------------

#[test]
fn d_tn_06_tone_key_after_backspace_replaces_the_tone() {
    assert_eq!(typed(telex(), "toans\u{8}f"), "tòa");
    assert_eq!(typed(telex(), "toans\u{8}s"), "tóa");
    assert_eq!(typed(telex(), "toans\u{8}r"), "tỏa");
    assert_eq!(typed(telex(), "hoais\u{8}f"), "hòa");
    assert_eq!(typed(telex(), "hoais\u{8}s"), "hóa");
}

#[test]
fn d_tn_06_vni_tone_digit_after_backspace() {
    assert_eq!(typed(vni(), "toa1n\u{8}2"), "tòa");
    assert_eq!(typed(vni(), "toa1n\u{8}1"), "tóa");
}

#[test]
fn d_tn_06_backspace_keeps_the_display_unchanged() {
    // A7 step 4: no `Replace` on Backspace, the surviving characters stay as
    // displayed; only the *next* key re-places the tone.
    assert_eq!(typed(telex(), "toans\u{8}"), "toá");
    assert_eq!(typed(telex(), "hoais\u{8}"), "hoá");
}

// ---------------------------------------------------------------------------
// D-TN-07 — a plain letter is not a trigger: it appends and the surviving
// marks stay, even when the syllable stops being valid (A10 handles commit).
// ---------------------------------------------------------------------------

#[test]
fn d_tn_07_plain_letter_after_backspace_appends_with_marks_intact() {
    assert_eq!(typed(telex(), "baajt\u{8}b"), "bậb");
    assert_eq!(typed(telex(), "tieengs\u{8}n"), "tiếnn");
    assert_eq!(typed(telex(), "Nguyeexn\u{8}nn"), "Nguyễnn");
}

// ---------------------------------------------------------------------------
// D-TN-08 — the doubled-letter escape needs two presses of the same LETTER key;
// a letter typed after the tone key appends.
// ---------------------------------------------------------------------------

#[test]
fn d_tn_08_letter_after_a_tone_key_appends_instead_of_escaping_the_mark() {
    assert_eq!(typed(telex(), "Nguyeexn\u{8}e"), "Nguyễe");
    assert_eq!(typed(telex(), "kieems\u{8}e"), "kiếe");
    assert_eq!(typed(telex(), "Nguyeexn\u{8}ee"), "Nguyễee");
}

#[test]
fn d_tn_08_the_real_doubled_letter_escape_still_fires() {
    assert_eq!(simulate(telex(), "aaa"), "aa");
    assert_eq!(simulate(telex(), "aaw"), "ă");
}

// ---------------------------------------------------------------------------
// Documented escapes that must not flip while the rules above narrow.
// ---------------------------------------------------------------------------

#[test]
fn documented_adjacent_escapes_after_backspace_are_unchanged() {
    assert_eq!(typed(telex(), "bans\u{8}s"), "bas");
    assert_eq!(typed(telex(), "kieems\u{8}s"), "kiees");
    assert_eq!(typed(telex(), "tieengs\u{8}s"), "tieens");
    assert_eq!(typed(telex(), "thuys\u{8}s"), "thus");
    assert_eq!(typed(telex(), "hoangf\u{8}f"), "hoanf");
    assert_eq!(typed(vni(), "ba65t\u{8}5"), "ba65");
}

// ---------------------------------------------------------------------------
// D-TN-06, the owner's canonical repro (2026-10-30):
//
//   "Viết thiếu dấu rồi đã cách ra, rồi backspace lại dấu cách đó mới thêm
//    dấu. Ví dụ như chữ `viêt` ư+ space + BS + 1, expected là `viết`."
//
// A word with no tone is committed by the separator, the Backspace removes that
// separator and re-opens the word (§A7 step 1, spec §D8), and the tone key then
// lands on the re-opened word.
// ---------------------------------------------------------------------------

/// `<SP>` = a real space key, `<BS>` = `KeyInput::backspace()`, `<CR>` = Enter.
fn flow(options: EngineOptions, keys: &str) -> String {
    let mut session = Session::new(options);
    let mut letters = String::new();
    let flush = |session: &mut Session, letters: &mut String| {
        if !letters.is_empty() {
            session.text(letters);
            letters.clear();
        }
    };
    for ch in keys.chars() {
        match ch {
            '\u{8}' => {
                flush(&mut session, &mut letters);
                session.backspace();
            }
            '\r' => {
                flush(&mut session, &mut letters);
                session.key(KeyInput::enter());
            }
            '\t' => {
                flush(&mut session, &mut letters);
                session.key(KeyInput::key(vkb_core::VK_TAB, Some('\t')));
            }
            _ => letters.push(ch),
        }
    }
    flush(&mut session, &mut letters);
    session.out
}

#[test]
fn d_tn_06_owner_repro_vni_tone_digit_after_deleted_space() {
    // The owner's exact case: `viêt` + space + BS + `1`.
    assert_eq!(flow(vni(), "vie6t \u{8}1"), "viết");
    assert_eq!(flow(vni(), "vie6t \u{8}5"), "việt");
    // 2/3/4 cannot stand before the coda `t` (§A9, D-TN-04): the digit is
    // literal text and the re-opened word is untouched.
    assert_eq!(flow(vni(), "vie6t \u{8}2"), "viêt2");
    assert_eq!(flow(vni(), "vie6t \u{8}3"), "viêt3");
    assert_eq!(flow(vni(), "vie6t \u{8}4"), "viêt4");
}

#[test]
fn d_tn_06_owner_repro_telex_tone_key_after_deleted_space() {
    assert_eq!(flow(telex(), "vieet \u{8}s"), "viết");
    assert_eq!(flow(telex(), "vieet \u{8}j"), "việt");
    // f/r/x before the coda `t` are illegal (§A9, D-TN-04): literal trigger.
    assert_eq!(flow(telex(), "vieet \u{8}f"), "viêtf");
    assert_eq!(flow(telex(), "vieet \u{8}r"), "viêtr");
    assert_eq!(flow(telex(), "vieet \u{8}x"), "viêtx");
}

#[test]
fn d_tn_06_owner_repro_every_tone_on_a_real_word() {
    // `toa` + space + BS + each tone key: all five are words/readings (§A4 T6 puts
    // the tone on `o`, the build's `hòa` style).
    assert_eq!(flow(telex(), "toa \u{8}s"), "tóa");
    assert_eq!(flow(telex(), "toa \u{8}f"), "tòa");
    assert_eq!(flow(telex(), "toa \u{8}r"), "tỏa");
    assert_eq!(flow(telex(), "toa \u{8}x"), "tõa");
    assert_eq!(flow(telex(), "toa \u{8}j"), "tọa");
}

#[test]
fn d_tn_06_owner_repro_reopen_a_two_unit_tone_target() {
    // Two letters in the word after the tone target: the tone still lands on the
    // reopened word, not on a raw tail.
    assert_eq!(flow(telex(), "hoang \u{8}f"), "hoàng");
    assert_eq!(flow(telex(), "hoang \u{8}s"), "hoáng");
    assert_eq!(flow(telex(), "dduowng \u{8}f"), "đường");
    // A horned, toned word: reopen keeps the `ươ` and the tone changes.
    assert_eq!(flow(telex(), "dduowcj \u{8}s"), "đước");
    assert_eq!(flow(telex(), "dduowcj \u{8}f"), "đượcf"); // D-TN-04: huyền is illegal before c
    // VNI twin of the same shape (`9` = đ, `7` = ơ/ư).
    assert_eq!(flow(vni(), "d9uo7ng \u{8}2"), "đường");
}

#[test]
fn d_tn_06_owner_repro_works_after_every_separator() {
    // §D8: the reopen memory covers space, Tab and ASCII punctuation (class 1).
    assert_eq!(flow(telex(), "vieet.\u{8}s"), "viết");
    assert_eq!(flow(telex(), "vieet,\u{8}s"), "viết");
    assert_eq!(flow(telex(), "vieet\t\u{8}s"), "viết");
    assert_eq!(flow(vni(), "vie6t.\u{8}1"), "viết");
    // Two Backspaces: the reopened word loses its last letter, then retones.
    assert_eq!(flow(telex(), "vieet \u{8}\u{8}s"), "viế");
}

#[test]
fn d_tn_06_owner_repro_after_enter_does_not_reopen() {
    // Enter is class 3 (spec §B3 / §D8): it ends the line, so the Backspace that
    // removes it does not bring the word back — the tone key starts a new word
    // with no vowel and is literal there. (`apply_action` does not append the CR
    // itself; the app owns that character.)
    assert_eq!(flow(telex(), "vieet\r\u{8}s"), "viêts");
    assert_eq!(flow(vni(), "vie6t\r\u{8}1"), "viêt1");
}

#[test]
fn d_tn_06_owner_repro_reopens_only_the_immediately_preceding_word() {
    // §A7 step 1 / §D8: the reopen memory is one word, not a history — only
    // `hoc` comes back, and the earlier `di` keeps its text.
    assert_eq!(flow(telex(), "toi di hoc \u{8}s"), "toi di hóc");
    // The same VNI flow reopens only the last word too.
    assert_eq!(flow(vni(), "toi di hoc \u{8}1"), "toi di hóc");
}

// ---------------------------------------------------------------------------
// D-TN-02 — Telex `z` removes the tone (§A1: `asz` → `a`), the mark stays (§A5).
// ---------------------------------------------------------------------------

#[test]
fn d_tn_02_z_removes_the_tone() {
    assert_eq!(typed(telex(), "basz"), "ba");
    assert_eq!(typed(telex(), "toansz"), "toan");
    assert_eq!(typed(telex(), "asz"), "a");
    assert_eq!(typed(telex(), "Basz"), "Ba");
}

#[test]
fn d_tn_02_z_keeps_the_circumflex() {
    assert_eq!(typed(telex(), "baajz"), "bâ");
    assert_eq!(typed(telex(), "baajtz"), "bât");
    assert_eq!(typed(telex_nfd(), "baajtz"), nfd("bât"));
    // Reopen route and partial deletion: `bật` BS `z` → `bâ`.
    assert_eq!(typed(telex(), "baajt\u{8}z"), "bâ");
    assert_eq!(flow(telex(), "baajt \u{8}z"), "bât");
}

#[test]
fn d_tn_02_z_without_a_tone_is_literal() {
    assert_eq!(typed(telex(), "az"), "az");
    assert_eq!(typed(telex(), "zalo"), "zalo");
    assert_eq!(typed(telex(), "baz"), "baz");
}

#[test]
fn d_tn_02_a_tone_can_be_typed_again_after_z() {
    assert_eq!(typed(telex(), "baszf"), "bà");
}

// ---------------------------------------------------------------------------
// D-TN-03 — VNI `0` removes the tone only (§A2, §A5).
// ---------------------------------------------------------------------------

#[test]
fn d_tn_03_zero_keeps_the_marks() {
    assert_eq!(typed(vni(), "a60"), "â");
    assert_eq!(typed(vni(), "ba650"), "bâ");
    assert_eq!(typed(vni(), "ba65t0"), "bât");
    assert_eq!(typed(vni(), "ba65t\u{8}0"), "bâ");
    assert_eq!(typed(vni(), "ba10"), "ba");
    assert_eq!(typed(vni(), "d90"), "đ");
    assert_eq!(typed(vni(), "tu7o7ng0"), "tương");
}

#[test]
fn d_tn_03_zero_with_nothing_to_remove_is_a_literal_digit() {
    assert_eq!(typed(vni(), "a0"), "a0");
    assert_eq!(typed(vni(), "ba0"), "ba0");
    assert_eq!(typed(vni(), "ban0"), "ban0");
    assert_eq!(typed(vni(), "ba00"), "ba00");
    // A tone or a mark is a target: unchanged.
    assert_eq!(typed(vni(), "a60"), "â");
    assert_eq!(typed(vni(), "ba10"), "ba");
    // A repeated 0 after a removal has nothing left to remove: literal.
    assert_eq!(typed(vni(), "ba100"), "ba0");
}

// ---------------------------------------------------------------------------
// D-TN-04 — a tone change obeys the A9 coda check: `c ch p t` take only sắc and
// nặng, an impossible tone is literal text and the word stays as it is.
// ---------------------------------------------------------------------------

#[test]
fn d_tn_04_illegal_tone_change_before_a_stop_coda_is_literal() {
    assert_eq!(typed(telex(), "baajtf"), "bậtf");
    assert_eq!(typed(telex(), "baajtr"), "bậtr");
    assert_eq!(typed(telex(), "baajtx"), "bậtx");
    assert_eq!(typed(telex(), "Baajtf"), "Bậtf");
    assert_eq!(typed(telex(), "BAATJf"), "BẬTf");
    assert_eq!(typed(telex(), "dduowcjf"), "đượcf");
    assert_eq!(typed(vni(), "ba65t2"), "bật2");
    assert_eq!(typed(vni(), "ba65t3"), "bật3");
    assert_eq!(typed(vni(), "ba65t4"), "bật4");
}

#[test]
fn d_tn_04_the_fresh_word_sibling_is_literal_too() {
    assert_eq!(typed(telex(), "tacf"), "tacf");
    assert_eq!(typed(telex(), "tachr"), "tachr");
    assert_eq!(typed(telex(), "tapx"), "tapx");
}

#[test]
fn d_tn_04_legal_changes_still_apply() {
    assert_eq!(typed(telex(), "baajts"), "bất");
    assert_eq!(typed(telex(), "dduowcjs"), "đước");
    assert_eq!(typed(telex(), "kieemsf"), "kiềm");
    assert_eq!(typed(telex(), "hoangfj"), "hoạng");
    assert_eq!(typed(telex(), "basf"), "bà");
    assert_eq!(typed(vni(), "ba65t1"), "bất");
    assert_eq!(typed(vni(), "ba12"), "bà");
}

#[test]
fn d_tn_04_spell_check_off_applies_the_tone() {
    let options = EngineOptions {
        spell_check: false,
        ..telex()
    };
    assert_eq!(typed(options, "baajtf"), "bầt");
}

// ---------------------------------------------------------------------------
// D-TN-05 — a trigger with no target is a literal digit; the word stays composed.
// ---------------------------------------------------------------------------

#[test]
fn d_tn_05_digit_with_no_target_is_literal() {
    assert_eq!(typed(vni(), "ba65t6"), "bật6");
    assert_eq!(typed(vni(), "ba6t6"), "bât6");
}

#[test]
fn d_tn_05_adjacent_digit_escape_is_unchanged() {
    assert_eq!(typed(vni(), "a66"), "a6");
    assert_eq!(typed(vni(), "ba11"), "ba1");
    assert_eq!(typed(vni(), "d99"), "d9");
}
