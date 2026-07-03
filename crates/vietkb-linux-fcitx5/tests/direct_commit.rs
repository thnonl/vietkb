use vietkb_core::EngineAction;
use vietkb_linux_fcitx5::{Fcitx5DirectCommit, Fcitx5Output};

#[test]
fn replacement_uses_surrounding_text_delete_then_commit() {
    let mut commit = Fcitx5DirectCommit::new();
    commit.apply(EngineAction::Commit("Vieet".to_owned()));
    commit.apply(EngineAction::Replace {
        delete_graphemes: 5,
        insert: "Việt".to_owned(),
    });

    assert_eq!(commit.committed(), "Việt");
    assert_eq!(
        commit.outputs(),
        &[
            Fcitx5Output::CommitString("Vieet".to_owned()),
            Fcitx5Output::DeleteSurroundingText {
                before: 5,
                after: 0
            },
            Fcitx5Output::CommitString("Việt".to_owned()),
        ]
    );
}
