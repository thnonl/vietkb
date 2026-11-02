//! The line history behind Backspace reopen (EVKey's letter array, bounded):
//! owner cases across separators, the cases that must forget the line, and the
//! cap. The screen is simulated the way a backend applies the actions.

use vkb_core::{
    apply_action, simulate, Engine, EngineOptions, InputMethod, KeyInput, Modifiers,
    VietnameseEngine, MAX_WORD, VK_ESCAPE, VK_TAB,
};

struct Session {
    engine: Engine,
    out: String,
}

impl Session {
    fn new(options: EngineOptions) -> Self {
        Self { engine: Engine::new(options), out: String::new() }
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

    fn backspaces(&mut self, n: usize) -> &str {
        for _ in 0..n {
            self.backspace();
        }
        &self.out
    }
}

fn vni() -> EngineOptions {
    EngineOptions { method: InputMethod::Vni, ..EngineOptions::default() }
}

fn telex() -> EngineOptions {
    EngineOptions::default()
}

// ---------------------------------------------------------------------------
// Owner cases: a word several words back comes back when Backspace walks into it.
// ---------------------------------------------------------------------------

#[test]
fn vieet_nam_backspace_four_then_j_gives_viet() {
    let mut s = Session::new(telex());
    s.text("Vieet nam");
    assert_eq!(s.backspaces(4), "Viêt");
    assert_eq!(s.text("j"), "Việt");
}

#[test]
fn vieet_nam_ha_backspace_seven_then_j_gives_viet() {
    let mut s = Session::new(telex());
    s.text("Vieet nam ha");
    assert_eq!(s.backspaces(7), "Viêt");
    assert_eq!(s.text("j"), "Việt");
}

#[test]
fn comma_separator_is_walked_back_through() {
    let mut s = Session::new(telex());
    s.text("Tha, ho");
    assert_eq!(s.backspaces(4), "Tha");
    assert_eq!(s.text("s"), "Thá");
}

#[test]
fn double_space_is_two_marks() {
    let mut s = Session::new(telex());
    s.text("Tha  x");
    assert_eq!(s.backspaces(3), "Tha");
    assert_eq!(s.text("s"), "Thá");
}

#[test]
fn vni_tone_digit_walks_back_across_a_space() {
    let mut s = Session::new(vni());
    s.text("Tha co");
    assert_eq!(s.backspaces(3), "Tha");
    assert_eq!(s.text("1"), "Thá");
}

#[test]
fn vni_modifier_walks_back_across_two_words() {
    let mut s = Session::new(vni());
    s.text("Tha ca ho");
    assert_eq!(s.backspaces(6), "Tha");
    assert_eq!(s.text("6"), "Thâ");
}

#[test]
fn a_tab_forgets_the_line() {
    // Owner ruling 2026-11-02: Tab may move focus to another field, so it never
    // reopens. Deleting the Tab and the word after it reaches nothing back.
    let mut s = Session::new(telex());
    s.text("Tha");
    s.key(KeyInput::key(VK_TAB, Some('\t')));
    s.text("co");
    assert_eq!(s.backspaces(3), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn a_tab_character_forgets_the_line_too() {
    // The same rule when the Tab arrives as a character, not as the Tab key.
    let mut s = Session::new(telex());
    s.text("Tha");
    s.text("\t");
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn a_toned_word_walked_back_into_matches_fresh_typing() {
    // `tieengs Vieetj ` then BS back into `Việt`: the next key must give what
    // typing `Vieetjs` fresh gives, the tone already on the word included.
    let mut s = Session::new(telex());
    assert_eq!(s.text("tieengs Vieetj "), "tiếng Việt ");
    assert_eq!(s.backspace(), "tiếng Việt");
    let fresh = simulate(telex(), "Vieetjs");
    assert_eq!(s.text("s"), format!("tiếng {fresh}"));
}

#[test]
fn a_separator_not_yet_deleted_keeps_the_next_word_fresh() {
    // Three presses delete `nam`; the space after `Viêt` is still on screen, so
    // the next key starts a new word and `s` is literal.
    let mut s = Session::new(telex());
    s.text("Vieet nam");
    assert_eq!(s.backspaces(3), "Viêt ");
    assert_eq!(s.text("s"), "Viêt s");
}

// ---------------------------------------------------------------------------
// Forgetting: anything that may change the text the history describes.
// ---------------------------------------------------------------------------

#[test]
fn an_arrow_key_forgets_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    s.key(KeyInput::key(0x25, None));
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn enter_forgets_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha xy ");
    s.key(KeyInput::enter());
    assert_eq!(s.backspaces(2), "Tha xy");
    assert_eq!(s.text("s"), "Tha xys");
}

#[test]
fn a_ctrl_chord_forgets_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    let ctrl_v = KeyInput {
        modifiers: Modifiers { ctrl: true, ..Modifiers::NONE },
        ..KeyInput::key(0x56, None)
    };
    s.key(ctrl_v);
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn reset_between_deleting_and_the_tone_key_forgets_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    // Backspace reopens `Tha` as the current word; then the caret moves.
    assert_eq!(s.backspace(), "Tha");
    s.engine.reset();
    // The engine no longer knows the text left of the caret: `s` is a new word.
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn escape_forgets_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    s.key(KeyInput::key(VK_ESCAPE, None));
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn a_bare_modifier_keeps_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    s.key(KeyInput::key(0x10, None));
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thá");
}

#[test]
fn a_symbol_the_history_cannot_describe_forgets_the_line() {
    // `€` is on screen but is neither a separator nor a control character the
    // history records: the line is forgotten, so the Backspace over the space
    // cannot reopen `Tha`.
    let mut s = Session::new(telex());
    s.text("Tha ");
    s.text("€");
    assert_eq!(s.backspace(), "Tha ");
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn a_word_cap_commit_forgets_the_line() {
    // The capped word has no separator after it, so the mark below it must not
    // reopen `Tha` once the capped word is gone.
    let mut s = Session::new(telex());
    s.text("Tha ");
    s.text(&"b".repeat(MAX_WORD + 1));
    s.backspaces(MAX_WORD + 1);
    assert_eq!(s.out, "Tha ");
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

// ---------------------------------------------------------------------------
// The cap: a bounded history never panics and never reopens beyond its reach.
// ---------------------------------------------------------------------------

#[test]
fn more_words_than_the_cap_then_delete_back_is_bounded() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    for _ in 0..40 {
        s.text("ba ");
    }
    // Delete everything: no panic, and the engine ends with an empty word.
    let chars = s.out.chars().count();
    s.backspaces(chars + 10);
    assert_eq!(s.out, "");
    assert!(s.engine.buffer().is_empty());
    // `Tha` is beyond the cap: a tone key does not reopen it.
    assert_eq!(s.text("s"), "s");
}

#[test]
fn the_newest_words_within_the_cap_still_reopen() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    for _ in 0..40 {
        s.text("ba ");
    }
    // The last word is `ba `: four presses delete its space, both letters and the
    // space before it, which brings the word before back as the current word.
    let before = s.out.clone();
    let kept = &before[..before.len() - 6];
    assert_eq!(s.backspaces(4), format!("{kept}ba"));
    assert_eq!(s.text("s"), format!("{kept}bá"));
}

#[test]
fn a_long_line_with_many_separators_never_panics() {
    let mut s = Session::new(telex());
    let line = "tieengs Vieetj nam, ho. cos ddi ";
    for _ in 0..30 {
        s.text(line);
    }
    let chars = s.out.chars().count();
    s.backspaces(chars + 5);
    assert_eq!(s.out, "");
    assert!(s.engine.buffer().is_empty());
}

// ---------------------------------------------------------------------------
// Review fixes: the bare modifier press, options changes, and pinned tones.
// ---------------------------------------------------------------------------

fn modifier_down(vk: u16, set: fn(&mut Modifiers)) -> KeyInput {
    let mut modifiers = Modifiers::NONE;
    set(&mut modifiers);
    KeyInput { modifiers, ..KeyInput::key(vk, None) }
}

#[test]
fn a_bare_ctrl_press_keeps_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    assert_eq!(s.key(modifier_down(0x11, |m| m.ctrl = true)), "Tha ");
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thá");
}

#[test]
fn a_bare_alt_press_keeps_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    assert_eq!(s.key(modifier_down(0x12, |m| m.alt = true)), "Tha ");
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thá");
}

#[test]
fn a_bare_win_press_keeps_the_line() {
    let mut s = Session::new(telex());
    s.text("Tha ");
    assert_eq!(s.key(modifier_down(0x5B, |m| m.win = true)), "Tha ");
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thá");
}

#[test]
fn a_ctrl_chord_after_a_word_still_forgets_the_line() {
    // Ctrl+V with the V key itself (not the bare Ctrl press) is a chord: forget.
    let mut s = Session::new(telex());
    s.text("Tha ");
    s.key(modifier_down(0x56, |m| m.ctrl = true));
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn any_options_change_forgets_the_line() {
    // The history was built under the old options: a changed option (here the
    // spell check) is a boundary, so the Backspace over the space reopens nothing.
    let mut s = Session::new(telex());
    s.text("Tha ");
    let mut changed = telex();
    changed.spell_check = !changed.spell_check;
    vkb_core::VietnameseEngine::set_options(&mut s.engine, changed);
    assert_eq!(s.backspace(), "Tha");
    assert_eq!(s.text("s"), "Thas");
}

#[test]
fn a_pinned_tone_is_restored_with_its_word() {
    // `toán` loses its last letter to `toá`, whose tone stays pinned to `a`. The
    // word that comes back is that pinned word: the next tone key continues from
    // it exactly as fresh typing of the kept keystrokes does.
    let mut s = Session::new(telex());
    assert_eq!(s.text("toans"), "toán");
    assert_eq!(s.backspace(), "toá");
    assert_eq!(s.text(" "), "toá ");
    assert_eq!(s.backspace(), "toá");
    assert_eq!(s.text("f"), simulate(telex(), "toasf"));
}
