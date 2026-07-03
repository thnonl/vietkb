use crate::engine::EngineAction;
use crate::transform::{apply_tone, change_mark, Tone};

pub fn apply_telex(word: &mut String, ch: char) -> EngineAction {
    if let Some(tone) = tone_from_key(ch) {
        if let Some(changed) = apply_tone(word, tone) {
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
    match ch.to_ascii_lowercase() {
        's' => Some(Tone::Acute),
        'f' => Some(Tone::Grave),
        'r' => Some(Tone::Hook),
        'x' => Some(Tone::Tilde),
        'j' => Some(Tone::Dot),
        _ => None,
    }
}

fn apply_mark(word: &str, mark: char) -> Option<String> {
    let mark = mark.to_ascii_lowercase();
    let mut chars: Vec<char> = word.chars().collect();

    if mark == 'd' {
        if let Some(idx) = chars.iter().rposition(|ch| matches!(ch, 'd' | 'D')) {
            chars[idx] = change_mark(chars[idx], mark)?;
            return Some(chars.into_iter().collect());
        }
        return None;
    }

    let idx = chars
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, ch)| change_mark(*ch, mark).map(|_| idx))?;
    chars[idx] = change_mark(chars[idx], mark)?;
    Some(chars.into_iter().collect())
}
