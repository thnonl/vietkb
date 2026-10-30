//! Backspace (one displayed character per press), reopen-after-boundary,
//! restore-word, boundaries, auto-capitalisation and the word cap.

use vkb_core::{
    apply_action, simulate, simulate_keys, Charset, Engine, EngineAction, EngineOptions,
    InputMethod, KeyInput, MacroTable, VietnameseEngine, MAX_WORD,
};

fn keys(input: &str) -> Vec<KeyInput> {
    input.chars().map(KeyInput::char).collect()
}

fn run(options: EngineOptions, all: &[KeyInput]) -> String {
    let mut engine = Engine::new(options);
    let mut out = String::new();
    for key in all {
        let action = engine.process_key(*key);
        apply_action(&mut out, action, *key);
    }
    out
}

/// One engine plus the text a backend would have typed, so a test can assert
/// what the user sees after every single key.
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

/// `text` rendered the way `unicode_nfd` emits it.
fn nfd(text: &str) -> String {
    String::from_utf8(Charset::UnicodeNfd.encode(text)).expect("NFD output is UTF-8")
}

fn nfd_options() -> EngineOptions {
    EngineOptions {
        charset: Charset::UnicodeNfd,
        ..EngineOptions::default()
    }
}

#[test]
fn backspace_deletes_one_displayed_character() {
    // The owner's example: `tiếng` is five characters, so three presses leave
    // `ti` — each press removes one letter, not one raw keystroke.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("tieengs"), "tiếng");
    assert_eq!(session.backspace(), "tiến");
    assert_eq!(session.backspace(), "tiế");
    assert_eq!(session.backspace(), "ti");
}

#[test]
fn backspace_deletes_a_marked_word_one_character_at_a_time() {
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("dduwowngf"), "đường");
    assert_eq!(session.backspace(), "đườn");
    assert_eq!(session.backspace(), "đườ");
    assert_eq!(session.backspace(), "đư");
}

#[test]
fn backspace_counts_a_capital_letter_as_one_character() {
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("Tieengs"), "Tiếng");
    assert_eq!(session.backspace(), "Tiến");
}

#[test]
fn backspace_removes_a_toned_letter_and_its_tone() {
    // The tone lives on the deleted letter: it goes with it.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("cas"), "cá");
    assert_eq!(session.backspace(), "c");
}

#[test]
fn nfd_backspace_deletes_one_character() {
    let mut session = Session::new(nfd_options());
    assert_eq!(session.text("tieengs"), nfd("tiếng"));
    assert_eq!(session.backspace(), nfd("tiến"));
    assert_eq!(session.engine.current_text(), nfd("tiến"));
}

#[test]
fn nfd_backspace_deletes_every_unit_of_a_multi_unit_character() {
    // `ế` is three code units in NFD (e + circumflex + acute): one press must
    // delete all three, so the action cannot be a pass-through backspace.
    let mut engine = Engine::new(nfd_options());
    for key in keys("tiees") {
        engine.process_key(key);
    }
    assert_eq!(engine.current_text(), nfd("tiế"));
    assert_eq!(
        engine.process_key(KeyInput::backspace()),
        EngineAction::Replace {
            delete_units: 3,
            insert: String::new(),
        }
    );
    assert_eq!(engine.current_text(), nfd("ti"));
}

#[test]
fn backspace_after_a_boundary_leaves_the_word_to_the_application() {
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("tieengs "), "tiếng ");
    // The space belongs to the application, not to the word buffer.
    assert_eq!(session.backspace(), "tiếng");
    // Then the committed word's own last letter, still untouched by the engine.
    assert_eq!(session.backspace(), "tiến");
}

#[test]
fn backspace_with_no_word_is_a_noop() {
    let mut engine = Engine::new(EngineOptions::default());
    assert!(engine.process_key(KeyInput::backspace()).is_noop());
    assert!(engine.buffer().is_empty());
}

#[test]
fn typing_after_a_deletion_composes_on_the_kept_history() {
    // The surviving keystrokes are a real history: typing the deleted key back
    // rebuilds the word instead of doubling a letter.
    let mut session = Session::new(EngineOptions::default());
    session.text("dduwowngf");
    assert_eq!(session.backspace(), "đườn");
    assert_eq!(session.text("g"), "đường");
}

#[test]
fn typing_after_a_deletion_composes_like_the_same_history_typed_at_once() {
    // After `tiếng` - `g` the buffer still holds `t i e e n s`, so a following
    // `t` is the same shape as typing `tieenst` in one go: the spell check
    // leaves both as raw keystrokes rather than inventing a syllable.
    let mut session = Session::new(EngineOptions::default());
    session.text("tieengs");
    assert_eq!(session.backspace(), "tiến");
    assert_eq!(session.text("t"), simulate(EngineOptions::default(), "tieenst"));
}

#[test]
fn backspace_keeps_the_original_case() {
    let mut all = keys("Thuongwf");
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "Thườn");
}

#[test]
fn backspace_over_a_toned_letter_removes_the_letter() {
    // `Hoas` shows `Hóa`; the press removes `a`, so the tone stays on `o`.
    let mut all = keys("Hoas");
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "Hó");
}

#[test]
fn backspace_over_vni_modifiers_deletes_a_character() {
    let vni = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    let mut session = Session::new(vni);
    assert_eq!(session.text("To6i1"), "Tối");
    assert_eq!(session.backspace(), "Tố");

    // `nghieng2` puts the tone on the `i` (A4), so the press removes the final
    // `g` and keeps it there.
    let mut session = Session::new(vni);
    assert_eq!(session.text("nghieng2"), "nghìeng");
    assert_eq!(session.backspace(), "nghìen");
}

#[test]
fn backspace_to_empty_leaves_nothing_behind() {
    let mut all = keys("as");
    assert_eq!(run(EngineOptions::default(), &all), "á");
    // `á` is one character: the press deletes the letter and its tone.
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "");
    // With no word of ours, backspace reaches the application.
    all.push(KeyInput::backspace());
    assert_eq!(run(EngineOptions::default(), &all), "");
}

// ---------------------------------------------------------------------------
// Reopen after a boundary: `Tha` + space + BS + `s` = `Thá`
// ---------------------------------------------------------------------------

#[test]
fn backspace_reopens_the_word_the_space_committed() {
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("Tha"), "Tha");
    assert_eq!(session.text(" "), "Tha ");
    // The app deletes the space; the engine puts the word back in the buffer.
    assert_eq!(session.backspace(), "Tha");
    assert_eq!(session.engine.buffer(), ['T', 'h', 'a']);
    assert_eq!(session.engine.current_text(), "Tha");
    // So the tone trigger composes on it instead of starting a new word.
    assert_eq!(session.text("s"), "Thá");
}

#[test]
fn vni_tone_digits_reopen_a_word_after_deleted_space() {
    let options = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    for (digit, expected) in [('1', "Thá"), ('2', "Thà"), ('3', "Thả"), ('4', "Thã"), ('5', "Thạ")] {
        let mut session = Session::new(options);
        assert_eq!(session.text("Tha"), "Tha");
        assert_eq!(session.text(" "), "Tha ");
        assert_eq!(session.backspace(), "Tha");
        assert_eq!(session.text(&digit.to_string()), expected, "digit {digit}");
    }
}

#[test]
fn vni_modifiers_reopen_a_word_after_deleted_space() {
    let options = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    for (word, digit, expected) in [("Tha", '6', "Thâ"), ("Thu", '7', "Thư"), ("Tha", '8', "Thă"), ("d", '9', "đ")] {
        let mut session = Session::new(options);
        assert_eq!(session.text(word), word);
        assert_eq!(session.text(" "), format!("{word} "));
        assert_eq!(session.backspace(), word);
        assert_eq!(session.text(&digit.to_string()), expected, "{word} + {digit}");
    }
}

#[test]
fn vni_modifier_digit_after_non_deleted_space_is_literal() {
    let options = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    let mut session = Session::new(options);
    assert_eq!(session.text("Tha "), "Tha ");
    assert_eq!(session.text("6"), "Tha 6");
}

#[test]
fn vni_tone_digit_works_without_deleted_boundary() {
    let options = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    let mut session = Session::new(options);
    assert_eq!(session.text("Tha"), "Tha");
    assert_eq!(session.text("1"), "Thá");
}

#[test]
fn vni_digit_after_non_deleted_space_is_literal() {
    let options = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    let mut session = Session::new(options);
    assert_eq!(session.text("Tha "), "Tha ");
    assert_eq!(session.text("1"), "Tha 1");
}

#[test]
fn a_second_backspace_after_reopening_deletes_a_character() {
    let mut session = Session::new(EngineOptions::default());
    session.text("Tha ");
    assert_eq!(session.backspace(), "Tha");
    assert_eq!(session.backspace(), "Th");
}

#[test]
fn a_keystroke_after_the_boundary_suppresses_reopening() {
    let mut session = Session::new(EngineOptions::default());
    session.text("Tha ");
    assert_eq!(session.text("x"), "Tha x");
    // `x` spent the memory: this press deletes `x`, it does not reopen `Tha`.
    assert_eq!(session.backspace(), "Tha ");
    assert_eq!(session.engine.buffer(), Vec::<char>::new());
}

#[test]
fn reopening_covers_separators_but_not_line_breaks() {
    // Enter ends the line, not just the word: no reopen, so `s` starts a word.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("Tha"), "Tha");
    assert_eq!(session.key(KeyInput::enter()), "Tha\r");
    assert_eq!(session.backspace(), "Tha");
    assert_eq!(session.text("s"), "Thas");

    // Escape is class 3 as well: the word is gone, so `s` is literal.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("Tha"), "Tha");
    assert_eq!(session.key(KeyInput::key(vkb_core::VK_ESCAPE, None)), "Tha");
    assert_eq!(session.text("s"), "Thas", "ESC hard-resets the word");

    // Tab is a separator (EVKey class 1): deleting it brings the word back.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("Tha"), "Tha");
    assert_eq!(session.key(KeyInput::key(vkb_core::VK_TAB, Some('\t'))), "Tha\t");
    assert_eq!(session.backspace(), "Tha");
    assert_eq!(session.text("s"), "Thá");

    // Sentence punctuation behaves the same way.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("Tha."), "Tha.");
    assert_eq!(session.backspace(), "Tha");
    assert_eq!(session.text("s"), "Thá");
}

#[test]
fn a_tone_trigger_on_a_reopened_toned_word_keeps_the_escape_rule() {
    // The word comes back with its tone; the second `s` is the engine's existing
    // doubled-trigger escape (same result as typing the keystrokes in one go),
    // so nothing is corrupted and nothing is duplicated.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("tieengs "), "tiếng ");
    assert_eq!(session.backspace(), "tiếng");
    assert_eq!(
        session.text("s"),
        simulate(EngineOptions::default(), "tieengss")
    );
}

#[test]
fn the_reopen_memory_is_one_word_not_a_history() {
    // The space after `ho` replaces the memory: reopening gives `ho`, not `Tha`.
    let mut session = Session::new(EngineOptions::default());
    session.text("Tha ho ");
    assert_eq!(session.backspace(), "Tha ho");
    assert_eq!(session.text("s"), "Tha hó");
}

#[test]
fn a_boundary_clears_the_reopen_memory() {
    // `Tha` + space + BS restores and re-commits; the second space commits it
    // again, so this press is a plain pass-through of that space.
    let mut session = Session::new(EngineOptions::default());
    session.text("Tha ");
    assert_eq!(session.backspace(), "Tha");
    assert_eq!(session.text(" "), "Tha ");
    assert_eq!(session.backspace(), "Tha");
}

#[test]
fn a_macro_commit_is_not_reopened() {
    // The screen shows the expansion, not the trigger: re-opening `vn` would
    // desync the buffer from the screen, so a macro commit leaves no memory.
    let mut engine = Engine::new(EngineOptions::default());
    engine.upsert_macro("vn", "Việt Nam");
    let mut out = String::new();
    for key in keys("vn ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "Việt Nam ");
    apply_action(&mut out, engine.process_key(KeyInput::backspace()), KeyInput::backspace());
    assert_eq!(out, "Việt Nam");
    assert!(engine.buffer().is_empty());
    apply_action(&mut out, engine.process_key(KeyInput::char('s')), KeyInput::char('s'));
    assert_eq!(out, "Việt Nams", "the trigger must not come back");
}

#[test]
fn backspace_after_restore_word_deletes_a_character() {
    // `restore_word` forces literal text from raw index 0; a press then deletes
    // one character of that literal word and keeps the rest literal.
    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    for key in keys("tieengs") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    apply_action(&mut out, engine.restore_word(), KeyInput::char(' '));
    assert_eq!(out, "tieengs");
    for key in [KeyInput::backspace(), KeyInput::char('s')] {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "tieengs", "typing stays literal after the restore");
}

#[test]
fn restore_word_puts_the_raw_keystrokes_back() {
    let mut engine = Engine::new(EngineOptions::default());
    let mut out = String::new();
    for key in keys("tieengs") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "tiếng");
    apply_action(&mut out, engine.restore_word(), KeyInput::char(' '));
    assert_eq!(out, "tieengs");
    // The word stays literal after the restore.
    for key in keys("s") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "tieengss");
}

#[test]
fn restore_word_on_an_untransformed_word_changes_nothing() {
    use vkb_core::EngineAction;
    let mut engine = Engine::new(EngineOptions::default());
    for key in keys("abc") {
        engine.process_key(key);
    }
    let action = engine.restore_word();
    assert!(matches!(
        action,
        EngineAction::Noop | EngineAction::Swallow
    ));
    assert_eq!(engine.current_text(), "abc");
}

#[test]
fn boundary_flushes_the_word() {
    let mut engine = Engine::new(EngineOptions::default());
    for key in keys("tieengs") {
        engine.process_key(key);
    }
    engine.boundary();
    assert!(engine.buffer().is_empty());
    // The next word starts fresh: a stray tone key is literal.
    let mut out = String::new();
    for key in keys("s") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "s");
}

#[test]
fn word_cap_commit_and_restart() {
    let input: String = "bn".repeat(MAX_WORD);
    assert_eq!(simulate(EngineOptions::default(), &input), input);
}

#[test]
fn word_cap_keeps_the_keystroke_that_starts_the_next_word() {
    // `a` cannot be used here: the D3 escape rule collapses repeated trigger letters.
    let telex = "b".repeat(MAX_WORD + 1);
    assert_eq!(simulate(EngineOptions::default(), &telex), telex);

    let vni_options = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    let vni_letters = "b".repeat(MAX_WORD + 1);
    assert_eq!(simulate(vni_options, &vni_letters), vni_letters);
    let vni_digit = format!("{}1", "b".repeat(MAX_WORD));
    assert_eq!(simulate(vni_options, &vni_digit), vni_digit);

    let jfwz_off = EngineOptions { jfwz_as_consonant: false, ..EngineOptions::default() };
    assert!(simulate(jfwz_off, &format!("{}w", "b".repeat(MAX_WORD))).ends_with("bư"));
    let nfd = EngineOptions { jfwz_as_consonant: false, charset: Charset::UnicodeNfd, ..EngineOptions::default() };
    // NFD renders `ư` as two scalars: `u` + U+031B.
    assert!(simulate(nfd, &format!("{}w", "b".repeat(MAX_WORD))).ends_with("bu\u{031B}"));

    let vni = "1".repeat(MAX_WORD + 1);
    let options = EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, &vni), vni);
}

#[test]
fn auto_capitalize_after_a_sentence_end() {
    let options = EngineOptions {
        auto_capitalize: true,
        ..EngineOptions::default()
    };
    assert_eq!(simulate(options, "tieengs. vieetj"), "tiếng. Việt");
    assert_eq!(simulate(options, "hoaf? hoaf"), "hòa? Hòa");
    assert_eq!(simulate(options, "hoaf hoaf"), "hòa hòa");
}

#[test]
fn switching_method_resets_the_word() {
    let mut engine = Engine::new(EngineOptions::default());
    for key in keys("tieengs") {
        engine.process_key(key);
    }
    engine.set_options(EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    });
    assert!(engine.buffer().is_empty());
}

#[test]
fn macros_expand_at_the_boundary_only() {
    let mut engine = Engine::new(EngineOptions::default());
    engine.set_macros(MacroTable::from_pairs([
        ("btw", "by the way"),
        ("vn", "Việt Nam"),
    ]));
    let mut out = String::new();
    for key in keys("btw ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "by the way ");

    let mut out = String::new();
    for key in keys("vn ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "Việt Nam ");

    // Capitalised trigger → capitalised expansion.
    let mut out = String::new();
    for key in keys("Btw ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "By the way ");

    // Not a boundary, not a macro hit.
    let mut out = String::new();
    for key in keys("btwx") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "btwx");
}

#[test]
fn macros_can_be_disabled() {
    let options = EngineOptions {
        macro_enabled: false,
        ..EngineOptions::default()
    };
    let mut engine = Engine::new(options);
    engine.set_macros(MacroTable::from_pairs([("btw", "by the way")]));
    let mut out = String::new();
    for key in keys("btw ") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "btw ");
}

#[test]
fn macro_expansion_types_the_boundary_itself() {
    let mut engine = Engine::new(EngineOptions::default());
    engine.upsert_macro("vn", "Việt Nam");
    let mut out = String::new();
    for key in keys("vn.") {
        apply_action(&mut out, engine.process_key(key), key);
    }
    assert_eq!(out, "Việt Nam.");
}

#[test]
fn simulate_keys_matches_simulate_on_plain_text() {
    let text = "tieengs Vieetj";
    assert_eq!(
        simulate_keys(EngineOptions::default(), &keys(text)),
        simulate(EngineOptions::default(), text)
    );
}

// ---------------------------------------------------------------------------
// D-BS-LIT-01: literal mode must not outlive the word's last vowel
// ---------------------------------------------------------------------------

fn vni_options() -> EngineOptions {
    EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    }
}

/// Type `first`, a space, Backspace back into the word, then delete until
/// `keep` characters of it remain on screen.
fn reopen_and_trim(options: EngineOptions, first: &str, keep: usize) -> Session {
    let mut session = Session::new(options);
    session.text(first);
    session.text(" ");
    session.backspace();
    while session.out.chars().count() > keep {
        session.backspace();
    }
    session
}

#[test]
fn vni_word_after_a_frozen_word_composes_once_back_at_its_consonant() {
    // Owner report: a non-Vietnamese word, space, Backspace into it, deleted
    // down to `d`; `9e6m` must then compose to `đêm`, not stay raw.
    for first in ["da11", "d99", "da66"] {
        let mut session = reopen_and_trim(vni_options(), first, 1);
        assert_eq!(session.out, "d", "{first}");
        assert_eq!(session.text("9e6m"), "đêm", "{first}");
    }
}

#[test]
fn vni_word_after_a_fully_deleted_frozen_word_composes() {
    for first in ["a11", "a66", "d99"] {
        let mut session = reopen_and_trim(vni_options(), first, 0);
        assert_eq!(session.out, "", "{first}");
        assert_eq!(session.text("d9e6m"), "đêm", "{first}");
    }
}

#[test]
fn telex_word_after_a_fully_deleted_frozen_word_composes() {
    for first in ["dass", "daaa", "dddd"] {
        let mut session = reopen_and_trim(EngineOptions::default(), first, 0);
        assert_eq!(session.out, "", "{first}");
        assert_eq!(session.text("ddeem"), "đêm", "{first}");
    }
}

#[test]
fn a_frozen_dd_remnant_stays_literal_after_backspace() {
    // `dddd` shows `ddd`; one Backspace leaves `dd`, which would turn into `đ`
    // if it were recomposed. The screen must not move and the word stays raw.
    let mut session = Session::new(EngineOptions::default());
    assert_eq!(session.text("dddd"), "ddd");
    assert_eq!(session.backspace(), "dd");
    assert_eq!(session.text("e"), "dde");
}
