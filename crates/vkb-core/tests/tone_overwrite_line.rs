//! "Gõ dấu đè" for words still in memory (owner, 2026-11-03): plain Left/Right/End move the
//! caret inside the remembered line, and a tone key typed right after one of its words re-tones
//! it in place. A word outside memory stays as it is and the key adds its character
//! (`tốt` + `x` = `tốtx`).

use vkb_core::{Engine, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine, VK_BACK};

const LEFT: u16 = 0x25;
const RIGHT: u16 = 0x27;
const END: u16 = 0x23;
const HOME: u16 = 0x24;

/// A tiny editor: the text, the caret, and the engine in front of it (as the platform drives it:
/// plain Left/Right/End go to the engine, other caret keys reset it).
struct Doc {
    engine: Engine,
    text: Vec<char>,
    caret: usize,
}

impl Doc {
    fn new(method: InputMethod) -> Self {
        Self { engine: Engine::new(EngineOptions { method, ..EngineOptions::default() }), text: Vec::new(), caret: 0 }
    }

    fn apply(&mut self, action: EngineAction, typed: Option<char>, back: bool) {
        match action {
            EngineAction::Noop => {
                if let Some(c) = typed {
                    self.text.insert(self.caret, c);
                    self.caret += 1;
                } else if back && self.caret > 0 {
                    self.caret -= 1;
                    self.text.remove(self.caret);
                }
            }
            EngineAction::Commit(s) => s.chars().for_each(|c| {
                self.text.insert(self.caret, c);
                self.caret += 1;
            }),
            EngineAction::Replace { delete_units, insert } | EngineAction::ReplaceThenPass { delete_units, insert } => {
                for _ in 0..delete_units {
                    self.caret -= 1;
                    self.text.remove(self.caret);
                }
                insert.chars().for_each(|c| {
                    self.text.insert(self.caret, c);
                    self.caret += 1;
                });
            }
            _ => {}
        }
    }

    /// Plain text, `<` = Left, `>` = Right, `$` = End, `^` = Home, `~` = Backspace.
    fn keys(&mut self, keys: &str) -> &mut Self {
        for c in keys.chars() {
            match c {
                '<' => {
                    let _ = self.engine.process_key(KeyInput::key(LEFT, None));
                    self.caret = self.caret.saturating_sub(1);
                }
                '>' => {
                    let _ = self.engine.process_key(KeyInput::key(RIGHT, None));
                    self.caret = (self.caret + 1).min(self.text.len());
                }
                '$' => {
                    let _ = self.engine.process_key(KeyInput::key(END, None));
                    self.caret = self.text.len();
                }
                '^' => {
                    // Home: the platform resets the engine.
                    self.engine.reset();
                    let _ = HOME;
                    self.caret = 0;
                }
                '~' => {
                    let action = self.engine.process_key(KeyInput::key(VK_BACK, None));
                    self.apply(action, None, true);
                }
                _ => {
                    let action = self.engine.process_key(KeyInput::char(c));
                    self.apply(action, Some(c), false);
                }
            }
        }
        self
    }

    fn text(&self) -> String {
        self.text.iter().collect()
    }
}

#[test]
fn left_to_the_end_of_a_word_then_a_tone_key_re_tones_it() {
    let mut d = Doc::new(InputMethod::Telex);
    d.keys("tooi nhas <f");
    assert_eq!(d.text(), "tôi nhà ");
    // Again, another tone: still the same word.
    d.keys("r");
    assert_eq!(d.text(), "tôi nhả ");
}

#[test]
fn several_words_back_and_two_fixes_in_one_line() {
    let mut d = Doc::new(InputMethod::Telex);
    d.keys("tooi awn cas kho ");
    // To the end of `cá`: back over " kho " (5).
    d.keys("<<<<<f");
    assert_eq!(d.text(), "tôi ăn cà kho ");
    // On to the end of `tôi`: back over " ăn " (4 more, the caret is right after `cà`... `cà` is 2).
    d.keys("<<<<<<s");
    assert_eq!(d.text(), "tối ăn cà kho ");
    // Back to the end with Right and keep typing: the line is still remembered.
    d.keys(">>>>>>>>>>>ngon ");
    assert_eq!(d.text(), "tối ăn cà kho ngon ");
    // Backspace back into `ngon` still reopens it (the history is intact).
    d.keys("~j ");
    assert_eq!(d.text(), "tối ăn cà kho ngọn ");
}

#[test]
fn vni_too() {
    let mut d = Doc::new(InputMethod::Vni);
    d.keys("to6i nha1 <2");
    assert_eq!(d.text(), "tôi nhà ");
    d.keys("0");
    assert_eq!(d.text(), "tôi nha ");
}

#[test]
fn a_word_outside_memory_stays_and_the_key_adds_its_character() {
    // Home is outside memory: the line is forgotten.
    let mut d = Doc::new(InputMethod::Telex);
    d.keys("toots ^$");
    let before = d.text();
    d.keys("<x");
    assert_eq!(before, "tốt ");
    assert_eq!(d.text(), "tốtx ", "a word outside memory keeps its text; the key is a letter");
}

#[test]
fn inside_a_word_the_key_is_a_letter() {
    let mut d = Doc::new(InputMethod::Telex);
    d.keys("tooi nhas <<f");
    assert_eq!(d.text(), "tôi nhfá ", "the caret is inside `nhá`: nothing is re-toned");
}

#[test]
fn typing_in_the_middle_forgets_the_line_and_end_then_knows_nothing() {
    let mut d = Doc::new(InputMethod::Telex);
    d.keys("tooi nhas <<<<<");
    // A tone key right after `tôi` re-tones it ...
    d.keys("x");
    assert_eq!(d.text(), "tỗi nhá ");
    // ... a letter typed there is inserted, and the line is forgotten.
    d.keys("k");
    assert_eq!(d.text(), "tỗik nhá ");
    // End, Left (before the last space), a tone key: nothing is remembered, it is a letter.
    d.keys("$<f");
    assert_eq!(d.text(), "tỗik nháf ", "after End nothing is remembered: the key is a letter");
}
