//! Engine action -> OS injection plan.
//!
//! Backends do not interpret `EngineAction` themselves: they call [`decide`] and
//! translate the resulting [`Injection`] into native events. Kept OS-free and pure
//! so the decision table is unit-tested without a live hook.

use vkb_core::EngineAction;

/// Native work a backend must perform for one engine decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Injection {
    /// Touch nothing.
    None,
    /// Audible feedback only (invalid keystroke).
    Beep,
    /// Type `text` once.
    Type(String),
    /// Delete `deletes` graphemes backwards, then type `insert` — one batch.
    Replace { deletes: u32, insert: String },
}

/// What to do with the physical key that produced the action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    /// `true` = do not let the app see the key (`CallNextHookEx` is skipped).
    pub swallow: bool,
    pub injection: Injection,
}

impl Decision {
    pub const PASS: Self = Self {
        swallow: false,
        injection: Injection::None,
    };
    pub const SWALLOW: Self = Self {
        swallow: true,
        injection: Injection::None,
    };

    pub fn is_pass(&self) -> bool {
        !self.swallow && matches!(self.injection, Injection::None)
    }
}

/// Decide hook/interception behaviour for `action`.
///
/// `ch` is the character the physical key would have produced (`None` for keys
/// that type nothing, e.g. Enter or a dead key). A `Commit` that is exactly the
/// character the app is about to type anyway is passed through untouched, which
/// keeps the common ASCII path allocation- and injection-free.
pub fn decide(action: &EngineAction, ch: Option<char>) -> Decision {
    match action {
        EngineAction::Noop => Decision::PASS,
        EngineAction::Swallow => Decision::SWALLOW,
        EngineAction::ToggleEnabled => Decision::PASS,
        EngineAction::Reject => Decision {
            swallow: true,
            injection: Injection::Beep,
        },
        EngineAction::Commit(text) => {
            if text.is_empty() {
                return Decision::SWALLOW;
            }
            if let Some(ch) = ch {
                let mut chars = text.chars();
                if chars.next() == Some(ch) && chars.next().is_none() {
                    // The app itself types this character; re-typing it would double it.
                    return Decision::PASS;
                }
            }
            Decision {
                swallow: true,
                injection: Injection::Type(text.clone()),
            }
        }
        EngineAction::Replace {
            delete_graphemes,
            insert,
        } => Decision {
            swallow: true,
            injection: Injection::Replace {
                deletes: *delete_graphemes,
                insert: insert.clone(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_and_toggle_pass_through() {
        assert!(decide(&EngineAction::Noop, Some('t')).is_pass());
        assert!(decide(&EngineAction::ToggleEnabled, None).is_pass());
    }

    #[test]
    fn swallow_is_silent() {
        let d = decide(&EngineAction::Swallow, Some(' '));
        assert_eq!(d, Decision::SWALLOW);
        assert_eq!(d.injection, Injection::None);
    }

    #[test]
    fn reject_beeps_and_swallows() {
        let d = decide(&EngineAction::Reject, Some('z'));
        assert!(d.swallow);
        assert_eq!(d.injection, Injection::Beep);
    }

    #[test]
    fn commit_equal_to_typed_char_passes_through() {
        let d = decide(&EngineAction::Commit("t".to_string()), Some('t'));
        assert!(d.is_pass(), "must not retype the ASCII char: {d:?}");
    }

    #[test]
    fn commit_with_more_text_is_typed() {
        let d = decide(&EngineAction::Commit("th".to_string()), Some('h'));
        assert!(d.swallow);
        assert_eq!(d.injection, Injection::Type("th".to_string()));
    }

    #[test]
    fn commit_without_char_is_typed() {
        let d = decide(&EngineAction::Commit("Đ".to_string()), None);
        assert!(d.swallow);
        assert_eq!(d.injection, Injection::Type("Đ".to_string()));
    }

    #[test]
    fn commit_multi_char_cannot_be_mistaken_for_pass_through() {
        // `ch` equals the first char but the commit carries more than one.
        let d = decide(&EngineAction::Commit("tt".to_string()), Some('t'));
        assert!(d.swallow);
    }

    #[test]
    fn empty_commit_swallows_without_typing() {
        let d = decide(&EngineAction::Commit(String::new()), Some('t'));
        assert!(d.swallow);
        assert_eq!(d.injection, Injection::None, "no stale backspace/typing");
    }

    #[test]
    fn replace_is_batched() {
        let d = decide(
            &EngineAction::Replace {
                delete_graphemes: 4,
                insert: "thái".to_string(),
            },
            Some('s'),
        );
        assert!(d.swallow);
        assert_eq!(
            d.injection,
            Injection::Replace {
                deletes: 4,
                insert: "thái".to_string()
            }
        );
    }

    #[test]
    fn replace_with_zero_deletes_still_inserts() {
        let d = decide(
            &EngineAction::Replace {
                delete_graphemes: 0,
                insert: "ă".to_string(),
            },
            Some('a'),
        );
        assert!(d.swallow);
        assert!(matches!(
            d.injection,
            Injection::Replace {
                deletes: 0,
                insert: _
            }
        ));
    }
}
