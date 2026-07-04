use crate::english::is_english_word;
use crate::key::{KeyCode, KeyInput};
use crate::telex::apply_telex_with_options;
use crate::viqr::apply_viqr;
use crate::vni::apply_vni;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    Telex,
    Vni,
    Viqr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Unicode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineOptions {
    pub old_oa_uy: bool,
    pub smart_mark: bool,
    pub spell_check: bool,
    pub skip_english_words: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            old_oa_uy: false,
            smart_mark: true,
            spell_check: true,
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
    }

    fn commit_boundary(&mut self, text: &str) -> EngineAction {
        self.reset_word();
        EngineAction::Commit(text.to_owned())
    }

    fn process_alphabetic(&mut self, ch: char) -> EngineAction {
        self.raw.push(ch);

        if self.english_bypass {
            return EngineAction::Commit(ch.to_string());
        }

        let previous_graphemes = self.word.chars().count();
        let action = match self.method {
            InputMethod::Telex => apply_telex_with_options(&mut self.word, ch, self.options),
            InputMethod::Vni => apply_vni(&mut self.word, ch),
            InputMethod::Viqr => apply_viqr(&mut self.word, ch),
        };

        if self.options.spell_check && self.options.skip_english_words && is_english_word(&self.raw)
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

        action
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
                if self.enabled && !self.english_bypass && matches!(self.method, InputMethod::Vni) {
                    return apply_vni(&mut self.word, ch);
                }
                self.reset_word();
                EngineAction::Commit(ch.to_string())
            }
            KeyCode::Char(ch) if is_ascii_punctuation(ch) => {
                if self.enabled && !self.english_bypass && matches!(self.method, InputMethod::Viqr)
                {
                    return apply_viqr(&mut self.word, ch);
                }
                self.reset_word();
                EngineAction::Commit(ch.to_string())
            }
            KeyCode::Space => self.commit_boundary(" "),
            KeyCode::Enter => self.commit_boundary("\n"),
            KeyCode::Backspace => {
                self.word.pop();
                self.raw.pop();
                EngineAction::Noop
            }
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
