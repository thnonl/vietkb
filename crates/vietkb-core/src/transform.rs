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

pub fn apply_tone(word: &str, tone: Tone) -> Option<String> {
    apply_tone_with_options(word, tone, false, true)
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

    for preferred in ['ê', 'Ê', 'ơ', 'Ơ', 'ô', 'Ô', 'â', 'Â', 'ă', 'Ă'] {
        if let Some(idx) = chars.iter().position(|ch| strip_tone(*ch) == preferred) {
            return Some(idx);
        }
    }

    if vowel_indices.len() >= 2 {
        let last = *vowel_indices.last()?;
        if matches!(strip_tone(chars[last]), 'i' | 'I' | 'u' | 'U' | 'y' | 'Y') {
            return vowel_indices.get(vowel_indices.len() - 2).copied();
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

fn is_vowel(ch: char) -> bool {
    LOWER_BASES.contains(&strip_tone(ch)) || UPPER_BASES.contains(&strip_tone(ch))
}
