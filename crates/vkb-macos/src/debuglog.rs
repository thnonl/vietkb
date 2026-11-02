//! Opt-in keystroke trace (`VKB_DEBUG_KEYS=1`) -> `<data dir>/keys.log`.
//! Counterpart of `vkb-windows/src/debuglog.rs`: same env var, same file name
//! in the app's data directory (`~/Library/Application Support/VietKB[ Dev]`,
//! published with `set_data_dir`), same `<unix millis> <kind> <body>` lines.
//!
//! Purpose: see the *ordering* of events around a replace ("gõ bằng" losing the
//! first letter of the next word): when the tap saw each event, what it decided,
//! and when every injected Backspace / text chunk was posted and through which
//! path.
//!
//! Cost model:
//!
//! * Off (default): [`enabled`] is one relaxed atomic load; nothing else runs,
//!   nothing is formatted or allocated, no thread exists.
//! * On: the tap thread builds a small `Copy` record (no heap) and `try_send`s
//!   it into a bounded channel. A full channel drops the record and bumps a
//!   counter that the writer reports as a `drop` line; the callback never waits.
//!   A writer thread formats, writes and flushes (when the queue runs dry, so a
//!   killed process still leaves its lines).
//! * Secure input (password fields): only a `secure` line, with no key code,
//!   character or text, is ever recorded.
//!
//! Line kinds (`t=` is monotonic microseconds, mach time since boot, the same
//! clock as the event timestamp, so `lag` is "tap saw it this long after the
//! OS created it"):
//!
//! ```text
//! 1730000000123 ev   t=… seq=7 down code=0x0B(11) uni="b" flags=0x0 src_pid=0 tag=physical repeat=0 lag=140us
//! 1730000000124 inj  t=… seq=7 text down code=0 uni="ằ" path=proxy post=18us
//! 1730000000125 dec  t=… seq=7 swallow replace{deletes:1,insert:"ằ"} stage=engine cb=210us
//! 1730000000126 !!TAP_DISABLED_BY_TIMEOUT t=… last_cb=4200000us
//! ```
//!
//! The file contains what was typed. It exists for diagnosis only: delete it
//! afterwards.

use std::fmt::{self, Write as _};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use vkb_platform::plan::{Decision, Injection};

use crate::state::{EventKind, MARKER};

/// `kCGEventSourceUserData` of the external e2e harness (`scripts/mac-e2e`).
pub const HARNESS_MARKER: i64 = 0x564B_4532_4545;

/// Records the tap thread may have queued before it starts dropping.
const QUEUE: usize = 4096;
/// Roll the file over once it is this large (kept as `keys.log.1`).
const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

static ENABLED: AtomicBool = AtomicBool::new(false);
static SECURE_NOW: AtomicBool = AtomicBool::new(false);
static CURRENT_SEQ: AtomicU32 = AtomicU32::new(0);
static LAST_CB_US: AtomicU64 = AtomicU64::new(0);

/// `true` when tracing is on. One relaxed load; the whole off-path cost.
#[inline(always)]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

// -- clock ------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod clock {
    use std::sync::OnceLock;

    #[repr(C)]
    struct Timebase {
        numer: u32,
        denom: u32,
    }
    extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut Timebase) -> i32;
    }

    fn timebase() -> (u64, u64) {
        static TB: OnceLock<(u64, u64)> = OnceLock::new();
        *TB.get_or_init(|| {
            let mut info = Timebase { numer: 0, denom: 0 };
            let ok = unsafe { mach_timebase_info(&mut info) } == 0 && info.denom != 0;
            if ok {
                (info.numer as u64, info.denom as u64)
            } else {
                (1, 1)
            }
        })
    }

    pub fn ticks_to_us(ticks: u64) -> u64 {
        let (numer, denom) = timebase();
        (ticks as u128 * numer as u128 / denom as u128 / 1000) as u64
    }

    pub fn now_us() -> u64 {
        ticks_to_us(unsafe { mach_absolute_time() })
    }
}

#[cfg(not(target_os = "macos"))]
mod clock {
    use std::sync::OnceLock;
    use std::time::Instant;

    pub fn ticks_to_us(ticks: u64) -> u64 {
        ticks / 1000
    }

    pub fn now_us() -> u64 {
        static START: OnceLock<Instant> = OnceLock::new();
        START.get_or_init(Instant::now).elapsed().as_micros() as u64
    }
}

/// Monotonic microseconds (mach time on macOS).
#[inline]
pub fn now_us() -> u64 {
    clock::now_us()
}

/// Convert a `CGEventTimestamp` (mach ticks) to the same microsecond clock.
#[inline]
pub fn event_ts_to_us(ticks: u64) -> u64 {
    clock::ticks_to_us(ticks)
}

// -- records ----------------------------------------------------------------

/// UTF-16 text in a fixed buffer: records stay `Copy` and heap-free.
#[derive(Clone, Copy)]
pub struct Units<const N: usize> {
    len: u8,
    cut: bool,
    buf: [u16; N],
}

impl<const N: usize> Units<N> {
    pub const fn empty() -> Self {
        Self {
            len: 0,
            cut: false,
            buf: [0; N],
        }
    }

    pub fn from_units(units: &[u16]) -> Self {
        let mut out = Self::empty();
        let n = units.len().min(N);
        out.buf[..n].copy_from_slice(&units[..n]);
        out.len = n as u8;
        out.cut = units.len() > N;
        out
    }

    pub fn from_text(text: &str) -> Self {
        let mut out = Self::empty();
        for unit in text.encode_utf16() {
            if (out.len as usize) == N {
                out.cut = true;
                break;
            }
            out.buf[out.len as usize] = unit;
            out.len += 1;
        }
        out
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Debug-quoted text, with `…` appended when it was cut.
    fn render(&self, out: &mut String) {
        let text = String::from_utf16_lossy(&self.buf[..self.len as usize]);
        if self.cut {
            let _ = write!(out, "{text:?}+…");
        } else {
            let _ = write!(out, "{text:?}");
        }
    }
}

/// What the engine decided for one event.
#[derive(Clone, Copy)]
pub enum Dec {
    Pass,
    Swallow,
    Beep,
    Type(Units<32>),
    Replace { deletes: u32, insert: Units<32> },
}

impl Dec {
    pub fn from_decision(decision: &Decision) -> Self {
        match &decision.injection {
            Injection::None if decision.swallow => Dec::Swallow,
            Injection::None => Dec::Pass,
            Injection::Beep => Dec::Beep,
            Injection::Type(text) => Dec::Type(Units::from_text(text)),
            Injection::Replace { deletes, insert } => Dec::Replace {
                deletes: *deletes,
                insert: Units::from_text(insert),
            },
        }
    }
}

/// Where an event came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// VietKB's own injection (our marker).
    Ours,
    /// The external e2e harness.
    Harness,
    /// A hardware keystroke: no marker, no source process.
    Physical,
    /// Another process posted it (no known marker).
    Other,
}

/// Classify an event by `kCGEventSourceUserData` and the source pid.
pub fn classify_source(user_data: i64, source_pid: i64) -> Source {
    if user_data == MARKER {
        Source::Ours
    } else if user_data == HARNESS_MARKER {
        Source::Harness
    } else if user_data == 0 && source_pid == 0 {
        Source::Physical
    } else {
        Source::Other
    }
}

impl Source {
    fn label(self) -> &'static str {
        match self {
            Source::Ours => "ours",
            Source::Harness => "harness",
            Source::Physical => "physical",
            Source::Other => "other",
        }
    }
}

/// Which API an injected event was handed to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostPath {
    Proxy,
    Hid,
    Session,
}

impl PostPath {
    fn label(self) -> &'static str {
        match self {
            PostPath::Proxy => "proxy",
            PostPath::Hid => "hid",
            PostPath::Session => "session",
        }
    }
}

/// An event as the tap received it.
#[derive(Clone, Copy)]
pub struct EvIn {
    pub t_us: u64,
    /// Time between the OS creating the event and the tap seeing it; `None`
    /// when the event carries no timestamp.
    pub lag_us: Option<u64>,
    pub seq: u32,
    pub kind: EventKind,
    pub keycode: u16,
    pub uni: Units<8>,
    pub flags: u64,
    pub source_pid: i64,
    pub target_pid: i64,
    pub user_data: i64,
    pub autorepeat: bool,
}

#[derive(Clone, Copy)]
pub enum Rec {
    Note {
        buf: Units<96>,
    },
    Ev(EvIn),
    /// One injected key event.
    Inj {
        t_us: u64,
        seq: u32,
        keycode: u16,
        down: bool,
        text: Units<16>,
        path: PostPath,
        post_us: u64,
    },
    Dec {
        t_us: u64,
        seq: u32,
        dec: Dec,
        stage: &'static str,
        cb_us: u64,
    },
    /// Secure input was on: nothing about the key is recorded.
    Secure { t_us: u64, cb_us: u64 },
    TapDisabled {
        t_us: u64,
        timeout: bool,
        last_cb_us: u64,
    },
}

// -- formatting (writer thread) ----------------------------------------------

fn kind_label(kind: EventKind) -> &'static str {
    match kind {
        EventKind::KeyDown => "down",
        EventKind::KeyUp => "up",
        EventKind::FlagsChanged => "flags",
        EventKind::MouseDown => "mouse",
    }
}

fn dec_label(dec: &Dec, out: &mut String) {
    match dec {
        Dec::Pass => out.push_str("passthrough"),
        Dec::Swallow => out.push_str("swallow"),
        Dec::Beep => out.push_str("swallow beep"),
        Dec::Type(text) => {
            out.push_str("swallow type{");
            text.render(out);
            out.push('}');
        }
        Dec::Replace { deletes, insert } => {
            let _ = write!(out, "swallow replace{{deletes:{deletes},insert:");
            insert.render(out);
            out.push('}');
        }
    }
}

/// Format one record (without timestamp prefix) into `(kind, body)`.
pub fn format_rec(rec: &Rec, body: &mut String) -> &'static str {
    match rec {
        Rec::Note { buf } => {
            let text = String::from_utf16_lossy(&buf.buf[..buf.len as usize]);
            body.push_str(&text);
            "note"
        }
        Rec::Ev(ev) => {
            let _ = write!(
                body,
                "t={} seq={} {}",
                ev.t_us,
                ev.seq,
                kind_label(ev.kind)
            );
            if ev.kind != EventKind::MouseDown {
                let _ = write!(body, " code=0x{:02X}({})", ev.keycode, ev.keycode);
                body.push_str(" uni=");
                ev.uni.render(body);
                let _ = write!(body, " flags=0x{:X}", ev.flags);
            }
            let _ = write!(
                body,
                " src_pid={} tag={}",
                ev.source_pid,
                classify_source(ev.user_data, ev.source_pid).label()
            );
            if ev.target_pid != 0 {
                let _ = write!(body, " target_pid={}", ev.target_pid);
            }
            if classify_source(ev.user_data, ev.source_pid) == Source::Other
                && ev.user_data != 0
            {
                let _ = write!(body, " ud=0x{:X}", ev.user_data);
            }
            let _ = write!(body, " repeat={}", u8::from(ev.autorepeat));
            match ev.lag_us {
                Some(lag) => {
                    let _ = write!(body, " lag={lag}us");
                }
                None => body.push_str(" lag=-"),
            }
            "ev"
        }
        Rec::Inj {
            t_us,
            seq,
            keycode,
            down,
            text,
            path,
            post_us,
        } => {
            let what = if !text.is_empty() {
                "text"
            } else if *keycode == crate::keys::MAC_BACKSPACE {
                "bs"
            } else {
                "key"
            };
            let _ = write!(
                body,
                "t={t_us} seq={seq} {what} {} code={keycode}",
                if *down { "down" } else { "up" }
            );
            if !text.is_empty() {
                body.push_str(" uni=");
                text.render(body);
            }
            let _ = write!(body, " path={} post={post_us}us", path.label());
            "inj"
        }
        Rec::Dec {
            t_us,
            seq,
            dec,
            stage,
            cb_us,
        } => {
            let _ = write!(body, "t={t_us} seq={seq} ");
            dec_label(dec, body);
            if !stage.is_empty() {
                let _ = write!(body, " stage={stage}");
            }
            let _ = write!(body, " cb={cb_us}us");
            "dec"
        }
        Rec::Secure { t_us, cb_us } => {
            let _ = write!(body, "t={t_us} cb={cb_us}us");
            "secure"
        }
        Rec::TapDisabled {
            t_us,
            timeout,
            last_cb_us,
        } => {
            let _ = write!(
                body,
                "t={t_us} tap was switched off by the system; events were lost; previous callback took {last_cb_us}us"
            );
            if *timeout {
                "!!TAP_DISABLED_BY_TIMEOUT"
            } else {
                "!!TAP_DISABLED_BY_USER_INPUT"
            }
        }
    }
}

/// Wall-clock milliseconds for a record stamped `t_us` on the monotonic clock.
fn wall_ms(base_wall_ms: u64, base_us: u64, t_us: u64) -> u64 {
    if t_us >= base_us {
        base_wall_ms + (t_us - base_us) / 1000
    } else {
        base_wall_ms.saturating_sub((base_us - t_us) / 1000)
    }
}

fn rec_time(rec: &Rec) -> Option<u64> {
    match rec {
        Rec::Note { .. } => None,
        Rec::Ev(ev) => Some(ev.t_us),
        Rec::Inj { t_us, .. }
        | Rec::Dec { t_us, .. }
        | Rec::Secure { t_us, .. }
        | Rec::TapDisabled { t_us, .. } => Some(*t_us),
    }
}

/// Writer loop: drain the queue, write, flush when idle. Returns when every
/// sender is gone. Generic over the sink so it is tested without a file.
fn run_writer<W: std::io::Write>(rx: Receiver<Rec>, dropped: Arc<AtomicU64>, mut out: W) {
    let base_wall = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let base_us = now_us();
    let mut line = String::with_capacity(256);
    let mut body = String::with_capacity(256);
    let emit = |out: &mut W, wall: u64, kind: &str, body: &str, line: &mut String| {
        line.clear();
        let _ = writeln!(line, "{wall} {kind} {body}");
        let _ = out.write_all(line.as_bytes());
    };
    loop {
        let rec = match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(rec) => Some(rec),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        let mut next = rec;
        while let Some(rec) = next {
            body.clear();
            let kind = format_rec(&rec, &mut body);
            let wall = rec_time(&rec).map_or_else(
                || wall_ms(base_wall, base_us, now_us()),
                |t| wall_ms(base_wall, base_us, t),
            );
            emit(&mut out, wall, kind, &body, &mut line);
            next = rx.try_recv().ok();
        }
        let lost = dropped.swap(0, Ordering::Relaxed);
        if lost > 0 {
            let wall = wall_ms(base_wall, base_us, now_us());
            emit(
                &mut out,
                wall,
                "drop",
                &format!("n={lost} records dropped: the writer fell behind"),
                &mut line,
            );
        }
        let _ = out.flush();
    }
    let _ = out.flush();
}

// -- channel ------------------------------------------------------------------

struct Pipe {
    tx: SyncSender<Rec>,
    dropped: Arc<AtomicU64>,
}

impl Pipe {
    #[inline]
    fn emit(&self, rec: Rec) {
        if self.tx.try_send(rec).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

static PIPE: OnceLock<Pipe> = OnceLock::new();

#[inline]
fn emit(rec: Rec) {
    if let Some(pipe) = PIPE.get() {
        pipe.emit(rec);
    }
}

/// `true` for the values of `VKB_DEBUG_KEYS` that switch the trace on.
fn env_on(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Where `keys.log` lives: the app's data directory, else the standard one.
pub fn log_path() -> PathBuf {
    let dir = crate::globals::DATA_DIR.lock().clone().unwrap_or_else(|| {
        std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(|home| {
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("VietKB")
            })
            .unwrap_or_else(std::env::temp_dir)
    });
    dir.join("keys.log")
}

fn open_log(path: &PathBuf) -> Option<std::fs::File> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::metadata(path).is_ok_and(|meta| meta.len() > MAX_FILE_BYTES) {
        let _ = std::fs::rename(path, path.with_extension("log.1"));
    }
    let mut options = std::fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).ok()
}

/// Read `VKB_DEBUG_KEYS` once and, when it is on, start the writer. Idempotent;
/// call it before the tap is created (it is, from `sys::install`).
pub fn init() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if !env_on(std::env::var("VKB_DEBUG_KEYS").ok().as_deref()) {
            return;
        }
        let (tx, rx) = sync_channel(QUEUE);
        let dropped = Arc::new(AtomicU64::new(0));
        let writer_dropped = dropped.clone();
        let spawned = std::thread::Builder::new()
            .name("vkb-keys-log".into())
            .spawn(move || {
                let Some(file) = open_log(&log_path()) else {
                    tracing::warn!("VKB_DEBUG_KEYS: cannot open keys.log; trace disabled");
                    // Keep draining so the tap's try_send keeps succeeding.
                    while rx.recv().is_ok() {}
                    return;
                };
                run_writer(rx, writer_dropped, std::io::BufWriter::new(file));
            });
        if spawned.is_ok() {
            let _ = PIPE.set(Pipe { tx, dropped });
            ENABLED.store(true, Ordering::Relaxed);
            note(format_args!(
                "start pid={} VKB_DEBUG_KEYS=1 (this file records typed characters; delete it after diagnosis)",
                std::process::id()
            ));
        }
    });
}

// -- tap-thread API (every call is a no-op while off) -----------------------------

struct StackBuf {
    units: Units<96>,
}

impl fmt::Write for StackBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for unit in s.encode_utf16() {
            if (self.units.len as usize) == 96 {
                self.units.cut = true;
                return Ok(());
            }
            self.units.buf[self.units.len as usize] = unit;
            self.units.len += 1;
        }
        Ok(())
    }
}

/// A free-form line (startup, input-source changes). At most 96 UTF-16 units.
pub fn note(args: fmt::Arguments<'_>) {
    if !enabled() {
        return;
    }
    let mut buf = StackBuf {
        units: Units::empty(),
    };
    let _ = buf.write_fmt(args);
    emit(Rec::Note { buf: buf.units });
}

/// Next event sequence number; also the one injected steps are filed under.
pub fn next_seq() -> u32 {
    CURRENT_SEQ.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
}

/// The tap received an event. `secure` is whether secure input is on: then
/// nothing about the event is recorded here.
pub fn event_in(ev: EvIn, secure: bool) {
    SECURE_NOW.store(secure, Ordering::Relaxed);
    if secure {
        return;
    }
    emit(Rec::Ev(ev));
}

/// An injected key event was posted (no-op while secure input is on).
pub fn injected(
    t_us: u64,
    keycode: u16,
    down: bool,
    text: Option<&[u16]>,
    path: PostPath,
    post_us: u64,
) {
    if SECURE_NOW.load(Ordering::Relaxed) {
        return;
    }
    emit(Rec::Inj {
        t_us,
        seq: CURRENT_SEQ.load(Ordering::Relaxed),
        keycode,
        down,
        text: text.map_or_else(Units::empty, Units::from_units),
        path,
        post_us,
    });
}

/// The callback is done with the event.
pub fn event_out(seq: u32, t0_us: u64, dec: Option<&Decision>, stage: &'static str, secure: bool) {
    let now = now_us();
    let cb_us = now.saturating_sub(t0_us);
    LAST_CB_US.store(cb_us, Ordering::Relaxed);
    if secure {
        emit(Rec::Secure { t_us: now, cb_us });
        return;
    }
    emit(Rec::Dec {
        t_us: now,
        seq,
        dec: dec.map_or(Dec::Pass, Dec::from_decision),
        stage,
        cb_us,
    });
}

/// The system disabled the tap (`kCGEventTapDisabledByTimeout` / `...ByUserInput`).
pub fn tap_disabled(timeout: bool) {
    emit(Rec::TapDisabled {
        t_us: now_us(),
        timeout,
        last_cb_us: LAST_CB_US.load(Ordering::Relaxed),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(kind: EventKind, code: u16, uni: &str, ud: i64, pid: i64) -> EvIn {
        EvIn {
            t_us: 1_000_500,
            lag_us: Some(140),
            seq: 7,
            kind,
            keycode: code,
            uni: Units::from_text(uni),
            flags: 0,
            source_pid: pid,
            target_pid: 0,
            user_data: ud,
            autorepeat: false,
        }
    }

    fn line(rec: Rec) -> String {
        let mut body = String::new();
        let kind = format_rec(&rec, &mut body);
        format!("{kind} {body}")
    }

    #[test]
    fn env_values() {
        assert!(env_on(Some("1")));
        assert!(env_on(Some("TRUE")));
        assert!(!env_on(Some("0")));
        assert!(!env_on(Some("")));
        assert!(!env_on(None));
    }

    #[test]
    fn sources_are_told_apart() {
        assert_eq!(classify_source(0, 0), Source::Physical);
        assert_eq!(classify_source(MARKER, 1234), Source::Ours);
        assert_eq!(classify_source(HARNESS_MARKER, 99), Source::Harness);
        assert_eq!(classify_source(0, 99), Source::Other);
        assert_eq!(classify_source(5, 0), Source::Other);
    }

    #[test]
    fn a_replace_reads_in_order_ev_inj_dec() {
        let lines = [
            line(Rec::Ev(ev(EventKind::KeyDown, 0x0B, "b", 0, 0))),
            line(Rec::Inj {
                t_us: 1_000_620,
                seq: 7,
                keycode: crate::keys::MAC_BACKSPACE,
                down: true,
                text: Units::empty(),
                path: PostPath::Proxy,
                post_us: 12,
            }),
            line(Rec::Inj {
                t_us: 1_000_650,
                seq: 7,
                keycode: 0,
                down: true,
                text: Units::from_text("ằ"),
                path: PostPath::Proxy,
                post_us: 18,
            }),
            line(Rec::Dec {
                t_us: 1_000_710,
                seq: 7,
                dec: Dec::Replace {
                    deletes: 1,
                    insert: Units::from_text("ằ"),
                },
                stage: "engine",
                cb_us: 210,
            }),
        ];
        assert_eq!(
            lines[0],
            "ev t=1000500 seq=7 down code=0x0B(11) uni=\"b\" flags=0x0 src_pid=0 tag=physical repeat=0 lag=140us"
        );
        assert_eq!(
            lines[1],
            "inj t=1000620 seq=7 bs down code=51 path=proxy post=12us"
        );
        assert_eq!(
            lines[2],
            "inj t=1000650 seq=7 text down code=0 uni=\"ằ\" path=proxy post=18us"
        );
        assert_eq!(
            lines[3],
            "dec t=1000710 seq=7 swallow replace{deletes:1,insert:\"ằ\"} stage=engine cb=210us"
        );
    }

    #[test]
    fn our_echo_and_foreign_events_are_tagged() {
        let ours = line(Rec::Ev(ev(EventKind::KeyUp, 51, "", MARKER, 4321)));
        assert!(ours.contains("tag=ours"), "{ours}");
        let other = line(Rec::Ev(ev(EventKind::KeyDown, 0, "x", 0x77, 4321)));
        assert!(other.contains("tag=other") && other.contains("ud=0x77"), "{other}");
        let mouse = line(Rec::Ev(ev(EventKind::MouseDown, 0, "", 0, 0)));
        assert!(mouse.starts_with("ev t=") && !mouse.contains("code="), "{mouse}");
    }

    #[test]
    fn tap_disabled_is_loud_and_secure_carries_no_key() {
        let timeout = line(Rec::TapDisabled {
            t_us: 5,
            timeout: true,
            last_cb_us: 4_200_000,
        });
        assert!(timeout.starts_with("!!TAP_DISABLED_BY_TIMEOUT"), "{timeout}");
        assert!(timeout.contains("4200000us"));
        let user = line(Rec::TapDisabled {
            t_us: 5,
            timeout: false,
            last_cb_us: 0,
        });
        assert!(user.starts_with("!!TAP_DISABLED_BY_USER_INPUT"));
        assert_eq!(line(Rec::Secure { t_us: 9, cb_us: 3 }), "secure t=9 cb=3us");
    }

    #[test]
    fn long_text_is_cut_not_overflowed() {
        let long = "ă".repeat(40);
        let text = line(Rec::Dec {
            t_us: 1,
            seq: 1,
            dec: Dec::Type(Units::from_text(&long)),
            stage: "",
            cb_us: 1,
        });
        assert!(text.contains("+…"), "{text}");
    }

    #[test]
    fn a_full_queue_drops_and_counts_without_blocking() {
        let (tx, rx) = sync_channel(2);
        let pipe = Pipe {
            tx,
            dropped: Arc::new(AtomicU64::new(0)),
        };
        for n in 0..5 {
            pipe.emit(Rec::Secure { t_us: n, cb_us: 0 });
        }
        assert_eq!(pipe.dropped.load(Ordering::Relaxed), 3);

        // The writer drains what was queued and reports the loss.
        let dropped = pipe.dropped.clone();
        drop(pipe);
        let sink = SharedSink::default();
        run_writer(rx, dropped, sink.clone());
        let text = sink.text();
        assert_eq!(text.matches(" secure t=").count(), 2, "{text}");
        assert!(text.contains(" drop n=3 "), "{text}");
    }

    #[test]
    fn secure_input_queues_only_secure_lines() {
        // Install a real pipe (the same one `init` builds) without the env var.
        let (tx, rx) = sync_channel(64);
        let installed = PIPE
            .set(Pipe {
                tx,
                dropped: Arc::new(AtomicU64::new(0)),
            })
            .is_ok();
        assert!(installed, "no other test installs the pipe");

        event_in(ev(EventKind::KeyDown, 0x0B, "b", 0, 0), true);
        assert!(SECURE_NOW.load(Ordering::Relaxed));
        injected(1, 0, true, Some(&[0x62]), PostPath::Proxy, 1);
        injected(2, crate::keys::MAC_BACKSPACE, true, None, PostPath::Proxy, 1);
        event_out(7, now_us(), Some(&Decision::SWALLOW), "engine", true);
        let queued: Vec<Rec> = rx.try_iter().collect();
        assert_eq!(queued.len(), 1, "exactly the secure line");
        assert!(matches!(queued[0], Rec::Secure { .. }));

        // Back to normal: the same calls now queue ev / inj / dec.
        event_in(ev(EventKind::KeyDown, 0x0B, "b", 0, 0), false);
        assert!(!SECURE_NOW.load(Ordering::Relaxed));
        injected(3, 0, true, Some(&[0x62]), PostPath::Proxy, 1);
        event_out(7, now_us(), Some(&Decision::SWALLOW), "engine", false);
        let kinds: Vec<&str> = rx
            .try_iter()
            .map(|rec| match rec {
                Rec::Ev(_) => "ev",
                Rec::Inj { .. } => "inj",
                Rec::Dec { .. } => "dec",
                _ => "other",
            })
            .collect();
        assert_eq!(kinds, ["ev", "inj", "dec"]);
    }

    #[derive(Clone, Default)]
    struct SharedSink(Arc<parking_lot::Mutex<Vec<u8>>>);

    impl SharedSink {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().clone()).unwrap()
        }
    }

    impl std::io::Write for SharedSink {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn the_writer_stamps_wall_time_from_the_record_clock() {
        assert_eq!(wall_ms(10_000, 1_000_000, 1_002_500), 10_002);
        assert_eq!(wall_ms(10_000, 1_000_000, 998_000), 9_998);
    }
}
