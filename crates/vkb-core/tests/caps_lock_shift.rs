//! Owner ruling (2026-11-06): with Caps Lock on, Shift on the first letter makes
//! it small on Windows (`Tieengs` gives the keys `tIEENGS`). "Nên ra tIẾNG vì
//! không có từ tiếng anh nào là tIEENGS cả." A capital Caps Lock made is not the
//! inner capital of a code word (`iPhone`, `getValue`), which keeps its keys.

use vkb_core::{apply_action, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

/// Type `keys`; an upper-case letter is typed under Caps Lock, a lower-case one
/// with Shift (as Windows gives them), digits and space plain. `caps` false types
/// the same characters with Caps Lock off (a capital is then Shift).
fn typed(method: InputMethod, keys: &str, caps: bool) -> String {
    let mut engine = Engine::new(EngineOptions { method, ..EngineOptions::default() });
    let mut out = String::new();
    for ch in keys.chars() {
        let mut key = KeyInput::char(ch);
        key.caps_lock = caps;
        key.modifiers.shift = if caps { ch.is_ascii_lowercase() } else { ch.is_ascii_uppercase() };
        let action = engine.process_key(key);
        apply_action(&mut out, action, key);
    }
    out
}

#[test]
fn caps_lock_with_shift_on_the_first_letter_composes() {
    assert_eq!(typed(InputMethod::Telex, "tIEENGS ", true), "tIẾNG ");
    assert_eq!(typed(InputMethod::Telex, "vIEETJ ", true), "vIỆT ");
    assert_eq!(typed(InputMethod::Telex, "nAMF ", true), "nÀM ");
    assert_eq!(typed(InputMethod::Vni, "tIE6NG1 ", true), "tIẾNG ");
    assert_eq!(typed(InputMethod::Vni, "nA2M ", true), "nÀM ");
}

#[test]
fn deliberate_inner_capitals_still_keep_their_keys() {
    assert_eq!(typed(InputMethod::Telex, "iPhone ", false), "iPhone ");
    assert_eq!(typed(InputMethod::Telex, "getValue ", false), "getValue ");
    assert_eq!(typed(InputMethod::Telex, "tIEENGS ", false), "tIEENGS ");
}
