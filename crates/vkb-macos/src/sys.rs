//! The Apple FFI layer: CGEventTap on a dedicated run-loop thread, event
//! injection, Accessibility, secure input, frontmost app, sounds.
//!
//! Raw `extern "C"` for the small CoreGraphics / CoreFoundation / Carbon
//! surface; `objc2` (already in the Tauri dependency tree) for NSWorkspace and
//! NSSound. All the decisions live in [`crate::state`]; this file only moves
//! data between the OS and that state machine.
//!
//! NOT exercised by the unit tests: creating a tap needs Accessibility and a
//! live session (`docs/evkey-clone-work/70-impl-notes-macos.md`).

use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU8, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use objc2::rc::{autoreleasepool, Allocated, Retained};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use parking_lot::Mutex;

use crate::globals;
use crate::inject::{self, Step};
use crate::keys::MAC_BACKSPACE;
use crate::state::{EventKind, Host, RawEvent, Runtime, MARKER};

type CFTypeRef = *const c_void;
type CGEventRef = *mut c_void;
type CGEventSourceRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFDictionaryRef = *const c_void;
type CFStringRef = *const c_void;

type TapCallback = extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef;

// CGEventType
const KEY_DOWN: u32 = 10;
const KEY_UP: u32 = 11;
const FLAGS_CHANGED: u32 = 12;
const LEFT_MOUSE_DOWN: u32 = 1;
const RIGHT_MOUSE_DOWN: u32 = 3;
const OTHER_MOUSE_DOWN: u32 = 25;
const TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
const TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;

// CGEventField
const FIELD_AUTOREPEAT: u32 = 8;
const FIELD_KEYCODE: u32 = 9;
const FIELD_SOURCE_USER_DATA: u32 = 42;

// CGEventTapLocation / Place / Options / source state
const SESSION_EVENT_TAP: u32 = 1;
const HID_EVENT_TAP: u32 = 0;
const HEAD_INSERT: u32 = 0;
const TAP_OPTION_DEFAULT: u32 = 0;
const SOURCE_STATE_PRIVATE: i32 = -1;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: TapCallback,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventTapIsEnabled(tap: CFMachPortRef) -> bool;
    fn CGEventTapPostEvent(proxy: CGEventTapProxy, event: CGEventRef);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_len: std::ffi::c_ulong,
        actual_len: *mut std::ffi::c_ulong,
        buffer: *mut u16,
    );
    fn CGEventKeyboardSetUnicodeString(
        event: CGEventRef,
        len: std::ffi::c_ulong,
        buffer: *const u16,
    );
    fn CGEventCreateKeyboardEvent(source: CGEventSourceRef, keycode: u16, down: bool)
        -> CGEventRef;
    fn CGEventSourceCreate(state: i32) -> CGEventSourceRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: CFRunLoopRef);
    fn CFRelease(cf: CFTypeRef);
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        count: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CFDictionaryRef;
    static kCFRunLoopCommonModes: CFStringRef;
    static kCFBooleanTrue: CFTypeRef;
    static kCFTypeDictionaryKeyCallBacks: u8;
    static kCFTypeDictionaryValueCallBacks: u8;
}

// `IsSecureEventInputEnabled` lives in HIToolbox, re-exported by Carbon.
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn IsSecureEventInputEnabled() -> bool;
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn NSBeep();
}

// -- Accessibility ----------------------------------------------------------

/// `AXIsProcessTrusted()`: may this process observe and filter key events?
pub fn has_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// `AXIsProcessTrustedWithOptions({prompt: true})`: shows the system prompt that
/// points the user to Privacy & Security -> Accessibility. Returns the current
/// trust state (the user answers asynchronously).
pub fn request_accessibility_permission() -> bool {
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::addr_of!(kCFTypeDictionaryKeyCallBacks).cast(),
            ptr::addr_of!(kCFTypeDictionaryValueCallBacks).cast(),
        );
        let trusted = AXIsProcessTrustedWithOptions(options);
        if !options.is_null() {
            CFRelease(options);
        }
        trusted
    }
}

/// `true` while any process holds secure event input (password fields).
pub fn secure_input_enabled() -> bool {
    unsafe { IsSecureEventInputEnabled() }
}

// -- Frontmost application --------------------------------------------------

fn with_frontmost<R>(f: impl FnOnce(*mut AnyObject) -> R) -> Option<R> {
    autoreleasepool(|_| unsafe {
        let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace.is_null() {
            return None;
        }
        let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
        if app.is_null() {
            return None;
        }
        Some(f(app))
    })
}

/// Pid of the frontmost application (0 when unknown). No allocation.
pub fn frontmost_pid() -> i32 {
    with_frontmost(|app| unsafe {
        let pid: i32 = msg_send![app, processIdentifier];
        pid
    })
    .unwrap_or(0)
}

/// Lowercased bundle identifier of the frontmost application.
pub fn active_app_bundle() -> Option<String> {
    with_frontmost(|app| unsafe {
        let bundle: *mut AnyObject = msg_send![app, bundleIdentifier];
        if bundle.is_null() {
            return None;
        }
        let utf8: *const c_char = msg_send![bundle, UTF8String];
        if utf8.is_null() {
            return None;
        }
        Some(CStr::from_ptr(utf8).to_string_lossy().to_ascii_lowercase())
    })
    .flatten()
}

// -- Sound ------------------------------------------------------------------

struct SendSound(Retained<AnyObject>);
// SAFETY: guarded by the surrounding Mutex; NSSound play/stop are only ever
// called while it is held.
unsafe impl Send for SendSound {}

static SOUNDS: Mutex<[Option<SendSound>; 2]> = Mutex::new([None, None]);

fn make_sound(wav: &'static [u8]) -> Option<SendSound> {
    autoreleasepool(|_| unsafe {
        let data: Option<Retained<AnyObject>> = msg_send![
            class!(NSData),
            dataWithBytes: wav.as_ptr().cast::<c_void>(),
            length: wav.len()
        ];
        let data = data?;
        let alloc: Allocated<AnyObject> = msg_send![class!(NSSound), alloc];
        let sound: Option<Retained<AnyObject>> = msg_send![alloc, initWithData: &*data];
        sound.map(SendSound)
    })
}

/// Play one embedded cue. `false` when the sound could not be created or played.
pub fn play_wav(enabled: bool, wav: &'static [u8]) -> bool {
    let slot = usize::from(enabled);
    let mut sounds = SOUNDS.lock();
    if sounds[slot].is_none() {
        sounds[slot] = make_sound(wav);
    }
    let Some(sound) = sounds[slot].as_ref() else {
        return false;
    };
    // Interrupt a still-playing cue so a fast ON -> OFF is heard.
    for other in sounds.iter().flatten() {
        let _: () = unsafe { msg_send![&*other.0, stop] };
    }
    let played: bool = unsafe { msg_send![&*sound.0, play] };
    played
}

pub fn beep() {
    unsafe { NSBeep() }
}

// -- Injection --------------------------------------------------------------

/// The proxy of the callback being served; events are posted through it so they
/// land right after the tap, ahead of any key the user typed meanwhile.
static PROXY: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static SOURCE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
/// 0 = unread, 1 = tap proxy (default), 2 = `CGEventPost(kCGHIDEventTap)`.
static POST_MODE: AtomicU8 = AtomicU8::new(0);

fn post_mode() -> u8 {
    let mode = POST_MODE.load(Ordering::Relaxed);
    if mode != 0 {
        return mode;
    }
    let mode = match std::env::var("VKB_MAC_POST").as_deref() {
        Ok("hid") => 2,
        _ => 1,
    };
    POST_MODE.store(mode, Ordering::Relaxed);
    mode
}

/// Private source: injected events do not inherit the physical modifier state.
fn event_source() -> CGEventSourceRef {
    let source = SOURCE.load(Ordering::Acquire);
    if !source.is_null() {
        return source;
    }
    let created = unsafe { CGEventSourceCreate(SOURCE_STATE_PRIVATE) };
    match SOURCE.compare_exchange(ptr::null_mut(), created, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => created,
        Err(existing) => {
            if !created.is_null() {
                unsafe { CFRelease(created) };
            }
            existing
        }
    }
}

/// Create, tag and post one key event. Returns `false` when it could not be made.
fn post_key(keycode: u16, down: bool, text: Option<&[u16]>) -> bool {
    unsafe {
        let event = CGEventCreateKeyboardEvent(event_source(), keycode, down);
        if event.is_null() {
            return false;
        }
        CGEventSetIntegerValueField(event, FIELD_SOURCE_USER_DATA, MARKER);
        // Explicit empty flags: whatever the user holds must not colour the text.
        CGEventSetFlags(event, 0);
        if let Some(units) = text {
            CGEventKeyboardSetUnicodeString(event, units.len() as std::ffi::c_ulong, units.as_ptr());
        }
        let proxy = PROXY.load(Ordering::Acquire);
        if !proxy.is_null() && post_mode() == 1 {
            CGEventTapPostEvent(proxy, event);
        } else if post_mode() == 2 {
            CGEventPost(HID_EVENT_TAP, event);
        } else {
            CGEventPost(SESSION_EVENT_TAP, event);
        }
        CFRelease(event.cast_const());
        true
    }
}

/// Delete `deletes` characters with real Backspace presses, then type `text`.
pub fn send_replace(deletes: u32, text: &str) {
    for step in inject::plan(deletes, text) {
        match step {
            Step::Backspace => {
                post_key(MAC_BACKSPACE, true, None);
                post_key(MAC_BACKSPACE, false, None);
            }
            Step::Text(units) => {
                // Virtual keycode 0 carries the string; the app reads the string.
                post_key(0, true, Some(&units));
                post_key(0, false, Some(&units));
            }
        }
    }
}

// -- Host -------------------------------------------------------------------

/// The production [`Host`]: everything straight from the OS.
pub struct MacHost;

impl Host for MacHost {
    fn frontmost_pid(&mut self) -> i32 {
        frontmost_pid()
    }
    fn frontmost_bundle(&mut self) -> Option<String> {
        active_app_bundle()
    }
    fn secure_input(&mut self) -> bool {
        secure_input_enabled()
    }
    fn replace(&mut self, deletes: u32, text: &str) {
        send_replace(deletes, text);
    }
    fn toggle_sound(&mut self, enabled: bool) {
        crate::sound::play_toggle(enabled);
    }
    fn error_sound(&mut self) {
        crate::sound::play_error();
    }
}

// -- The tap ----------------------------------------------------------------

struct TapContext {
    runtime: Runtime,
}

static TAP: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
/// Run loop of the tap thread, as a usize so the mutex is `Send`.
static RUN_LOOP: Mutex<usize> = Mutex::new(0);
static THREAD: Mutex<Option<std::thread::JoinHandle<()>>> = Mutex::new(None);
static TAP_BUSY: AtomicBool = AtomicBool::new(false);

fn read_event(event_type: u32, event: CGEventRef) -> Option<RawEvent> {
    unsafe {
        let kind = match event_type {
            KEY_DOWN => EventKind::KeyDown,
            KEY_UP => EventKind::KeyUp,
            FLAGS_CHANGED => EventKind::FlagsChanged,
            LEFT_MOUSE_DOWN | RIGHT_MOUSE_DOWN | OTHER_MOUSE_DOWN => EventKind::MouseDown,
            _ => return None,
        };
        let user_data = CGEventGetIntegerValueField(event, FIELD_SOURCE_USER_DATA);
        if kind == EventKind::MouseDown {
            return Some(RawEvent {
                kind,
                keycode: 0,
                flags: 0,
                autorepeat: false,
                typed: None,
                user_data,
            });
        }
        let keycode = CGEventGetIntegerValueField(event, FIELD_KEYCODE) as u16;
        let flags = CGEventGetFlags(event);
        let autorepeat = kind == EventKind::KeyDown
            && CGEventGetIntegerValueField(event, FIELD_AUTOREPEAT) != 0;
        let typed = if kind == EventKind::KeyDown {
            let mut buffer = [0u16; 8];
            let mut length: std::ffi::c_ulong = 0;
            CGEventKeyboardGetUnicodeString(
                event,
                buffer.len() as std::ffi::c_ulong,
                &mut length,
                buffer.as_mut_ptr(),
            );
            crate::keys::typed_char(&buffer[..(length as usize).min(buffer.len())])
        } else {
            None
        };
        Some(RawEvent {
            kind,
            keycode,
            flags,
            autorepeat,
            typed,
            user_data,
        })
    }
}

extern "C" fn tap_callback(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    refcon: *mut c_void,
) -> CGEventRef {
    if event_type == TAP_DISABLED_BY_TIMEOUT || event_type == TAP_DISABLED_BY_USER_INPUT {
        // The system turned the tap off (slow callback, or secure input toggled).
        // Turn it back on and forget half-seen state: events were lost meanwhile.
        let tap = TAP.load(Ordering::Acquire);
        if !tap.is_null() {
            unsafe { CGEventTapEnable(tap, true) };
        }
        if !refcon.is_null() {
            let ctx = unsafe { &mut *refcon.cast::<TapContext>() };
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ctx.runtime.reset()));
        }
        tracing::warn!(event_type, "event tap was disabled by the system; re-enabled");
        return event;
    }
    if refcon.is_null() || event.is_null() {
        return event;
    }
    let Some(raw) = read_event(event_type, event) else {
        return event;
    };
    let ctx = unsafe { &mut *refcon.cast::<TapContext>() };
    PROXY.store(proxy, Ordering::Release);
    let decision = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ctx.runtime.poll_config();
        ctx.runtime.on_event(&raw)
    }));
    PROXY.store(ptr::null_mut(), Ordering::Release);
    match decision {
        Ok(decision) if decision.swallow => ptr::null_mut(),
        _ => event,
    }
}

fn tap_thread(ready: mpsc::Sender<std::result::Result<(), String>>) {
    let cfg = globals::snapshot();
    let sink = globals::SINK.lock().take();
    let runtime = Runtime::new(Box::new(MacHost), cfg, sink);
    let ctx = Box::into_raw(Box::new(TapContext { runtime }));

    let mask: u64 = (1 << KEY_DOWN)
        | (1 << KEY_UP)
        | (1 << FLAGS_CHANGED)
        | (1 << LEFT_MOUSE_DOWN)
        | (1 << RIGHT_MOUSE_DOWN)
        | (1 << OTHER_MOUSE_DOWN);
    let tap = unsafe {
        CGEventTapCreate(
            SESSION_EVENT_TAP,
            HEAD_INSERT,
            TAP_OPTION_DEFAULT,
            mask,
            tap_callback,
            ctx.cast(),
        )
    };
    if tap.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
        let _ = ready.send(Err(
            "CGEventTapCreate failed: grant VietKB Accessibility in System Settings -> Privacy & Security".into(),
        ));
        return;
    }
    let source = unsafe { CFMachPortCreateRunLoopSource(ptr::null(), tap, 0) };
    if source.is_null() {
        unsafe { CFRelease(tap.cast_const()) };
        drop(unsafe { Box::from_raw(ctx) });
        let _ = ready.send(Err("CFMachPortCreateRunLoopSource failed".into()));
        return;
    }
    let run_loop = unsafe { CFRunLoopGetCurrent() };
    unsafe {
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
    }
    TAP.store(tap, Ordering::Release);
    *RUN_LOOP.lock() = run_loop as usize;
    globals::READY.store(true, Ordering::SeqCst);
    let _ = ready.send(Ok(()));

    if !globals::STOP.load(Ordering::SeqCst) {
        unsafe { CFRunLoopRun() };
    }

    globals::READY.store(false, Ordering::SeqCst);
    *RUN_LOOP.lock() = 0;
    TAP.store(ptr::null_mut(), Ordering::Release);
    unsafe {
        CGEventTapEnable(tap, false);
        CFRelease(source.cast_const());
        CFRelease(tap.cast_const());
        drop(Box::from_raw(ctx));
    }
}

/// Start the tap thread. Fails (never panics) without Accessibility or when the
/// tap cannot be created.
pub fn install() -> Result<()> {
    if globals::READY.load(Ordering::SeqCst) {
        return Ok(());
    }
    if TAP_BUSY.swap(true, Ordering::SeqCst) {
        bail!("event tap is already starting");
    }
    let result = install_inner();
    TAP_BUSY.store(false, Ordering::SeqCst);
    result
}

fn install_inner() -> Result<()> {
    if !has_accessibility_permission() {
        bail!("Accessibility permission is not granted");
    }
    // Reap a previous thread that ended on its own.
    if let Some(handle) = THREAD.lock().take() {
        let _ = handle.join();
    }
    globals::STOP.store(false, Ordering::SeqCst);
    let (tx, rx) = mpsc::channel();
    let handle = std::thread::Builder::new()
        .name("vkb-event-tap".into())
        .spawn(move || tap_thread(tx))
        .map_err(|err| anyhow!("spawning the event tap thread: {err}"))?;
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {
            *THREAD.lock() = Some(handle);
            Ok(())
        }
        Ok(Err(message)) => {
            let _ = handle.join();
            Err(anyhow!(message))
        }
        Err(_) => {
            globals::STOP.store(true, Ordering::SeqCst);
            Err(anyhow!("event tap thread did not start in time"))
        }
    }
}

/// Stop the tap thread. Idempotent.
pub fn uninstall() {
    globals::STOP.store(true, Ordering::SeqCst);
    {
        let run_loop = RUN_LOOP.lock();
        if *run_loop != 0 {
            unsafe { CFRunLoopStop(*run_loop as CFRunLoopRef) };
        }
    }
    if let Some(handle) = THREAD.lock().take() {
        let _ = handle.join();
    }
    globals::READY.store(false, Ordering::SeqCst);
}

/// Tap created and currently enabled.
pub fn is_ready() -> bool {
    if !globals::READY.load(Ordering::SeqCst) {
        return false;
    }
    let tap = TAP.load(Ordering::Acquire);
    !tap.is_null() && unsafe { CGEventTapIsEnabled(tap) }
}
