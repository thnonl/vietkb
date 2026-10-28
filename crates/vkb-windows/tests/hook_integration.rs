//! Real-hook integration test.
//!
//! It installs a global `WH_KEYBOARD_LL` hook and injects one untagged `F24`,
//! so it is `#[ignore]` by default:
//!
//! ```text
//! cargo test -p vkb-windows -- --ignored --nocapture
//! ```
//!
//! Manual steps and the preconditions are in
//! `docs/evkey-clone-work/70-impl-notes-windows.md` §"Manual verification".
//! Run it on an interactive desktop (not a service session): a headless session
//! never delivers hook events.

use std::sync::mpsc;
use std::time::Duration;

use vkb_config::{Hotkey, HotkeySet};
use vkb_platform::Hotkey as Action;

/// `VK_F24`.
const VK_F24: u16 = 0x87;

fn only_f24() -> HotkeySet {
    let mut set = HotkeySet::default();
    // Bind exactly one action so nothing else can fire during the test.
    set.enable = None;
    set.disable = None;
    set.toggle = None;
    set.restore_word = None;
    set.app_exclude = None;
    set.convert_tool = None;
    set.macro_panel = Hotkey::parse("F24").ok();
    set.macro_toggle = None;
    set.open_settings = None;
    set
}

#[test]
#[ignore = "installs a global keyboard hook; run manually on an interactive desktop"]
fn hook_installs_fires_hotkey_and_uninstalls() {
    let (sender, receiver) = mpsc::channel();
    vkb_windows::install_hotkeys(&only_f24(), move |hotkey| {
        let _ = sender.send(hotkey);
    })
    .expect("hook install");

    assert!(vkb_windows::is_hook_ready(), "hook must report ready");

    // Untagged injection: the hook treats it exactly like a physical press.
    let injected = vkb_windows::input::send_untagged_key(VK_F24);
    assert_eq!(injected, 2, "SendInput rejected the synthetic key");

    let fired = receiver
        .recv_timeout(Duration::from_secs(3))
        .expect("F24 did not reach the hook");
    assert_eq!(fired, Action::MacroPanel);

    vkb_windows::uninstall_hook().expect("uninstall");
    assert!(!vkb_windows::is_hook_ready(), "hook must be gone");
}

#[test]
#[ignore = "installs a global keyboard hook; run manually on an interactive desktop"]
fn install_is_idempotent_and_stop_is_safe() {
    let set = only_f24();
    vkb_windows::install_hotkeys(&set, |_| {}).expect("first install");
    let ready = vkb_windows::is_hook_ready();
    vkb_windows::install_hotkeys(&set, |_| {}).expect("second install");
    assert_eq!(ready, vkb_windows::is_hook_ready(), "install must be idempotent");

    vkb_windows::uninstall_hook().expect("first uninstall");
    vkb_windows::uninstall_hook().expect("second uninstall is a no-op");
    assert!(!vkb_windows::is_hook_ready());
}

#[test]
fn enabled_flag_never_sticks() {
    // No hook needed: the flag is what the hook reads.
    vkb_windows::set_enabled(true);
    assert!(vkb_windows::is_enabled());
    vkb_windows::set_enabled(false);
    assert!(!vkb_windows::is_enabled());
    vkb_windows::set_enabled(true);
    assert!(vkb_windows::is_enabled(), "toggle back must restore state");
}
