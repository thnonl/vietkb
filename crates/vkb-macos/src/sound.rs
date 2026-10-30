//! Vietnamese on/off cue: the same embedded `go_moc` woodblock pair the Windows
//! build ships (`assets/sounds/go_moc/{on,off}.wav`), played through `NSSound`.
//!
//! Every toggle route calls [`play_toggle`] with the **resulting** state. Playback
//! happens on a spawned thread (never the tap thread); a failure is silence and
//! at most one `debug` line per session. `Injection::Beep` keeps the system
//! alert via [`play_error`] (`NSBeep`).

use std::sync::atomic::{AtomicBool, Ordering};

/// The one shipped pair. `include_bytes!` resolves at build time.
pub static ON: &[u8] = include_bytes!("../../../assets/sounds/go_moc/on.wav");
pub static OFF: &[u8] = include_bytes!("../../../assets/sounds/go_moc/off.wav");

/// The embedded cue for the state that was just set.
pub fn wav_for(enabled: bool) -> &'static [u8] {
    if enabled {
        ON
    } else {
        OFF
    }
}

static FAILURE_LOGGED: AtomicBool = AtomicBool::new(false);

fn log_failure_once() {
    if !FAILURE_LOGGED.swap(true, Ordering::Relaxed) {
        tracing::debug!("toggle sound could not be played");
    }
}

/// Play the cue for the state that was just set.
#[cfg(target_os = "macos")]
pub fn play_toggle(enabled: bool) {
    let wav = wav_for(enabled);
    std::thread::spawn(move || {
        if !crate::sys::play_wav(enabled, wav) {
            log_failure_once();
        }
    });
}

/// `Injection::Beep` signals *error*: the system alert, never the toggle pair.
#[cfg(target_os = "macos")]
pub fn play_error() {
    std::thread::spawn(crate::sys::beep);
}

#[cfg(not(target_os = "macos"))]
pub fn play_toggle(_enabled: bool) {
    log_failure_once();
}

#[cfg(not(target_os = "macos"))]
pub fn play_error() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn le32(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    }

    #[test]
    fn embedded_cues_are_distinct_riff_wave_files() {
        for wav in [ON, OFF] {
            assert!(wav.len() > 44, "truncated WAV");
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
            assert_eq!(le32(wav, 4) as usize + 8, wav.len(), "RIFF length matches the file");
        }
        assert_ne!(ON, OFF, "ON and OFF must never sound the same");
    }

    #[test]
    fn the_cue_follows_the_resulting_state() {
        assert_eq!(wav_for(true), ON);
        assert_eq!(wav_for(false), OFF);
    }
}
