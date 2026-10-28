//! Opt-in latency probe for budget **P2**: keystroke → injection return.
//!
//! A *sample* is the wall time from entering the low-level hook callback for a
//! key to the moment the injection call (`SendInput`, one batched array) returns
//! for that same key. That is the part of the path this project controls; how
//! fast the target app paints the glyph afterwards is not observable from here
//! and is deliberately outside the budget's numerator.
//!
//! ## Cost when the probe is off
//!
//! `VKB_PERF` is read **once**, at [`init_from_env`] (hook install / app start)
//! and cached in [`ENABLED`]. The keystroke path then pays:
//!
//! * one atomic load of [`ENABLED`] and the branch it guards ([`begin`]);
//! * [`end`]: a thread-local `Cell::take` that is already `None`, so no clock
//!   read, no lock and no allocation happen.
//!
//! ## Cost when the probe is on
//!
//! Two `Instant::now()` calls (QPC-backed, no allocation), one uncontended
//! `parking_lot::Mutex` around a ring whose `Vec` was reserved at startup, and
//! one `u32` push. Overflow is dropped, never reallocated, so a pathological
//! typing session cannot make the hook thread allocate.
//!
//! ## Output
//!
//! [`finish`] writes `%APPDATA%\VietKB\perf-<timestamp>.tsv` (one microsecond
//! value per row, `#` header) and logs count/p50/p95/p99/max plus a verdict, so
//! a run answers the budget without extra tooling. The procedure that consumes
//! this file is `docs/evkey-clone-work/81-p2-procedure.md`.

use std::cell::Cell;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use parking_lot::Mutex;

use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;

/// Fixed ring capacity. A 500-word paragraph is ~3k keystrokes per app; 16k
/// covers both sessions plus retries without a single reallocation.
const CAPACITY: usize = 16 * 1024;

/// A mark older than this means the key never reached an injection call (or the
/// thread-local was left behind); such a sample is not a keystroke latency.
const MAX_SAMPLE: Duration = Duration::from_secs(2);

/// Below this many samples the percentiles are noise, not a verdict.
const MIN_SAMPLES_FOR_VERDICT: usize = 100;

/// P2 budgets, microseconds.
const BUDGET_P95_US: u32 = 8_000;
const BUDGET_P99_US: u32 = 15_000;

/// Cached `VKB_PERF=1`. Written once at init, read once per keystroke.
static ENABLED: AtomicBool = AtomicBool::new(false);
static INIT: OnceLock<()> = OnceLock::new();
static RING: OnceLock<Mutex<Ring>> = OnceLock::new();
static META: OnceLock<Meta> = OnceLock::new();
static FLUSHED: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Start tick of the keystroke currently inside the hook callback. Only the
    /// hook thread ever writes it; `None` whenever the probe is disabled.
    static MARK: Cell<Option<Instant>> = const { Cell::new(None) };
}

struct Ring {
    samples: Vec<u32>,
    /// Samples that were in range (including the ones dropped on overflow).
    total: u64,
    dropped: u64,
}

struct Meta {
    started: SystemTime,
    version: String,
    profile: &'static str,
    layout: String,
}

/// One run's numbers, all in microseconds except the counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    /// Samples actually kept in the ring.
    pub count: usize,
    /// In-range samples lost to a full ring.
    pub dropped: u64,
    /// In-range samples seen in total (`count + dropped`).
    pub total: u64,
    pub p50: u32,
    pub p95: u32,
    pub p99: u32,
    pub max: u32,
}

impl Summary {
    /// P2: p95 ≤ 8 ms and p99 ≤ 15 ms. Only meaningful with enough samples.
    pub fn passes(&self) -> bool {
        self.count >= MIN_SAMPLES_FOR_VERDICT
            && self.p95 <= BUDGET_P95_US
            && self.p99 <= BUDGET_P99_US
    }
}

/// Read `VKB_PERF` once and arm the probe. Idempotent; returns whether the probe
/// is on. Call it at startup (the hook does, and so does the app).
pub fn init_from_env() -> bool {
    INIT.get_or_init(|| {
        let on = matches!(std::env::var("VKB_PERF"), Ok(v) if v == "1" || v.eq_ignore_ascii_case("true"));
        if !on {
            return;
        }
        let ring = Ring {
            samples: Vec::with_capacity(CAPACITY),
            total: 0,
            dropped: 0,
        };
        let _ = RING.set(Mutex::new(ring));
        let meta = Meta {
            started: SystemTime::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            profile: if cfg!(debug_assertions) { "debug" } else { "release" },
            layout: layout_label(),
        };
        tracing::info!(
            target: "vkb_windows::perf",
            capacity = CAPACITY,
            build = meta.profile,
            version = %meta.version,
            layout = %meta.layout,
            "perf probe active (VKB_PERF=1)"
        );
        let _ = META.set(meta);
        // Last, and with `Release` so the ring/meta writes are visible to any
        // thread that observes the flag.
        ENABLED.store(true, Ordering::Release);
    });
    enabled()
}

/// `true` when the probe is armed. The single flag the hot path consults.
#[inline]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

/// Stamp the start of the keystroke. Callers gate it on [`enabled`], so the
/// off-path never reaches this function: no clock read, no allocation.
#[inline]
pub(crate) fn begin() {
    let _ = MARK.try_with(|mark| mark.set(Some(Instant::now())));
}

/// Close the sample after the injection call returned.
///
/// Off-path: the thread-local is `None`, so nothing else runs.
#[inline]
pub(crate) fn end() {
    let _ = MARK.try_with(|mark| {
        if let Some(start) = mark.take() {
            push(start.elapsed());
        }
    });
}

/// Append one sample. Runs inside the hook callback: no allocation is possible
/// because the vector was reserved to `CAPACITY` at startup.
#[inline]
fn push(elapsed: Duration) {
    if elapsed > MAX_SAMPLE {
        // Stale mark, not a keystroke latency.
        return;
    }
    let micros = elapsed.as_micros().min(u32::MAX as u128) as u32;
    let Some(ring) = RING.get() else {
        return;
    };
    let mut ring = ring.lock();
    ring.total += 1;
    if ring.samples.len() < CAPACITY {
        ring.samples.push(micros);
    } else {
        ring.dropped += 1;
    }
}

/// Current numbers without touching the ring's contents.
pub fn snapshot() -> Option<Summary> {
    collect().map(|(summary, _)| summary)
}

/// Sorted copy of the samples plus the derived percentiles.
fn collect() -> Option<(Summary, Vec<u32>)> {
    let ring = RING.get()?.lock();
    let mut samples = ring.samples.clone();
    let total = ring.total;
    let dropped = ring.dropped;
    drop(ring);
    samples.sort_unstable();
    let summary = Summary {
        count: samples.len(),
        dropped,
        total,
        p50: percentile(&samples, 50),
        p95: percentile(&samples, 95),
        p99: percentile(&samples, 99),
        max: samples.last().copied().unwrap_or(0),
    };
    Some((summary, samples))
}

/// Nearest-rank percentile: the smallest value at or above `pct`% of the data.
fn percentile(sorted: &[u32], pct: u32) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (u64::from(pct) * sorted.len() as u64).div_ceil(100);
    let index = rank.saturating_sub(1).min(sorted.len() as u64 - 1) as usize;
    sorted[index]
}

/// Write the TSV and log the verdict. Safe to call more than once; only the
/// first call in a process writes a file. No-op when the probe is off.
pub fn finish() -> Option<PathBuf> {
    if !ENABLED.load(Ordering::Acquire) {
        return None;
    }
    if FLUSHED.swap(true, Ordering::SeqCst) {
        return None;
    }
    match write_report() {
        Ok(path) => Some(path),
        Err(err) => {
            FLUSHED.store(false, Ordering::SeqCst);
            tracing::warn!(target: "vkb_windows::perf", error = %err, "cannot write the perf report");
            None
        }
    }
}

fn write_report() -> Result<PathBuf> {
    let meta = META.get().ok_or_else(|| anyhow!("probe metadata missing"))?;
    let (summary, samples) = collect().ok_or_else(|| anyhow!("sample ring missing"))?;

    let path = output_path(meta.started);
    let mut body = String::with_capacity(samples.len() * 6 + 512);
    body.push_str("# VietKB P2 keystroke-latency probe\n");
    body.push_str("# sample = low-level hook callback entry -> injection (SendInput) return, microseconds\n");
    let _ = writeln!(body, "# build: {}", meta.profile);
    let _ = writeln!(body, "# version: {}", meta.version);
    let _ = writeln!(body, "# keyboard_layout: {}", meta.layout);
    let _ = writeln!(body, "# started_utc: {}", iso_utc(meta.started));
    let _ = writeln!(body, "# count: {}", summary.count);
    let _ = writeln!(body, "# dropped: {}", summary.dropped);
    let _ = writeln!(body, "# p50: {} us", summary.p50);
    let _ = writeln!(body, "# p95: {} us", summary.p95);
    let _ = writeln!(body, "# p99: {} us", summary.p99);
    let _ = writeln!(body, "# max: {} us", summary.max);
    body.push_str("# verdict: ");
    body.push_str(if summary.count < MIN_SAMPLES_FOR_VERDICT {
        "insufficient-samples"
    } else if summary.passes() {
        "PASS"
    } else {
        "FAIL"
    });
    body.push('\n');
    body.push_str("# column: micros\n");
    for sample in &samples {
        let _ = writeln!(body, "{sample}");
    }

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, body)?;

    if summary.count < MIN_SAMPLES_FOR_VERDICT {
        tracing::warn!(
            target: "vkb_windows::perf",
            count = summary.count,
            "P2: too few samples for a verdict"
        );
    }
    tracing::info!(
        target: "vkb_windows::perf",
        count = summary.count,
        p50_us = summary.p50,
        p95_us = summary.p95,
        p99_us = summary.p99,
        max_us = summary.max,
        dropped = summary.dropped,
        path = %path.display(),
        verdict = if summary.passes() { "PASS" } else { "FAIL" },
        "P2 keystroke -> injection latency"
    );
    if summary.dropped > 0 {
        tracing::warn!(
            target: "vkb_windows::perf",
            dropped = summary.dropped,
            "P2: sample ring overflowed; the run is still usable but the tail may be short"
        );
    }
    Ok(path)
}

fn output_path(started: SystemTime) -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("VietKB")
        .join(format!("perf-{}.tsv", compact_utc(started)))
}

/// Layout the probe started under, e.g. `0x0409 (en-US)`. Sampled once so a
/// mid-session layout switch cannot silently mix two layouts into one file.
fn layout_label() -> String {
    let hkl = unsafe { GetKeyboardLayout(0) };
    let langid = (hkl.0 as usize) & 0xFFFF;
    format!("0x{langid:04X} ({})", language_tag(langid as u16))
}

fn language_tag(langid: u16) -> &'static str {
    match langid & 0x3FF {
        0x09 => "en-US",
        0x2A => "vi-VN",
        0x1E => "th-TH",
        0x04 => "zh-CN",
        0x11 => "ja-JP",
        0x12 => "ko-KR",
        0x07 => "de-DE",
        0x0C => "fr-FR",
        0x0A => "es-ES",
        0x10 => "it-IT",
        0x19 => "ru-RU",
        _ => "unknown",
    }
}

struct Utc {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

fn utc(at: SystemTime) -> Utc {
    let secs = at.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    Utc {
        year,
        month,
        day,
        hour: (rem / 3_600) as u32,
        minute: ((rem % 3_600) / 60) as u32,
        second: (rem % 60) as u32,
    }
}

fn compact_utc(at: SystemTime) -> String {
    let t = utc(at);
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        t.year, t.month, t.day, t.hour, t.minute, t.second
    )
}

fn iso_utc(at: SystemTime) -> String {
    let t = utc(at);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        t.year, t.month, t.day, t.hour, t.minute, t.second
    )
}

/// Days since the Unix epoch → civil date (Howard Hinnant's `civil_from_days`),
/// so the probe needs no date crate.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles_are_exact_on_a_ramp() {
        let ramp: Vec<u32> = (1..=100).collect();
        assert_eq!(percentile(&ramp, 50), 50);
        assert_eq!(percentile(&ramp, 95), 95);
        assert_eq!(percentile(&ramp, 99), 99);
        assert_eq!(percentile(&[], 99), 0);
        assert_eq!(percentile(&[7], 99), 7);
    }

    #[test]
    fn budget_verdict_needs_enough_samples() {
        let fast = Summary {
            count: MIN_SAMPLES_FOR_VERDICT,
            total: MIN_SAMPLES_FOR_VERDICT as u64,
            p95: BUDGET_P95_US,
            p99: BUDGET_P99_US,
            ..Summary::default()
        };
        assert!(fast.passes(), "exactly on budget passes");
        let thin = Summary {
            count: MIN_SAMPLES_FOR_VERDICT - 1,
            ..fast
        };
        assert!(!thin.passes(), "a handful of keys is not a verdict");
        let slow = Summary {
            p95: BUDGET_P95_US + 1,
            ..fast
        };
        assert!(!slow.passes());
        let tail = Summary {
            p99: BUDGET_P99_US + 1,
            ..fast
        };
        assert!(!tail.passes(), "the tail is part of the budget");
    }

    #[test]
    fn civil_dates_match_known_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_000), (2022, 1, 8));
        // 2024-02-29, a leap day.
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    }

    #[test]
    fn report_lands_under_appdata_vietkb() {
        let path = output_path(UNIX_EPOCH + Duration::from_secs(1_700_000_000));
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(name, "perf-20231114-221320.tsv", "UTC stamp, sortable");
        assert_eq!(
            path.parent().unwrap().file_name().unwrap().to_string_lossy(),
            "VietKB"
        );
    }

    #[test]
    fn probe_is_off_unless_the_environment_says_so() {
        // The test runner does not set VKB_PERF; nothing may be armed by default.
        if std::env::var_os("VKB_PERF").is_none() {
            assert!(!enabled(), "probe must stay off without VKB_PERF");
            // Both hot-path hooks are no-ops and must not panic.
            begin();
            end();
            assert!(snapshot().is_none());
            assert!(finish().is_none());
        }
    }
}
