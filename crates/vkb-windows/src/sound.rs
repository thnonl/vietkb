//! Toggle sounds use the current Windows sound scheme.

use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_NODEFAULT};

/// Play an enabled/disabled Windows sound-scheme alias asynchronously.
///
/// Volume and character follow the user's scheme; when the scheme has no such
/// alias the call fails and the toggle stays silent instead of making a tone.
pub fn play_toggle(enabled: bool) {
    std::thread::spawn(move || {
        let alias = if enabled { "SystemAsterisk" } else { "SystemHand" };
        let alias: Vec<u16> = alias.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let _ = PlaySoundW(
                PCWSTR(alias.as_ptr()),
                None,
                SND_ALIAS | SND_ASYNC | SND_NODEFAULT,
            );
        }
    });
}

