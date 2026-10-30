//! Zero-focus end-to-end harness.
//!
//! Drives the real decision path of the backend - `hook::process_event` (the code
//! the `WH_KEYBOARD_LL` procedure runs), the hotkey matcher, the engine and the
//! real injection packet builders - and compares the rebuilt text with the
//! expected text. It never delivers a key to any window and never touches focus.
//!
//! # Default mode: in-process, zero global events
//!
//! No `SendInput`, no hook is installed. The harness builds `KBDLLHOOKSTRUCT`
//! values and feeds them to the backend directly; the backend's injections are
//! diverted into a capture buffer instead of `SendInput` (feature `e2e-harness`).
//! The text is rebuilt from the captured injections (backspaces, unicode packets,
//! VK+scancode keys) plus the keys the hook decided to pass. Only the OS
//! transport is skipped. Captured packets are also fed back into the runtime to
//! prove it ignores its own injections.
//!
//! ```text
//! cargo run -p vkb-windows --features e2e-harness --example zero_focus_e2e --release
//! ```
//!
//! # `--live-hooks`: real hooks (explicit opt-in)
//!
//! ```text
//!   SendInput(HARNESS tag) --> H_high --> backend --> H_low --> (nothing)
//!                                  ^          |
//!                                  +----------+ injections (TEST tag) re-enter at the top
//! ```
//!
//! Refuses to start while any other `vietkb*` / `vkb*` process is alive: such an
//! instance refreshes its own hook every 30 s, can become the newest hook, and
//! would compose the harness's keys and inject into the focused window with the
//! production marker, which neither observer could catch. H_high is re-installed
//! before every case and the run aborts on the first unaccounted event.
//!
//! Options: `--probe-only` (live), `--methods telex,vni`, `--modes unicode,standard`,
//! `--pace-ms N` (live, default 6), `--only <id-prefix>`.
//! Results: `target/zero-focus-e2e.txt`.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use vkb_config::AppConfig;
use vkb_core::InputMethod;
use vkb_platform::BackendCommand;
use vkb_windows::keys;
use vkb_windows::state::HookRuntime;
use windows::Win32::Foundation::{CloseHandle, BOOL, HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, PeekMessageW,
    PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, KBDLLHOOKSTRUCT_FLAGS, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN,
};

/// Tag on every event the harness feeds or sends.
const HARNESS_TAG: usize = 0x5645_5448; // "VETH"
/// Tag the backend stamps on its own injections.
const TEST_TAG: usize = 0x5645_5449; // "VETI"

const VK_LSHIFT: u16 = 0xA0;
const VK_LCONTROL: u16 = 0xA2;
const VK_F23: u16 = 0x86;
const VK_F24: u16 = 0x87;

static ABORT: AtomicBool = AtomicBool::new(false);
static EPOCH: OnceLock<Instant> = OnceLock::new();

fn now_ms() -> u64 {
    EPOCH.get().map_or(0, |t| t.elapsed().as_millis() as u64)
}

// ---------------------------------------------------------------------------
// Observation
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Src {
    /// The backend's injection (in-process capture, or live H_high with the TEST tag).
    Injected,
    /// A key the backend let through (in-process, or live H_low with the HARNESS tag).
    Passed,
}

#[derive(Clone, Copy, Debug)]
struct Ev {
    src: Src,
    vk: u16,
    scan: u16,
    down: bool,
}

/// One synthetic key event the harness feeds or sends.
#[derive(Clone, Copy, Debug)]
struct Syn {
    vk: u16,
    scan: u16,
    up: bool,
    unicode: bool,
}

// ---------------------------------------------------------------------------
// Live mode: lock-free recording inside the hook procedures
// ---------------------------------------------------------------------------

const RING: usize = 1 << 16;
const EMPTY: AtomicU64 = AtomicU64::new(0);
/// Packed events: bit 63 valid, bit 41 down, bit 40 src, bits 16..32 scan, 0..16 vk.
static RING_BUF: [AtomicU64; RING] = [EMPTY; RING];
static HEAD: AtomicU64 = AtomicU64::new(0);

static SENT_H: AtomicU64 = AtomicU64::new(0);
static SENT_T: AtomicU64 = AtomicU64::new(0);
static HIGH_H: AtomicU64 = AtomicU64::new(0); // harness events H_high saw (passes them)
static HIGH_T: AtomicU64 = AtomicU64::new(0); // test events H_high swallowed
static LOW_H: AtomicU64 = AtomicU64::new(0); // harness events H_low swallowed
static LOW_T: AtomicU64 = AtomicU64::new(0); // test events that ESCAPED H_high (leak)
static FOREIGN: AtomicU64 = AtomicU64::new(0); // other traffic passed untouched
static LAST_EVENT_MS: AtomicU64 = AtomicU64::new(0);

fn record(src: Src, info: &KBDLLHOOKSTRUCT, wparam: WPARAM) {
    let down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
    let idx = HEAD.fetch_add(1, Ordering::SeqCst) as usize % RING;
    let packed = (1u64 << 63)
        | (u64::from(down) << 41)
        | (u64::from(src == Src::Passed) << 40)
        | (u64::from(info.scanCode as u16) << 16)
        | u64::from(info.vkCode as u16);
    RING_BUF[idx].store(packed, Ordering::Release);
}

/// Move recorded events from the ring into `out`, in observation order.
fn drain_ring(cursor: &mut u64, out: &mut Vec<Ev>) {
    let head = HEAD.load(Ordering::SeqCst);
    while *cursor < head {
        let idx = (*cursor as usize) % RING;
        let deadline = Instant::now() + Duration::from_millis(200);
        let mut packed = RING_BUF[idx].load(Ordering::Acquire);
        while packed >> 63 == 0 && Instant::now() < deadline {
            std::hint::spin_loop();
            packed = RING_BUF[idx].load(Ordering::Acquire);
        }
        RING_BUF[idx].store(0, Ordering::Release);
        out.push(Ev {
            src: if packed >> 40 & 1 == 1 { Src::Passed } else { Src::Injected },
            vk: packed as u16,
            scan: (packed >> 16) as u16,
            down: packed >> 41 & 1 == 1,
        });
        *cursor += 1;
    }
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

/// Everything installed in live mode, torn down on every exit path.
struct Sandwich {
    high: Option<HookThread>,
    low: Option<HookThread>,
    backend_up: bool,
}

impl Sandwich {
    /// Re-install H_high so it is the newest hook again.
    fn reinstall_high(&mut self) {
        self.high = None;
        self.high = Some(HookThread::start("h-high", high_proc));
    }
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
    }
}

/// Only Ctrl+C / Ctrl+Break are ours; anything else is left to the default handler.
unsafe extern "system" fn ctrl_handler(kind: u32) -> BOOL {
    if kind == 0 || kind == 1 {
        ABORT.store(true, Ordering::SeqCst);
        BOOL(1)
    } else {
        BOOL(0)
    }
}

/// Names of other VietKB-like processes (empty = clear to run live hooks).
fn other_vietkb_processes() -> Result<Vec<String>, String> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(|e| format!("process snapshot failed: {e}"))?;
    let own = unsafe { GetCurrentProcessId() };
    let mut found = Vec::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while ok {
        let len = entry.szExeFile.iter().position(|c| *c == 0).unwrap_or(entry.szExeFile.len());
        let name = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
        if entry.th32ProcessID != own && (name.starts_with("vietkb") || name.starts_with("vkb")) {
            found.push(format!("{name} (pid {})", entry.th32ProcessID));
        }
        ok = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    unsafe {
        let _ = CloseHandle(snapshot);
    }
    Ok(found)
}

// ---------------------------------------------------------------------------
// Building events
// ---------------------------------------------------------------------------

fn vk_syn(vk: u16, up: bool) -> Syn {
    let scan = match vk {
        VK_LSHIFT => 0x2A,
        VK_LCONTROL => 0x1D,
        other => keys::scan_for_vk(other),
    };
    Syn {
        vk,
        scan,
        up,
        unicode: false,
    }
}

fn unicode_syn(unit: u16, up: bool) -> Syn {
    Syn {
        vk: 0,
        scan: unit,
        up,
        unicode: true,
    }
}

fn tap_vk(vk: u16, shift: bool) -> Vec<Syn> {
    let mut out = Vec::with_capacity(4);
    if shift {
        out.push(vk_syn(VK_LSHIFT, false));
    }
    out.push(vk_syn(vk, false));
    out.push(vk_syn(vk, true));
    if shift {
        out.push(vk_syn(VK_LSHIFT, true));
    }
    out
}

fn char_syns(ch: char) -> Vec<Syn> {
    match ch {
        '\n' => tap_vk(keys::VK_RETURN, false),
        '\t' => tap_vk(keys::VK_TAB, false),
        other => match keys::us_vk_for_char(other) {
            Some((vk, shift)) => tap_vk(vk, shift),
            None => {
                let mut buf = [0u16; 2];
                let mut out = Vec::new();
                for unit in other.encode_utf16(&mut buf).iter().copied() {
                    out.push(unicode_syn(unit, false));
                    out.push(unicode_syn(unit, true));
                }
                out
            }
        },
    }
}

fn to_input(s: &Syn, extra: usize) -> INPUT {
    let mut flags = if s.up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) };
    if s.unicode {
        flags |= KEYEVENTF_UNICODE;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(s.vk),
                wScan: s.scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: extra,
            },
        },
    }
}

/// Live mode: send one batch. A short send aborts the run.
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

#[derive(Clone, Debug)]
enum Step {
    /// Type characters one key at a time.
    Text(String),
    /// Press and release one virtual key.
    Tap(u16),
    /// Hold / release a modifier or key.
    Down(u16),
    Up(u16),
    /// A mouse button goes down (in-process mode only).
    Mouse,
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
    live: bool,
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
        if !live {
            cases.push(Case {
                id: id("mouse-click-ends-word"),
                settings: base(method_of(method)),
                steps: vec![typed("a"), Step::Mouse, typed("a")],
                // Without the mouse boundary `aa` would compose a circumflex.
                expected: "aa".to_string(),
                forbid_passed_vks: Vec::new(),
                want_command: None,
            });
        }
    }

    if let Some(prefix) = only {
        cases.retain(|c| c.id.starts_with(prefix.as_str()));
    }
    cases
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

/// Commands the backend fired, as small codes in a fixed ring (the callback runs
/// on the hook thread: no lock, no allocation).
const CMD_RING: usize = 256;
const CMD_EMPTY: AtomicU32 = AtomicU32::new(0);
static CMDS: [AtomicU32; CMD_RING] = [CMD_EMPTY; CMD_RING];
static CMD_HEAD: AtomicU64 = AtomicU64::new(0);
static CMD_MARK: AtomicU64 = AtomicU64::new(0);

fn command_code(command: &BackendCommand) -> u32 {
    match command {
        BackendCommand::Enable(true) => 1,
        BackendCommand::Enable(false) => 2,
        BackendCommand::RestoreWord => 3,
        _ => 99,
    }
}

fn note_command(command: &BackendCommand) {
    let idx = CMD_HEAD.fetch_add(1, Ordering::SeqCst) as usize % CMD_RING;
    CMDS[idx].store(command_code(command), Ordering::SeqCst);
}

fn commands_clear() {
    CMD_MARK.store(CMD_HEAD.load(Ordering::SeqCst), Ordering::SeqCst);
}

/// Codes fired since the last [`commands_clear`].
fn commands_since_clear() -> Vec<u32> {
    let head = CMD_HEAD.load(Ordering::SeqCst);
    let from = CMD_MARK.load(Ordering::SeqCst).max(head.saturating_sub(CMD_RING as u64));
    (from..head)
        .map(|i| CMDS[i as usize % CMD_RING].load(Ordering::SeqCst))
        .collect()
}
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

#[derive(Default)]
struct Stats {
    fed: u64,
    injected: u64,
    passed: u64,
    swallowed: u64,
}

struct Driver {
    /// `Some` = in-process mode: the backend runtime driven directly.
    rt: Option<Box<HookRuntime>>,
    pace: Duration,
    shared: Arc<RwLock<AppConfig>>,
    applied: Option<Settings>,
    events: Vec<Ev>,
    cursor: u64,
    stats: Stats,
}

fn kbd(syn: &Syn, extra: usize) -> KBDLLHOOKSTRUCT {
    KBDLLHOOKSTRUCT {
        vkCode: if syn.unicode { u32::from(keys::VK_PACKET) } else { u32::from(syn.vk) },
        scanCode: u32::from(syn.scan),
        flags: KBDLLHOOKSTRUCT_FLAGS(if syn.up { 0x80 } else { 0 }),
        time: 0,
        dwExtraInfo: extra,
    }
}

impl Driver {
    fn apply(&mut self, settings: &Settings) {
        if self.applied.as_ref() != Some(settings) {
            match self.rt.as_mut() {
                Some(rt) => rt.apply_config(Arc::new(config_for(settings))),
                None => {
                    *self.shared.write() = config_for(settings);
                    vkb_windows::set_config(self.shared.clone());
                    thread::sleep(Duration::from_millis(80));
                }
            }
            self.applied = Some(settings.clone());
        }
        vkb_windows::set_enabled(true);
    }

    /// Live: wait until every harness event is accounted for and the stream is
    /// quiet. Returns the number of unaccounted events (0 = clean).
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

    /// Feed / send a run of events. `false` = the run must stop.
    fn send_syns(&mut self, syns: &[Syn]) -> bool {
        if ABORT.load(Ordering::SeqCst) {
            return false;
        }
        if self.rt.is_none() {
            let inputs: Vec<INPUT> = syns.iter().map(|s| to_input(s, HARNESS_TAG)).collect();
            send_batch(&inputs, true);
            thread::sleep(self.pace);
            if self.settle() != 0 {
                return false;
            }
            drain_ring(&mut self.cursor, &mut self.events);
            return true;
        }
        for syn in syns {
            let message = if syn.up { WM_KEYUP } else { WM_KEYDOWN };
            let info = kbd(syn, HARNESS_TAG);
            let rt = self.rt.as_mut().expect("in-process runtime");
            let swallow = vkb_windows::e2e::feed(rt, message, &info);
            self.stats.fed += 1;
            if swallow {
                self.stats.swallowed += 1;
            }
            for cap in vkb_windows::e2e::take_captured() {
                if cap.extra != TEST_TAG {
                    eprintln!("an injection carried tag {:#x}, not the test tag", cap.extra);
                    return false;
                }
                self.events.push(Ev {
                    src: Src::Injected,
                    vk: cap.vk,
                    scan: cap.scan,
                    down: !cap.up,
                });
                self.stats.injected += 1;
                // The runtime must ignore its own injection when it re-enters.
                let echo = KBDLLHOOKSTRUCT {
                    vkCode: u32::from(cap.vk),
                    scanCode: u32::from(cap.scan),
                    flags: KBDLLHOOKSTRUCT_FLAGS(if cap.up { 0x80 } else { 0 }),
                    time: 0,
                    dwExtraInfo: cap.extra,
                };
                let echo_msg = if cap.up { WM_KEYUP } else { WM_KEYDOWN };
                if rt.on_event(echo_msg, &echo).swallow {
                    eprintln!("the runtime reacted to its own injection");
                    return false;
                }
            }
            if !swallow {
                self.stats.passed += 1;
                self.events.push(Ev {
                    src: Src::Passed,
                    vk: info.vkCode as u16,
                    scan: info.scanCode as u16,
                    down: !syn.up,
                });
            }
        }
        true
    }

    fn send_step(&mut self, step: &Step) -> bool {
        match step {
            Step::Text(text) => {
                for ch in text.chars() {
                    if !self.send_syns(&char_syns(ch)) {
                        return false;
                    }
                }
                true
            }
            Step::Tap(vk) => self.send_syns(&tap_vk(*vk, false)),
            Step::Down(vk) => self.send_syns(&[vk_syn(*vk, false)]),
            Step::Up(vk) => self.send_syns(&[vk_syn(*vk, true)]),
            Step::Mouse => {
                if self.rt.is_none() {
                    return false;
                }
                vkb_windows::e2e::mouse_button_down();
                true
            }
        }
    }

    /// Run one case. Returns `(verdict, detail)`; `Err` means the run must stop.
    fn run(&mut self, case: &Case) -> Result<(bool, String), String> {
        self.apply(&case.settings);
        // A non-text key ends any half-typed word from the previous case.
        if !self.send_syns(&[vk_syn(VK_F24, false), vk_syn(VK_F24, true)]) {
            return Err(format!("{}: harness events unaccounted before the case", case.id));
        }
        self.events.clear();
        commands_clear();
        let start_leak = LOW_T.load(Ordering::SeqCst);

        for step in &case.steps {
            if !self.send_step(step) {
                return Err(format!("{}: harness events unaccounted for or send aborted", case.id));
            }
        }
        if LOW_T.load(Ordering::SeqCst) != start_leak {
            return Err(format!("{}: injected event escaped the top hook", case.id));
        }

        let (actual, _) = rebuild(&self.events);
        let mut ok = actual == case.expected;
        let mut detail = String::new();
        if !ok {
            let _ = write!(detail, "expected=[{}] actual=[{}]", case.expected, actual);
        }
        let reached: Vec<u16> = self
            .events
            .iter()
            .filter(|e| e.src == Src::Passed && e.down && case.forbid_passed_vks.contains(&e.vk))
            .map(|e| e.vk)
            .collect();
        if !reached.is_empty() {
            ok = false;
            let _ = write!(
                detail,
                " trigger-key-down-reached-app={:?} (would have hit the focused window)",
                reached.iter().map(|vk| format!("0x{vk:02X}")).collect::<Vec<_>>()
            );
        }
        if let Some(want) = &case.want_command {
            let fired = commands_since_clear();
            if !fired.contains(&command_code(want)) {
                ok = false;
                let _ = write!(detail, " command {want:?} not fired (codes fired: {fired:?})");
            }
        }
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

/// Live-mode safety gate. `Ok` only when every probe event is accounted for and
/// none escaped.
fn probe_live(report: &mut String) -> Result<(), String> {
    macro_rules! say {
        ($($arg:tt)*) => {{ let line = format!($($arg)*); println!("{line}"); report.push_str(&line); report.push('\n'); }};
    }
    say!("PROBE(live) start");
    let (h0, t0, hi_h0, hi_t0, lo_h0, lo_t0) = (
        SENT_H.load(Ordering::SeqCst),
        SENT_T.load(Ordering::SeqCst),
        HIGH_H.load(Ordering::SeqCst),
        HIGH_T.load(Ordering::SeqCst),
        LOW_H.load(Ordering::SeqCst),
        LOW_T.load(Ordering::SeqCst),
    );
    // Keys no application reacts to (F23 / F24).
    send_batch(
        &[
            to_input(&vk_syn(VK_F24, false), HARNESS_TAG),
            to_input(&vk_syn(VK_F24, true), HARNESS_TAG),
        ],
        true,
    );
    send_batch(
        &[
            to_input(&vk_syn(VK_F23, false), TEST_TAG),
            to_input(&vk_syn(VK_F23, true), TEST_TAG),
        ],
        false,
    );
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
        "PROBE sent: harness={sent_h} test={sent_t} | H_high: harness-passed={high_h} test-swallowed={high_t} | backend: seen={seen} swallowed={swallowed} | H_low: harness-swallowed={low_h} test-escaped={low_t}"
    );
    let unaccounted = (sent_h + sent_t).saturating_sub(high_t + swallowed + low_h);
    say!("PROBE escaped events (unaccounted + test-tag escapes) = {}", unaccounted + low_t);
    if !ok || unaccounted != 0 || low_t != 0 || sent_h != 2 || sent_t != 2 || high_h != 2 || high_t != 2 || low_h != 2 {
        return Err("probe: hook order or accounting wrong (see counters above)".to_string());
    }
    say!("PROBE hook order H_high -> backend -> H_low and swallow accounting: OK");
    Ok(())
}

fn default_settings() -> Settings {
    Settings {
        method: InputMethod::Telex,
        old_tone: false,
        no_jfwz: false,
        no_spell: false,
        no_free_tone: false,
        standard: false,
    }
}

fn say(report: &mut String, line: String) {
    println!("{line}");
    report.push_str(&line);
    report.push('\n');
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let live = args.iter().any(|a| a == "--live-hooks");
    let probe_only = args.iter().any(|a| a == "--probe-only");
    if probe_only && !live {
        eprintln!("--probe-only is a --live-hooks option");
        return ExitCode::from(2);
    }
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
    let _ = EPOCH.set(Instant::now());

    // Live hooks need the machine to themselves.
    if live {
        match other_vietkb_processes() {
            Err(err) => {
                eprintln!("refusing --live-hooks: {err}");
                return ExitCode::from(2);
            }
            Ok(found) if !found.is_empty() => {
                eprintln!(
                    "refusing --live-hooks: {} is running. A running VietKB refreshes its keyboard hook every \
                     30 s and can become the newest hook; it would compose the harness's keys and inject them \
                     into the focused window with the production marker, which this harness cannot catch. \
                     Use the default in-process mode, or quit it yourself first (this harness never stops it).",
                    found.join(", ")
                );
                return ExitCode::from(2);
            }
            Ok(_) => {}
        }
        unsafe {
            let _ = SetConsoleCtrlHandler(Some(ctrl_handler), true);
        }
    }

    // Preconditions: nothing held, caps lock off (the backend seeds its modifier
    // state from the OS at construction time). Read-only.
    let held = [0x10u16, 0x11, 0x12, 0x5B, 0x5C]
        .iter()
        .any(|vk| (unsafe { GetAsyncKeyState(i32::from(*vk)) } as u16) & 0x8000 != 0);
    let caps = (unsafe { GetKeyState(i32::from(keys::VK_CAPITAL)) } & 1) != 0;
    if held || caps {
        eprintln!("refusing to start: a modifier is held or Caps Lock is on (retry in a moment)");
        return ExitCode::from(2);
    }

    let shared: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(config_for(&default_settings())));
    vkb_windows::e2e::configure(HARNESS_TAG, TEST_TAG);
    vkb_windows::set_command_callback(|command| note_command(&command));

    let mut sandwich = Sandwich {
        high: None,
        low: None,
        backend_up: false,
    };
    let mut driver = Driver {
        rt: None,
        pace,
        shared: shared.clone(),
        applied: None,
        events: Vec::new(),
        cursor: 0,
        stats: Stats::default(),
    };

    if live {
        // Install order matters: H_low, backend, H_high (newest runs first).
        sandwich.low = Some(HookThread::start("h-low", low_proc));
        vkb_windows::set_config(shared.clone());
        if let Err(err) = vkb_windows::install() {
            eprintln!("backend hook did not install: {err}");
            return ExitCode::from(3);
        }
        sandwich.backend_up = true;
        thread::sleep(Duration::from_millis(100));
        sandwich.reinstall_high();
        thread::sleep(Duration::from_millis(100));
    } else {
        vkb_windows::e2e::set_capture(true);
        driver.rt = Some(Box::new(HookRuntime::new(
            Some(Arc::new(config_for(&default_settings()))),
            None,
        )));
        driver.applied = Some(default_settings());
    }
    LAST_FG.store(fg(), Ordering::SeqCst);

    let outcome = (|| -> Result<String, String> {
        sanity_check_tags()?;
        say(&mut report, "PROBE injection builders tag every event with the test tag: OK".to_string());
        if live {
            probe_live(&mut report)?;
            if probe_only {
                return Ok("PROBE-ONLY clean".to_string());
            }
        } else {
            say(&mut report, "MODE in-process: no hooks installed, no SendInput, zero global events".to_string());
        }

        // Engine probe: the first real letters, only after the gate passed.
        let probe_case = Case::text(
            "probe/engine".to_string(),
            default_settings(),
            "vieetj".to_string(),
            "việt".to_string(),
        );
        let (ok, detail) = driver.run(&probe_case)?;
        say(&mut report, format!("PROBE engine: {} {detail}", if ok { "OK" } else { "FAIL" }));
        if !ok {
            return Err("engine probe failed: pipeline broken".to_string());
        }

        let mut per_group: BTreeMap<String, (u32, u32)> = BTreeMap::new();
        let mut failing = Vec::new();
        for mode in &modes {
            let standard = mode == "standard";
            let cases = load_cases(&qa_root, &methods, standard, live, &only);
            for case in &cases {
                if ABORT.load(Ordering::SeqCst) {
                    return Err("aborted (ctrl-c or send failure)".to_string());
                }
                if live {
                    sandwich.reinstall_high();
                    thread::sleep(Duration::from_millis(20));
                }
                let (ok, detail) = driver.run(case)?;
                let parts: Vec<&str> = case.id.split('/').collect();
                let method = match case.settings.method {
                    InputMethod::Vni => "vni",
                    _ => "telex",
                };
                let group = format!("{}-{}-{}", parts[0], method, if standard { "standard" } else { "unicode" });
                let entry = per_group.entry(group).or_default();
                if ok {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                    failing.push(case.id.clone());
                }
                say(
                    &mut report,
                    format!("{} {} {}", if ok { "PASS" } else { "FAIL" }, case.id, detail),
                );
            }
        }

        // Negative control (in-process): with the pre-fix behaviour switched on,
        // the D-E2E-03 assertion must FAIL.
        let mut control_note = String::new();
        if !live {
            let cases = load_cases(&qa_root, &["telex".to_string()], false, false, &Some("hotkey/f1-enable-composes".to_string()));
            let control = cases.first().ok_or("negative control case missing")?;
            vkb_windows::e2e::set_legacy_trigger_leak(true);
            let result = driver.run(control);
            vkb_windows::e2e::set_legacy_trigger_leak(false);
            let (ok, detail) = result?;
            let caught = !ok && detail.contains("trigger-key-down-reached-app");
            say(
                &mut report,
                format!(
                    "CONTROL {} {} (pre-fix behaviour: {})",
                    if caught { "PASS" } else { "FAIL" },
                    control.id,
                    if caught { "the F1/F2 trigger key-down reached the app and the assertion failed, as it must" } else { "assertion did NOT detect the leak" }
                ),
            );
            if !caught {
                failing.push(format!("control/{}", control.id));
            }
            control_note = format!(" control={}", if caught { "caught" } else { "MISSED" });
        }

        let mut summary = String::from("SUMMARY");
        let (mut p, mut f) = (0, 0);
        for (group, (pass, fail)) in &per_group {
            let _ = write!(summary, " {group}={pass}/{}", pass + fail);
            p += pass;
            f += fail;
        }
        if live {
            let (_, swallowed) = vkb_windows::e2e::counters();
            let _ = write!(
                summary,
                " | PASS={p} FAIL={f} | mode=live sent_harness={} H_high_swallowed={} backend_swallowed={} H_low_swallowed={} escaped={} foreign_untouched={}",
                SENT_H.load(Ordering::SeqCst),
                HIGH_T.load(Ordering::SeqCst),
                swallowed,
                LOW_H.load(Ordering::SeqCst),
                LOW_T.load(Ordering::SeqCst),
                FOREIGN.load(Ordering::Relaxed),
            );
        } else {
            let _ = write!(
                summary,
                " | PASS={p} FAIL={f} |{control_note} mode=in-process events_fed={} injected_captured={} passed={} swallowed={} global_events_sent=0 hooks_installed=0",
                driver.stats.fed, driver.stats.injected, driver.stats.passed, driver.stats.swallowed,
            );
        }
        if !failing.is_empty() {
            let _ = write!(summary, "\nFAILING: {}", failing.join(", "));
        }
        Ok(summary)
    })();

    let fg_note = if live && fg() != LAST_FG.load(Ordering::SeqCst) {
        "foreground window changed during the run (nothing was delivered to it)"
    } else {
        "no focus interaction"
    };

    drop(sandwich);
    vkb_windows::e2e::set_capture(false);
    vkb_windows::e2e::reset();
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

    let failed = match &outcome {
        Err(_) => true,
        Ok(summary) => summary.contains("FAILING:"),
    };
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
