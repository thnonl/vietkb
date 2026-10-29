//! Opt-in keystroke trace (`VKB_DEBUG_KEYS=1`) → `<data dir>\keys.log`, where
//! the data directory is the one the app published with [`crate::set_data_dir`]
//! (channel-aware), or `%APPDATA%\VietKB` when the crate runs standalone.
//!
//! Only for diagnosis: it answers "what did the hook decide for this key, and
//! what did the injection call return?" without a debugger. Kept out of the hot
//! path's cost model:
//!
//! * `VKB_DEBUG_KEYS` is read once (`OnceLock`) into [`enabled`], a plain atomic.
//! * [`Trace`] holds plain fields; every setter starts with `if !armed { return }`
//!   so the off path neither formats nor allocates.
//! * The log file is opened lazily **on the thread that writes it** (the hook
//!   thread) into a `thread_local`. No lock is taken on the keystroke path, and
//!   a key is never delayed waiting for another thread.
//! * Each line is flushed immediately so a killed process still leaves evidence.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use vkb_core::{EngineAction, Modifiers};
use vkb_platform::plan::{Decision, Injection};
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;

use crate::input::SendResult;

static ENABLED: OnceLock<bool> = OnceLock::new();

thread_local! {
    /// Writer for the current thread. Only the hook thread ever touches it, so
    /// a `RefCell` is enough; nothing else can observe it mid-write.
    static WRITER: RefCell<Option<File>> = const { RefCell::new(None) };
}

/// `true` when `VKB_DEBUG_KEYS=1` was set at startup. One atomic load.
#[inline]
pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        matches!(std::env::var("VKB_DEBUG_KEYS"), Ok(v) if v == "1" || v.eq_ignore_ascii_case("true"))
    })
}

pub fn log_path() -> PathBuf {
    crate::paths::data_dir().join("keys.log")
}

/// Append one already-formatted line (adds the timestamp and newline).
fn write_line(prefix: &str, body: &str) {
    if !enabled() {
        return;
    }
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let line = format!("{millis} {prefix} {body}\n");
    WRITER.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            if let Some(parent) = log_path().parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            *slot = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path())
                .ok();
        }
        if let Some(file) = slot.as_mut() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    });
}

/// Startup/config line: proves what the runtime actually initialised from.
pub fn note(body: &str) {
    write_line("note", body);
}

/// Per-key record. Cheap to create and to finish when the channel is off.
pub struct Trace {
    armed: bool,
    msg: u32,
    vk: u16,
    scan: u16,
    down: bool,
    ch: Option<char>,
    mods: Modifiers,
    caps: bool,
    stage: &'static str,
    state: Option<&'static str>,
    action: Option<EngineAction>,
    send: Option<SendResult>,
}

impl Trace {
    pub fn new(msg: u32, info: &KBDLLHOOKSTRUCT) -> Self {
        Self {
            armed: enabled(),
            msg,
            vk: info.vkCode as u16,
            scan: info.scanCode as u16,
            down: matches!(msg, 0x0100 | 0x0104),
            ch: None,
            mods: Modifiers::NONE,
            caps: false,
            stage: "start",
            state: None,
            action: None,
            send: None,
        }
    }

    #[inline]
    pub fn stage(&mut self, stage: &'static str) {
        if self.armed {
            self.stage = stage;
        }
    }

    /// Why the key stopped being text (disabled / IME / layout / rule).
    #[inline]
    pub fn inactive(&mut self, reason: &'static str) {
        if self.armed {
            self.state = Some(reason);
        }
    }

    #[inline]
    pub fn mods(&mut self, mods: Modifiers, caps_lock: bool) {
        if self.armed {
            self.mods = mods;
            self.caps = caps_lock;
        }
    }

    #[inline]
    pub fn ch(&mut self, ch: Option<char>) {
        if self.armed {
            self.ch = ch;
        }
    }

    #[inline]
    pub fn action(&mut self, action: &EngineAction) {
        if self.armed {
            self.action = Some(action.clone());
        }
    }

    #[inline]
    pub fn send(&mut self, result: Option<SendResult>) {
        if self.armed {
            self.send = result;
        }
    }

    /// One line per key. Always called, whether the key passed or was swallowed.
    pub fn finish(self, decision: &Decision) {
        if !self.armed {
            return;
        }
        let mut body = String::with_capacity(220);
        let _ = write!(
            body,
            "{} vk=0x{:02X} scan=0x{:02X} msg=0x{:04X} ch={:?} mods={}{}{}{}{} stage={}",
            if self.down { "down" } else { "up" },
            self.vk,
            self.scan,
            self.msg,
            self.ch,
            if self.mods.shift { "S" } else { "-" },
            if self.mods.ctrl { "C" } else { "-" },
            if self.mods.alt { "A" } else { "-" },
            if self.mods.win { "W" } else { "-" },
            if self.caps { "L" } else { "-" },
            self.stage,
        );
        if let Some(reason) = self.state {
            let _ = write!(body, " inactive={reason}");
        }
        let _ = write!(body, " action={}", action_label(self.action.as_ref()));
        let _ = write!(body, " decision={}", decision_label(decision));
        match self.send {
            Some(send) => {
                let _ = write!(
                    body,
                    " send={}/{} lasterror={} attempts={}",
                    send.sent, send.total, send.error, send.attempts
                );
            }
            None => body.push_str(" send=-"),
        }
        write_line("key", &body);
    }
}

fn action_label(action: Option<&EngineAction>) -> String {
    match action {
        None => "none".to_string(),
        Some(EngineAction::Noop) => "Noop".to_string(),
        Some(EngineAction::Swallow) => "Swallow".to_string(),
        Some(EngineAction::ToggleEnabled) => "ToggleEnabled".to_string(),
        Some(EngineAction::Reject) => "Reject".to_string(),
        Some(EngineAction::Commit(text)) => format!("Commit({text:?})"),
        Some(EngineAction::Replace {
            delete_units,
            insert,
        }) => format!("Replace{{delete:{delete_units},insert:{insert:?}}}"),
        Some(EngineAction::ReplaceThenPass {
            delete_units,
            insert,
        }) => format!("ReplaceThenPass{{delete:{delete_units},insert:{insert:?}}}"),
    }
}

fn decision_label(decision: &Decision) -> String {
    let injection = match &decision.injection {
        Injection::None => "None".to_string(),
        Injection::Beep => "Beep".to_string(),
        Injection::Type(text) => format!("Type({text:?})"),
        Injection::Replace { deletes, insert } => {
            format!("Replace{{deletes:{deletes},insert:{insert:?}}}")
        }
    };
    format!("{{swallow:{},injection:{injection}}}", decision.swallow)
}
