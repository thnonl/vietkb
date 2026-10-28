//! Notification sound. No file assets: the shell's own beep.

use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONASTERISK;

/// Play the enabled/disabled notification. Never fails the caller.
///
/// `MessageBeep` plays the sound the user's scheme assigns to "asterisk"; if the
/// scheme has none assigned the call is a no-op, which is what EVKey users
/// expect from a machine with sound effects turned off.
pub fn play_notification_sound() {
    let _ = unsafe { MessageBeep(MB_ICONASTERISK) };
}
