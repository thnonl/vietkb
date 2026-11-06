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

/// The keyboard layout of the window in front (`HKL` as an integer), or 0 when
/// it cannot be read. Unit tests and the zero-focus harness read a US layout.
#[cfg(not(any(test, feature = "e2e-harness")))]
pub fn foreground_layout(hwnd: isize) -> isize {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
    if hwnd == 0 {
        return 0;
    }
    let thread = unsafe { GetWindowThreadProcessId(HWND(hwnd as _), None) };
    unsafe { GetKeyboardLayout(thread) }.0 as isize
}

#[cfg(test)]
thread_local! {
    /// What [`foreground_layout`] reports in unit tests.
    pub static TEST_LAYOUT: std::cell::Cell<isize> = const { std::cell::Cell::new(US_LAYOUT) };
}

#[cfg(test)]
pub fn foreground_layout(_hwnd: isize) -> isize {
    TEST_LAYOUT.with(|layout| layout.get())
}

#[cfg(feature = "e2e-harness")]
#[cfg(not(test))]
pub fn foreground_layout(_hwnd: isize) -> isize {
    US_LAYOUT
}

/// English (US) language with the US keyboard.
pub const US_LAYOUT: isize = 0x0409_0409;

/// The physical layout is US (the high word), whatever the language: Vietnamese
/// with the US keyboard is `0x0409042A`. Unknown (0) counts as US, as before.
pub fn is_us_layout(hkl: isize) -> bool {
    hkl == 0 || (hkl as usize >> 16) & 0xFFFF == 0x0409
}

/// The character `vk` types on layout `hkl`, without touching the thread's
/// dead-key state (flag 4, Windows 10 1607+).
#[cfg(not(test))]
pub fn to_unicode_on(vk: u16, scan: u16, state: &[u8; 256], hkl: isize) -> Option<char> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{ToUnicodeEx, HKL};
    let mut buffer = [0u16; 8];
    let length = unsafe { ToUnicodeEx(vk as u32, scan as u32, state, &mut buffer, 4, HKL(hkl as _)) };
    if length != 1 {
        return None;
    }
    char::from_u32(buffer[0] as u32).filter(|ch| !ch.is_control())
}

#[cfg(test)]
thread_local! {
    /// What [`to_unicode_on`] types in unit tests for a non-US layout.
    pub static TEST_LAYOUT_CHAR: std::cell::Cell<Option<char>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub fn to_unicode_on(_vk: u16, _scan: u16, _state: &[u8; 256], _hkl: isize) -> Option<char> {
    TEST_LAYOUT_CHAR.with(|ch| ch.get())
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
