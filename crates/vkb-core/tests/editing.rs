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

fn telex_spell(spell: bool) -> EngineOptions {
    EngineOptions {
        spell_check: spell,
        ..EngineOptions::default()
    }
}

/// Typed with `⌫` (U+232B) standing for Backspace.
fn typed_script(options: EngineOptions, script: &str) -> String {
    let mut session = Session::new(options);
    for ch in script.chars() {
        if ch == '\u{232B}' {
            session.backspace();
        } else {
            session.key(KeyInput::char(ch));
        }
    }
    session.out
}

#[test]
fn a_fully_deleted_frozen_word_is_a_fresh_word() {
    let o = telex_spell(true);
    assert_eq!(typed_script(o, "pass⌫⌫⌫dduwowcj"), "được");
    assert_eq!(typed_script(o, "test⌫⌫⌫⌫hif"), "hì");
    assert_eq!(typed_script(o, "class⌫⌫⌫⌫⌫dduwowcj"), "được");
}

#[test]
fn emptied_word_then_an_invalid_syllable_is_judged_fresh() {
    // After the word is gone, `hoafk` is typed raw by the spell check, and a
    // Backspace on it re-judges it like any other fresh word.
    let o = telex_spell(true);
    assert_eq!(typed_script(o, "class⌫⌫⌫⌫⌫hoafk"), "hoafk");
    assert_eq!(typed_script(o, "class⌫⌫⌫⌫⌫hoafk⌫"), "hòa");
}

#[test]
fn a_typo_then_backspace_rejudges_the_word() {
    let o = telex_spell(true);
    assert_eq!(typed_script(o, "dduwowcjk⌫"), "được");
    assert_eq!(typed_script(o, "gifk⌫"), "gì");
    assert_eq!(typed_script(o, "owk⌫n"), "ơn");
    assert_eq!(typed_script(o, "ddk⌫uwowcj"), "được");
    assert_eq!(typed_script(vni_options(), "a1k⌫"), "á");
}

#[test]
fn backspace_re_runs_tone_placement_for_a_word_that_was_not_a_syllable() {
    let o = EngineOptions {
        spell_check: false,
        modern_tone: true,
        ..EngineOptions::default()
    };
    assert_eq!(typed_script(o, "hoafk⌫"), "hòa");
    // A real coda deleted keeps the display (A7 step 4, D-TN-06).
    assert_eq!(typed_script(o, "toans⌫"), "toá");
}

#[test]
fn tone_before_a_final_stop_consonant_composes() {
    let o = telex_spell(true);
    for (keys, word) in [("hojc", "học"), ("nhajt", "nhạt"), ("cajc", "cạc"), ("bojc", "bọc"), ("thojc", "thọc")] {
        assert_eq!(typed_script(o, keys), word, "{keys}");
    }
    // English still passes through: `test` is t-e-s-t, not `tét`.
    assert_eq!(typed_script(o, "test"), "test");
}

#[test]
fn a_word_after_an_edit_gets_the_fresh_spell_verdict() {
    // `êi` is no syllable: fresh typing leaves it raw, and so does a word whose
    // first letters were deleted and retyped.
    let v = vni_options();
    assert_eq!(typed_script(v, "d9e6i"), "d9e6i");
    assert_eq!(typed_script(v, "dog⌫⌫9e6i"), "d9e6i");
    let t = telex_spell(true);
    assert_eq!(typed_script(t, "dog⌫⌫deei"), "ddeei");
    // With the spell check off the same keys compose.
    assert_eq!(typed_script(telex_spell(false), "dog⌫⌫deei"), "đêi");
}

#[test]
fn a_digit_ends_a_word_and_backspace_reopens_it() {
    assert_eq!(typed_script(telex_spell(false), "d9⌫deei"), "đêi");
    assert_eq!(typed_script(EngineOptions::default(), "a1⌫s"), "á");
}

#[test]
fn double_toggle_then_backspace_revalidates() {
    let o = telex_spell(true);
    assert_eq!(typed_script(o, "giff⌫s"), "gí");
    assert_eq!(typed_script(vni_options(), "gi11⌫1"), "gí");
}

#[test]
fn vni_tone_digit_then_backspace_deletes_the_whole_letter() {
    // `á` is one character, so one Backspace empties the word; a digit with no
    // letter before it is plain text.
    assert_eq!(typed_script(vni_options(), "a1⌫1"), "1");
}

#[test]
fn doubled_key_after_a_kept_consonant_composes() {
    assert_eq!(typed_script(EngineOptions::default(), "dd9⌫ejp"), "đẹp");
    assert_eq!(typed_script(telex_spell(true), "dass⌫⌫d"), "đ");
}

#[test]
fn digits_before_letters_are_plain_text() {
    // Owner ruling: leading digits never block composing (Telex `duowcj` is
    // `dược`; `dduwowcj` is `được`).
    let t = telex_spell(true);
    assert_eq!(typed_script(t, "1dduwowcj"), "1được");
    assert_eq!(typed_script(t, "1duowcj"), "1dược");
    assert_eq!(typed_script(t, "3giowf"), "3giờ");
    assert_eq!(typed_script(t, "1⌫dduwowcj"), "được");
    assert_eq!(typed_script(vni_options(), "2d9e6m"), "2đêm");
    assert_eq!(typed_script(vni_options(), "12d9e6m"), "12đêm");
    assert_eq!(typed_script(vni_options(), "1⌫d9e6m"), "đêm");
    // A word that starts with letters and carries a digit later is unchanged.
    assert_eq!(typed_script(t, "mp3"), "mp3");
    assert_eq!(typed_script(t, "a1b"), "a1b");
    // The code zone still freezes what follows it.
    assert_eq!(typed_script(t, "C:\\dduwowcj"), "C:\\dduwowcj");
}

#[test]
fn restore_hotkey_does_not_claim_a_word_it_would_not_change() {
    let mut session = Session::new(telex_spell(true));
    session.text("dass");
    session.backspace();
    session.backspace();
    assert_eq!(session.out, "d");
    // A bare `d` restores to `d`: let Ctrl+Shift+Z reach the application.
    assert!(!session.engine.can_restore());
    session.text("d");
    assert_eq!(session.out, "đ");
    assert!(session.engine.can_restore());
}

#[test]
fn trigger_key_remnant_followed_by_a_vowel_composes_like_fresh_typing() {
    // A consonant-only remnant made of trigger keys is just letters.
    for key in ['s', 'f', 'r', 'x', 'j'] {
        let o = telex_spell(true);
        let fresh_word = typed_script(o, &format!("{key}a"));
        assert_eq!(typed_script(o, &format!("{key}an⌫⌫a")), fresh_word, "{key}");
    }
    for digit in ['1', '2', '3', '4', '5'] {
        let o = vni_options();
        let fresh_word = typed_script(o, &format!("d{digit}a"));
        assert_eq!(typed_script(o, &format!("d{digit}an⌫⌫a")), fresh_word, "d{digit}");
    }
}

// ---------------------------------------------------------------------------
// Owner: fresh `d9e6i` stays raw (êi is no syllable), but deleting from it and
// retyping Vietnamese must compose.
// ---------------------------------------------------------------------------

/// (label, options, keys of the raw word, keys that finish `đêm`, `đến`)
fn owner_cfgs() -> Vec<(String, EngineOptions, &'static str, &'static str, &'static str)> {
    let mut v = Vec::new();
    for restore in [false, true] {
        let vni = EngineOptions {
            method: InputMethod::Vni,
            spell_check: true,
            restore_wrong_word: restore,
            ..EngineOptions::default()
        };
        let telex = EngineOptions {
            spell_check: true,
            restore_wrong_word: restore,
            ..EngineOptions::default()
        };
        v.push((format!("VNI restore={restore}"), vni, "d9e6i", "d9e6m", "d9e6n1"));
        v.push((format!("Telex restore={restore}"), telex, "ddeei", "ddeem", "ddeens"));
    }
    v
}

/// Screen after every key of `script` (`⌫` = Backspace), joined with ` | `.
fn trace(options: EngineOptions, script: &str) -> (String, String) {
    let mut session = Session::new(options);
    let mut steps = Vec::new();
    for ch in script.chars() {
        if ch == '\u{232B}' {
            session.backspace();
        } else {
            session.key(KeyInput::char(ch));
        }
        steps.push(format!("{:?}", session.out));
    }
    (session.out.clone(), steps.join(" | "))
}

/// Backspaces needed to bring the screen after `base` down to `len` chars.
fn presses_down_to(options: EngineOptions, base: &str, len: usize) -> usize {
    let mut session = Session::new(options);
    for ch in base.chars() {
        session.key(KeyInput::char(ch));
    }
    let mut n = 0;
    while session.out.chars().count() > len && n < 32 {
        session.backspace();
        n += 1;
    }
    n
}

fn bs_n(n: usize) -> String {
    "\u{232B}".repeat(n)
}

#[test]
fn owner_raw_word_is_still_raw_when_typed_fresh() {
    for (label, options, raw, _, _) in owner_cfgs() {
        assert_eq!(typed_script(options, raw), raw, "{label}");
    }
}

#[test]
fn owner_one_backspace_then_m_composes() {
    for (label, options, raw, _, _) in owner_cfgs() {
        let script = format!("{raw}\u{232B}m");
        let (out, steps) = trace(options, &script);
        println!("{label}: {script} => {steps}");
        assert_eq!(out, "đêm", "{label}: {steps}");
    }
}

/// The keystrokes still missing from `em` (`d9e6m` / `ddeem`) once only
/// `shown` is left on screen: a plain `d` still needs the bar, an `đ` does not.
fn missing_after<'a>(shown: &str, em: &'a str) -> &'a str {
    match shown {
        "d" => &em[1..],
        "đ" => &em[2..],
        other => panic!("unexpected one-character screen {other:?}"),
    }
}

#[test]
fn owner_delete_to_one_character_then_complete() {
    // Backspacing the raw `d9e6i` re-judges it: the screen goes `d9e6i` → `đê`
    // → `đ`, so the last character is the finished `đ` (typing `9` onto it
    // again would escape the bar, exactly like fresh `d99`). Completion is
    // therefore `e6m`; a frozen word that really does end on a bare `d` takes
    // the whole `9e6m`.
    for (label, options, raw, em, _) in owner_cfgs() {
        let n = presses_down_to(options, raw, 1);
        let mut session = Session::new(options);
        session.text(raw);
        for _ in 0..n {
            session.backspace();
        }
        let shown = session.out.clone();
        let script = format!("{raw}{}{}", bs_n(n), missing_after(&shown, em));
        let (out, steps) = trace(options, &script);
        println!("{label}: {script} => {steps}");
        assert_eq!(out, "đêm", "{label}: {steps}");
        // A word frozen as plain text, deleted down to a bare `d`.
        for first in ["dxcv", "dpqr"] {
            let n = presses_down_to(options, first, 1);
            let script = format!("{first}{}{}", bs_n(n), &em[1..]);
            let (out, steps) = trace(options, &script);
            println!("{label}: {script} => {steps}");
            assert_eq!(out, "đêm", "{label}: {steps}");
        }
    }
}

#[test]
fn owner_delete_to_empty_then_retype() {
    for (label, options, raw, em, en) in owner_cfgs() {
        let n = presses_down_to(options, raw, 0);
        for word in [em, en] {
            let script = format!("{raw}{}{word}", bs_n(n));
            let (out, steps) = trace(options, &script);
            println!("{label}: {script} => {steps}");
            let expected = if word == em { "đêm" } else { "đến" };
            assert_eq!(out, expected, "{label}: {steps}");
        }
    }
}

#[test]
fn owner_every_intermediate_prefix_then_complete() {
    for (label, options, raw, em, en) in owner_cfgs() {
        let mut session = Session::new(options);
        for ch in raw.chars() {
            session.key(KeyInput::char(ch));
        }
        // Every screen the Backspace key passes through, from the raw word down.
        let mut stops = vec![session.out.clone()];
        let mut presses = vec![0usize];
        let mut n = 0;
        while !session.out.is_empty() && n < 32 {
            session.backspace();
            n += 1;
            stops.push(session.out.clone());
            presses.push(n);
        }
        println!("{label}: screens after each Backspace: {stops:?}");
        for (k, shown) in presses.iter().zip(stops.iter()) {
            // What is left on screen, typed as the prefix of the finished word.
            for (word, expected) in [(em, "đêm"), (en, "đến")] {
                // Keystrokes of the finished word that the screen does not show yet.
                let have: String = match shown.as_str() {
                    "" => String::new(),
                    s if s.starts_with('d') && s.chars().count() == 1 => "d".into(),
                    "đ" => if raw == "d9e6i" { "d9".into() } else { "dd".into() },
                    "đê" => if raw == "d9e6i" { "d9e6".into() } else { "ddee".into() },
                    s if s == "d9" || s == "dd" || s == "d9e" || s == "dde" || s == "d9e6" || s == "ddee" || s == "d9e6i" || s == "ddeei" => s.to_string(),
                    other => {
                        println!("{label}: no keystroke prefix for screen {other:?}");
                        continue;
                    }
                };
                if !word.starts_with(&have) {
                    continue;
                }
                let script = format!("{raw}{}{}", bs_n(*k), &word[have.len()..]);
                let (out, steps) = trace(options, &script);
                println!("{label}: k={k} shown={shown:?} -> {script} => {steps}");
                assert_eq!(out, expected, "{label} k={k}: {steps}");
            }
        }
    }
}

#[test]
fn owner_through_the_space() {
    for (label, options, raw, em, en) in owner_cfgs() {
        // The separator commits the word; with restore_wrong_word it also rewrites
        // the raw word (A10), so count what is really on screen.
        let base = format!("{raw} ");
        let mut session = Session::new(options);
        for ch in base.chars() {
            session.key(KeyInput::char(ch));
        }
        let total = session.out.chars().count();
        // Through the space and back into the word, to the first `d`.
        let to_d = presses_down_to(options, &base, 1);
        let to_empty = presses_down_to(options, &base, 0);
        println!("{label}: screen after space {:?} ({total} chars), to_d={to_d}, to_empty={to_empty}", session.out);
        let mut one = Session::new(options);
        one.text(&base);
        for _ in 0..to_d {
            one.backspace();
        }
        let rest = missing_after(&one.out.clone(), em);
        let script = format!("{base}{}{rest}", bs_n(to_d));
        let (out, steps) = trace(options, &script);
        println!("{label}: {script} => {steps}");
        assert_eq!(out, "đêm", "{label}: to d: {steps}");
        for (word, expected) in [(em, "đêm"), (en, "đến")] {
            let script = format!("{base}{}{word}", bs_n(to_empty));
            let (out, steps) = trace(options, &script);
            println!("{label}: {script} => {steps}");
            assert_eq!(out, expected, "{label}: to empty: {steps}");
        }
        // One Backspace through the space only reopens the word: `m` then continues it.
        let script = format!("{base}\u{232B}\u{232B}m");
        let (out, steps) = trace(options, &script);
        println!("{label}: {script} => {steps}");
        assert_eq!(out, "đêm", "{label}: through space + 1: {steps}");
    }
}

// ---------------------------------------------------------------------------
// Round 3 rulings (D-BS-LIT-01)
// ---------------------------------------------------------------------------

#[test]
fn tone_pin_depends_on_the_old_word_being_a_syllable_not_on_spell_check() {
    // A real coda deleted keeps the display; a word that was never a syllable
    // re-runs A4 now. Both hold with the spell check on and off.
    for spell in [true, false] {
        for modern_tone in [true, false] {
            let o = EngineOptions {
                spell_check: spell,
                modern_tone,
                ..EngineOptions::default()
            };
            assert_eq!(typed_script(o, "toans⌫"), "toá", "spell={spell} modern={modern_tone}");
        }
    }
    let off = EngineOptions {
        spell_check: false,
        modern_tone: true,
        ..EngineOptions::default()
    };
    assert_eq!(typed_script(off, "hoafk⌫"), "hòa");
}

#[test]
fn keystrokes_that_survived_any_backspace_keep_their_marks() {
    // `toan` BS `s` types the tone after the first edit; BS and a second edit
    // make those middle keys part of the surviving word, so a plain letter that
    // only appends keeps `toá` instead of freezing the word as raw text.
    let o = telex_spell(true);
    assert_eq!(typed_script(o, "toan⌫sn⌫nn"), "toánn");
    assert_eq!(typed_script(o, "toan⌫sn⌫n⌫nn"), "toánn");
    // Without the second Backspace the tone was typed after the only edit, so the
    // appended `nn` is judged like fresh typing: raw.
    assert_eq!(typed_script(o, "toan⌫snnn"), "toasnnn");
}

#[test]
fn post_edit_tone_then_repeated_trigger_matches_fresh_typing() {
    let o = telex_spell(true);
    // The first key after a Backspace that kept a stale tone re-runs A4.
    assert_eq!(typed_script(o, "toans⌫s"), "tóa");
    // The next one is a repeated trigger: the documented escape, exactly what
    // fresh typing of `toasss` shows, and the word is not frozen by the edit.
    assert_eq!(typed_script(o, "toans⌫ss"), "toass");
    assert_eq!(typed_script(o, "toasss"), "toass");
    assert_eq!(typed_script(o, "toans⌫sss"), "toasss");
    // Adjacent repeat right after the Backspace is the plain escape.
    assert_eq!(typed_script(o, "bans⌫s"), "bas");
}

#[test]
fn j_before_a_stop_coda_is_not_an_english_cluster_but_the_other_tones_are() {
    let o = telex_spell(true);
    assert_eq!(typed_script(o, "hojc"), "học");
    assert_eq!(typed_script(o, "nhajt"), "nhạt");
    // `s`, `f`, `r`, `x` before a stop coda still read as English.
    for word in ["hosc", "fast", "best", "test"] {
        assert_eq!(typed_script(o, word), word, "{word}");
    }
    // A tone typed after the coda is the Vietnamese order and still works.
    assert_eq!(typed_script(o, "hocj"), "học");
}
