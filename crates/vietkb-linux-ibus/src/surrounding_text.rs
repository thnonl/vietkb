#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SurroundingText {
    text: String,
}

impl SurroundingText {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn replace_suffix(&mut self, delete_graphemes: usize, insert: &str) {
        for _ in 0..delete_graphemes {
            self.text.pop();
        }
        self.text.push_str(insert);
    }

    pub fn commit(&mut self, text: &str) {
        self.text.push_str(text);
    }
}
