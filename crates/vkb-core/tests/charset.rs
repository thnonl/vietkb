//! Charset output: Unicode NFC and NFD through the engine, and table coverage.

use vkb_core::{simulate, Charset, EngineOptions, InputMethod};

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
fn ascii_passes_through_every_charset() {
    for charset in Charset::ALL {
        let bytes = charset.encode("abc 123");
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("abc"), "{charset:?}");
        assert!(text.contains("123"), "{charset:?}");
    }
}
