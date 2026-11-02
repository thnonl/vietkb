//! IME awareness (mirror of `vkb-windows/src/ime.rs`).
//!
//! While another input method owns the keyboard (Apple Vietnamese Telex/VNI,
//! Japanese, Chinese, Korean, any third-party IME), VietKB's replacements would
//! fight the composition, so the tap stands aside and passes every key through.
//!
//! macOS has no per-window "is composing" query, so the unit is the *selected
//! input source*: a plain keyboard layout (ABC, US, AZERTY, Dvorak...) keeps
//! VietKB active, an input method does not.
//!
//! Cost model: the keystroke path reads one relaxed atomic ([`composing`]). The
//! TIS API is touched only off that path (`sys::refresh_input_source`, on the
//! main queue), on the system's input-source-changed notification, at start and
//! as a rate-limited safety net. This file holds the pure classification and
//! the cache, so both compile and are tested on every platform.

use std::sync::atomic::{AtomicBool, Ordering};

/// `kTISCategoryKeyboardInputSource`: layouts and input methods alike. Palettes
/// (emoji picker) and ink sources use other categories and never compose text.
pub const CATEGORY_KEYBOARD: &str = "TISCategoryKeyboardInputSource";

/// `kTISTypeKeyboardLayout`: a plain layout, never an IME.
pub const TYPE_LAYOUT: &str = "TISTypeKeyboardLayout";
/// A mode of an input method (Telex of Apple Vietnamese, Hiragana of Kotoeri...).
pub const TYPE_INPUT_MODE: &str = "TISTypeKeyboardInputMode";
/// An input method that has no modes.
pub const TYPE_METHOD_WITHOUT_MODES: &str = "TISTypeKeyboardInputMethodWithoutModes";
/// The parent entry of an input method that has modes.
pub const TYPE_METHOD_MODE_ENABLED: &str = "TISTypeKeyboardInputMethodModeEnabled";

/// Pure rule: does the selected input source compose text on its own?
///
/// `id` is `kTISPropertyInputSourceID`, `kind` is `kTISPropertyInputSourceType`
/// and `category` is `kTISPropertyInputSourceCategory`; any may be empty when
/// the lookup failed. Unknown means "not an IME": VietKB stays active, which is
/// the behaviour from before this guard existed.
pub fn is_composing_source(id: &str, kind: &str, category: &str) -> bool {
    if !category.is_empty() && category != CATEGORY_KEYBOARD {
        return false;
    }
    if id.starts_with("com.apple.keylayout.") {
        return false;
    }
    match kind {
        TYPE_INPUT_MODE | TYPE_METHOD_WITHOUT_MODES | TYPE_METHOD_MODE_ENABLED => true,
        TYPE_LAYOUT => false,
        // Type unreadable: fall back to the id convention of Apple's own
        // input methods. A type this table does not know is not an IME.
        "" => id.starts_with("com.apple.inputmethod."),
        _ => false,
    }
}

static COMPOSING: AtomicBool = AtomicBool::new(false);

/// `true` while the selected input source is an input method. One relaxed load.
#[inline]
pub fn composing() -> bool {
    COMPOSING.load(Ordering::Relaxed)
}

/// Store the latest classification. Returns `true` when it changed.
pub fn set_composing(value: bool) -> bool {
    COMPOSING.swap(value, Ordering::Relaxed) != value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_layouts_keep_vietkb_active() {
        for id in [
            "com.apple.keylayout.ABC",
            "com.apple.keylayout.US",
            "com.apple.keylayout.French",
            "com.apple.keylayout.Dvorak",
            "com.apple.keylayout.Vietnamese",
        ] {
            assert!(!is_composing_source(id, TYPE_LAYOUT, CATEGORY_KEYBOARD), "{id}");
        }
    }

    #[test]
    fn input_methods_and_their_modes_stand_vietkb_down() {
        let cases = [
            ("com.apple.inputmethod.VietnameseIM.Telex", TYPE_INPUT_MODE),
            ("com.apple.inputmethod.VietnameseIM.VNI", TYPE_INPUT_MODE),
            ("com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese", TYPE_INPUT_MODE),
            ("com.apple.inputmethod.SCIM.ITABC", TYPE_INPUT_MODE),
            ("com.apple.inputmethod.Korean.2SetKorean", TYPE_INPUT_MODE),
            ("com.google.inputmethod.Japanese.base", TYPE_INPUT_MODE),
            ("com.example.inputmethod.Plain", TYPE_METHOD_WITHOUT_MODES),
            ("com.apple.inputmethod.Kotoeri", TYPE_METHOD_MODE_ENABLED),
        ];
        for (id, kind) in cases {
            assert!(is_composing_source(id, kind, CATEGORY_KEYBOARD), "{id}");
        }
    }

    #[test]
    fn palettes_and_other_categories_never_count() {
        assert!(!is_composing_source(
            "com.apple.CharacterPaletteIM",
            "TISTypeCharacterPalette",
            "TISCategoryPaletteInputSource"
        ));
        assert!(!is_composing_source(
            "com.apple.inputmethod.ink",
            TYPE_INPUT_MODE,
            "TISCategoryInkInputSource"
        ));
    }

    #[test]
    fn an_unreadable_type_falls_back_to_the_id_convention() {
        assert!(is_composing_source("com.apple.inputmethod.VietnameseIM", "", ""));
        assert!(!is_composing_source("com.apple.keylayout.ABC", "", ""));
        assert!(!is_composing_source("", "", ""));
        assert!(!is_composing_source("com.vendor.unknown", "TISTypeSomethingNew", ""));
    }

    #[test]
    fn a_keylayout_id_wins_over_a_surprising_type() {
        assert!(!is_composing_source(
            "com.apple.keylayout.ABC",
            TYPE_INPUT_MODE,
            CATEGORY_KEYBOARD
        ));
    }

    #[test]
    fn the_cache_reports_changes_only() {
        let _guard = crate::globals::TEST_LOCK.lock();
        set_composing(false);
        assert!(set_composing(true));
        assert!(composing());
        assert!(!set_composing(true));
        assert!(set_composing(false));
        assert!(!composing());
    }
}
