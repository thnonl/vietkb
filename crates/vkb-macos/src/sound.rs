//! Vietnamese on/off cue: the same embedded `go_moc` woodblock pair the Windows
//! build ships (`assets/sounds/go_moc/{on,off}.wav`), played through `NSSound`.
//!
//! Every toggle route calls [`play_toggle`] with the **resulting** state. Playback
//! happens on one dedicated sound thread (never the tap thread); a failure is silence and
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

#[cfg(target_os = "macos")]
enum Job {
    Toggle(bool),
    Beep,
}

/// One long-lived sound thread: `NSSound` objects are created and played from
/// the same thread, and a toggle never spawns anything.
#[cfg(target_os = "macos")]
fn sender() -> &'static parking_lot::Mutex<std::sync::mpsc::Sender<Job>> {
    use std::sync::OnceLock;
    static SENDER: OnceLock<parking_lot::Mutex<std::sync::mpsc::Sender<Job>>> = OnceLock::new();
    SENDER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<Job>();
        let _ = std::thread::Builder::new()
            .name("vkb-sound".into())
            .spawn(move || {
                while let Ok(job) = rx.recv() {
                    match job {
                        Job::Toggle(enabled) => {
                            if !crate::sys::play_wav(enabled, wav_for(enabled)) {
                                log_failure_once();
                            }
                        }
                        Job::Beep => crate::sys::beep(),
                    }
                }
            });
        parking_lot::Mutex::new(tx)
    })
}

/// Play the cue for the state that was just set (queued; never blocks).
#[cfg(target_os = "macos")]
pub fn play_toggle(enabled: bool) {
    let _ = sender().lock().send(Job::Toggle(enabled));
}

/// `Injection::Beep` signals *error*: the system alert, never the toggle pair.
#[cfg(target_os = "macos")]
pub fn play_error() {
    let _ = sender().lock().send(Job::Beep);
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
