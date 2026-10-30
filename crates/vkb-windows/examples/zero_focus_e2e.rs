//! Zero-focus end-to-end harness ("hook sandwich").
//!
//! Drives the **real** pipeline - a real `WH_KEYBOARD_LL` hook, the real engine,
//! the real `SendInput` injection packets - without delivering a single key to
//! any window and without touching focus:
//!
//! ```text
//!   SendInput(HARNESS tag) --> H_high --> VietKB backend --> H_low --> (nothing)
//!                                  ^            |
//!                                  |            +-- injections (TEST tag) re-enter at the top
//!                                  +----------------- H_high swallows them
//!                                                      owner's installed VietKB (older hook) is never reached
//! ```
//!
//! * hooks run most-recently-installed first, so the order of installation is
//!   H_low, backend, H_high;
//! * the backend (feature `e2e-harness`) processes only HARNESS-tagged events and
//!   stamps its own injections with the TEST tag, so physical keys and the owner's
//!   installed VietKB are never touched;
//! * H_high swallows every TEST-tagged event and records it; H_low swallows every
//!   HARNESS-tagged event the backend let through (literal passthrough) and also
//!   swallows - and counts as a LEAK - any TEST-tagged event H_high missed.
//!
//! Run (never touches the desktop; safe while the owner types):
//!
//! ```text
//! cargo run -p vkb-windows --features e2e-harness --example zero_focus_e2e --release
//! cargo run -p vkb-windows --features e2e-harness --example zero_focus_e2e --release -- --probe-only
//! ```
//!
//! Options: `--probe-only`, `--methods telex,vni`, `--modes unicode,standard`,
//! `--pace-ms N` (default 6), `--only <id-prefix>`.
//! Results: `target/zero-focus-e2e.txt`.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use parking_lot::{Mutex, RwLock};
use vkb_config::AppConfig;
use vkb_core::InputMethod;
use vkb_platform::BackendCommand;
use vkb_windows::keys;
use windows::Win32::Foundation::{BOOL, HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, PeekMessageW,
    PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

/// Tag on every event the harness sends.
const HARNESS_TAG: usize = 0x5645_5448; // "VETH"
/// Tag the backend stamps on its own injections.
const TEST_TAG: usize = 0x5645_5449; // "VETI"

const VK_LSHIFT: u16 = 0xA0;
const VK_LCONTROL: u16 = 0xA2;
const VK_F23: u16 = 0x86;
const VK_F24: u16 = 0x87;

// ---------------------------------------------------------------------------
// Observation (written by the two hook threads, read by the driver)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Src {
    /// Seen by H_high with the TEST tag: the backend's injection.
    Injected,
    /// Seen by H_low with the HARNESS tag: a key the backend let through.
    Passed,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Ev {
    seq: u64,
    src: Src,
    vk: u16,
    scan: u16,
    down: bool,
}

static SEQ: AtomicU64 = AtomicU64::new(0);
static LOG: Mutex<Vec<Ev>> = Mutex::new(Vec::new());

static SENT_H: AtomicU64 = AtomicU64::new(0);
static SENT_T: AtomicU64 = AtomicU64::new(0);
static HIGH_H: AtomicU64 = AtomicU64::new(0); // harness events H_high saw (passes them)
static HIGH_T: AtomicU64 = AtomicU64::new(0); // test events H_high swallowed
static LOW_H: AtomicU64 = AtomicU64::new(0); // harness events H_low swallowed
static LOW_T: AtomicU64 = AtomicU64::new(0); // test events that ESCAPED H_high (leak)
static FOREIGN: AtomicU64 = AtomicU64::new(0); // physical / other-app events passed untouched
static LAST_EVENT_MS: AtomicU64 = AtomicU64::new(0);
static ABORT: AtomicBool = AtomicBool::new(false);
static T0: Mutex<Option<Instant>> = Mutex::new(None);

fn now_ms() -> u64 {
    T0.lock().map_or(0, |t| t.elapsed().as_millis() as u64)
}

fn record(src: Src, info: &KBDLLHOOKSTRUCT, wparam: WPARAM) {
    let down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
    // The sequence number is taken under the lock so the log order is the
    // order in which the two hook threads observed the events.
    let mut log = LOG.lock();
    let ev = Ev {
        seq: SEQ.fetch_add(1, Ordering::SeqCst),
        src,
        vk: info.vkCode as u16,
        scan: info.scanCode as u16,
        down,
    };
    log.push(ev);
}

unsafe extern "system" fn high_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if info.dwExtraInfo == TEST_TAG {
            record(Src::Injected, info, wparam);
            HIGH_T.fetch_add(1, Ordering::SeqCst);
            LAST_EVENT_MS.store(now_ms(), Ordering::SeqCst);
            return LRESULT(1);
        }
        if info.dwExtraInfo == HARNESS_TAG {
            HIGH_H.fetch_add(1, Ordering::SeqCst);
            LAST_EVENT_MS.store(now_ms(), Ordering::SeqCst);
        } else {
            FOREIGN.fetch_add(1, Ordering::Relaxed);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

unsafe extern "system" fn low_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if info.dwExtraInfo == HARNESS_TAG {
            record(Src::Passed, info, wparam);
            LOW_H.fetch_add(1, Ordering::SeqCst);
            LAST_EVENT_MS.store(now_ms(), Ordering::SeqCst);
            return LRESULT(1);
        }
        if info.dwExtraInfo == TEST_TAG {
            // H_high should have swallowed this. Swallow it here as well so it
            // still cannot reach a window, and count it as a leak.
            LOW_T.fetch_add(1, Ordering::SeqCst);
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// One LL hook on its own thread with its own message loop; unhooked on drop.
struct HookThread {
    thread_id: u32,
    join: Option<thread::JoinHandle<()>>,
}

impl HookThread {
    fn start(name: &str, proc_: unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT) -> Self {
        let (tx, rx) = mpsc::channel::<Result<u32, String>>();
        let join = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let mut msg = MSG::default();
                let _ = unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE) };
                let hook: HHOOK = match unsafe {
                    SetWindowsHookExW(WH_KEYBOARD_LL, Some(proc_), HINSTANCE::default(), 0)
                } {
                    Ok(hook) => hook,
                    Err(err) => {
                        let _ = tx.send(Err(err.to_string()));
                        return;
                    }
                };
                let _ = tx.send(Ok(unsafe { GetCurrentThreadId() }));
                while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
                    unsafe {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
                unsafe {
                    let _ = UnhookWindowsHookEx(hook);
                }
            })
            .expect("spawn hook thread");
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(thread_id)) => Self {
                thread_id,
                join: Some(join),
            },
            Ok(Err(err)) => panic!("{name}: SetWindowsHookExW failed: {err}"),
            Err(_) => panic!("{name}: hook thread did not start"),
        }
    }
}

impl Drop for HookThread {
    fn drop(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// Everything installed, torn down on every exit path (including unwinding).
struct Sandwich {
    high: Option<HookThread>,
    low: Option<HookThread>,
    backend_up: bool,
}

impl Drop for Sandwich {
    fn drop(&mut self) {
        // Top of the chain first, so nothing the backend still emits can leak.
        self.high.take();
        if self.backend_up {
            let _ = vkb_windows::uninstall_hook();
            self.backend_up = false;
        }
        self.low.take();
        vkb_windows::e2e::reset();
    }
}

unsafe extern "system" fn ctrl_handler(_kind: u32) -> BOOL {
    ABORT.store(true, Ordering::SeqCst);
    BOOL(1)
}

// ---------------------------------------------------------------------------
// Sending
// ---------------------------------------------------------------------------

fn key_input(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS, extra: usize) -> INPUT {
    INPUT {
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
    }
}

fn vk_event(vk: u16, up: bool, extra: usize) -> INPUT {
    let scan = match vk {
        VK_LSHIFT => 0x2A,
        VK_LCONTROL => 0x1D,
        other => keys::scan_for_vk(other),
    };
    key_input(vk, scan, if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) }, extra)
}

fn unicode_event(unit: u16, up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    key_input(0, unit, flags, HARNESS_TAG)
}

/// Send one batch. Aborts the process-level run on a short send (events that
/// vanish would corrupt the accounting).
fn send_batch(inputs: &[INPUT], harness: bool) {
    if inputs.is_empty() {
        return;
    }
    let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) } as u64;
    if harness {
        SENT_H.fetch_add(sent, Ordering::SeqCst);
    } else {
        SENT_T.fetch_add(sent, Ordering::SeqCst);
    }
    if sent != inputs.len() as u64 {
        ABORT.store(true, Ordering::SeqCst);
        eprintln!("SendInput accepted {sent}/{}: aborting", inputs.len());
    }
}

fn tap_vk(vk: u16, shift: bool) -> Vec<INPUT> {
    let mut out = Vec::with_capacity(4);
    if shift {
        out.push(vk_event(VK_LSHIFT, false, HARNESS_TAG));
    }
    out.push(vk_event(vk, false, HARNESS_TAG));
    out.push(vk_event(vk, true, HARNESS_TAG));
    if shift {
        out.push(vk_event(VK_LSHIFT, true, HARNESS_TAG));
    }
    out
}

fn char_events(ch: char) -> Vec<INPUT> {
    match ch {
        '\n' => tap_vk(keys::VK_RETURN, false),
        '\t' => tap_vk(keys::VK_TAB, false),
        other => match keys::us_vk_for_char(other) {
            Some((vk, shift)) => tap_vk(vk, shift),
            None => {
                let mut buf = [0u16; 2];
                let mut out = Vec::new();
                for unit in other.encode_utf16(&mut buf).iter().copied() {
                    out.push(unicode_event(unit, false));
                    out.push(unicode_event(unit, true));
                }
                out
            }
        },
    }
}

#[derive(Clone, Debug)]
enum Step {
    /// Type characters one key at a time.
    Text(String),
    /// Press and release one virtual key.
    Tap(u16),
    /// Hold / release a modifier or key.
    Down(u16),
    Up(u16),
}

// ---------------------------------------------------------------------------
// Rebuilding the text
// ---------------------------------------------------------------------------

fn rebuild(events: &[Ev]) -> (String, Vec<u16>) {
    let mut text: Vec<char> = Vec::new();
    let mut leaked_vks: Vec<u16> = Vec::new();
    let mut shift = [false; 2]; // per source
    let mut pending_high: [Option<u16>; 2] = [None, None];
    for ev in events {
        let idx = if ev.src == Src::Injected { 0 } else { 1 };
        if matches!(ev.vk, keys::VK_SHIFT | VK_LSHIFT | keys::VK_RSHIFT) {
            shift[idx] = ev.down;
            continue;
        }
        if !ev.down {
            continue;
        }
        match ev.vk {
            keys::VK_BACK => {
                text.pop();
            }
            keys::VK_RETURN => text.push('\n'),
            keys::VK_TAB => text.push('\t'),
            keys::VK_PACKET => {
                let unit = ev.scan;
                if let Some(high) = pending_high[idx].take() {
                    if let Some(ch) = char::decode_utf16([high, unit]).next().and_then(Result::ok) {
                        text.push(ch);
                    }
                } else if (0xD800..0xDC00).contains(&unit) {
                    pending_high[idx] = Some(unit);
                } else if let Some(ch) = char::from_u32(u32::from(unit)) {
                    text.push(ch);
                }
            }
            vk => {
                if let Some(ch) = keys::us_char(vk, shift[idx], false) {
                    text.push(ch);
                } else if ev.src == Src::Passed {
                    // A non-text key (F-key, modifier chord) reached the app side.
                    leaked_vks.push(vk);
                }
            }
        }
    }
    (text.into_iter().collect(), leaked_vks)
}

// ---------------------------------------------------------------------------
// Vietnamese -> keystrokes (port of qa/e2e/encode-keys.ps1)
// ---------------------------------------------------------------------------

/// base letter, shape (0 none, 1 circumflex, 2 breve, 3 horn), tone (0..5 = none f s r x j)
fn decompose(ch: char) -> Option<(char, u8, u8)> {
    const ROWS: [(char, [&str; 3]); 6] = [
        ('a', ["aàáảãạ", "ăằắẳẵặ", "âầấẩẫậ"]),
        ('e', ["eèéẻẽẹ", "", "êềếểễệ"]),
        ('i', ["iìíỉĩị", "", ""]),
        ('o', ["oòóỏõọ", "", "ôồốổỗộ"]),
        ('u', ["uùúủũụ", "", ""]),
        ('y', ["yỳýỷỹỵ", "", ""]),
    ];
    // Shape groups per base: [plain, breve|horn, circumflex]; o and u carry horn.
    let lower = ch.to_lowercase().next()?;
    let upper = ch != lower;
    let hit = |base: char, shape: u8, tone: u8| Some((if upper { base.to_ascii_uppercase() } else { base }, shape, tone));
    for (base, groups) in ROWS {
        for (g, set) in groups.iter().enumerate() {
            if let Some(pos) = set.chars().position(|c| c == lower) {
                let shape = match (base, g) {
                    (_, 0) => 0,
                    ('a', 1) => 2,
                    (_, 2) => 1,
                    _ => 0,
                };
                return hit(base, shape, pos as u8);
            }
        }
    }
    // ơ and ư (horn): own rows.
    const HORN: [(char, &str); 2] = [('o', "ơờớởỡợ"), ('u', "ưừứửữự")];
    for (base, set) in HORN {
        if let Some(pos) = set.chars().position(|c| c == lower) {
            return hit(base, 3, pos as u8);
        }
    }
    None
}

fn encode_word(word: &str, vni: bool) -> String {
    let mut out = String::new();
    let mut tone = 0u8;
    for ch in word.chars() {
        if ch == 'đ' || ch == 'Đ' {
            out.push(if ch == 'Đ' { 'D' } else { 'd' });
            out.push(if vni { '9' } else { 'd' });
            continue;
        }
        match decompose(ch) {
            None => out.push(ch),
            Some((base, shape, t)) => {
                out.push(base);
                let lower = base.to_ascii_lowercase();
                if vni {
                    match shape {
                        1 => out.push('6'),
                        2 => out.push('8'),
                        3 => out.push('7'),
                        _ => {}
                    }
                } else {
                    match shape {
                        1 => out.push(lower),
                        2 | 3 => out.push('w'),
                        _ => {}
                    }
                }
                if t != 0 {
                    tone = t;
                }
            }
        }
    }
    if tone != 0 {
        let key = if vni {
            ['0', '2', '1', '3', '4', '5'][tone as usize]
        } else {
            ['0', 'f', 's', 'r', 'x', 'j'][tone as usize]
        };
        out.push(key);
    }
    out
}

fn encode_keys(text: &str, vni: bool) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if !word.is_empty() {
            out.push_str(&encode_word(word, vni));
            word.clear();
        }
    };
    for ch in text.chars() {
        if ch.is_alphabetic() {
            word.push(ch);
        } else {
            flush(&mut word, &mut out);
            out.push(ch);
        }
    }
    flush(&mut word, &mut out);
    out
}

// ---------------------------------------------------------------------------
// Cases
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
struct Settings {
    method: InputMethod,
    old_tone: bool,
    no_jfwz: bool,
    no_spell: bool,
    no_free_tone: bool,
    standard: bool,
}

struct Case {
    id: String,
    settings: Settings,
    steps: Vec<Step>,
    expected: String,
    /// Extra assertion on the observation: `(vks that must NOT reach H_low, command that must fire)`.
    forbid_passed_vks: Vec<u16>,
    want_command: Option<BackendCommand>,
}

impl Case {
    fn text(id: String, settings: Settings, keys_: String, expected: String) -> Self {
        Self {
            id,
            settings,
            steps: vec![Step::Text(keys_)],
            expected,
            forbid_passed_vks: Vec::new(),
            want_command: None,
        }
    }
}

fn method_of(name: &str) -> InputMethod {
    match name {
        "vni" => InputMethod::Vni,
        _ => InputMethod::Telex,
    }
}

fn read_utf8(path: &Path) -> String {
    let raw = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let text = String::from_utf8(raw).expect("utf-8");
    text.trim_start_matches('\u{feff}').to_string()
}

fn load_cases(
    root: &Path,
    methods: &[String],
    standard: bool,
    only: &Option<String>,
) -> Vec<Case> {
    let mode = if standard { "std" } else { "uni" };
    let mut cases = Vec::new();
    let base = |method: InputMethod| Settings {
        method,
        old_tone: false,
        no_jfwz: false,
        no_spell: false,
        no_free_tone: false,
        standard,
    };

    // --- edge cases ---
    let edge = read_utf8(&root.join("edge-cases.tsv"));
    for (n, line) in edge.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if n == 0 || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 {
            continue;
        }
        let (id, method, flags, keys_, expected) = (f[0], f[1], f[2], f[3], f[4]);
        if !methods.iter().any(|m| m == method) {
            continue;
        }
        let mut settings = base(method_of(method));
        settings.old_tone = flags.contains("--old-tone-style");
        settings.no_jfwz = flags.contains("--no-jfwz-consonant");
        settings.no_spell = flags.contains("--no-spell-check");
        settings.no_free_tone = flags.contains("--no-free-tone");
        cases.push(Case::text(
            format!("edge/{id}/{mode}"),
            settings,
            keys_.to_string(),
            expected.to_string(),
        ));
    }

    // --- paragraph corpus ---
    let mut files: Vec<PathBuf> = std::fs::read_dir(root.join("corpus"))
        .expect("corpus dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
        .collect();
    files.sort();
    for method in methods {
        for file in &files {
            let expected = read_utf8(file).trim().to_string();
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            let keys_ = encode_keys(&expected, method == "vni");
            cases.push(Case::text(
                format!("corpus/{stem}/{method}/{mode}"),
                base(method_of(method)),
                keys_,
                expected,
            ));
        }
    }

    // --- hotkeys (default hotkey set, release mode) ---
    for method in methods {
        let vni = method == "vni";
        let word = if vni { "vie6t5" } else { "vieetj" };
        let m = base(method_of(method));
        let id = |name: &str| format!("hotkey/{name}/{method}/{mode}");
        let typed = |s: &str| Step::Text(s.to_string());
        cases.push(Case {
            id: id("f2-disable-literal"),
            settings: m.clone(),
            steps: vec![Step::Tap(keys::VK_F2), typed(word)],
            expected: word.to_string(),
            forbid_passed_vks: vec![keys::VK_F2],
            want_command: Some(BackendCommand::Enable(false)),
        });
        cases.push(Case {
            id: id("f1-enable-composes"),
            settings: m.clone(),
            steps: vec![Step::Tap(keys::VK_F2), Step::Tap(keys::VK_F1), typed(word)],
            expected: "việt".to_string(),
            // D-E2E-03: neither trigger may reach the application.
            forbid_passed_vks: vec![keys::VK_F1, keys::VK_F2],
            want_command: Some(BackendCommand::Enable(true)),
        });
        cases.push(Case {
            id: id("f2-then-f1-then-f2"),
            settings: m.clone(),
            steps: vec![
                Step::Tap(keys::VK_F2),
                Step::Tap(keys::VK_F1),
                Step::Tap(keys::VK_F2),
                typed(word),
            ],
            expected: word.to_string(),
            forbid_passed_vks: vec![keys::VK_F1, keys::VK_F2],
            want_command: None,
        });
        cases.push(Case {
            id: id("ctrl-shift-toggle-twice"),
            settings: m.clone(),
            steps: vec![
                Step::Down(VK_LCONTROL),
                Step::Down(VK_LSHIFT),
                Step::Up(VK_LSHIFT),
                Step::Up(VK_LCONTROL),
                typed(word),
                typed(" "),
                Step::Down(VK_LCONTROL),
                Step::Down(VK_LSHIFT),
                Step::Up(VK_LSHIFT),
                Step::Up(VK_LCONTROL),
                typed(word),
            ],
            expected: format!("{word} việt"),
            forbid_passed_vks: Vec::new(),
            want_command: None,
        });
        cases.push(Case {
            id: id("restore-word-chord"),
            settings: m,
            steps: vec![
                typed(word),
                Step::Down(VK_LCONTROL),
                Step::Down(VK_LSHIFT),
                Step::Down(0x5A),
                Step::Up(0x5A),
                Step::Up(VK_LSHIFT),
                Step::Up(VK_LCONTROL),
            ],
            // The word is rewritten to the raw keys, and the command fires.
            expected: word.to_string(),
            forbid_passed_vks: Vec::new(),
            want_command: Some(BackendCommand::RestoreWord),
        });
    }

    if let Some(prefix) = only {
        cases.retain(|c| c.id.starts_with(prefix.as_str()));
    }
    cases
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

static COMMANDS: Mutex<Vec<BackendCommand>> = Mutex::new(Vec::new());
static LAST_FG: AtomicU32 = AtomicU32::new(0);

fn fg() -> u32 {
    unsafe { GetForegroundWindow() }.0 as usize as u32
}

fn config_for(settings: &Settings) -> AppConfig {
    let mut cfg = AppConfig::default();
    cfg.input.enabled = true;
    cfg.input.method = settings.method;
    cfg.typing.modern_tone = !settings.old_tone;
    cfg.typing.jfwz_as_consonant = !settings.no_jfwz;
    cfg.typing.spell_check = !settings.no_spell;
    cfg.typing.free_tone = !settings.no_free_tone;
    cfg.typing.send_standard = settings.standard;
    cfg.typing.use_clipboard = false;
    cfg.apps.remember_per_app = false;
    // Never make a sound on the owner's machine.
    cfg.system.notification_sound = false;
    cfg
}

struct Driver {
    pace: Duration,
    shared: Arc<RwLock<AppConfig>>,
    applied: Option<Settings>,
}

impl Driver {
    fn apply(&mut self, settings: &Settings) {
        if self.applied.as_ref() != Some(settings) {
            *self.shared.write() = config_for(settings);
            vkb_windows::set_config(self.shared.clone());
            thread::sleep(Duration::from_millis(80));
            self.applied = Some(settings.clone());
        }
        vkb_windows::set_enabled(true);
    }

    /// Wait until every harness event is accounted for and the stream is quiet.
    /// Returns the number of unaccounted events (0 = clean).
    fn settle(&self) -> u64 {
        let deadline = Instant::now() + Duration::from_millis(1500);
        loop {
            let (_, swallowed_by_backend) = vkb_windows::e2e::counters();
            let sent = SENT_H.load(Ordering::SeqCst);
            let accounted = swallowed_by_backend + LOW_H.load(Ordering::SeqCst);
            let quiet = now_ms().saturating_sub(LAST_EVENT_MS.load(Ordering::SeqCst)) >= 4;
            if sent == accounted && sent == HIGH_H.load(Ordering::SeqCst) && quiet {
                return 0;
            }
            if Instant::now() >= deadline {
                return sent.abs_diff(accounted).max(1);
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn send_step(&self, step: &Step) -> bool {
        match step {
            Step::Text(text) => {
                for ch in text.chars() {
                    if ABORT.load(Ordering::SeqCst) {
                        return false;
                    }
                    send_batch(&char_events(ch), true);
                    thread::sleep(self.pace);
                    if self.settle() != 0 {
                        return false;
                    }
                }
            }
            Step::Tap(vk) => {
                send_batch(&tap_vk(*vk, false), true);
                thread::sleep(self.pace);
                if self.settle() != 0 {
                    return false;
                }
            }
            Step::Down(vk) => {
                send_batch(&[vk_event(*vk, false, HARNESS_TAG)], true);
                thread::sleep(self.pace);
                if self.settle() != 0 {
                    return false;
                }
            }
            Step::Up(vk) => {
                send_batch(&[vk_event(*vk, true, HARNESS_TAG)], true);
                thread::sleep(self.pace);
                if self.settle() != 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Run one case. Returns `(verdict, detail)`; `Err` means the run must stop.
    fn run(&mut self, case: &Case) -> Result<(bool, String), String> {
        self.apply(&case.settings);
        // A non-text key ends any half-typed word from the previous case.
        send_batch(&tap_vk(VK_F24, false), true);
        thread::sleep(self.pace);
        if self.settle() != 0 {
            return Err(format!("{}: unaccounted events before the case", case.id));
        }
        LOG.lock().clear();
        COMMANDS.lock().clear();
        let start_leak = LOW_T.load(Ordering::SeqCst);

        for step in &case.steps {
            if !self.send_step(step) {
                return Err(format!("{}: harness events unaccounted for or send aborted", case.id));
            }
        }
        if LOW_T.load(Ordering::SeqCst) != start_leak {
            return Err(format!("{}: injected event escaped the top hook", case.id));
        }

        let events = LOG.lock().clone();
        let (actual, leaked) = rebuild(&events);
        let mut ok = actual == case.expected;
        let mut detail = String::new();
        if !ok {
            let _ = write!(detail, "expected=[{}] actual=[{}]", case.expected, actual);
        }
        let reached: Vec<u16> = events
            .iter()
            .filter(|e| e.src == Src::Passed && e.down && case.forbid_passed_vks.contains(&e.vk))
            .map(|e| e.vk)
            .collect();
        if !reached.is_empty() {
            ok = false;
            let _ = write!(
                detail,
                " trigger-key-down-reached-app={:?} (would have hit the focused window)",
                reached
                    .iter()
                    .map(|vk| format!("0x{vk:02X}"))
                    .collect::<Vec<_>>()
            );
        }
        if let Some(want) = &case.want_command {
            let seen = COMMANDS.lock().iter().any(|c| c == want);
            if !seen {
                ok = false;
                let _ = write!(detail, " command {want:?} not fired (got {:?})", COMMANDS.lock());
            }
        }
        let _ = leaked; // non-text keys that passed are reported only via forbid_passed_vks
        Ok((ok, detail))
    }
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn sanity_check_tags() -> Result<(), String> {
    use vkb_platform::plan::Injection;
    use vkb_windows::input::{build, SendMode};
    let samples = [
        Injection::Type("Việt Nam\n".to_string()),
        Injection::Replace {
            deletes: 3,
            insert: "tâ😀".to_string(),
        },
    ];
    for mode in [SendMode::Unicode, SendMode::Standard] {
        for sample in &samples {
            for input in build(sample, mode) {
                let extra = unsafe { input.Anonymous.ki.dwExtraInfo };
                if extra != TEST_TAG {
                    return Err(format!("an injected event carries tag {extra:#x}, not the test tag"));
                }
            }
        }
    }
    for input in vkb_windows::input::ctrl_v_sequence() {
        let extra = unsafe { input.Anonymous.ki.dwExtraInfo };
        if extra != TEST_TAG {
            return Err("the clipboard fallback chord is not test-tagged".to_string());
        }
    }
    Ok(())
}

/// Wait until `cond` holds or `ms` elapse.
fn wait_for(ms: u64, cond: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < deadline {
        if cond() {
            return true;
        }
        thread::sleep(Duration::from_millis(2));
    }
    cond()
}

/// The safety gate. `Ok` only when every probe event is accounted for and none escaped.
fn probe(report: &mut String) -> Result<(), String> {
    macro_rules! say {
        ($($arg:tt)*) => {{ let line = format!($($arg)*); println!("{line}"); report.push_str(&line); report.push('\n'); }};
    }
    say!("PROBE start");

    // 0. Pure check: every event the backend can build carries the test tag.
    sanity_check_tags()?;
    say!("PROBE step0 injection builders tag every event: OK");

    // 1. Infrastructure with keys no application reacts to (F23 / F24).
    let (h0, t0, hi_h0, hi_t0, lo_h0, lo_t0) = (
        SENT_H.load(Ordering::SeqCst),
        SENT_T.load(Ordering::SeqCst),
        HIGH_H.load(Ordering::SeqCst),
        HIGH_T.load(Ordering::SeqCst),
        LOW_H.load(Ordering::SeqCst),
        LOW_T.load(Ordering::SeqCst),
    );
    // 1a. harness-tagged: top hook passes, backend passes (non-text), bottom hook swallows.
    send_batch(&[vk_event(VK_F24, false, HARNESS_TAG), vk_event(VK_F24, true, HARNESS_TAG)], true);
    // 1b. test-tagged: the top hook must swallow them; the bottom hook must never see them.
    send_batch(&[vk_event(VK_F23, false, TEST_TAG), vk_event(VK_F23, true, TEST_TAG)], false);
    let ok = wait_for(1500, || {
        LOW_H.load(Ordering::SeqCst) - lo_h0 == 2 && HIGH_T.load(Ordering::SeqCst) - hi_t0 == 2
    });
    thread::sleep(Duration::from_millis(50));
    let sent_h = SENT_H.load(Ordering::SeqCst) - h0;
    let sent_t = SENT_T.load(Ordering::SeqCst) - t0;
    let high_h = HIGH_H.load(Ordering::SeqCst) - hi_h0;
    let high_t = HIGH_T.load(Ordering::SeqCst) - hi_t0;
    let low_h = LOW_H.load(Ordering::SeqCst) - lo_h0;
    let low_t = LOW_T.load(Ordering::SeqCst) - lo_t0;
    let (seen, swallowed) = vkb_windows::e2e::counters();
    say!(
        "PROBE step1 sent: harness={sent_h} test={sent_t} | H_high: harness-passed={high_h} test-swallowed={high_t} | backend: seen={seen} swallowed={swallowed} | H_low: harness-swallowed={low_h} test-escaped={low_t}"
    );
    let unaccounted = (sent_h + sent_t).saturating_sub(high_t + swallowed + low_h);
    say!("PROBE escaped events (unaccounted + test-tag escapes) = {}", unaccounted + low_t);
    if !ok || unaccounted != 0 || low_t != 0 || sent_h != 2 || sent_t != 2 || high_h != 2 || high_t != 2 || low_h != 2 {
        return Err("probe: hook order or accounting wrong (see counters above)".to_string());
    }
    say!("PROBE step1 hook order H_high -> backend -> H_low and swallow accounting: OK");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let probe_only = args.iter().any(|a| a == "--probe-only");
    let pace = Duration::from_millis(
        arg_value(&args, "--pace-ms")
            .and_then(|v| v.parse().ok())
            .unwrap_or(6),
    );
    let methods: Vec<String> = arg_value(&args, "--methods")
        .unwrap_or_else(|| "telex,vni".to_string())
        .split(',')
        .map(str::to_string)
        .collect();
    let modes: Vec<String> = arg_value(&args, "--modes")
        .unwrap_or_else(|| "unicode,standard".to_string())
        .split(',')
        .map(str::to_string)
        .collect();
    let only = arg_value(&args, "--only");

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest.join("../../target/zero-focus-e2e.txt");
    let qa_root = manifest.join("../../docs/evkey-clone-work/qa/e2e");
    let mut report = String::new();

    unsafe {
        let _ = SetConsoleCtrlHandler(Some(ctrl_handler), true);
    }
    *T0.lock() = Some(Instant::now());

    // Preconditions: nothing held, caps lock off (the backend seeds its modifier
    // state from the OS at install time).
    let held = [0x10u16, 0x11, 0x12, 0x5B, 0x5C]
        .iter()
        .any(|vk| (unsafe { GetAsyncKeyState(i32::from(*vk)) } as u16) & 0x8000 != 0);
    let caps = (unsafe { GetKeyState(i32::from(keys::VK_CAPITAL)) } & 1) != 0;
    if held || caps {
        eprintln!("refusing to start: a modifier is held or Caps Lock is on (retry in a moment)");
        std::process::exit(2);
    }

    let shared: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(config_for(&Settings {
        method: InputMethod::Telex,
        old_tone: false,
        no_jfwz: false,
        no_spell: false,
        no_free_tone: false,
        standard: false,
    })));

    // Install order matters: H_low, backend, H_high (newest runs first).
    vkb_windows::e2e::configure(HARNESS_TAG, TEST_TAG);
    let mut sandwich = Sandwich {
        high: None,
        low: None,
        backend_up: false,
    };
    sandwich.low = Some(HookThread::start("h-low", low_proc));
    vkb_windows::set_command_callback(|command| COMMANDS.lock().push(command));
    vkb_windows::set_config(shared.clone());
    if let Err(err) = vkb_windows::install() {
        eprintln!("backend hook did not install: {err}");
        std::process::exit(3);
    }
    sandwich.backend_up = true;
    thread::sleep(Duration::from_millis(100));
    sandwich.high = Some(HookThread::start("h-high", high_proc));
    thread::sleep(Duration::from_millis(100));
    LAST_FG.store(fg(), Ordering::SeqCst);

    let outcome = (|| -> Result<String, String> {
        probe(&mut report)?;
        if probe_only {
            return Ok("PROBE-ONLY clean".to_string());
        }

        let mut driver = Driver {
            pace,
            shared: shared.clone(),
            applied: None,
        };

        // Engine-level probe: the first real letters, only after the gate passed.
        let probe_case = Case::text(
            "probe/engine".to_string(),
            Settings {
                method: InputMethod::Telex,
                old_tone: false,
                no_jfwz: false,
                no_spell: false,
                no_free_tone: false,
                standard: false,
            },
            "vieetj".to_string(),
            "việt".to_string(),
        );
        let (ok, detail) = driver.run(&probe_case)?;
        let line = format!("PROBE step2 engine: {} {detail}", if ok { "OK" } else { "FAIL" });
        println!("{line}");
        report.push_str(&line);
        report.push('\n');
        if !ok {
            return Err("engine probe failed: backend inactive in the foreground app (IME/rule?) or pipeline broken".to_string());
        }

        let mut per_method: BTreeMap<String, (u32, u32)> = BTreeMap::new();
        let mut failing = Vec::new();
        for mode in &modes {
            let standard = mode == "standard";
            let cases = load_cases(&qa_root, &methods, standard, &only);
            for case in &cases {
                if ABORT.load(Ordering::SeqCst) {
                    return Err("aborted (ctrl-c or send failure)".to_string());
                }
                let (ok, detail) = driver.run(case)?;
                let group = {
                    let parts: Vec<&str> = case.id.split('/').collect();
                    let method = match case.settings.method {
                        InputMethod::Vni => "vni",
                        _ => "telex",
                    };
                    format!("{}-{}-{}", parts[0], method, if standard { "standard" } else { "unicode" })
                };
                let entry = per_method.entry(group).or_default();
                if ok {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                    failing.push(case.id.clone());
                }
                let line = format!("{} {} {}", if ok { "PASS" } else { "FAIL" }, case.id, detail);
                println!("{line}");
                report.push_str(&line);
                report.push('\n');
            }
        }

        let mut summary = String::from("SUMMARY");
        let (mut p, mut f) = (0, 0);
        for (group, (pass, fail)) in &per_method {
            let _ = write!(summary, " {group}={pass}/{}", pass + fail);
            p += pass;
            f += fail;
        }
        let (_, swallowed) = vkb_windows::e2e::counters();
        let _ = write!(
            summary,
            " | PASS={p} FAIL={f} | sent_harness={} sent_test={} H_high_swallowed={} backend_swallowed={} H_low_swallowed={} escaped={} foreign_untouched={}",
            SENT_H.load(Ordering::SeqCst),
            SENT_T.load(Ordering::SeqCst),
            HIGH_T.load(Ordering::SeqCst),
            swallowed,
            LOW_H.load(Ordering::SeqCst),
            LOW_T.load(Ordering::SeqCst),
            FOREIGN.load(Ordering::Relaxed),
        );
        if !failing.is_empty() {
            let _ = write!(summary, "\nFAILING: {}", failing.join(", "));
        }
        Ok(summary)
    })();

    let final_fg = fg();
    let fg_note = if final_fg != LAST_FG.load(Ordering::SeqCst) {
        "foreground window changed during the run (owner switched windows; nothing was delivered to it)"
    } else {
        "foreground window unchanged"
    };

    drop(sandwich);
    let line = match &outcome {
        Ok(summary) => summary.clone(),
        Err(err) => format!("ABORT {err}"),
    };
    println!("{line}\n{fg_note}");
    report.push_str(&line);
    report.push('\n');
    report.push_str(fg_note);
    report.push('\n');
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&out_path, &report);
    println!("results: {}", out_path.display());
    if outcome.is_err() {
        std::process::exit(1);
    }
}
