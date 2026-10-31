# 70 — Implementation notes: Windows backend (`vkb-platform`, `vkb-windows`)

Owner: Windows dev (worktree `wt-windows`, branch `feat/windows`).
Scope touched: `crates/vkb-platform/**`, `crates/vkb-windows/**`, `crates/vkb-macos/**` (untouched, still the Sprint 3 skeleton).

---

## 1. File map

| File | Contents |
|---|---|
| `vkb-platform/src/plan.rs` | `EngineAction` → `Decision { swallow, Injection }`; the only place that decides interception. Pure, 10 tests. |
| `vkb-platform/src/rules.rs` | `resolve()` per-app enable/charset/method/macro + `normalize_process_name()`. Pure, 8 tests. |
| `vkb-platform/src/hotkeys.rs` | `Matcher` / `Combo`: allocation-free hotkey matching with per-combo latch. Pure, 11 tests. |
| `vkb-windows/src/keys.rs` | VK constants, modifier tracking, US-layout char/VK tables, scancode table, autorepeat bitset helpers. |
| `vkb-windows/src/input.rs` | Pure `INPUT` array builder (`build`, `build_segment`) + single-call `SendInput` + `Ctrl+V`. |
| `vkb-windows/src/state.rs` | `HookRuntime`: engine, enablement, per-app resolution, IME throttle, injection. |
| `vkb-windows/src/hook.rs` | Hook thread, message loop, install timeout, watchdog, 30 s self-refresh, teardown. |
| `vkb-windows/src/foreground.rs` | `GetForegroundWindow` → pid → `QueryFullProcessImageNameW`, layout probe, cache helpers. |
| `vkb-windows/src/ime.rs` | `ImmGetCompositionStringW` composition probe, `ToUnicode` fallback for non-US layouts. |
| `vkb-windows/src/autostart.rs` | `HKCU\...\Run` create/read/delete. |
| `vkb-windows/src/clipboard.rs` | Opt-in fallback: capture all HGLOBAL clipboard formats, paste, restore. |
| `vkb-windows/src/globals.rs` | Atomics + callback slots shared with the UI thread. |
| `vkb-windows/src/backend.rs` | `WindowsBackend: PlatformBackend`, `Injector: EngineSink`. |
| `vkb-windows/tests/hook_integration.rs` | `#[ignore]` real-hook tests + one non-ignored enabled-flag test. |

`vkb-platform/src/lib.rs` gained only `pub mod hotkeys; pub mod plan; pub mod rules;` — the
frozen traits, types and re-exports are byte-identical to the contract.

---

## 2. Hotkey reliability design (no `RegisterHotKey`)

1. **Matching inside the hook.** The configured `HotkeySet` is compiled once per config
   change into a fixed 9-slot `Matcher` (`Combo { action, exact modifiers, Option<VK> }`).
   On every transition the hook asks `Matcher::on_key(vk, down, mods, is_modifier)`, so the
   trigger key is swallowed *before* the app sees it — impossible with `RegisterHotKey`,
   which also cannot express `Ctrl+Shift`.
2. **Latches, never flags that can stick.** Each combo has one latch bit, set when the combo
   completes and cleared as soon as the combo can no longer be held (key released, or a
   required modifier released). Autorepeat therefore fires once, and releasing + pressing the
   toggle again always produces exactly one more toggle — verified by
   `hotkeys::tests::releasing_and_repressing_toggles_back` and
   `state::tests::toggle_twice_returns_to_previous_state`.
3. **Watchdog.** A supervisor thread checks a heartbeat bumped every 200 ms by the hook
   thread, and an `ALIVE` flag set/cleared inside the worker. Dead thread → respawn; no
   heartbeat for 2.5 s → post `WM_QUIT` and respawn. The worker additionally re-installs its
   own hook every 30 s, which heals the case Windows silently drops the hook after
   `LowLevelHooksTimeout` (the only reliable detection: the OS gives no signal).

Modifier tracking is our own (updated from the hook stream, seeded once from `GetAsyncKeyState`
at install), so a missed `GetKeyState` cannot leave the toggle latched.

**Deliberate deviation:** modifier-only combos (`Ctrl+Shift`) are **matched and fired but not
swallowed**. Swallowing a bare `Ctrl`/`Shift` key down leaves the target with an unbalanced
modifier state, so `Ctrl+Shift+C` would reach the app as `Ctrl+C`. Modifiers produce no text,
so the app never receives the trigger *as text*, which is the requirement's intent. Flip
`state::on_event` step 1 if the Lead wants EVKey-verbatim swallowing.

---

## 3. Injection sequence

1. One `EngineAction` → one `Injection` (`Type(text)` or `Replace { deletes, insert }`).
2. `Replace` builds **one** `Vec<INPUT>`: `deletes × (VK_BACK down, up)` followed by the text
   events, then a **single** `SendInput` call. The target window never observes a half-applied
   replacement (no flicker, no intermediate commit), which is the guard-list requirement.
3. Text is `KEYEVENTF_UNICODE` (UTF-16 units, surrogate pairs split correctly) — layout
   independent and works in edit controls, Chrome, VS Code, Excel.
4. `send_standard` (config `typing.send_standard`) switches to virtual key + scancode from a
   static set-1 table (`keys::scan_for_vk`), wraps shifted characters in `Shift` down/up, and
   still falls back to Unicode events for characters no US key can produce (Vietnamese
   diacritics). Backspace always carries the real scancode in this mode.
5. Every injected event carries `dwExtraInfo = 0x5649_454B` ("VIEK"); the hook drops those
   events immediately, so injection can never feed back into the engine.
   **D-WIN-09 (candidate, needs Windows measurement):** in the default mode a `Replace` is
   two `SendInput` calls, the Backspace block first (`input::split_point`, after the
   `deletes`-th Backspace key-up, so a restore chord's release stays with the deletes) and
   the text second, so Notepad has processed the deletes before the `Shift`-wrapped VK text
   arrives (`qa1` -> `QA ONE` was right only 25-50% in one batch, identical emitted sequence
   for hits and misses). Shift-needing ASCII stays VK + `Shift`: as packets it would be a run
   right after backspaces, which collapses (D-E2E-01). `send_standard` and plain typing keep
   one call. `SendResult.sent` counts across both calls, so the short-send recovery resumes
   from the same index. The alternate `Injector` sink stays one call (it may run off the hook
   thread, where a physical key could land between two calls). Measure: `qa1` x20 after a VNI switch, `ba6t5/ta8t1`, `VIEETJ`
   all-caps.
   **Measurement build (D-WIN-09):** a `dev-channel` build reads `VKB_INJECT_STRATEGY`
   (`split`, `single`, `split-delay-N`, `per-char`, `scancode-shift`, `unicode-after-split`,
   `capslock-free`; `crates/vkb-windows/src/input_dev.rs`) and routes a hook-path `Replace`
   through it; unset = the path above, untouched; release builds never read it. Protocol:
   `qa/win-dwin09-protocol.md`.
6. `Noop` never reaches the injector: `plan::decide` returns pass-through and no array is
   built, so the pass-through path allocates nothing.

---

## 4. Other decisions

- **Per-app rules.** `rules::resolve` (pure) applies `AppRule.enabled > remembered > global`;
  charset/method/macro come from the rule only. Resolution runs on foreground-window change
  (or every 1.5 s TTL), never per keystroke; the process name is compared as a `String` only
  when it changed. `remember_per_app` stores the state per process in the hook thread and is
  written on both hotkey and `set_enabled()` transitions.
- **Enablement.** `input.enabled` is adopted from the config file only when the value in the
  file actually changes, so a save of unrelated settings does not undo a hotkey toggle.
- **IME.** `ImmGetContext` + `GCS_COMPSTR` length > 0 → pass through and reset the word.
  Throttled to once per 60 ms. `VK_PACKET` is always passed through.
- **Layout.** Read-only: `GetKeyboardLayout(thread)` compared against `0x0409`. Nothing in this
  crate ever calls `LoadKeyboardLayout`/`ActivateKeyboardLayout` — the EVKey bug is
  structurally absent (grep-able: the strings do not appear).
- **Autostart.** Single `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value `VietKB`
  containing the quoted exe path. No admin, no Task Scheduler.
- **Clean teardown.** `HookSlot::drop` unhooks on every worker exit path, including unwind.
  A hard kill needs nothing: Windows removes low-level hooks when the installing thread dies.
- **Panics.** `catch_unwind` around the runtime and around `engine.process_key`; a panicking
  engine is disabled once instead of killing the hook thread. Note the release profile uses
  `panic = "abort"`, so in a release build a panic still ends the process.

---

## 5. Tests

`cargo test -p vkb-platform` — **29 passed**, pure: injection decisions (10), per-app rules (8),
hotkey matcher (11).

`cargo test -p vkb-windows` — **47 unit + 1 integration passed**, no live hook needed: key tables
+ scancode coverage, `INPUT` builder (order, batching, tagging, surrogate pairs, standard mode),
config→matcher compile, runtime enable/remember logic, autostart registry round-trip (restores
the previous value), clipboard helpers (`#[ignore]`d), foreground probe smoke test.

`cargo test -p vkb-windows -- --ignored` — 3 more tests, all needing a live desktop: the two
real-hook tests below and the clipboard round-trip.

`cargo check -p vkb-macos` — compiles (skeleton unchanged, nothing to check on this host).

---

## 6. Manual verification (needs a live desktop)

Preconditions: interactive session, no other IME currently composing, Notepad open.

1. `cargo test -p vkb-windows -- --ignored --nocapture` — expects
   `hook_installs_fires_hotkey_and_uninstalls` and `install_is_idempotent_and_stop_is_safe`
   to pass (they inject one `F24`).
2. Run a small harness that calls `vkb_windows::install_hotkeys(&HotkeySet::default(), |hk| println!("{hk:?}"))`,
   then press `F1`, `F2`, `F6`, `F8`, `F9`, `F11`: each must print, and no key may reach
   Notepad (they are swallowed).
3. Toggle reliability: bind `toggle = "Ctrl+Shift"`, press it twice in Notepad — the state must
   return to the original and `is_enabled()` must match the printed callback.
4. Hold `F1` for two seconds: exactly one callback (autorepeat latch).
5. Injection: with a stub engine (or once `vkb-core` lands) type `as` → expect `á` in Notepad
   with no visible intermediate state; repeat in Chrome, VS Code, Excel, Zalo.
6. Per-app: `[[apps.rules]] process = "notepad.exe", enabled = false` → typing in Notepad is
   untouched, Chrome still converts.
7. IME: enable a Japanese IME, start a composition in Notepad, type — VietKB must stay out.
8. Autostart: `set_autostart(true)`, check `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v VietKB`,
   sign out and back in; `set_autostart(false)` removes it.
9. Watchdog: kill the hook thread from a debug build (`globals::STOP` or a panic injection) and
   verify re-install within ~1 s and that hotkeys work again.

---

## 7. Known gaps / not verified

- **No live-hook verification in this session.** Everything above §6 is automated; the hook
  behaviour itself (swallow/CallNextHookEx, IME, per-app, watchdog) needs the manual pass —
  the build machine is shared and a global hook affects every keystroke.
- `vkb-core::Engine::process_key` is still `todo!()` upstream; the hook catches the panic and
  disables the engine once, so Sprint 1 typing cannot be demoed until `wt-engine` lands.
  Hotkeys, injection plumbing and rules work regardless.
- `macro_when_disabled` needs the engine to be callable while Vietnamese is off. The backend
  currently short-circuits when disabled, because `EngineOptions` has no "disabled" flag;
  see §8.
- `send_standard` for IME-style scancode-only apps is best-effort: Vietnamese diacritics have
  no key and fall back to Unicode events.
- Clipboard fallback restores HGLOBAL-backed formats (all text formats, CF_HTML, CF_LOCALE,
  private HGLOBAL formats) but not GDI-handle formats (CF_BITMAP, CF_METAFILEPICT,
  CF_ENHMETAFILE) or owner-rendered formats. It is off by default and never used by the hook
  path.
- `ToUnicode` in the non-US-layout fallback has the documented dead-key side effect; it is only
  called for non-US layouts and only for keys the US table cannot resolve.
- `set_enabled()` from the UI thread does not itself fire the enabled callback (the caller
  already knows); the hook fires it for hotkey-driven changes.
- Perf budgets P2/P3/P6 are not measured yet: no end-to-end typing until the engine lands.
  The design targets them (no allocation, no locks, cached app probe, ≤400 ms install).

---

## 8. Contract changes requested (for the Lead, not applied)

1. **`EngineOptions` needs `enabled: bool`** (or `VietnameseEngine::set_enabled`). Without it
   the backend cannot implement `macro_when_disabled` ("vẫn cho phép gõ tắt khi tắt tiếng Việt")
   without risking double typing.
2. **`install_hotkeys` callback lifetime**: stored as `Arc`, invoked on the hook thread.
   `HotkeyCallback = Box<dyn Fn(Hotkey) + Send>` works unchanged, but the app must not block
   inside it (already stated in the contract).
3. **Additive API** (no frozen signature changed) that `vkb-app` may want to call:
   `vkb_windows::install()`, `set_config(SharedConfig)`, `is_hook_ready()`,
   `WindowsBackend`, `Injector`, `active_app()`.

---

## 9. Clipboard send mode (D-WIN-12..18)

Opt-in only (`typing.use_clipboard`, charter C4: never on the default path). This section is the ONE current description; older wording elsewhere (defect rows, history below) is superseded by it.

Code map: pure rules in `vkb_platform::clipboard_plan` (formats, proofs, verdicts, decisions), pure bookkeeping in `vkb_platform::clipboard_book::SessionBook` (a state machine: events in, explicit `Action`s out), Win32 execution in `crates/vkb-windows/src/clipboard.rs`. Everything pure is unit tested on every host, including deterministic randomized property tests over a fake clipboard; the Win32 side is checked by compile + clippy and by live desktop QA (section 9.2).

### 9.1 Design

**Ordering by construction.** The replacement runs synchronously inside the hook callback, like the Unicode path, so no physical key can overtake it. `Injector` (non-hook sinks) always types: only the hook thread has the pumped owner window.

**Hook thread (`replace`, never blocks).** `decide_paste(enabled, session numbers, snapshot number, GetClipboardSequenceNumber())` pastes only when a ready faithful snapshot was taken at exactly the current sequence number, or our own temporary text from the previous paste of this session is still there. Anything else (mode off, no / lossy / stale snapshot, lock busy, a restore in flight, sequence 0, previous owner not answering a 20 ms `WM_NULL` probe, `OpenClipboard` busy: single try, the CURRENT owner not answering a second 20 ms probe taken right after the open and before the first write, any failed call) types the text with Unicode `SendInput`. A paste: re-probe the current owner after `OpenClipboard` (`EmptyClipboard` sends it `WM_DESTROYCLIPBOARD` synchronously; our own window or nobody needs no probe), record our text identity (`OurText`: fingerprint + length, BEFORE the write), take the snapshot into the plan, write the temporary text (+ `CanIncludeInClipboardHistory` / `CanUploadToCloudClipboard` = 0, `ExcludeClipboardContentFromMonitorProcessing`, the ownership marker `VietKB.Owned`), close, verify (below), then ONE `SendInput` (Backspaces, held chord / Shift lifted, LCtrl+V, Shift back) and post `WM_APP_PASTED`. The owner window is a message-only window created on the hook thread (it pumps messages).

**Ownership is proved, not guessed.** `CloseClipboard` advances the sequence number, so a number read while open never matches the notification that follows. After the close, under the shared lock, `verify_write` reads `s1`, the owner (`OwnerCheck::Window{matches}`, or `Null` for the NULL-owner fallback, which compares `post_close`), the marker, `s2`; the pure `write_verdict` answers `Ours(seq)`, `Foreign` (only then is old data retired) or `Unstable(latest)` (kept: user data is never dropped on uncertainty). A write is identified by `OwnSeqs { in_open, after_close }`; both values are ours.

**Worker thread (`vkb-clipboard`, pumped).** Keeps the fresh snapshot (`AddClipboardFormatListener`; each foreign change drops it and, after a 100 ms debounce and only when nothing is pending, takes a new one) and executes restores. A snapshot lists the formats first and classifies them before any data is read; `HGLOBAL` formats are copied byte for byte within 32 MiB / 64 MiB; GDI handle formats are never dereferenced (Windows rebuilds CF_BITMAP from the DIB); unknown / handle-only / marker-carrying / privacy-flagged (`ExcludeClipboardContentFromMonitorProcessing`, `CanInclude...`/`CanUpload...` = 0) / > 512-format clipboards are not snapshotted (the hook types). The snapshot number is read in a stable bracket after the close. Snapshots are dropped when the mode is switched off.

**Reads of a clipboard we do not own.** The current owner is probed ONCE PER OPEN, after opening and before the first read (`read_allowed()`, with `WM_NULL`, 50 ms, unless our window or nobody owns it; pure `may_read`). The probe proves the owner's message loop answered a moment ago and nothing more: an owner that blocks while it renders a delayed format can still stall the read (section 9.2). No answer: no read, no snapshot. The owner cannot change between probe and read because the clipboard stays open. Fail closed: an elevated (higher-integrity) owner we may not message answers 0 too, so content copied from an elevated program is never snapshotted (the mode simply types for it). `GetClipboardOwner` is only called by `current_owner`, which tells "no owner" (NULL, last error cleared first, pure `owner_state`) from a genuine error (unknown owner: treated as "probe failed", the hook types / no read). `read_dword` is only called by `capture`, after the gate. A restore only WRITES: it reads nothing from a foreign clipboard. `clear_if_ours` reads text only from our own window, the text is accepted when the allocation is at least as big as ours and NUL-terminated within our length (never decoded beyond it).

**Restores (`SessionBook`).** Generations: only the newest is ever restored; `start_session`, a failed write and any finished restore (done or foreign) retire every older entry (zeroed on the worker); a restore that comes back busy after a newer generation appeared is retired. The settle wait is `SETTLE_MS` = 1200 ms from the moment the paste batch has been SENT (the worker stamps the time on `WM_APP_PASTED`; every further paste restarts it); for that window the clipboard holds our text, so a manual Ctrl+V pastes the expansion (accepted tradeoff, no UI). A restore proves ownership while the clipboard is open (`restore_decision`: sequence matches either of our numbers, or owner window + marker; urgent data of a failed write: owner window or number; the NULL owner alone is never proof). Not provable -> `Foreign`: the data is not put back and never overwrites what is there. Busy / incomplete: retried 500, 500, 1000, 1000, 2000, 2000 ms, then every 5 s, forever (data is never dropped in normal operation; an incomplete write records its post-write numbers). On success the restored data is ADOPTED as the next snapshot (the restored clipboard carries the privacy flags we add, a re-capture would refuse it); the restored data keeps those flags on purpose (the user's original copy is already in Win+V / cloud history). It also keeps the `VietKB.Owned` marker on purpose: the bracket after the restore's close uses owner + marker to prove the restored state is ours before adopting it (an empty restore carries neither). Another app's next copy replaces the whole clipboard, marker included.

**Mode off and quit (flush).** Mode off: restore once (the newest), drop every snapshot, retry a busy clipboard every 250 ms for at most 5 s, then zero the data, empty our leftover text (`clear_if_ours`) and stop the timers (`keys.log`: `clipboard-restore GIVE UP`). Quit (`hook::uninstall` -> `shutdown`): same against a fixed 700 ms deadline (the 800 ms ack wait includes it), urgent jobs included. Every stored payload is due immediately and is attempted at least once; a not-yet-settled current session first waits `min(settle remaining, budget - 100 ms)` so the target can read our text (exit-latency cost: up to ~0.7 s after a paste in the last second; mode off up to 1.2 s). `open_retry_delay` gives flush jobs a budget (`max_wait_ms` = deadline - now): normal restores back off 20/40/80/160 ms on a busy clipboard, a flush never sleeps past its budget. The give-up is exact on the book's clock; the real `SetTimer` adds ~15 ms granularity.

**After a give-up (`clear_if_ours`).** If OUR window still owns the clipboard and the marker is present or its text equals the text recorded before the write (also the leftover of a failing write), the clipboard is emptied so a later Ctrl+V cannot paste our text. A foreign owner is never cleared or read.

**Other rules.** The hook refuses to paste while a restore is in flight. A failed write is planned by what it changed (`failed_write_plan`): if `EmptyClipboard` itself failed nothing changed and the user's data is still on the clipboard, so the snapshot is just retired (a fresh one follows after the debounce) and a running session stays; any later failure means the clipboard was emptied, so it is closed and the data handed to the worker (`urgent`); nothing heavy runs on the hook thread (retired buffers are zeroed and freed by the worker; above the 128 MiB queue cap the hook zeroes inline, worst case 2 x 64 MiB). Release is `panic = "abort"`: `catch_unwind` covers dev / test builds only, the release guarantee is that every API result is checked.

### 9.2 Documented limits

- A clipboard manager / cloud sync / RDP that re-sets our text WITHOUT our marker looks foreign: the restore is abandoned and the user's original clipboard (possibly an image) is not put back; our text stays. There is no text-based rescue (reading a foreign clipboard from a restore was too risky, see the changelog).
- Content copied from an elevated program, or from an owner that does not answer in 50 ms, is not snapshotted; the mode types instead.
- The probe proves the owner's message loop answered a moment ago, nothing more: an owner that answers `WM_NULL` and then blocks while it renders a delayed format (`WM_RENDERFORMAT` / `WM_RENDERALLFORMATS`, triggered by `GetClipboardData`) still stalls the worker in that read with the clipboard locked. The 50 ms probe makes this rare, it cannot make it impossible; a source-level guard test (`crates/vkb-platform/tests/clipboard_source_guard.rs`) pins that every read goes through `read_allowed()`.
- A previous owner that answers `WM_NULL` and then blocks inside `WM_DESTROYCLIPBOARD` can still stall the hook in `EmptyClipboard` (typing whenever the previous owner is not ours would disable the mode).
- A foreign write between the close and the snapshot bracket makes the number newer than the data; the write's own `WM_CLIPBOARDUPDATE` drops that snapshot, but the hook could paste in the gap (microseconds wide).
- Every clipboard change is copied once while the mode is on (capped), and reading it makes delayed-render owners render.
- Live Windows desktop QA (runs 2–3, 2026-11-01, at 2f38b5f): WinForms RichTextBox, isolated Chrome and isolated Edge/bing.com; clipboard text / bitmap / PNG / empty / HTML; all expansions exact, every original format restored (flags + marker kept, as above); a newer copy inside the settle window (external process, Ctrl+C in another app) survives; a busy clipboard (held 2.5 s) is restored after release; quit 2 ms after an expansion restores before exit. Not covered: Win+V history count (history off on the test machine), byte-for-byte compare after a busy retry, the `#[ignore]`d round-trip test.

### 9.3 Changelog (condensed; the design above is authoritative)

- Initial fix (D-WIN-12..15): classify formats before reading any data (the crash was a GDI handle treated as `HGLOBAL`), one ordered worker, Ctrl+V as a proper chord in the same `SendInput`, sequence-checked late restore, privacy opt-out formats.
- Ordering by construction: the first async design (physical keys waiting on a pending counter) was replaced by the synchronous hook paste + listener-kept snapshot.
- D-WIN-16/17: sequence numbers are read after the close; the listener debounces and never opens the clipboard during a pending restore.
- Marker + owner + stable bracket replaced a "close bumps by at most 8" heuristic.
- `SessionBook` refactor, newest-wins generations, mode-off / quit deadlines, due-now flush, per-job open budget, byte-capped garbage queue, property tests with a fake clipboard (conservation, newest-only restores, single flight, timer liveness, no give-up outside a flush, exact deadlines, settle period, no foreign overwrite, no text left behind).
- The text rescue ("clipboard text equals ours -> restore anyway") existed in several forms and was REMOVED: it read a foreign clipboard from a restore (hang risk, owner-race windows) for a rare benefit. `text_fingerprint` stays only to recognise our own leftover text in `clear_if_ours`.
- Simplification round: the current owner is probed once per open, after opening and before the first read (replaces probe-before-open + identity re-check; a delayed-render owner that blocks while rendering can still stall the read, see 9.2); our text recorded before the write; snapshot moved into the start plan; `write_verdict` without the unused in-open parameter.
- Guard hardening: the source guards pin usage (the exact `if !read_allowed() {` -> `return None;`, the exact `if !clipboard_owner_responsive(20) {` -> `return false;` between open and write, the exact `owner_is_ours && ` conjunction), collect impl methods and qualified fns, and count every `GetClipboardData(` call site against the collected bodies; the NotEmptied failure restores the previously recorded text.
- Latest round: the hook re-probes the current owner after `OpenClipboard` and before its first write; a write failing at `EmptyClipboard` retires the snapshot instead of queueing a restore; the text read accepts allocations at least as big as our text; source-level guard tests (`crates/vkb-platform/tests/clipboard_source_guard.rs`, line-ending independent: they normalise CRLF and are also run over a CRLF copy) pin the call-order rules.
