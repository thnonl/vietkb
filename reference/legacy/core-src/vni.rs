use crate::engine::{EngineAction, EngineOptions};
use crate::transform::{
    apply_telex_w_mark, apply_tone_with_options, change_mark, strip_tone, strip_vietnamese_mark,
    Tone,
};

pub fn apply_vni_with_options(word: &mut String, ch: char, options: EngineOptions) -> EngineAction {
    if ch == '0' {
        let delete_graphemes = word.chars().count();
        let insert: String = word.chars().map(strip_vietnamese_mark).collect();
        *word = insert.clone();
        return EngineAction::Replace {
            delete_graphemes,
            insert,
        };
    }

    if let Some(tone) = tone_from_key(ch) {
        if let Some(changed) =
            apply_tone_with_options(word, tone, options.old_oa_uy, options.smart_mark)
        {
            let delete_graphemes = word.chars().count();
            let insert = if changed == *word {
                let mut raw: String = word.chars().map(strip_tone).collect();
                raw.push(ch);
                raw
            } else {
                changed
            };
            *word = insert.clone();
            return EngineAction::Replace {
                delete_graphemes,
                insert,
            };
        }
    }

    if let Some(mark) = mark_from_key(ch) {
        if let Some(changed) = apply_mark(word, mark) {
            let delete_graphemes = word.chars().count();
            *word = changed.clone();
            return EngineAction::Replace {
                delete_graphemes,
                insert: changed,
            };
        }
    }

    word.push(ch);
    EngineAction::Commit(ch.to_string())
}

fn tone_from_key(ch: char) -> Option<Tone> {
    match ch {
        '1' => Some(Tone::Acute),
        '2' => Some(Tone::Grave),
        '3' => Some(Tone::Hook),
        '4' => Some(Tone::Tilde),
        '5' => Some(Tone::Dot),
        _ => None,
    }
}

fn mark_from_key(ch: char) -> Option<char> {
    match ch {
        '6' => Some('a'),
        '7' => Some('w'),
        '8' => Some('w'),
        '9' => Some('d'),
        _ => None,
    }
}

fn apply_mark(word: &str, mark: char) -> Option<String> {
    let mut chars: Vec<char> = word.chars().collect();
    if mark == 'd' {
        let idx = chars.iter().rposition(|ch| matches!(ch, 'd' | 'D'))?;
        chars[idx] = change_mark(chars[idx], mark)?;
        return Some(chars.into_iter().collect());
    }

    if mark == 'w' {
        apply_telex_w_mark(&mut chars)?;
        return Some(chars.into_iter().collect());
    }

    let idx = chars
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, ch)| vni_mark(*ch, mark).map(|_| idx))?;
    chars[idx] = vni_mark(chars[idx], mark)?;
    Some(chars.into_iter().collect())
}

fn vni_mark(ch: char, mark: char) -> Option<char> {
    match mark {
        'a' => change_mark(ch, ch.to_ascii_lowercase()),
        'w' => change_mark(ch, 'w'),
        _ => None,
    }
}
