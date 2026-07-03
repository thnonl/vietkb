use vietkb_core::EngineAction;

use crate::SurroundingText;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextOutput {
    Commit(String),
    DeleteSurrounding { graphemes: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DirectCommit {
    surrounding_text: SurroundingText,
    outputs: Vec<TextOutput>,
}

impl DirectCommit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn surrounding_text(&self) -> &str {
        self.surrounding_text.text()
    }

    pub fn outputs(&self) -> &[TextOutput] {
        &self.outputs
    }

    pub fn apply(&mut self, action: EngineAction) {
        match action {
            EngineAction::Noop
            | EngineAction::Preedit(_)
            | EngineAction::ToggleVietnamese
            | EngineAction::BeepOrReject => {}
            EngineAction::Commit(text) => {
                self.surrounding_text.commit(&text);
                self.outputs.push(TextOutput::Commit(text));
            }
            EngineAction::Replace {
                delete_graphemes,
                insert,
            } => {
                self.surrounding_text
                    .replace_suffix(delete_graphemes, &insert);
                self.outputs.push(TextOutput::DeleteSurrounding {
                    graphemes: delete_graphemes,
                });
                self.outputs.push(TextOutput::Commit(insert));
            }
        }
    }
}
