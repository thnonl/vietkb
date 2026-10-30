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
            delete_units,
            insert,
        } => Decision {
            swallow: true,
            injection: Injection::Replace {
                deletes: *delete_units,
                insert: insert.clone(),
            },
        },
        // Type the replacement, then let the key through (macro committed with Enter/Tab).
        // The injector runs synchronously inside the hook callback, so the expansion
        // reaches the target before the physical key does.
        EngineAction::ReplaceThenPass {
            delete_units,
            insert,
        } => Decision {
            swallow: false,
            injection: Injection::Replace {
                deletes: *delete_units,
                insert: insert.clone(),
            },
        },
    }
}

/// The case keys the user holds at the event being decided, taken from the
/// hook's own ordered modifier tracking (it sees every event, in order, inside
/// the callback; the OS key-state reads can lag one event there).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeldKeys {
    /// Shift keys down, as a bit set: 1 = left, 2 = right, 4 = the generic
    /// `VK_SHIFT` that synthetic senders emit.
    pub shift: u8,
    /// Caps Lock is on.
    pub caps: bool,
}

impl HeldKeys {
    pub fn shift_down(self) -> bool {
        self.shift != 0
    }
}

/// Keep the head of the word the app already shows.
///
/// The engine rewrites the whole word (`delete_units == on_screen.len()`), but
/// only its tail usually changes: `Vie` + `e` is `Viê`. Retyping the unchanged
/// head costs nothing the user can see, except when the retyped letters are
/// capitals: an injected `Shift` pair around a capital is not reliably honoured
/// by every text stack (D-E2E-06, Win11 Notepad turns the retyped `V` of `Việt`
/// into `v`), while the physical key the user pressed always was. Leaving the
/// head alone keeps the user's own capital and shortens the batch.
///
/// `on_screen` is the engine's text *before* the key. Actions that do not
/// rewrite exactly that text are returned untouched.
pub fn keep_unchanged_head(on_screen: &[char], mut action: EngineAction) -> EngineAction {
    let (delete_units, insert, pass) = match &mut action {
        EngineAction::Replace {
            delete_units,
            insert,
        } => (delete_units, insert, false),
        EngineAction::ReplaceThenPass {
            delete_units,
            insert,
        } => (delete_units, insert, true),
        _ => return action,
    };
    if *delete_units as usize != on_screen.len() {
        return action;
    }
    let mut keep = on_screen
        .iter()
        .zip(insert.chars())
        .take_while(|(shown, new)| *shown == new)
        .count();
    // Never cut between a base letter and a combining mark (`unicode_nfd`
    // renders `ệ` as `e` + `U+0302` + `U+0323`): the base is retyped with them.
    while keep > 0
        && (on_screen.get(keep).copied().is_some_and(is_combining)
            || insert.chars().nth(keep).is_some_and(is_combining))
    {
        keep -= 1;
    }
    if keep == 0 {
        return action;
    }
    // In place: the engine already allocated `insert`; this only shortens it.
    let cut = insert.char_indices().nth(keep).map_or(insert.len(), |(at, _)| at);
    insert.drain(..cut);
    *delete_units -= keep as u32;
    if *delete_units == 0 && insert.is_empty() {
        // The word is already what the engine wants: nothing to rewrite.
        return if pass {
            EngineAction::Noop
        } else {
            EngineAction::Swallow
        };
    }
    action
}

/// Combining marks the charsets emit (`unicode_nfd` uses U+0300..=U+036F; the
/// other blocks are listed so a future charset cannot be cut apart either).
fn is_combining(ch: char) -> bool {
    matches!(ch,
        '\u{0300}'..='\u{036F}'
        | '\u{1AB0}'..='\u{1AFF}'
        | '\u{1DC0}'..='\u{1DFF}'
        | '\u{20D0}'..='\u{20FF}'
        | '\u{FE20}'..='\u{FE2F}')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    fn replace(delete_units: u32, insert: &str) -> EngineAction {
        EngineAction::Replace {
            delete_units,
            insert: insert.to_string(),
        }
    }

    #[test]
    fn head_is_kept_when_only_the_tail_changes() {
        assert_eq!(
            keep_unchanged_head(&shown("Vie"), replace(3, "Viê")),
            replace(1, "ê"),
            "the capital V is never retyped"
        );
        assert_eq!(
            keep_unchanged_head(&shown("Viêt"), replace(4, "Việt")),
            replace(2, "ệt")
        );
        assert_eq!(
            keep_unchanged_head(&shown("đep"), replace(3, "đẹp")),
            replace(2, "ẹp")
        );
    }

    #[test]
    fn a_base_letter_is_never_separated_from_its_combining_mark() {
        // `unicode_nfd`: `Viê` is `Vie` + U+0302; adding the tone appends U+0323.
        // The shared head is `Vie`, but cutting after the `e` would retype a bare
        // mark: the base goes with it.
        let on_screen = shown("Vie\u{302}");
        let action = replace(4, "Vie\u{302}\u{323}");
        assert_eq!(keep_unchanged_head(&on_screen, action), replace(2, "e\u{302}\u{323}"));
        // Swapping one mark for another on the same base.
        let action = replace(3, "Vi\u{e9}");
        assert_eq!(keep_unchanged_head(&shown("Vie"), action), replace(1, "\u{e9}"));
        let action = replace(4, "Vie\u{301}");
        assert_eq!(
            keep_unchanged_head(&shown("Vie\u{302}"), action),
            replace(2, "e\u{301}"),
            "the old mark and the new one are both tied to `e`"
        );
        // Marks that do not change keep the whole head, marks included.
        let action = replace(4, "Vie\u{302}t");
        assert_eq!(keep_unchanged_head(&shown("Vie\u{302}"), action), replace(0, "t"));
    }

    #[test]
    fn macro_only_replace_is_not_a_rewrite_of_the_screen() {
        // Macro-only mode keeps `on_screen` empty while the Replace deletes the
        // trigger the app typed: the counts differ, so the action is untouched.
        assert_eq!(keep_unchanged_head(&[], replace(3, "tắt ")), replace(3, "tắt "));
    }

    #[test]
    fn nothing_in_common_is_a_full_rewrite() {
        assert_eq!(keep_unchanged_head(&shown("dd"), replace(2, "đ")), replace(2, "đ"));
        assert_eq!(keep_unchanged_head(&shown("a"), replace(1, "")), replace(1, ""));
    }

    #[test]
    fn a_replace_that_is_not_the_whole_word_is_left_alone() {
        // Delete count differs from the word on screen: the engine knows
        // something this helper does not.
        assert_eq!(keep_unchanged_head(&shown("Vie"), replace(2, "Viê")), replace(2, "Viê"));
        assert_eq!(keep_unchanged_head(&[], replace(0, "x")), replace(0, "x"));
    }

    #[test]
    fn an_identical_rewrite_swallows_the_key() {
        assert_eq!(
            keep_unchanged_head(&shown("ab"), replace(2, "ab")),
            EngineAction::Swallow
        );
        let pass = EngineAction::ReplaceThenPass {
            delete_units: 2,
            insert: "ab".to_string(),
        };
        assert_eq!(keep_unchanged_head(&shown("ab"), pass), EngineAction::Noop);
    }

    #[test]
    fn replace_then_pass_keeps_its_pass() {
        let pass = EngineAction::ReplaceThenPass {
            delete_units: 3,
            insert: "tắt!".to_string(),
        };
        assert_eq!(
            keep_unchanged_head(&shown("tat"), pass),
            EngineAction::ReplaceThenPass {
                delete_units: 2,
                insert: "ắt!".to_string()
            }
        );
    }

    #[test]
    fn other_actions_are_untouched() {
        for action in [
            EngineAction::Noop,
            EngineAction::Swallow,
            EngineAction::Reject,
            EngineAction::Commit("x".to_string()),
        ] {
            assert_eq!(keep_unchanged_head(&shown("x"), action.clone()), action);
        }
    }

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
                delete_units: 4,
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
                delete_units: 0,
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
