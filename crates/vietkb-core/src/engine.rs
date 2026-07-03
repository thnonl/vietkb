use crate::key::{KeyCode, KeyInput};
use crate::telex::apply_telex;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineAction {
    Noop,
    Commit(String),
    Replace { delete_graphemes: usize, insert: String },
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
    word: String,
}

impl Default for SimpleVietnameseEngine {
    fn default() -> Self {
        Self {
            method: InputMethod::Telex,
            charset: Charset::Unicode,
            enabled: true,
            word: String::new(),
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

    fn commit_boundary(&mut self, text: &str) -> EngineAction {
        self.word.clear();
        EngineAction::Commit(text.to_owned())
    }
}

impl VietnameseEngine for SimpleVietnameseEngine {
    fn reset(&mut self) {
        self.word.clear();
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

                match self.method {
                    InputMethod::Telex => apply_telex(&mut self.word, ch),
                    InputMethod::Vni => apply_vni(&mut self.word, ch),
                    InputMethod::Viqr => apply_viqr(&mut self.word, ch),
                }
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                if self.enabled && matches!(self.method, InputMethod::Vni) {
                    return apply_vni(&mut self.word, ch);
                }
                self.word.clear();
                EngineAction::Commit(ch.to_string())
            }
            KeyCode::Char(ch) if is_ascii_punctuation(ch) => {
                if self.enabled && matches!(self.method, InputMethod::Viqr) {
                    return apply_viqr(&mut self.word, ch);
                }
                self.word.clear();
                EngineAction::Commit(ch.to_string())
            }
            KeyCode::Space => self.commit_boundary(" "),
            KeyCode::Enter => self.commit_boundary("\n"),
            KeyCode::Backspace => {
                self.word.pop();
                EngineAction::Noop
            }
            KeyCode::Char(ch) => self.commit_boundary(&ch.to_string()),
            KeyCode::Other => {
                self.word.clear();
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
