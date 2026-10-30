//! IME awareness.
//!
//! When an IME is composing (Japanese/Chinese/Korean input, or a Vietnamese IME
//! the user launched), our replacements would fight the composition, so the hook
//! stays out of the way. The check is throttled by the caller — the IMM calls are
//! cheap but not free.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR,
};
use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicode;

fn hwnd_from(raw: isize) -> HWND {
    HWND(raw as _)
}

/// `true` when an IME has an active composition in `hwnd`.
pub fn composition_active(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }
    let window = hwnd_from(hwnd);
    unsafe {
        let context = ImmGetContext(window);
        if context.0 as isize == 0 {
            return false;
        }
        let length = ImmGetCompositionStringW(context, GCS_COMPSTR, None, 0);
        let _ = ImmReleaseContext(window, context);
        length > 0
    }
}

/// Resolve a character with the OS translator as a general fallback.
///
/// `ToUnicode` consults the *thread's* active layout; the common path uses the
/// pure US table in [`crate::keys`].
pub fn to_unicode(vk: u16, scan: u16, state: &[u8; 256]) -> Option<char> {
    let mut buffer = [0u16; 8];
    let length = unsafe { ToUnicode(vk as u32, scan as u32, Some(state), &mut buffer, 0) };
    if length != 1 {
        // -1 = dead key (its state must survive untouched), 0/2+ = nothing usable.
        return None;
    }
    char::from_u32(buffer[0] as u32).filter(|ch| !ch.is_control())
}

/// Keyboard state array for `ToUnicode`.
pub fn keyboard_state(mods: &crate::keys::ModifierState) -> [u8; 256] {
    let mut state = [0u8; 256];
    if mods.shift {
        state[crate::keys::VK_SHIFT as usize] = 0x80;
    }
    if mods.ctrl {
        state[crate::keys::VK_CONTROL as usize] = 0x80;
    }
    if mods.alt {
        state[crate::keys::VK_MENU as usize] = 0x80;
    }
    if mods.caps_lock {
        state[crate::keys::VK_CAPITAL as usize] = 0x01;
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::ModifierState;

    #[test]
    fn no_window_means_no_composition() {
        assert!(!composition_active(0));
    }

    #[test]
    fn keyboard_state_reflects_modifiers() {
        let mods = ModifierState {
            shift: true,
            ctrl: true,
            alt: true,
            win: false,
            caps_lock: true,
            ..ModifierState::default()
        };
        let state = keyboard_state(&mods);
        assert_eq!(state[crate::keys::VK_SHIFT as usize], 0x80);
        assert_eq!(state[crate::keys::VK_CONTROL as usize], 0x80);
        assert_eq!(state[crate::keys::VK_MENU as usize], 0x80);
        assert_eq!(state[crate::keys::VK_CAPITAL as usize], 0x01);
        assert_eq!(state[0x41], 0, "letters are not modifiers");
    }
}
