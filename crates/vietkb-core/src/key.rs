#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Backspace,
    Space,
    Enter,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInput {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyInput {
    pub fn from_char(ch: char) -> Self {
        Self {
            code: KeyCode::Char(ch),
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    pub fn plain(code: KeyCode) -> Self {
        Self {
            code,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    pub fn is_plain_text(self) -> bool {
        !self.ctrl && !self.alt && !self.meta
    }
}
