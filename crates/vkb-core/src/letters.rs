//! The Vietnamese letter model.
//!
//! Every Vietnamese letter is a triple of *base letter* + *mark* + *tone*.
//! The alphabet has 13 base shapes (`a ă â e ê i o ô ơ u ư y đ`) and 6 tone
//! states (`không, sắc, huyền, hỏi, ngã, nặng`), which gives the **67 lowercase
//! letters** that every legacy 8-bit charset encodes, and the same 67 in
//! uppercase (134 total).
//!
//! `LOWER`/`UPPER` below are the single source of truth for composition. The
//! code points are pinned by `tests/charset.rs::letter_table_matches_unicode_names`,
//! which checks all 146 entries against the Unicode names.

use smallvec::SmallVec;

/// `MAX_WORD` letters fit in the engine buffers; see [`crate::MAX_WORD`].
pub type LetterBuf = SmallVec<[Letter; 32]>;

/// One Vietnamese tone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tone {
    #[default]
    None,
    /// `sắc` — á
    Acute,
    /// `huyền` — à
    Grave,
    /// `hỏi` — ả
    Hook,
    /// `ngã` — ã
    Tilde,
    /// `nặng` — ạ
    Dot,
}

impl Tone {
    pub const ALL: [Tone; 5] = [
        Tone::Acute,
        Tone::Grave,
        Tone::Hook,
        Tone::Tilde,
        Tone::Dot,
    ];

    /// Row index into [`LOWER`]/[`UPPER`].
    pub const fn index(self) -> usize {
        match self {
            Tone::None => 0,
            Tone::Acute => 1,
            Tone::Grave => 2,
            Tone::Hook => 3,
            Tone::Tilde => 4,
            Tone::Dot => 5,
        }
    }

    pub const fn from_index(index: usize) -> Tone {
        match index {
            1 => Tone::Acute,
            2 => Tone::Grave,
            3 => Tone::Hook,
            4 => Tone::Tilde,
            5 => Tone::Dot,
            _ => Tone::None,
        }
    }
}

/// A vowel modification that is not a tone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mark {
    #[default]
    None,
    /// `ă` (breve). Only `a`.
    Breve,
    /// `â ê ô` (circumflex). Only `a e o`.
    Circumflex,
    /// `ơ ư` (horn). Only `o u`.
    Horn,
    /// `đ` (bar). Only `d`.
    Bar,
}

/// One letter of the word being typed.
///
/// `ch` keeps the case the user typed. Letters that are not Vietnamese vowels
/// or `d` simply carry `Mark::None` and are rendered as-is, which is how
/// literal text (digits, punctuation after an escape, English letters) lives in
/// the same buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Letter {
    pub ch: char,
    pub mark: Mark,
}

impl Letter {
    pub const fn new(ch: char, mark: Mark) -> Self {
        Self { ch, mark }
    }

    pub const fn literal(ch: char) -> Self {
        Self { ch, mark: Mark::None }
    }

    /// Lowercase base letter, e.g. `Ơ` → `o`.
    ///
    /// Keystrokes are ASCII, but a composed letter can still reach the model
    /// (a Vietnamese keyboard layout, a test), so non-ASCII is decomposed.
    pub fn base(self) -> char {
        let lower = self.ch.to_ascii_lowercase();
        if lower.is_ascii() {
            lower
        } else {
            decompose(lower).map(|(base, _, _)| base).unwrap_or(lower)
        }
    }

    pub fn is_upper(self) -> bool {
        self.ch.is_ascii_uppercase()
    }

    pub fn is_vowel(self) -> bool {
        // `đ` is a consonant: the bar is a modification mark but the letter can
        // never carry a tone (A5), so it is never a tone target either.
        vowel_kind(self.base(), self.mark).is_some_and(|kind| kind != D_KIND)
    }

    /// `true` when this letter can carry the given mark.
    pub fn accepts(self, mark: Mark) -> bool {
        vowel_kind(self.base(), mark).is_some() && self.mark != mark
    }

    pub fn kind(self) -> Option<usize> {
        vowel_kind(self.base(), self.mark)
    }
}

/// `(base, mark)` of every letter shape, in table order.
pub const BASES: [(char, Mark); 13] = [
    ('a', Mark::None),
    ('a', Mark::Breve),
    ('a', Mark::Circumflex),
    ('e', Mark::None),
    ('e', Mark::Circumflex),
    ('i', Mark::None),
    ('o', Mark::None),
    ('o', Mark::Circumflex),
    ('o', Mark::Horn),
    ('u', Mark::None),
    ('u', Mark::Horn),
    ('y', Mark::None),
    ('d', Mark::Bar),
];

/// Index of `đ` in [`BASES`].
pub const D_KIND: usize = 12;

/// The 73 lowercase letters: `[kind][tone]`. `đ` only has `[12][0]`.
#[rustfmt::skip]
pub const LOWER: [[char; 6]; 13] = [
    ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
    ['đ', '\u{0}', '\u{0}', '\u{0}', '\u{0}', '\u{0}'],
];

/// The 73 uppercase letters, same layout as [`LOWER`].
#[rustfmt::skip]
pub const UPPER: [[char; 6]; 13] = [
    ['A', 'Á', 'À', 'Ả', 'Ã', 'Ạ'],
    ['Ă', 'Ắ', 'Ằ', 'Ẳ', 'Ẵ', 'Ặ'],
    ['Â', 'Ấ', 'Ầ', 'Ẩ', 'Ẫ', 'Ậ'],
    ['E', 'É', 'È', 'Ẻ', 'Ẽ', 'Ẹ'],
    ['Ê', 'Ế', 'Ề', 'Ể', 'Ễ', 'Ệ'],
    ['I', 'Í', 'Ì', 'Ỉ', 'Ĩ', 'Ị'],
    ['O', 'Ó', 'Ò', 'Ỏ', 'Õ', 'Ọ'],
    ['Ô', 'Ố', 'Ồ', 'Ổ', 'Ỗ', 'Ộ'],
    ['Ơ', 'Ớ', 'Ờ', 'Ở', 'Ỡ', 'Ợ'],
    ['U', 'Ú', 'Ù', 'Ủ', 'Ũ', 'Ụ'],
    ['Ư', 'Ứ', 'Ừ', 'Ử', 'Ữ', 'Ự'],
    ['Y', 'Ý', 'Ỳ', 'Ỷ', 'Ỹ', 'Ỵ'],
    ['Đ', '\u{0}', '\u{0}', '\u{0}', '\u{0}', '\u{0}'],
];

/// `(base, mark)` → table index, for the 12 vowel shapes and `đ`.
pub const fn vowel_kind(base: char, mark: Mark) -> Option<usize> {
    match (base, mark) {
        ('a', Mark::None) => Some(0),
        ('a', Mark::Breve) => Some(1),
        ('a', Mark::Circumflex) => Some(2),
        ('e', Mark::None) => Some(3),
        ('e', Mark::Circumflex) => Some(4),
        ('i', Mark::None) => Some(5),
        ('o', Mark::None) => Some(6),
        ('o', Mark::Circumflex) => Some(7),
        ('o', Mark::Horn) => Some(8),
        ('u', Mark::None) => Some(9),
        ('u', Mark::Horn) => Some(10),
        ('y', Mark::None) => Some(11),
        ('d', Mark::Bar) => Some(12),
        _ => None,
    }
}

/// Compose one letter. Returns `None` for impossible combinations
/// (`ê` with a breve, `ạ` on `đ`, …) so callers can fall back to a literal.
pub fn compose(ch: char, mark: Mark, tone: Tone) -> Option<char> {
    let base = ch.to_ascii_lowercase();
    let kind = vowel_kind(base, mark)?;
    if kind == D_KIND && tone != Tone::None {
        return None;
    }
    let table = if ch.is_ascii_uppercase() { &UPPER } else { &LOWER };
    let out = table[kind][tone.index()];
    if out == '\u{0}' {
        None
    } else {
        Some(out)
    }
}

/// Inverse of [`compose`]: a composed letter back to `(lowercase base, mark, tone)`.
pub fn decompose(ch: char) -> Option<(char, Mark, Tone)> {
    for (kind, (base, mark)) in BASES.iter().enumerate() {
        for (tone_idx, lower) in LOWER[kind].iter().enumerate() {
            if *lower == ch || UPPER[kind][tone_idx] == ch {
                return Some((*base, *mark, Tone::from_index(tone_idx)));
            }
        }
    }
    None
}

/// Combining marks used by the NFD encoding, in canonical order:
/// modification first, tone second.
pub fn combining_marks(mark: Mark, tone: Tone) -> &'static [char] {
    match (mark, tone) {
        (Mark::None, Tone::None) => &[],
        (Mark::None, Tone::Acute) => &['\u{301}'],
        (Mark::None, Tone::Grave) => &['\u{300}'],
        (Mark::None, Tone::Hook) => &['\u{309}'],
        (Mark::None, Tone::Tilde) => &['\u{303}'],
        (Mark::None, Tone::Dot) => &['\u{323}'],
        (Mark::Breve, Tone::None) => &['\u{306}'],
        (Mark::Breve, Tone::Acute) => &['\u{306}', '\u{301}'],
        (Mark::Breve, Tone::Grave) => &['\u{306}', '\u{300}'],
        (Mark::Breve, Tone::Hook) => &['\u{306}', '\u{309}'],
        (Mark::Breve, Tone::Tilde) => &['\u{306}', '\u{303}'],
        (Mark::Breve, Tone::Dot) => &['\u{306}', '\u{323}'],
        (Mark::Circumflex, Tone::None) => &['\u{302}'],
        (Mark::Circumflex, Tone::Acute) => &['\u{302}', '\u{301}'],
        (Mark::Circumflex, Tone::Grave) => &['\u{302}', '\u{300}'],
        (Mark::Circumflex, Tone::Hook) => &['\u{302}', '\u{309}'],
        (Mark::Circumflex, Tone::Tilde) => &['\u{302}', '\u{303}'],
        (Mark::Circumflex, Tone::Dot) => &['\u{302}', '\u{323}'],
        (Mark::Horn, Tone::None) => &['\u{31B}'],
        (Mark::Horn, Tone::Acute) => &['\u{31B}', '\u{301}'],
        (Mark::Horn, Tone::Grave) => &['\u{31B}', '\u{300}'],
        (Mark::Horn, Tone::Hook) => &['\u{31B}', '\u{309}'],
        (Mark::Horn, Tone::Tilde) => &['\u{31B}', '\u{303}'],
        (Mark::Horn, Tone::Dot) => &['\u{31B}', '\u{323}'],
        // `đ` has no canonical decomposition and neither does a bare `d`.
        (Mark::Bar, _) => &[],
    }
}

/// Case-fold-free uppercase of a composed letter, for `auto_capitalize`.
pub fn uppercase_of(ch: char) -> char {
    match decompose(ch) {
        Some((base, mark, tone)) => compose(
            base.to_ascii_uppercase(),
            mark,
            tone,
        )
        .unwrap_or(ch),
        None => ch.to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_roundtrips_every_letter() {
        for (kind, (base, mark)) in BASES.iter().enumerate() {
            for tone_idx in 0..6 {
                let lower = LOWER[kind][tone_idx];
                let upper = UPPER[kind][tone_idx];
                if lower == '\u{0}' {
                    assert_eq!(upper, '\u{0}', "both tables must leave gaps");
                    continue;
                }
                assert_eq!(compose(*base, *mark, Tone::from_index(tone_idx)), Some(lower));
                assert_eq!(
                    compose(base.to_ascii_uppercase(), *mark, Tone::from_index(tone_idx)),
                    Some(upper)
                );
                assert_eq!(decompose(lower), Some((*base, *mark, Tone::from_index(tone_idx))));
                assert_eq!(decompose(upper), Some((*base, *mark, Tone::from_index(tone_idx))));
            }
        }
    }

    #[test]
    fn impossible_marks_are_rejected() {
        assert_eq!(compose('e', Mark::Breve, Tone::None), None);
        assert_eq!(compose('i', Mark::Circumflex, Tone::None), None);
        assert_eq!(compose('o', Mark::Bar, Tone::None), None);
        assert_eq!(compose('d', Mark::Bar, Tone::Acute), None);
    }

    #[test]
    fn letter_flags() {
        assert!(Letter::literal('a').is_vowel());
        assert!(Letter::new('ư', Mark::Horn).is_vowel());
        assert!(!Letter::literal('d').is_vowel());
        // `đ` carries a mark but is never a vowel, so it never takes a tone.
        assert!(!Letter::new('d', Mark::Bar).is_vowel());
        assert!(!Letter::literal('n').is_vowel());
        assert!(Letter::new('A', Mark::Breve).is_upper());
        assert!(Letter::literal('a').accepts(Mark::Breve));
        assert!(!Letter::new('ă', Mark::Breve).accepts(Mark::Breve));
    }
}
