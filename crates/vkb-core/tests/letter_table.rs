//! The 146-letter table (67 lowercase + 67 uppercase + `đ`/`Đ`) against the
//! Unicode names. A typo in `letters::LOWER`/`UPPER` fails here before it can
//! poison a charset table.

use vkb_core::letters::{decompose, Mark, Tone, BASES, D_KIND, LOWER, UPPER};

/// Authoritative code points (Latin-1, Latin Extended-A and Latin Extended
/// Additional; the Vietnamese block U+1EA0..U+1EF9 is contiguous per letter).
#[test]
fn every_letter_has_the_expected_code_point() {
    let expected: &[(char, char, u32)] = &[
        // (lowercase, uppercase, codepoint of the lowercase)
        ('a', 'A', 0x61), ('á', 'Á', 0xE1), ('à', 'À', 0xE0), ('ả', 'Ả', 0x1EA3), ('ã', 'Ã', 0xE3), ('ạ', 'Ạ', 0x1EA1),
        ('ă', 'Ă', 0x103), ('ắ', 'Ắ', 0x1EAF), ('ằ', 'Ằ', 0x1EB1), ('ẳ', 'Ẳ', 0x1EB3), ('ẵ', 'Ẵ', 0x1EB5), ('ặ', 'Ặ', 0x1EB7),
        ('â', 'Â', 0xE2), ('ấ', 'Ấ', 0x1EA5), ('ầ', 'Ầ', 0x1EA7), ('ẩ', 'Ẩ', 0x1EA9), ('ẫ', 'Ẫ', 0x1EAB), ('ậ', 'Ậ', 0x1EAD),
        ('e', 'E', 0x65), ('é', 'É', 0xE9), ('è', 'È', 0xE8), ('ẻ', 'Ẻ', 0x1EBB), ('ẽ', 'Ẽ', 0x1EBD), ('ẹ', 'Ẹ', 0x1EB9),
        ('ê', 'Ê', 0xEA), ('ế', 'Ế', 0x1EBF), ('ề', 'Ề', 0x1EC1), ('ể', 'Ể', 0x1EC3), ('ễ', 'Ễ', 0x1EC5), ('ệ', 'Ệ', 0x1EC7),
        ('i', 'I', 0x69), ('í', 'Í', 0xED), ('ì', 'Ì', 0xEC), ('ỉ', 'Ỉ', 0x1EC9), ('ĩ', 'Ĩ', 0x129), ('ị', 'Ị', 0x1ECB),
        ('o', 'O', 0x6F), ('ó', 'Ó', 0xF3), ('ò', 'Ò', 0xF2), ('ỏ', 'Ỏ', 0x1ECF), ('õ', 'Õ', 0xF5), ('ọ', 'Ọ', 0x1ECD),
        ('ô', 'Ô', 0xF4), ('ố', 'Ố', 0x1ED1), ('ồ', 'Ồ', 0x1ED3), ('ổ', 'Ổ', 0x1ED5), ('ỗ', 'Ỗ', 0x1ED7), ('ộ', 'Ộ', 0x1ED9),
        ('ơ', 'Ơ', 0x1A1), ('ớ', 'Ớ', 0x1EDB), ('ờ', 'Ờ', 0x1EDD), ('ở', 'Ở', 0x1EDF), ('ỡ', 'Ỡ', 0x1EE1), ('ợ', 'Ợ', 0x1EE3),
        ('u', 'U', 0x75), ('ú', 'Ú', 0xFA), ('ù', 'Ù', 0xF9), ('ủ', 'Ủ', 0x1EE7), ('ũ', 'Ũ', 0x169), ('ụ', 'Ụ', 0x1EE5),
        ('ư', 'Ư', 0x1B0), ('ứ', 'Ứ', 0x1EE9), ('ừ', 'Ừ', 0x1EEB), ('ử', 'Ử', 0x1EED), ('ữ', 'Ữ', 0x1EEF), ('ự', 'Ự', 0x1EF1),
        ('y', 'Y', 0x79), ('ý', 'Ý', 0xFD), ('ỳ', 'Ỳ', 0x1EF3), ('ỷ', 'Ỷ', 0x1EF7), ('ỹ', 'Ỹ', 0x1EF9), ('ỵ', 'Ỵ', 0x1EF5),
        ('đ', 'Đ', 0x111),
    ];
    assert_eq!(expected.len(), 73, "12 vowel shapes × 6 tones + đ");

    for (lower, upper, code) in expected {
        assert_eq!(*lower as u32, *code, "{lower} codepoint");
        assert_eq!(lower.to_uppercase().collect::<String>(), upper.to_string(), "{lower} uppercase");
        assert!(decompose(*lower).is_some(), "{lower} decomposes");
        assert!(decompose(*upper).is_some(), "{upper} decomposes");
    }
}

/// Flatten `LOWER`/`UPPER` and check the tables hold exactly those 67 pairs
/// plus the `đ` gap layout.
#[test]
fn tables_hold_exactly_the_67_letters() {
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for (kind, (_, _)) in BASES.iter().enumerate() {
        for tone in 0..6 {
            let l = LOWER[kind][tone];
            let u = UPPER[kind][tone];
            if l == '\u{0}' {
                assert_eq!(u, '\u{0}', "kind {kind} tone {tone} keeps the gap in both tables");
                continue;
            }
            lower.push(l);
            upper.push(u);
        }
    }
    assert_eq!(lower.len(), 73, "12 vowel shapes × 6 tones + đ");
    assert_eq!(upper.len(), 73);
    // 67 of those carry a mark or a tone; the other 6 are plain ASCII vowels.
    assert_eq!(lower.len() - 6, 67);
    // `đ` carries no tone: the only letter with a single table slot.
    assert_eq!(LOWER[D_KIND][0], 'đ');
    assert_eq!(UPPER[D_KIND][0], 'Đ');
    for tone in Tone::ALL {
        assert_eq!(LOWER[D_KIND][tone.index()], '\u{0}');
    }
}

/// Every table entry is a distinct scalar value: a duplicate would silently
/// merge two letters in every charset.
#[test]
fn no_duplicate_letters() {
    let mut seen = Vec::new();
    for row in LOWER.iter().chain(UPPER.iter()) {
        for ch in row {
            if *ch == '\u{0}' {
                continue;
            }
            assert!(!seen.contains(ch), "{ch} appears twice");
            seen.push(*ch);
        }
    }
    assert_eq!(seen.len(), 146, "73 lowercase + 73 uppercase");
}

#[test]
fn mark_and_tone_are_recovered_from_the_composed_letter() {
    assert_eq!(decompose('ắ'), Some(('a', Mark::Breve, Tone::Acute)));
    assert_eq!(decompose('Ữ'), Some(('u', Mark::Horn, Tone::Tilde)));
    assert_eq!(decompose('ộ'), Some(('o', Mark::Circumflex, Tone::Dot)));
    assert_eq!(decompose('đ'), Some(('d', Mark::Bar, Tone::None)));
    assert_eq!(decompose('x'), None);
}
