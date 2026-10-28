//! Charset output: exact byte tables, engine-level output, table coverage and
//! provenance status.

use vkb_core::charset::TableStatus;
use vkb_core::{simulate, Charset, EngineOptions, InputMethod};

/// Legacy byte tables are carried as `char::from(byte)`, so the engine's output
/// string is the Latin-1 view of the charset bytes.
fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|b| char::from(*b)).collect()
}

fn with_charset(charset: Charset, method: InputMethod) -> EngineOptions {
    EngineOptions {
        charset,
        method,
        ..EngineOptions::default()
    }
}

#[test]
fn unicode_output_is_nfc() {
    assert_eq!(
        simulate(with_charset(Charset::Unicode, InputMethod::Telex), "tieengs"),
        "tiếng"
    );
    assert_eq!(
        simulate(with_charset(Charset::Utf8, InputMethod::Telex), "tieengs"),
        "tiếng"
    );
}

#[test]
fn nfd_output_decomposes_every_letter() {
    let options = with_charset(Charset::UnicodeNfd, InputMethod::Telex);
    assert_eq!(
        simulate(options, "tieengs"),
        "tie\u{302}\u{301}ng",
        "ế must be e + circumflex + acute"
    );
    assert_eq!(
        simulate(options, "dduwowngf"),
        "\u{111}u\u{31b}o\u{31b}\u{300}ng"
    );
}

#[test]
fn tcvn3_bytes() {
    let options = with_charset(Charset::Tcvn3, InputMethod::Telex);
    assert_eq!(simulate(options, "tieengs"), latin1(&[0x74, 0x69, 0xD5, 0x6E, 0x67]));
    assert_eq!(simulate(options, "Vieetj"), latin1(&[0x56, 0x69, 0xD6, 0x74]));
    assert_eq!(
        simulate(options, "dduwowngf"),
        latin1(&[0xAE, 0xAD, 0xEA, 0x6E, 0x67])
    );
}

#[test]
fn viscii_bytes() {
    let options = with_charset(Charset::Viscii, InputMethod::Telex);
    assert_eq!(simulate(options, "tieengs"), latin1(&[0x74, 0x69, 0xAA, 0x6E, 0x67]));
    assert_eq!(simulate(options, "Vieetj"), latin1(&[0x56, 0x69, 0xAE, 0x74]));
    assert_eq!(
        simulate(options, "dduwowngf"),
        latin1(&[0xF0, 0xDF, 0xB6, 0x6E, 0x67])
    );
}

#[test]
fn vps_bytes() {
    let options = with_charset(Charset::Vps, InputMethod::Telex);
    assert_eq!(simulate(options, "tieengs"), latin1(&[0x74, 0x69, 0x89, 0x6E, 0x67]));
    assert_eq!(simulate(options, "Vieetj"), latin1(&[0x56, 0x69, 0x8C, 0x74]));
    assert_eq!(
        simulate(options, "dduwowngf"),
        latin1(&[0xC7, 0xDC, 0xA9, 0x6E, 0x67])
    );
}

#[test]
fn vni_windows_bytes() {
    let options = with_charset(Charset::VniWindows, InputMethod::Telex);
    assert_eq!(
        simulate(options, "tieengs"),
        latin1(&[0x74, 0x69, 0x65, 0xE1, 0x6E, 0x67])
    );
    assert_eq!(
        simulate(options, "Vieetj"),
        latin1(&[0x56, 0x69, 0x65, 0xE4, 0x74])
    );
    assert_eq!(
        simulate(options, "dduwowngf"),
        latin1(&[0xF1, 0xF6, 0xF4, 0xF8, 0x6E, 0x67])
    );
}

#[test]
fn viqr_output_is_ascii() {
    let options = with_charset(Charset::Viqr, InputMethod::Telex);
    assert_eq!(simulate(options, "tieengs"), "tie^'ng");
    assert_eq!(simulate(options, "Vieetj"), "Vie^.t");
    assert_eq!(simulate(options, "dduwowngf"), "ddu+o+`ng");
    assert_eq!(simulate(options, "hoaf"), "ho`a");
}

#[test]
fn bk_hcm1_follows_the_vni_layout() {
    let hcm = with_charset(Charset::BkHcm1, InputMethod::Telex);
    let vni = with_charset(Charset::VniWindows, InputMethod::Telex);
    for input in ["tieengs", "Vieetj", "dduwowngf"] {
        assert_eq!(simulate(hcm, input), simulate(vni, input), "input = {input}");
    }
}

#[test]
fn every_charset_covers_all_134_letters() {
    for charset in Charset::ALL {
        for (kind, _) in vkb_core::letters::BASES.iter().enumerate() {
            for tone in 0..6 {
                for table in [&vkb_core::letters::LOWER, &vkb_core::letters::UPPER] {
                    let ch = table[kind][tone];
                    if ch == '\u{0}' {
                        continue;
                    }
                    let bytes = charset.encode(&ch.to_string());
                    assert!(
                        !bytes.is_empty(),
                        "{charset:?} has no code for {ch} (kind {kind} tone {tone})"
                    );
                }
            }
        }
    }
}

#[test]
fn table_status_is_recorded_for_every_charset() {
    let expected = [
        (Charset::Unicode, TableStatus::Verified),
        (Charset::UnicodeNfd, TableStatus::Verified),
        (Charset::Utf8, TableStatus::Verified),
        (Charset::Viscii, TableStatus::Verified),
        (Charset::Viqr, TableStatus::Verified),
        (Charset::Tcvn3, TableStatus::Documented),
        (Charset::Vps, TableStatus::Documented),
        (Charset::VniWindows, TableStatus::Documented),
        (Charset::BkHcm1, TableStatus::Unverified),
    ];
    for (charset, status) in expected {
        assert_eq!(charset.status(), status, "{charset:?}");
        assert!(!charset.status_note().is_empty(), "{charset:?} needs a note");
    }
}

#[test]
fn unverified_charset_says_so_in_its_note() {
    assert!(Charset::BkHcm1
        .status_note()
        .to_uppercase()
        .contains("UNVERIFIED"));
}

#[test]
fn ascii_passes_through_every_charset() {
    for charset in Charset::ALL {
        let bytes = charset.encode("abc 123");
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("abc"), "{charset:?}");
        assert!(text.contains("123"), "{charset:?}");
    }
}
