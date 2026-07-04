use crate::charset::convert_output;
use crate::english::is_english_word;
use crate::key::{KeyCode, KeyInput};
use crate::telex::apply_telex_with_options;
use crate::viqr::apply_viqr_with_options;
use crate::vni::apply_vni_with_options;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    Telex,
    Vni,
    Viqr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Unicode,
    VniWindows,
    Tcvn3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineOptions {
    pub old_oa_uy: bool,
    pub smart_mark: bool,
    pub use_j_f_w_as_consonants: bool,
    pub spell_check: bool,
    pub restore_wrong_word: bool,
    pub skip_english_words: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            old_oa_uy: false,
            smart_mark: true,
            use_j_f_w_as_consonants: true,
            spell_check: true,
            restore_wrong_word: false,
            skip_english_words: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineAction {
    Noop,
    Commit(String),
    Replace {
        delete_graphemes: usize,
        insert: String,
    },
    Preedit(String),
    ToggleVietnamese,
    BeepOrReject,
}

pub trait VietnameseEngine {
    fn reset(&mut self);
    fn process_key(&mut self, input: KeyInput) -> EngineAction;
    fn set_input_method(&mut self, method: InputMethod);
    fn set_charset(&mut self, charset: Charset);
}

#[derive(Debug, Clone)]
pub struct SimpleVietnameseEngine {
    method: InputMethod,
    charset: Charset,
    enabled: bool,
    options: EngineOptions,
    word: String,
    raw: String,
    english_bypass: bool,
    viqr_escape: bool,
}

impl Default for SimpleVietnameseEngine {
    fn default() -> Self {
        Self {
            method: InputMethod::Telex,
            charset: Charset::Unicode,
            enabled: true,
            options: EngineOptions::default(),
            word: String::new(),
            raw: String::new(),
            english_bypass: false,
            viqr_escape: false,
        }
    }
}

impl SimpleVietnameseEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    /// Restore the raw keystrokes for the current word, discarding Vietnamese
    /// transformation. Used by the restore-word hotkey.
    pub fn restore_word(&mut self) -> EngineAction {
        if self.raw.is_empty() || self.raw == self.word {
            return EngineAction::Noop;
        }
        let delete_graphemes = self.word.chars().count();
        let insert = self.raw.clone();
        self.word = self.raw.clone();
        self.english_bypass = true;
        EngineAction::Replace {
            delete_graphemes,
            insert,
        }
    }

    fn recompute_word(&self) -> String {
        let mut word = String::new();
        for ch in self.raw.chars() {
            if ch.is_ascii_alphabetic() {
                match self.method {
                    InputMethod::Telex => {
                        apply_telex_with_options(&mut word, ch, self.options);
                    }
                    InputMethod::Vni => {
                        apply_vni_with_options(&mut word, ch, self.options);
                    }
                    InputMethod::Viqr => {
                        apply_viqr_with_options(&mut word, ch, self.options);
                    }
                }
            } else if ch.is_ascii_digit() && matches!(self.method, InputMethod::Vni) {
                apply_vni_with_options(&mut word, ch, self.options);
            } else if is_ascii_punctuation(ch) && matches!(self.method, InputMethod::Viqr) {
                apply_viqr_with_options(&mut word, ch, self.options);
            } else {
                word.push(ch);
            }
        }
        word
    }

    fn process_backspace(&mut self) -> EngineAction {
        if self.raw.is_empty() {
            self.word.pop();
            return EngineAction::Noop;
        }

        let old_graphemes = self.word.chars().count();
        self.raw.pop();
        self.english_bypass = false;
        let new_word = self.recompute_word();
        self.word = new_word.clone();

        if old_graphemes == 0 {
            return EngineAction::Noop;
        }

        EngineAction::Replace {
            delete_graphemes: old_graphemes,
            insert: convert_output(&new_word, self.charset),
        }
    }

    fn convert_action(&self, action: EngineAction) -> EngineAction {
        match action {
            EngineAction::Commit(text) => EngineAction::Commit(convert_output(&text, self.charset)),
            EngineAction::Replace {
                delete_graphemes,
                insert,
            } => EngineAction::Replace {
                delete_graphemes,
                insert: convert_output(&insert, self.charset),
            },
            EngineAction::Preedit(text) => {
                EngineAction::Preedit(convert_output(&text, self.charset))
            }
            action => action,
        }
    }

    pub fn set_skip_english_words(&mut self, value: bool) {
        self.options.skip_english_words = value;
    }

    pub fn set_options(&mut self, options: EngineOptions) {
        self.options = options;
        self.reset();
    }

    fn reset_word(&mut self) {
        self.word.clear();
        self.raw.clear();
        self.english_bypass = false;
        self.viqr_escape = false;
    }

    fn commit_boundary(&mut self, text: &str) -> EngineAction {
        if self.options.restore_wrong_word && should_restore_raw(&self.raw, &self.word) {
            let delete_graphemes = self.word.chars().count();
            let mut insert = self.raw.clone();
            insert.push_str(text);
            self.reset_word();
            return EngineAction::Replace {
                delete_graphemes,
                insert: convert_output(&insert, self.charset),
            };
        }

        self.reset_word();
        EngineAction::Commit(convert_output(text, self.charset))
    }

    fn process_alphabetic(&mut self, ch: char) -> EngineAction {
        self.raw.push(ch);

        if self.english_bypass {
            if matches!(self.method, InputMethod::Telex) && is_telex_transform_key(ch) {
                self.word = self.raw[..self.raw.len() - ch.len_utf8()].to_owned();
                self.english_bypass = false;
            } else {
                return EngineAction::Commit(ch.to_string());
            }
        }

        if self.options.use_j_f_w_as_consonants
            && matches!(self.method, InputMethod::Telex)
            && should_keep_telex_consonant(&self.raw, ch)
        {
            self.word.push(ch);
            return EngineAction::Commit(ch.to_string());
        }

        let previous_graphemes = self.word.chars().count();
        let action = match self.method {
            InputMethod::Telex => apply_telex_with_options(&mut self.word, ch, self.options),
            InputMethod::Vni => apply_vni_with_options(&mut self.word, ch, self.options),
            InputMethod::Viqr => apply_viqr_with_options(&mut self.word, ch, self.options),
        };

        if self.options.spell_check
            && self.options.skip_english_words
            && (matches!(self.method, InputMethod::Telex) || self.raw == self.word)
            && is_english_word(&self.raw)
        {
            self.english_bypass = true;
            let delete_graphemes = previous_graphemes;
            let insert = self.raw.clone();
            self.word.clear();
            return EngineAction::Replace {
                delete_graphemes,
                insert,
            };
        }

        self.convert_action(action)
    }
}

impl VietnameseEngine for SimpleVietnameseEngine {
    fn reset(&mut self) {
        self.reset_word();
    }

    fn process_key(&mut self, input: KeyInput) -> EngineAction {
        if !input.is_plain_text() {
            return EngineAction::Noop;
        }

        match input.code {
            KeyCode::Char('\u{0014}') => {
                self.enabled = !self.enabled;
                EngineAction::ToggleVietnamese
            }
            KeyCode::Char(ch) if ch.is_ascii_alphabetic() => {
                if !self.enabled {
                    return EngineAction::Commit(ch.to_string());
                }

                self.process_alphabetic(ch)
            }
            KeyCode::Char(ch) if ch.is_whitespace() => self.commit_boundary(&ch.to_string()),
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                if self.enabled && matches!(self.method, InputMethod::Vni) {
                    if self.english_bypass {
                        self.word = self.raw.clone();
                        self.english_bypass = false;
                    }
                    return apply_vni_with_options(&mut self.word, ch, self.options);
                }
                self.reset_word();
                EngineAction::Commit(ch.to_string())
            }
            KeyCode::Char(ch) if is_ascii_punctuation(ch) => {
                if self.enabled && matches!(self.method, InputMethod::Viqr) {
                    if ch == '\\' {
                        self.viqr_escape = true;
                        return EngineAction::Noop;
                    }
                    if self.viqr_escape {
                        self.viqr_escape = false;
                        return self.commit_boundary(&ch.to_string());
                    }
                    if is_viqr_key(ch) {
                        if self.english_bypass {
                            self.word = self.raw.clone();
                            self.english_bypass = false;
                        }
                        return apply_viqr_with_options(&mut self.word, ch, self.options);
                    }
                }
                self.commit_boundary(&ch.to_string())
            }
            KeyCode::Space => self.commit_boundary(" "),
            KeyCode::Enter => self.commit_boundary("\n"),
            KeyCode::Backspace => self.process_backspace(),
            KeyCode::Char(ch) => self.commit_boundary(&ch.to_string()),
            KeyCode::Other => {
                self.reset_word();
                EngineAction::Noop
            }
        }
    }

    fn set_input_method(&mut self, method: InputMethod) {
        self.method = method;
        self.reset();
    }

    fn set_charset(&mut self, charset: Charset) {
        self.charset = charset;
        self.reset();
    }
}

fn is_ascii_punctuation(ch: char) -> bool {
    ch.is_ascii_punctuation()
}

fn is_viqr_key(ch: char) -> bool {
    matches!(ch, '\'' | '`' | '?' | '~' | '.' | '^' | '(' | '+' | '-')
}

fn is_telex_transform_key(ch: char) -> bool {
    matches!(ch.to_ascii_lowercase(), 's' | 'f' | 'r' | 'x' | 'j' | 'a' | 'e' | 'o' | 'w' | 'd')
}

fn should_keep_telex_consonant(raw: &str, ch: char) -> bool {
    let ch = ch.to_ascii_lowercase();
    if !matches!(ch, 'f' | 'j' | 'w') {
        return false;
    }

    let raw = raw.to_ascii_lowercase();
    matches!(
        raw.as_str(),
        "of" | "oj" | "awf" | "owf" | "uwf" | "aww" | "oww" | "uww"
    )
}

fn should_restore_raw(raw: &str, transformed: &str) -> bool {
    if raw.is_empty() || raw == transformed {
        return false;
    }

    let raw = raw.to_ascii_lowercase();
    matches!(
        raw.as_str(),
        "of" | "oj" | "awf" | "owf" | "uwf" | "aww" | "oww" | "uww"
    )
}
