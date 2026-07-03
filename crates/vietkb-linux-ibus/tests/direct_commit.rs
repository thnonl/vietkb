use vietkb_core::EngineAction;
use vietkb_linux_ibus::{DirectCommit, TextOutput};

#[test]
fn commits_text_immediately() {
    let mut commit = DirectCommit::new();
    commit.apply(EngineAction::Commit("a".to_owned()));

    assert_eq!(commit.surrounding_text(), "a");
    assert_eq!(commit.outputs(), &[TextOutput::Commit("a".to_owned())]);
}

#[test]
fn replacement_deletes_surrounding_text_then_commits() {
    let mut commit = DirectCommit::new();
    commit.apply(EngineAction::Commit("tieeng".to_owned()));
    commit.apply(EngineAction::Replace {
        delete_graphemes: 6,
        insert: "tiếng".to_owned(),
    });

    assert_eq!(commit.surrounding_text(), "tiếng");
    assert_eq!(
        commit.outputs(),
        &[
            TextOutput::Commit("tieeng".to_owned()),
            TextOutput::DeleteSurrounding { graphemes: 6 },
            TextOutput::Commit("tiếng".to_owned()),
        ]
    );
}
