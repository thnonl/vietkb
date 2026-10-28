//! `INPUT` array construction and delivery.
//!
//! [`build`] is pure: one [`EngineAction`] decision becomes exactly one array,
//! which is handed to a **single** `SendInput` call. For
//! `EngineAction::Replace` that means the target window sees all backspaces and
//! the replacement text in one atomic batch — no intermediate state, no flicker.
//!
//! Clipboard is never part of this path (charter C4); the opt-in fallback lives
//! in [`crate::clipboard`].

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

use crate::keys;
use vkb_platform::plan::Injection;

/// Stamped on every synthetic event so the hook can recognise its own input and
/// never feed it back into the engine.
pub const INJECTED_MAGIC: usize = 0x5649_454B; // "VIEK"

/// How text is delivered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendMode {
    /// `KEYEVENTF_UNICODE`: layout-independent, the default.
    Unicode,
    /// `send_standard` (config): virtual key + scancode, for apps that ignore
    /// Unicode events (some games, legacy VB/Delphi controls).
    Standard,
}

/// Build the array for one injection. Empty for `None`/`Beep`.
pub fn build(injection: &Injection, mode: SendMode) -> Vec<INPUT> {
    match injection {
        Injection::None | Injection::Beep => Vec::new(),
        Injection::Type(text) => build_segment(0, text, mode),
        Injection::Replace { deletes, insert } => build_segment(*deletes, insert, mode),
    }
}

/// Backspaces + text in one array, so `Replace` is delivered atomically.
pub fn build_segment(deletes: u32, text: &str, mode: SendMode) -> Vec<INPUT> {
    let mut out = Vec::with_capacity((deletes as usize) * 2 + text.encode_utf16().count() * 2);
    for _ in 0..deletes {
        push_backspace(&mut out, mode);
    }
    push_text(&mut out, text, mode);
    out
}

fn push_backspace(out: &mut Vec<INPUT>, mode: SendMode) {
    let scan = match mode {
        SendMode::Unicode => 0,
        SendMode::Standard => keys::scan_for_vk(keys::VK_BACK),
    };
    push_key(out, keys::VK_BACK, scan, false);
    push_key(out, keys::VK_BACK, scan, true);
}

fn push_text(out: &mut Vec<INPUT>, text: &str, mode: SendMode) {
    match mode {
        SendMode::Unicode => {
            for unit in text.encode_utf16() {
                push_unicode(out, unit, false);
                push_unicode(out, unit, true);
            }
        }
        SendMode::Standard => {
            for ch in text.chars() {
                match keys::us_vk_for_char(ch) {
                    Some((vk, shift)) => {
                        if shift {
                            push_key(out, keys::VK_SHIFT, keys::scan_for_vk(keys::VK_SHIFT), false);
                        }
                        let scan = keys::scan_for_vk(vk);
                        push_key(out, vk, scan, false);
                        push_key(out, vk, scan, true);
                        if shift {
                            push_key(out, keys::VK_SHIFT, keys::scan_for_vk(keys::VK_SHIFT), true);
                        }
                    }
                    // No key produces this character on a US layout (Vietnamese
                    // diacritics): fall back to the Unicode event so the text is
                    // never lost.
                    None => {
                        let mut buf = [0u16; 2];
                        for unit in ch.encode_utf16(&mut buf).iter().copied() {
                            push_unicode(out, unit, false);
                            push_unicode(out, unit, true);
                        }
                    }
                }
            }
        }
    }
}

fn push_key(out: &mut Vec<INPUT>, vk: u16, scan: u16, up: bool) {
    push_key_tagged(out, vk, scan, up, INJECTED_MAGIC);
}

fn push_key_tagged(out: &mut Vec<INPUT>, vk: u16, scan: u16, up: bool, extra: usize) {
    let flags = if up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    out.push(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: extra,
            },
        },
    });
}

fn push_unicode(out: &mut Vec<INPUT>, unit: u16, up: bool) {
    let mut flags = KEYEVENTF_UNICODE;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    out.push(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INJECTED_MAGIC,
            },
        },
    });
}

/// Deliver the batch in one call. Returns the number of events the OS accepted
/// (fewer than `inputs.len()` means the target blocked input).
pub fn send(inputs: &[INPUT]) -> u32 {
    if inputs.is_empty() {
        return 0;
    }
    unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) }
}

/// `Ctrl+V` with scancodes. Only the opt-in clipboard fallback uses this.
pub fn ctrl_v_sequence() -> Vec<INPUT> {
    let mut out = Vec::with_capacity(4);
    let ctrl_scan = keys::scan_for_vk(keys::VK_CONTROL);
    let v_scan = keys::scan_for_vk(0x56);
    push_key(&mut out, keys::VK_CONTROL, ctrl_scan, false);
    push_key(&mut out, 0x56, v_scan, false);
    push_key(&mut out, 0x56, v_scan, true);
    push_key(&mut out, keys::VK_CONTROL, ctrl_scan, true);
    out
}

/// Inject one untagged press+release so the hook treats it as real input.
///
/// Only the `#[ignore]`d integration test uses this: every production injection
/// carries [`INJECTED_MAGIC`].
pub fn send_untagged_key(vk: u16) -> u32 {
    let mut out = Vec::with_capacity(2);
    let scan = keys::scan_for_vk(vk);
    push_key_tagged(&mut out, vk, scan, false, 0);
    push_key_tagged(&mut out, vk, scan, true, 0);
    send(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bake(injection: &Injection, mode: SendMode) -> Vec<INPUT> {
        build(injection, mode)
    }

    /// (vk, scan, flags, extra_info) of one event.
    fn field(input: &INPUT) -> (u16, u16, u32, usize) {
        let ki = unsafe { input.Anonymous.ki };
        (ki.wVk.0, ki.wScan, ki.dwFlags.0, ki.dwExtraInfo)
    }

    fn is_up(input: &INPUT) -> bool {
        field(input).2 & KEYEVENTF_KEYUP.0 != 0
    }

    fn unicode_seqs(inputs: &[INPUT]) -> Vec<(u16, bool)> {
        inputs
            .iter()
            .map(|input| {
                let ki = unsafe { input.Anonymous.ki };
                (
                    ki.wScan,
                    ki.dwFlags.0 & KEYEVENTF_KEYUP.0 != 0,
                )
            })
            .collect()
    }

    #[test]
    fn nothing_is_built_for_noop_and_beep() {
        assert!(bake(&Injection::None, SendMode::Unicode).is_empty());
        assert!(bake(&Injection::Beep, SendMode::Unicode).is_empty());
    }

    #[test]
    fn replace_is_one_batch_backspaces_then_text() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 2,
                insert: "á".to_string(),
            },
            SendMode::Unicode,
        );
        assert_eq!(inputs.len(), 2 * 2 + 2);
        // 0..4 backspace down/up pairs, then the unicode pair.
        for input in &inputs[..4] {
            assert_eq!(field(input).0, keys::VK_BACK);
        }
        assert!(!is_up(&inputs[0]), "first event is the key-down");
        assert!(is_up(&inputs[1]));
        let seqs = unicode_seqs(&inputs[4..]);
        assert_eq!(seqs, vec![(0x00E1, false), (0x00E1, true)]);
    }

    #[test]
    fn every_event_is_tagged_as_ours() {
        let inputs = bake(&Injection::Type("Việt".to_string()), SendMode::Unicode);
        assert!(!inputs.is_empty());
        for input in &inputs {
            assert_eq!(field(input).3, INJECTED_MAGIC);
        }
    }

    #[test]
    fn unicode_mode_splits_surrogate_pairs() {
        // U+1F600 = D83D DE00 -> four events.
        let inputs = bake(&Injection::Type("😀".to_string()), SendMode::Unicode);
        assert_eq!(inputs.len(), 4);
        assert_eq!(unicode_seqs(&inputs), vec![(0xD83D, false), (0xD83D, true), (0xDE00, false), (0xDE00, true)]);
    }

    #[test]
    fn standard_mode_uses_scancodes_for_ascii() {
        let inputs = bake(&Injection::Type("a".to_string()), SendMode::Standard);
        assert_eq!(inputs.len(), 2, "one key down + one key up");
        let ki = unsafe { inputs[0].Anonymous.ki };
        assert_eq!(ki.wVk.0, 0x41);
        assert_eq!(ki.wScan, keys::scan_for_vk(0x41));
        assert_ne!(ki.wScan, 0, "standard mode must carry a scancode");
        assert_eq!(ki.dwFlags.0 & KEYEVENTF_UNICODE.0, 0);
    }

    #[test]
    fn standard_mode_wraps_uppercase_in_shift() {
        let inputs = bake(&Injection::Type("A".to_string()), SendMode::Standard);
        assert_eq!(inputs.len(), 4);
        assert_eq!(field(&inputs[0]).0, keys::VK_SHIFT);
        assert_eq!(field(&inputs[3]).0, keys::VK_SHIFT);
        assert!(is_up(&inputs[3]), "shift must be released last");
    }

    #[test]
    fn standard_mode_falls_back_to_unicode_for_diacritics() {
        let inputs = bake(&Injection::Type("Đ".to_string()), SendMode::Standard);
        assert_eq!(inputs.len(), 2);
        let ki = unsafe { inputs[0].Anonymous.ki };
        assert_eq!(ki.wVk.0, 0);
        assert_eq!(ki.dwFlags.0 & KEYEVENTF_UNICODE.0, KEYEVENTF_UNICODE.0);
    }

    #[test]
    fn standard_mode_backspace_has_scan_code() {
        let inputs = bake(
            &Injection::Replace {
                deletes: 1,
                insert: String::new(),
            },
            SendMode::Standard,
        );
        assert_eq!(inputs.len(), 2);
        assert_eq!(field(&inputs[0]).1, keys::scan_for_vk(keys::VK_BACK));
    }

    #[test]
    fn empty_send_is_a_no_op() {
        assert_eq!(send(&[]), 0);
    }

    #[test]
    fn ctrl_v_presses_and_releases_in_order() {
        let inputs = ctrl_v_sequence();
        assert_eq!(inputs.len(), 4);
        let keys: Vec<(u16, bool)> = inputs
            .iter()
            .map(|input| {
                let ki = unsafe { input.Anonymous.ki };
                (ki.wVk.0, ki.dwFlags.0 & KEYEVENTF_KEYUP.0 != 0)
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                (keys::VK_CONTROL, false),
                (0x56, false),
                (0x56, true),
                (keys::VK_CONTROL, true)
            ]
        );
    }
}
