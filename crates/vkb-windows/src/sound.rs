//! Short synthesized toggle sounds. No file assets.

use std::f64::consts::TAU;

use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{PlaySoundW, SND_MEMORY, SND_NODEFAULT};

const SAMPLE_RATE: u32 = 22_050;
const ON_PITCH_HZ: f64 = 880.0;
const OFF_PITCH_HZ: f64 = 520.0;
const DURATION_MS: u32 = 70;

fn wav_bytes(pitch_hz: f64, ms: u32) -> Vec<u8> {
    let samples = (SAMPLE_RATE * ms / 1000) as usize;
    let data_len = samples * 2;
    let mut wav = Vec::with_capacity(44 + data_len);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36u32 + data_len as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_len as u32).to_le_bytes());

    for i in 0..samples {
        let t = i as f64 / SAMPLE_RATE as f64;
        let phase = if pitch_hz < 700.0 {
            TAU * pitch_hz * t * (1.0 - 0.2 * t / (ms as f64 / 1000.0))
        } else {
            TAU * pitch_hz * t * (1.0 + 0.2 * t / (ms as f64 / 1000.0))
        };
        let edge = (samples as f64 * 0.08).max(1.0);
        let envelope = (i as f64 / edge).min(1.0).min(((samples - 1 - i) as f64 / edge).min(1.0));
        let value = (phase.sin() * envelope * 5_500.0).round() as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

/// Play an enabled/disabled sound asynchronously; hook caller never waits.
pub fn play_toggle(enabled: bool) {
    let pitch = if enabled { ON_PITCH_HZ } else { OFF_PITCH_HZ };
    std::thread::spawn(move || {
        let wav = wav_bytes(pitch, DURATION_MS);
        unsafe {
            let _ = PlaySoundW(PCWSTR(wav.as_ptr().cast()), None, SND_MEMORY | SND_NODEFAULT);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_fields_and_data_length_are_valid() {
        let wav = wav_bytes(ON_PITCH_HZ, DURATION_MS);
        let samples = (SAMPLE_RATE * DURATION_MS / 1000) as usize;
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize, wav.len() - 8);
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(wav[16..20].try_into().unwrap()), 16);
        assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), SAMPLE_RATE);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize, samples * 2);
        assert_eq!(wav.len(), 44 + samples * 2);
    }

    #[test]
    fn on_and_off_buffers_differ() {
        assert_ne!(wav_bytes(ON_PITCH_HZ, DURATION_MS), wav_bytes(OFF_PITCH_HZ, DURATION_MS));
    }

    #[test]
    fn peak_amplitude_fits_signed_16_bit_range() {
        let wav = wav_bytes(ON_PITCH_HZ, DURATION_MS);
        let peak = wav[44..].chunks_exact(2).map(|s| i16::from_le_bytes([s[0], s[1]]).unsigned_abs()).max().unwrap();
        assert!(peak > 0 && peak <= i16::MAX as u16);
    }
}
