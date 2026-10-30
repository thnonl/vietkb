//! Vietnamese on/off cue: one embedded woodblock pair (`go_moc`), nothing else.
//!
//! Every toggle route — hotkey, UI switch, tray — calls [`play_toggle`] with the
//! **resulting** state, so ON and OFF never sound the same. Playback is
//! `PlaySoundW` with `SND_MEMORY | SND_ASYNC | SND_NODEFAULT` on a spawned
//! thread: no file on disk, no system sound scheme, never the hook thread, and a
//! failure (no device, muted endpoint) is silence — never a panic and at most
//! one `debug` line per session. `Injection::Beep` is not a toggle and keeps the
//! OS error alias via [`play_error`].
//!
//! Spec: `docs/evkey-clone-work/22-sound-spec.md` §1–§5.

use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};

/// The one shipped pair. `include_bytes!` resolves at build time: a missing,
/// renamed or malformed path is a compile error, never runtime silence.
pub static ON: &[u8] = include_bytes!("../../../assets/sounds/go_moc/on.wav");
pub static OFF: &[u8] = include_bytes!("../../../assets/sounds/go_moc/off.wav");

/// `Injection::Beep` alias — the OS critical-stop sound, never a pack.
const ALIAS_OFF: &str = "SystemHand";

/// At most one `debug` line per session: the next silent toggle is then one log
/// lookup instead of a diagnosis (`D-SND-04`). Never once per keystroke.
static FAILURE_LOGGED: AtomicBool = AtomicBool::new(false);

/// The embedded cue for the state that was just set.
fn wav_for(enabled: bool) -> &'static [u8] {
    if enabled {
        ON
    } else {
        OFF
    }
}

fn log_failure_once() {
    if FAILURE_LOGGED.swap(true, Ordering::Relaxed) {
        return;
    }
    tracing::debug!("toggle sound could not be played");
}

/// `SND_ASYNC` returns immediately; a new toggle interrupts the previous sound
/// (no `SND_NOSTOP`), which a fast ON→OFF needs. `SND_NODEFAULT` keeps "nothing"
/// as the fallback instead of a random system default. The buffer is `'static`,
/// which is what makes `SND_MEMORY` safe.
fn play_memory(wav: &'static [u8]) -> bool {
    unsafe {
        PlaySoundW(
            PCWSTR(wav.as_ptr() as *const u16),
            None,
            SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
        )
        .as_bool()
    }
}

fn play_alias(alias: &str) -> bool {
    let wide: Vec<u16> = alias.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        PlaySoundW(
            PCWSTR(wide.as_ptr()),
            None,
            SND_ALIAS | SND_ASYNC | SND_NODEFAULT,
        )
        .as_bool()
    }
}

/// Play the cue for the state that was just set. Never on the hook thread: one
/// spawned thread per toggle, exactly as before.
pub fn play_toggle(enabled: bool) {
    let wav = wav_for(enabled);
    std::thread::spawn(move || {
        if !play_memory(wav) {
            log_failure_once();
        }
    });
}

/// `Injection::Beep` signals *error*, so it is deliberately asymmetric: the OS
/// critical-stop alias, not the Vietnamese-on pair. Failure is silence.
pub fn play_error() {
    std::thread::spawn(|| {
        let _ = play_alias(ALIAS_OFF);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- WAV parsing helpers (no audio device involved) ---------------------

    fn le16(bytes: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([bytes[at], bytes[at + 1]])
    }

    fn le32(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    }

    /// `fmt ` fields of a canonical 44-byte-header PCM WAV.
    #[derive(Debug)]
    struct Format {
        tag: u16,
        channels: u16,
        rate: u32,
        byte_rate: u32,
        block_align: u16,
        bits: u16,
        data_len: usize,
    }

    fn parse_wav(wav: &'static [u8]) -> Format {
        assert!(wav.len() > 44, "truncated WAV ({} B)", wav.len());
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ", "fmt must be the first chunk");
        assert_eq!(le32(wav, 16), 16, "fmt chunk must be 16 B (PCM)");
        assert_eq!(&wav[36..40], b"data", "data must follow fmt directly");
        Format {
            tag: le16(wav, 20),
            channels: le16(wav, 22),
            rate: le32(wav, 24),
            byte_rate: le32(wav, 28),
            block_align: le16(wav, 32),
            bits: le16(wav, 34),
            data_len: le32(wav, 40) as usize,
        }
    }

    fn pcm(wav: &'static [u8]) -> Vec<i16> {
        wav[44..]
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    }

    fn peak_dbfs(samples: &[i16]) -> f64 {
        let peak = samples.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0);
        20.0 * (peak as f64 / 32768.0).log10()
    }

    fn asset_files() -> [(&'static str, bool, &'static [u8]); 2] {
        [("go_moc", true, ON), ("go_moc", false, OFF)]
    }

    // -- 1. container -------------------------------------------------------

    #[test]
    fn packs_are_pcm16_22k_mono() {
        let files = asset_files();
        assert_eq!(files.len(), 2);
        for (slug, enabled, wav) in files {
            let fmt = parse_wav(wav);
            let label = format!("{slug}/{}", if enabled { "on" } else { "off" });
            assert_eq!(fmt.tag, 1, "{label}: PCM only");
            assert_eq!(fmt.channels, 1, "{label}: mono");
            assert_eq!(fmt.rate, 22_050, "{label}: rate");
            assert_eq!(fmt.bits, 16, "{label}: bit depth");
            assert_eq!(fmt.block_align, 2, "{label}: block align");
            assert_eq!(fmt.byte_rate, 44_100, "{label}: byte rate");
            assert_eq!(fmt.data_len % 2, 0, "{label}: odd data chunk");
            assert_eq!(
                fmt.data_len,
                wav.len() - 44,
                "{label}: data length must cover the file"
            );
            assert_eq!(le32(wav, 4) as usize, wav.len() - 8, "{label}: RIFF size");
        }
    }

    // -- 2. durations and levels vs §1/§2 ----------------------------------

    #[test]
    fn durations_and_peaks_match_spec() {
        // (on ms, off ms, on peak dBFS, off peak dBFS) — `22-sound-spec.md` §1.
        const SPEC: (usize, usize, f64, f64) = (100, 90, -18.0, -18.0);
        let (on_ms, off_ms, on_peak, off_peak) = SPEC;
        for (enabled, ms, nominal) in [(true, on_ms, on_peak), (false, off_ms, off_peak)] {
            let bytes = wav_for(enabled);
            let samples = pcm(bytes);
            let expected = (22.05 * ms as f64).round() as usize;
            assert!(
                samples.len().abs_diff(expected) <= 1,
                "go_moc/{}: {} samples, spec {expected} ±1",
                if enabled { "on" } else { "off" },
                samples.len()
            );
            let measured = peak_dbfs(&samples);
            assert!(
                (measured - nominal).abs() <= 1.0,
                "go_moc/{}: peak {measured:.2} dBFS, spec {nominal} ±1",
                if enabled { "on" } else { "off" }
            );
        }
    }

    // -- 3. hard ceiling, DC offset, fades ---------------------------------

    #[test]
    fn levels_dc_offset_and_fades() {
        for (slug, enabled, wav) in asset_files() {
            let label = format!("{slug}/{}", if enabled { "on" } else { "off" });
            let samples = pcm(wav);
            let peak = peak_dbfs(&samples);
            assert!(peak <= -6.0, "{label}: peak {peak:.2} dBFS over the ceiling");

            let mean = samples.iter().map(|s| *s as i64).sum::<i64>() / samples.len() as i64;
            assert!(mean.abs() <= 4, "{label}: DC offset {mean} LSB");

            assert_eq!(&samples[..2], &[0, 0], "{label}: must start at zero");
            assert_eq!(
                &samples[samples.len() - 2..],
                &[0, 0],
                "{label}: must end at zero"
            );

            // A linear ≥ 2 ms fade-in: the first 40 samples (< 2 ms) are
            // quieter than the loudest sample of the file.
            let head = samples[..40.min(samples.len())]
                .iter()
                .map(|s| (*s as i32).abs())
                .max()
                .unwrap_or(0);
            let overall = samples.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0);
            assert!(head < overall, "{label}: no fade-in");
        }
    }

    // -- 4. ON/OFF symmetry: the state selects the cue ---------------------

    #[test]
    fn on_and_off_are_a_distinct_pair() {
        assert!(std::ptr::eq(wav_for(true), ON), "ON must play on.wav");
        assert!(std::ptr::eq(wav_for(false), OFF), "OFF must play off.wav");
        assert_eq!((ON.len(), OFF.len()), (4_454, 4_014), "§2 sizes");
        assert_ne!(ON, OFF, "the pair must not be byte-identical");
        // ON is the higher pitch (§1): a higher zero-crossing count.
        fn zcr(samples: &[i16]) -> f64 {
            let crossings = samples
                .windows(2)
                .filter(|pair| pair[0] <= 0 && pair[1] > 0)
                .count();
            crossings as f64 / (samples.len() as f64 / 22_050.0)
        }
        let on_pcm = pcm(ON);
        let off_pcm = pcm(OFF);
        let on_head = zcr(&on_pcm[..660]);
        let off_head = zcr(&off_pcm[..660]);
        assert!(on_head > off_head * 1.2, "ON {on_head:.0} Hz vs OFF {off_head:.0} Hz");
    }
}
