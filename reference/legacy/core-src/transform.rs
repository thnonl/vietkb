#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Acute,
    Grave,
    Hook,
    Tilde,
    Dot,
}

const LOWER_BASES: &[char] = &['a', 'ă', 'â', 'e', 'ê', 'i', 'o', 'ô', 'ơ', 'u', 'ư', 'y'];
const UPPER_BASES: &[char] = &['A', 'Ă', 'Â', 'E', 'Ê', 'I', 'O', 'Ô', 'Ơ', 'U', 'Ư', 'Y'];

const LOWER_TONES: [[char; 5]; 12] = [
    ['á', 'à', 'ả', 'ã', 'ạ'],
    ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    ['é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    ['ế', 'ề', 'ể', 'ễ', 'ệ'],
    ['í', 'ì', 'ỉ', 'ĩ', 'ị'],
    ['ó', 'ò', 'ỏ', 'õ', 'ọ'],
    ['ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    ['ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    ['ú', 'ù', 'ủ', 'ũ', 'ụ'],
    ['ứ', 'ừ', 'ử', 'ữ', 'ự'],
    ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
];

const UPPER_TONES: [[char; 5]; 12] = [
    ['Á', 'À', 'Ả', 'Ã', 'Ạ'],
    ['Ắ', 'Ằ', 'Ẳ', 'Ẵ', 'Ặ'],
    ['Ấ', 'Ầ', 'Ẩ', 'Ẫ', 'Ậ'],
    ['É', 'È', 'Ẻ', 'Ẽ', 'Ẹ'],
    ['Ế', 'Ề', 'Ể', 'Ễ', 'Ệ'],
    ['Í', 'Ì', 'Ỉ', 'Ĩ', 'Ị'],
    ['Ó', 'Ò', 'Ỏ', 'Õ', 'Ọ'],
    ['Ố', 'Ồ', 'Ổ', 'Ỗ', 'Ộ'],
    ['Ớ', 'Ờ', 'Ở', 'Ỡ', 'Ợ'],
    ['Ú', 'Ù', 'Ủ', 'Ũ', 'Ụ'],
    ['Ứ', 'Ừ', 'Ử', 'Ữ', 'Ự'],
    ['Ý', 'Ỳ', 'Ỷ', 'Ỹ', 'Ỵ'],
];

pub fn change_mark(ch: char, mark: char) -> Option<char> {
    let changed = match (ch, mark) {
        ('a', 'w') => 'ă',
        ('A', 'W') | ('A', 'w') => 'Ă',
        ('a', 'a') => 'â',
        ('A', 'A') | ('A', 'a') => 'Â',
        ('e', 'e') => 'ê',
        ('E', 'E') | ('E', 'e') => 'Ê',
        ('o', 'o') => 'ô',
        ('O', 'O') | ('O', 'o') => 'Ô',
        ('o', 'w') => 'ơ',
        ('O', 'W') | ('O', 'w') => 'Ơ',
        ('u', 'w') => 'ư',
        ('U', 'W') | ('U', 'w') => 'Ư',
        ('d', 'd') => 'đ',
        ('D', 'D') | ('D', 'd') => 'Đ',
        _ => return None,
    };
    Some(changed)
}

pub fn apply_tone_with_options(
    word: &str,
    tone: Tone,
    old_oa_uy: bool,
    smart_mark: bool,
) -> Option<String> {
    let chars: Vec<char> = word.chars().collect();
    let idx = tone_target(&chars, old_oa_uy, smart_mark)?;
    let toned = tone_char(chars[idx], tone)?;
    let mut out = String::new();
    for (i, ch) in chars.into_iter().enumerate() {
        out.push(if i == idx { toned } else { strip_tone(ch) });
    }
    Some(out)
}

fn tone_target(chars: &[char], old_oa_uy: bool, smart_mark: bool) -> Option<usize> {
    let vowel_indices: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter_map(|(i, ch)| is_vowel(*ch).then_some(i))
        .collect();

    if vowel_indices.is_empty() {
        return None;
    }

    if !smart_mark {
        return vowel_indices.last().copied();
    }

    if let Some(idx) = oa_oe_uy_target(chars, old_oa_uy) {
        return Some(idx);
    }

    for (idx, triple) in chars.windows(3).enumerate() {
        let first = strip_tone(triple[0]).to_ascii_lowercase();
        let second = strip_tone(triple[1]).to_ascii_lowercase();
        let third = strip_tone(triple[2]).to_ascii_lowercase();
        if matches!((first, second, third), ('u', 'y', 'u')) {
            return Some(idx + 1);
        }
    }

    for preferred in ['ê', 'Ê', 'ơ', 'Ơ', 'ô', 'Ô', 'â', 'Â', 'ă', 'Ă', 'ư', 'Ư'] {
        if let Some(idx) = chars.iter().position(|ch| strip_tone(*ch) == preferred) {
            return Some(idx);
        }
    }

    if vowel_indices.len() >= 2 {
        if let Some(idx) = modern_oa_oe_uy_target(chars) {
            return Some(idx);
        }

        let first = vowel_indices[vowel_indices.len() - 2];
        let last = *vowel_indices.last()?;
        let first_base = strip_tone(chars[first]).to_ascii_lowercase();
        let last_base = strip_tone(chars[last]).to_ascii_lowercase();
        let has_trailing_consonant = chars[last + 1..].iter().any(|ch| ch.is_ascii_alphabetic());

        if first_base == 'u' && last_base == 'y' {
            if has_trailing_consonant
                || (first > 0 && matches!(chars[first - 1].to_ascii_lowercase(), 'q'))
                || (first > 1
                    && matches!(chars[first - 1].to_ascii_lowercase(), 'h')
                    && matches!(chars[first - 2].to_ascii_lowercase(), 'k'))
                || (first > 1
                    && matches!(chars[first - 1].to_ascii_lowercase(), 'g')
                    && matches!(chars[first - 2].to_ascii_lowercase(), 'n'))
            {
                return Some(last);
            }
            return Some(first);
        }

        if matches!((first_base, last_base), ('a' | 'e', 'o')) {
            return Some(first);
        }

        if matches!((first_base, last_base), ('i' | 'u', 'a')) {
            if first > 0 && matches!(chars[first - 1].to_ascii_lowercase(), 'g' | 'q') {
                return Some(last);
            }
            return Some(first);
        }

        if matches!((first_base, last_base), ('i', 'u')) {
            return Some(if has_trailing_consonant { last } else { first });
        }

        if matches!(last_base, 'i' | 'u' | 'y') {
            return Some(first);
        }
    }

    vowel_indices.last().copied()
}

fn oa_oe_uy_target(chars: &[char], old_oa_uy: bool) -> Option<usize> {
    if !old_oa_uy {
        return None;
    }

    for (idx, pair) in chars.windows(2).enumerate() {
        let first = strip_tone(pair[0]).to_ascii_lowercase();
        let second = strip_tone(pair[1]).to_ascii_lowercase();
        if matches!((first, second), ('o', 'a') | ('o', 'e') | ('u', 'y')) {
            return Some(idx + 1);
        }
    }
    None
}

fn modern_oa_oe_uy_target(chars: &[char]) -> Option<usize> {
    for (idx, pair) in chars.windows(2).enumerate() {
        let first = strip_tone(pair[0]).to_ascii_lowercase();
        let second = strip_tone(pair[1]).to_ascii_lowercase();
        let has_trailing_consonant = chars[idx + 2..].iter().any(|ch| ch.is_ascii_alphabetic());
        let is_quy = first == 'u'
            && second == 'y'
            && idx > 0
            && matches!(chars[idx - 1].to_ascii_lowercase(), 'q');
        if !has_trailing_consonant && !is_quy && matches!((first, second), ('o', 'a') | ('o', 'e')) {
            return Some(idx);
        }
    }
    None
}

fn tone_char(ch: char, tone: Tone) -> Option<char> {
    let base = strip_tone(ch);
    let tone_idx = match tone {
        Tone::Acute => 0,
        Tone::Grave => 1,
        Tone::Hook => 2,
        Tone::Tilde => 3,
        Tone::Dot => 4,
    };

    if let Some(base_idx) = LOWER_BASES.iter().position(|candidate| *candidate == base) {
        return Some(LOWER_TONES[base_idx][tone_idx]);
    }
    if let Some(base_idx) = UPPER_BASES.iter().position(|candidate| *candidate == base) {
        return Some(UPPER_TONES[base_idx][tone_idx]);
    }
    None
}

pub fn strip_tone(ch: char) -> char {
    for (base_idx, tones) in LOWER_TONES.iter().enumerate() {
        if tones.contains(&ch) {
            return LOWER_BASES[base_idx];
        }
    }
    for (base_idx, tones) in UPPER_TONES.iter().enumerate() {
        if tones.contains(&ch) {
            return UPPER_BASES[base_idx];
        }
    }
    ch
}

pub fn strip_vietnamese_mark(ch: char) -> char {
    match strip_tone(ch) {
        'ă' | 'â' => 'a',
        'Ă' | 'Â' => 'A',
        'ê' => 'e',
        'Ê' => 'E',
        'ô' | 'ơ' => 'o',
        'Ô' | 'Ơ' => 'O',
        'ư' => 'u',
        'Ư' => 'U',
        'đ' => 'd',
        'Đ' => 'D',
        base => base,
    }
}

pub fn apply_telex_w_mark(chars: &mut [char]) -> Option<()> {
    let last = telex_w_target(chars)?;

    if matches!(strip_tone(chars[last]), 'o' | 'O') {
        if let Some(prev) = chars[..last]
            .iter()
            .rposition(|ch| matches!(strip_tone(*ch), 'u' | 'U'))
        {
            chars[prev] = change_mark(strip_tone(chars[prev]), 'w')?;
            chars[last] = change_mark(strip_tone(chars[last]), 'w')?;
            return Some(());
        }
    }

    chars[last] = change_mark(chars[last], 'w')?;
    Some(())
}

pub fn telex_w_target(chars: &[char]) -> Option<usize> {
    let last = chars
        .iter()
        .rposition(|ch| matches!(strip_tone(*ch), 'a' | 'A' | 'u' | 'U' | 'o' | 'O'))?;
    if matches!(strip_tone(chars[last]), 'u' | 'U') {
        if let Some(prev) = chars[..last]
            .iter()
            .rposition(|ch| matches!(strip_tone(*ch), 'u' | 'U'))
        {
            return Some(prev);
        }
    }
    Some(last)
}

fn is_vowel(ch: char) -> bool {
    LOWER_BASES.contains(&strip_tone(ch)) || UPPER_BASES.contains(&strip_tone(ch))
}
