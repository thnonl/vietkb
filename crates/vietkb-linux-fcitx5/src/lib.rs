use vietkb_core::EngineAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fcitx5Output {
    CommitString(String),
    DeleteSurroundingText { before: usize, after: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Fcitx5DirectCommit {
    committed: String,
    outputs: Vec<Fcitx5Output>,
}

impl Fcitx5DirectCommit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn committed(&self) -> &str {
        &self.committed
    }

    pub fn outputs(&self) -> &[Fcitx5Output] {
        &self.outputs
    }

    pub fn apply(&mut self, action: EngineAction) {
        match action {
            EngineAction::Noop
            | EngineAction::Preedit(_)
            | EngineAction::ToggleVietnamese
            | EngineAction::BeepOrReject => {}
            EngineAction::Commit(text) => {
                self.committed.push_str(&text);
                self.outputs.push(Fcitx5Output::CommitString(text));
            }
            EngineAction::Replace {
                delete_graphemes,
                insert,
            } => {
                for _ in 0..delete_graphemes {
                    self.committed.pop();
                }
                self.committed.push_str(&insert);
                self.outputs.push(Fcitx5Output::DeleteSurroundingText {
                    before: delete_graphemes,
                    after: 0,
                });
                self.outputs.push(Fcitx5Output::CommitString(insert));
            }
        }
    }
}
