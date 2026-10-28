//! Charset conversion for the "chuyển mã văn bản" tool.
//!
//! Sprint 1 supports the charsets whose mapping is exact and derivable from
//! Unicode: `unicode` (NFC), `unicode_nfd` (NFD), `utf8` (same bytes as NFC in a
//! Rust string) and `viqr`. The legacy single-byte tables (TCVN3, VNI Windows,
//! VISCII, BK HCM1, VPS) live in `vkb-core` and are not wired here yet — those
//! conversions return a clear error instead of emitting wrong bytes, which would
//! silently corrupt the user's text.
//!
//! This module is a deliberate duplicate of logic that belongs in `vkb-core`
//! (owned by the engine stream this sprint); see 70-impl-notes-ui.md.

use unicode_normalization::UnicodeNormalization;

const TONE_MARKS: [(char, &str); 5] = [
    ('\u{0301}', "'"), // acute
    ('\u{0300}', "`"), // grave
    ('\u{0309}', "?"), // hook above
    ('\u{0303}', "~"), // tilde
    ('\u{0323}', "."), // dot below
];

const TONE_FROM_VIQR: [(char, char); 5] = [
    ('\'', '\u{0301}'),
    ('`', '\u{0300}'),
    ('?', '\u{0309}'),
    ('~', '\u{0303}'),
    ('.', '\u{0323}'),
];

fn canon(slug: &str) -> Result<&'static str, String> {
    match slug.trim().to_ascii_lowercase().as_str() {
        "unicode" | "nfc" => Ok("unicode"),
        "unicode_nfd" | "nfd" | "unicode_to_hop" => Ok("unicode_nfd"),
        "utf8" | "utf-8" => Ok("utf8"),
        "viqr" => Ok("viqr"),
        "tcvn3" | "tcvn3_abc" => Ok("tcvn3"),
        "vni_windows" | "vni" => Ok("vni_windows"),
        "viscii" => Ok("viscii"),
        "bk_hcm1" => Ok("bk_hcm1"),
        "vps" => Ok("vps"),
        other => Err(format!("bảng mã không hợp lệ: {other}")),
    }
}

fn display_name(slug: &str) -> &'static str {
    match slug {
        "tcvn3" => "TCVN3 (ABC)",
        "vni_windows" => "VNI Windows",
        "viscii" => "VISCII",
        "bk_hcm1" => "BK HCM1",
        "vps" => "VPS",
        _ => "bảng mã này",
    }
}

/// Convert `input` from `from` to `to`. Both are the snake_case slugs used by
/// the config schema.
pub fn convert(input: &str, from: &str, to: &str) -> Result<String, String> {
    let from = canon(from)?;
    let to = canon(to)?;
    if from == to {
        return Ok(input.to_string());
    }

    // 1. Decode into NFC Unicode.
    let nfc = match from {
        "unicode" | "utf8" => input.nfc().collect::<String>(),
        "unicode_nfd" => input.nfc().collect::<String>(),
        "viqr" => decode_viqr(input),
        other => {
            return Err(format!(
                "Chuyển từ {} chưa được hỗ trợ trong bản này — cần bảng mã đầy đủ từ vkb-core.",
                display_name(other)
            ))
        }
    };

    // 2. Encode into the target charset.
    match to {
        "unicode" | "utf8" => Ok(nfc),
        "unicode_nfd" => Ok(nfc.nfd().collect()),
        "viqr" => Ok(encode_viqr(&nfc)),
        other => Err(format!(
            "Chuyển sang {} chưa được hỗ trợ trong bản này — cần bảng mã đầy đủ từ vkb-core.",
            display_name(other)
        )),
    }
}

fn is_vowel(ch: char) -> bool {
    // Decompose first: precomposed Vietnamese letters (ế, ẹ, ợ…) and the
    // modified bases (ă, â, ê, ô, ơ, ư) all reduce to one of these six.
    let base = ch
        .to_string()
        .nfd()
        .next()
        .unwrap_or(ch)
        .to_lowercase()
        .next()
        .unwrap_or(ch);
    matches!(base, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

fn is_upper_vietnamese(ch: char) -> bool {
    ch.is_uppercase()
}

/// Split one composed letter into (base, modifier marks, tone mark).
fn decompose(ch: char) -> (char, Vec<char>, Option<char>) {
    let mut base = ch;
    let mut modifiers = Vec::new();
    let mut tone = None;
    let decomposed: Vec<char> = ch.to_string().nfd().collect();
    if let Some(first) = decomposed.first() {
        base = *first;
    }
    for mark in decomposed.iter().skip(1) {
        match *mark {
            '\u{0302}' => modifiers.push('^'),
            '\u{0306}' => modifiers.push('('),
            '\u{031B}' => modifiers.push('+'),
            other => {
                if let Some((_, viqr)) = TONE_MARKS.iter().find(|(cp, _)| *cp == other) {
                    tone = Some(viqr.chars().next().unwrap());
                }
            }
        }
    }
    (base, modifiers, tone)
}

fn encode_viqr(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len() + 8);
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == 'đ' || ch == 'Đ' {
            out.push(if ch == 'Đ' { 'D' } else { 'd' });
            out.push(if ch == 'Đ' { 'D' } else { 'd' });
            i += 1;
            continue;
        }
        if is_vowel(ch) {
            let start = i;
            let mut end = i;
            while end < chars.len() && is_vowel(chars[end]) {
                end += 1;
            }
            let mut tone: Option<char> = None;
            for item in &chars[start..end] {
                let (base, modifiers, this_tone) = decompose(*item);
                out.push(base);
                for mark in modifiers {
                    out.push(mark);
                }
                if this_tone.is_some() {
                    tone = this_tone;
                }
            }
            if let Some(mark) = tone {
                out.push(mark);
            }
            i = end;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

fn decode_viqr(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // dd -> đ
        if (ch == 'd' || ch == 'D') && chars.get(i + 1).is_some_and(|next| *next == ch) {
            out.push(if ch == 'D' { 'Đ' } else { 'đ' });
            i += 2;
            continue;
        }

        // Diacritic modifiers apply to the previous letter.
        if matches!(ch, '^' | '(' | '+') {
            if let Some(last) = out.last().copied() {
                if let Some(replacement) = apply_modifier(last, ch) {
                    *out.last_mut().unwrap() = replacement;
                    i += 1;
                    continue;
                }
            }
            out.push(ch);
            i += 1;
            continue;
        }

        // Tone marks apply to the vowel cluster that precedes them.
        if let Some((_, combining)) = TONE_FROM_VIQR.iter().find(|(key, _)| *key == ch) {
            if apply_tone(&mut out, *combining) {
                i += 1;
                continue;
            }
        }

        out.push(ch);
        i += 1;
    }

    out.into_iter().collect::<String>().nfc().collect()
}

fn apply_modifier(ch: char, mark: char) -> Option<char> {
    let lower = ch.to_lowercase().next()?;
    let composed = match (lower, mark) {
        ('a', '(') => 'ă',
        ('a', '^') => 'â',
        ('e', '^') => 'ê',
        ('o', '^') => 'ô',
        ('o', '+') => 'ơ',
        ('u', '+') => 'ư',
        _ => return None,
    };
    Some(if is_upper_vietnamese(ch) {
        composed.to_uppercase().next().unwrap_or(composed)
    } else {
        composed
    })
}

/// Apply a combining tone mark to the trailing vowel cluster, honouring the
/// marked-vowel rule then "modern" placement (first vowel of an open two-vowel
/// cluster).
fn apply_tone(chars: &mut [char], tone: char) -> bool {
    let end = chars.len();
    let mut start = end;
    while start > 0 && is_vowel(chars[start - 1]) {
        start -= 1;
    }
    if start == end {
        return false;
    }

    let prefer_marked: Vec<usize> = (start..end)
        .filter(|index| {
            matches!(
                chars[*index].to_lowercase().next().unwrap_or(chars[*index]),
                'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư'
            )
        })
        .collect();
    let target = if let Some(&index) = prefer_marked.last() {
        index
    } else if end - start >= 2 {
        start
    } else {
        end - 1
    };

    let base = chars[target];
    let combined = format!("{}{}", base, tone).nfc().collect::<String>();
    let replacement = combined.chars().next().unwrap_or(base);
    chars[target] = replacement;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_nfd_roundtrip() {
        let src = "Tiếng Việt";
        let nfd = convert(src, "unicode", "unicode_nfd").unwrap();
        assert_ne!(nfd, src);
        assert_eq!(convert(&nfd, "unicode_nfd", "unicode").unwrap(), src);
    }

    #[test]
    fn viqr_roundtrip() {
        let src = "Tiếng Việt đẹp";
        let viqr = convert(src, "unicode", "viqr").unwrap();
        assert_eq!(viqr, "Tie^'ng Vie^.t dde.p");
        let back = convert(&viqr, "viqr", "unicode").unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn viqr_dd_and_horn() {
        assert_eq!(convert("được", "unicode", "viqr").unwrap(), "ddu+o+.c");
        assert_eq!(convert("ddu+o+.c", "viqr", "unicode").unwrap(), "được");
    }

    #[test]
    fn legacy_charsets_report_a_clear_error() {
        let err = convert("Tiếng Việt", "unicode", "tcvn3").unwrap_err();
        assert!(err.contains("TCVN3"), "{err}");
    }

    #[test]
    fn plain_ascii_passes_through() {
        assert_eq!(convert("hello world", "unicode", "viqr").unwrap(), "hello world");
    }
}
