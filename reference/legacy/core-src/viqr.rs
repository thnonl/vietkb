use crate::engine::{EngineAction, EngineOptions};
use crate::transform::{apply_tone_with_options, change_mark, telex_w_target, Tone};

pub fn apply_viqr_with_options(
    word: &mut String,
    ch: char,
    options: EngineOptions,
) -> EngineAction {
    if let Some(tone) = tone_from_key(ch) {
        if let Some(changed) =
            apply_tone_with_options(word, tone, options.old_oa_uy, options.smart_mark)
        {
            let delete_graphemes = word.chars().count();
            *word = changed.clone();
            return EngineAction::Replace {
                delete_graphemes,
                insert: changed,
            };
        }
    }

    if let Some(changed) = apply_mark(word, ch) {
        let delete_graphemes = word.chars().count();
        *word = changed.clone();
        return EngineAction::Replace {
            delete_graphemes,
            insert: changed,
        };
    }

    word.push(ch);
    EngineAction::Commit(ch.to_string())
}

fn tone_from_key(ch: char) -> Option<Tone> {
    match ch {
        '\'' => Some(Tone::Acute),
        '`' => Some(Tone::Grave),
        '?' => Some(Tone::Hook),
        '~' => Some(Tone::Tilde),
        '.' => Some(Tone::Dot),
        _ => None,
    }
}

fn apply_mark(word: &str, mark: char) -> Option<String> {
    let mut chars: Vec<char> = word.chars().collect();
    let idx = if matches!(mark, '+' | '(') {
        telex_w_target(&chars)?
    } else {
        chars
            .iter()
            .enumerate()
            .rev()
            .find_map(|(idx, ch)| viqr_mark(*ch, mark).map(|_| idx))?
    };
    chars[idx] = viqr_mark(chars[idx], mark)?;
    Some(chars.into_iter().collect())
}

fn viqr_mark(ch: char, mark: char) -> Option<char> {
    match mark {
        '^' => change_mark(ch, ch.to_ascii_lowercase()),
        '(' => change_mark(ch, 'w'),
        '+' => change_mark(ch, 'w'),
        '-' if matches!(ch, 'd' | 'D') => change_mark(ch, 'd'),
        _ => None,
    }
}
