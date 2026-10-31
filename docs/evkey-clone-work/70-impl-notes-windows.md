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

## 9. Clipboard send mode (D-WIN-12..15)

Opt-in only (`typing.use_clipboard`). Pure rules: `vkb_platform::clipboard_plan` (tested on every host). Win32 side: `crates/vkb-windows/src/clipboard.rs`.

**Ordering by construction.** The replacement runs synchronously inside the hook callback, like the Unicode path, so no physical key can overtake it. There is no pending counter and no wait in the hook (the earlier worker-queue design could deadlock `SendInput` against a parked hook and let a stuck worker delay every key).

Hook thread (`clipboard::replace`, never blocks):
1. `decide_paste(enabled, session_seq, snapshot_seq, GetClipboardSequenceNumber())`: paste only when a ready faithful snapshot was taken at exactly the current sequence number, or when our own temp text from the previous paste is still there (session continues). Mode off, missing/lossy/stale snapshot, `try_lock` miss, sequence 0 -> **type with Unicode `SendInput`**.
2. Hung previous owner (`IsHungAppWindow`) -> type. Single-try `OpenClipboard` (no retries) with the hook thread's own message-only owner window (created in `worker_main` right after the message queue exists, destroyed in `teardown`; the hook thread pumps, so other programs' `EmptyClipboard`/`WM_DESTROYCLIPBOARD` never waits on us). Busy -> type. Sequence re-checked under the open clipboard.
3. Write the temp text (+ `CanIncludeInClipboardHistory`/`CanUploadToCloudClipboard` = 0, `ExcludeClipboardContentFromMonitorProcessing`), close, then ONE `SendInput`: Backspaces, held Shift up (re-read with `GetAsyncKeyState`), LCtrl down, V down, V up, LCtrl up, Shift down. `SendInput` accepting 0 events -> type instead. Any failed call after the clipboard was emptied restores the snapshot on the spot.
4. Post `WM_APP_PASTED` to the worker (re-arms the settle timer).
Non-hook sinks (`Injector`) always type: only the hook thread has the pumped owner.

Worker thread `vkb-clipboard` (own window + `GetMessage` loop, so it is pumped):
- While the mode is on: `AddClipboardFormatListener`; every `WM_CLIPBOARDUPDATE` (ours excluded by sequence number) drops the snapshot and takes a new one: formats listed first (`GetLastError` checked after the enumeration, >512 formats = lossy), classified with `classify_format`/`snapshot_verdict` before any data is read; `HGLOBAL` formats copied byte for byte within 32 MiB/64 MiB; GDI handle formats never dereferenced (CF_BITMAP is rebuilt by Windows from the DIB); hung owner, unreadable or oversized data = no snapshot. A slow or hung `GetClipboardData` can only stall this thread; the hook then sees a stale sequence and types.
- Restore: 1200 ms (`SETTLE_MS`, fixed on purpose: no consumed-paste signal exists) after the last write, re-armed if a paste raced the timer (`settle_remaining_ms`), only if `GetClipboardSequenceNumber` still equals the value right after our write (checked with the clipboard open), 5 attempts with backoff, no privacy flags for an empty snapshot. The restored data is written through the hook thread's owner window, so a stuck worker can never be the owner others wait on.
- A foreign clipboard change during a session abandons it (never restored over). Switching the mode off, `clipboard::shutdown()` (first thing in `hook::uninstall`, bounded 800 ms) restore at once and drop every snapshot: the user's clipboard is not held in memory unless the mode is on.

Cost (opt-in mode): each clipboard change is copied once (capped), and reading it makes delayed-render owners render. Release is `panic = "abort"`: `catch_unwind` covers dev/test builds only; the guarantee in release is that every API result is checked and non-text/handle formats are never touched.

Not verified on a live Windows desktop (no host here). To do: the `#[ignore]`d round-trip test, a bitmap / PNG / Win+V / quit-within-400 ms manual check, Ctrl+C in other apps while the mode is on (must not stall), fast typing of expansions (ordering).

Review round 3 (all in `clipboard_plan` as pure functions + tests): `notification_is_ours` (the listener reads the sequence number AFTER taking `SHARED`, where the hook records its write, so a paste in flight never looks foreign); `restore_due` (age check and take of the session in ONE locked section; flush / mode-off / failed write stay unconditional); `flag_blocks_snapshot` (data marked `ExcludeClipboardContentFromMonitorProcessing`, or `CanIncludeInClipboardHistory` / `CanUploadToCloudClipboard` = 0, is never read or copied: the hook types; snapshot buffers are zeroed on drop). The hook probes a foreign previous owner with `SendMessageTimeoutW(WM_NULL, SMTO_ABORTIFHUNG|SMTO_BLOCK, 20 ms)` before `EmptyClipboard` (our own owner skips the probe); no answer -> type. Shift comes from the hook model (`held`), and a restore-chord's modifiers are released first (`build_paste(.., chord, ..)`, the typing fallback uses `build_restore` like the normal path). The restore reads `HOOK_OWNER` per attempt, `destroy_hook_owner` clears it under `SHARED`, and the last two attempts use the NULL owner if the handle went stale.

Review round 4: a re-capture after our own restore would see the privacy flags the restore adds and (by design, `flag_blocks_snapshot` is not relaxed) refuse to snapshot, silently degrading the mode to typing. So `restore_session` now **adopts** the restored data: after a complete `restore_locked` it reads the sequence number under the open clipboard and stores `Snap { seq, data }` plus `restored_seq`; `notification_is_ours(session_expected, restored_seq, current)` recognises that notification, so no re-capture happens and the next expansion pastes again (`decide_paste` test: adopted snapshot -> `StartSession`). The owner window is read in the same locked section that takes the session; the hook retires old session/snapshot data into `Shared.garbage` and the worker (`WM_APP_GC`) zeroes (`fill(0)` + `black_box` + `compiler_fence`) and frees it, so the hook thread never frees large buffers. A foreign window that UIPI forbids us to message answers the owner probe with 0: fail closed, the hook types.

Review round 5 (lows): the retired-buffer queue is capped (`GARBAGE_CAP` = 4, `push_bounded` evicts and zeroes the oldest) and drained explicitly on flush and mode-off; a failed paste write now keeps the data it put back as the snapshot (sequence read under the open clipboard, `restored_seq` set) instead of discarding it; mode-off clears `restored_seq` with the snapshot. Decision: on re-enable the first snapshot is a fresh capture, not an adoption of the earlier restored data, because adopting would keep the user's clipboard in memory while the mode is off. Consequence: right after re-enabling, a clipboard that still carries our privacy flags is not snapshotted (the hook types) until the user copies something.

Review round 6 (real-app QA, D-WIN-16..18): `CloseClipboard` advances `GetClipboardSequenceNumber`, so every number recorded while the clipboard was open was wrong for the listener (our own write looked foreign, the session was dropped, the restore never ran). All numbers that identify OUR write (paste, restore adoption, failed-write adoption, snapshot capture) are now read after the close, under `SHARED`, through `seq_after_close(in_open, after_close)`: it accepts at most `MAX_CLOSE_BUMP` steps past the in-open value (wrapping) and rejects anything else as a foreign write. The listener no longer opens the clipboard right after any change: it arms a `REFRESH_DELAY_MS` timer (debounce, and room for a target's own `OpenClipboard` for a paste), and `refresh_snapshot` returns at once while a paste session is active. D-WIN-18 (Shift over the trigger) is documented as needs-info in the defect row.

Review round 7 (replaces the `seq_after_close` / `MAX_CLOSE_BUMP` heuristic of round 6, which could lose data both ways): ownership of a clipboard state is now **proved**, not inferred from sequence-number distance. Every content we write (temporary text AND restored data) also carries the private format `VietKB.Owned` (`OWNED_MARKER`, one byte) next to the privacy flags; a write that cannot set the marker counts as a failed write. After `CloseClipboard`, under `SHARED`, without opening anything, `verify_write` reads `s1`, `GetClipboardOwner()` == our owner window (NULL-owner fallback: marker + bracket only), `IsClipboardFormatAvailable(marker)`, `s2`, up to 3 times, and the pure `classify_write(owner_ok, marker, s1, s2)` answers `Ours(seq)` (stable), `Foreign` (owner or marker prove somebody else wrote: only then is the old copy retired) or `Unstable(latest)` (owner+marker ours but the number is still moving: the session is kept and the latest number recorded; user data is never discarded on uncertainty). A write is identified by `OwnSeqs { in_open, after_close }`; `notification_is_ours`, `decide_paste` and `restore_action` accept either value. The foreign-clipboard snapshot is taken in a bracket too (`stable_seq` / `snapshot_stable`: both reads equal, else no snapshot); the marker is `FormatClass::Marker`, never copied and not user data. Residual window: a foreign write between the close and the snapshot bracket is caught by its own `WM_CLIPBOARDUPDATE`, which drops the snapshot.

Review round 8: (1) a sequence-number mismatch alone no longer drops or abandons a session / adopted snapshot: `ownership(own, owner_marker_ours, current)` returns `Ours`, `OursAtNewSeq(current)` (owner window + marker still ours: the recorded number is refreshed, and the adopted snapshot's number with it) or `Foreign`; used by the hook before it decides, by the listener and by the restore (`restore_action(own, proof, current)`). (2) A clipboard that carries `VietKB.Owned` is our own state: `snapshot_verdict` says `Lossy`, so it is never snapshotted. (3) Restoring an EMPTY user clipboard leaves it empty: no marker, no adoption. (4) With `VKB_DEBUG_KEYS=1` the first capture logs `clipboard-capture-seq in_open=.. after_close=.. delta=..` in keys.log, so QA can tell whether a read-only open/close advances the sequence number. Residual window (documented as required): the snapshot's number is read in a bracket AFTER the close; a foreign write between the close and that bracket makes the number newer than the data. It is caught by that write's own `WM_CLIPBOARDUPDATE` (the listener drops the snapshot), but the hook could paste in the gap: the foreign copy would then be replaced by our text and restored from the older snapshot. The gap is microseconds wide and cannot be closed without holding the clipboard open.

Review round 9 (Windows Lead review of 3fe4dcb..b0150c1):
- **Never drop user data on a failed restore.** `restore_session` returns `Done` or `Retry(session)`; busy clipboard (5 opens) or an incomplete restore puts the session back and re-arms the timer (`restore_retry`: 500, 500, 1000, 1000, 2000, 2000 ms), after `MAX_RESTORE_ROUNDS` the session is parked in `Shared.stranded` (still retried every 5 s, and by the mode-off / quit flush). Only the quit flush gives up (logged), because the process ends.
- **Restore timing.** The settle wait starts when the paste batch has been SENT (the hook posts `WM_APP_PASTED` after `SendInput`; the worker stamps `last_write` on receipt) and the floor is `SETTLE_MS` = 1200 ms (slow WinForms UI threads read the clipboard late: likely D-WIN-17 root cause); each further paste restarts it. Tradeoff: for that window the clipboard holds our text, so a user Ctrl+V pastes the expansion, not their data.
- **Flush in one lock section.** Mode-off and quit take the session, the parked sessions and clear snapshot / `restored_seq` under one lock; `try_paste` re-checks `ENABLED` after `try_lock`.
- **Nothing heavy on the hook thread.** A failed write closes the clipboard and hands the user's data to the worker (`Shared.urgent`, `WM_APP_URGENT`; restored on owner proof alone, then adopted); `retire` only queues, the worker zeroes and frees (no eviction on the hook).
- **EmptyClipboard on the hook.** The previous-owner probe (`WM_NULL`, 20 ms, hung check) bounds most stalls; an owner that answers the probe and then blocks inside `WM_DESTROYCLIPBOARD` can still stall the hook. Typing whenever the previous owner is not ours would disable the mode (that owner is the program the user copied from), so this is documented, not removed.
- **Managers re-setting our text.** The session keeps `text_fingerprint` of what we wrote; a sequence mismatch is no longer dropped by the listener, and at restore time (clipboard open) `restore_action_text` restores when the clipboard text equals ours. A genuinely foreign clipboard is retired and re-snapshotted.
- **NULL-owner fallback** now also requires the sequence number to equal the in-open one (`null_owner_ok`); with the usual close bump it simply skips the adoption (the data is still restored).
- **Delayed-render owners.** `capture` skips the snapshot unless the clipboard owner answers `WM_NULL` within 50 ms (`GetClipboardData` has no timeout).
- **Privacy flags on restored user data are kept on purpose**: the user's original copy is already in Win+V / cloud history, re-adding it would duplicate it. No concrete harm found.
- **D-WIN-18.** input.rs checked: with Shift held the trailing Space of an expansion goes out as a Unicode packet and the held Shift is never touched (unit test `held_shift_does_not_disturb_a_trailing_space`); the raw `qa1` with Shift is explained by VNI `1` -> `!` (see the defect row).

Review round 10: (1) **Generations.** Every `Session` has a monotonic `generation` (under `SHARED`). Only the newest session is ever restored (`pick_newest`, `take_newest`); a finished restore (restored or foreign) retires every older parked session, a new `StartSession` retires all parked ones, `requeue` keeps at most one parked (`keep_newest_stranded`), and the mode-off / quit flush restores only the newest and retires the rest, so an older copy of the user's data can never overwrite a newer one. (2) **No proof, no restore.** The failed-write job records the sequence number read while the clipboard is still open and the one after the close (`OwnSeqs`); the urgent restore needs our owner window OR one of those numbers (`urgent_proof`), the NULL owner alone is refused. `owner_marker_ours(None)` is now `false`: in the NULL-owner path the marker alone is never proof, the sequence must match. (3) **Text rescue is narrow.** The "clipboard text equals what we wrote" rescue only fires for a plain-text clipboard (CF_TEXT / OEMTEXT / UNICODETEXT / LOCALE plus our flags and marker, `plain_text_only`) and only in round 0 (`text_rescue_allowed`); when it fires `keys.log` says `clipboard-text-rescue fired` (and a debug trace). (4) **Delayed-render probe fails closed for elevated owners.** The 50 ms `WM_NULL` probe before a capture cannot message a higher-integrity window (UIPI) and reads that as "not answering": a clipboard owned by an elevated program is never snapshotted, the hook types instead. This is intended (we cannot bound `GetClipboardData` on it) and means the clipboard mode does nothing for content copied out of an elevated app.


Review round 11 (refactor): all session / generation bookkeeping now lives in ONE pure state machine, `vkb_platform::clipboard_book::SessionBook<T>` (entries = current / parked / urgent, `inflight`, mode-off / quit deadline), driven by events (`start_session`, `continue_session`, `paste_sent`, `current_to_urgent`, `urgent_failed_write`, `discard_current`, `timer`, `restore_result(Done | Busy | Incomplete | Foreign)`, `notification`, `mode_on` / `mode_off` / `quit`) and answering with explicit `Action`s (`Restore(job)`, `Retire` = zero, `Finished`, `ArmTimer`, `StopTimer`, `Snapshot`, `SkipSnapshot`, `DropSnapshot`, `GiveUp`); `clipboard.rs` only executes them with Win32 calls (`exec_locked` / `drive`). Invariants, each with unit tests: (1) only the newest generation is ever restored; `start_session`, a failed write and ANY finished restore (`Done` or `Foreign`) retire every stored entry of every kind with an older generation, and a restore in flight that comes back busy after a newer generation appeared is retired, not requeued (two urgent jobs + busy; parked + new session). (2) Normal operation never drops user data: busy / incomplete restores back off 500 .. 2000 ms, then every 5 s, forever; an incomplete write records its post-write sequence numbers (`Incomplete { own }`) so even the NULL-owner path can prove ownership next round. (3) Mode off: restore once, drop every snapshot, retry a busy clipboard every 250 ms for at most 5 s, then zero and give up (logged, `keys.log` says `clipboard-restore GIVE UP`), timers stopped. (4) Quit: same against a fixed 700 ms deadline (+ the ack stays under 800 ms), urgent jobs included, give-up logged. (5) No re-snapshot while any session, parked or urgent data, or a restore in flight is pending (`refresh_allowed`); a notification during a pending restore answers `SkipSnapshot`. (6) The text-fingerprint "same text" rescue is REMOVED (it could mis-fire on a user copy of the same plain text inside the settle window): a clipboard manager that strips our marker now simply makes us abandon the restore (documented limit); `text_fingerprint`, `restore_action_text`, `plain_text_only`, `text_rescue_allowed` are gone. (7) The retired-buffer queue is capped by bytes (`MAX_GARBAGE_BYTES` = 128 MiB, `garbage_admit`); above it the calling thread zeroes and frees (rare, logged). (8) The owner-responsiveness probe now runs BEFORE `OpenClipboard` in `refresh_snapshot`.

Review round 12 (deadlines that hold): (1) `begin_flush` makes every stored entry due NOW (`retry_at = now`), so mode-off / quit attempt each payload at least once, whatever pace it was parked at. (2) A flush `RestoreJob` carries `max_wait_ms` = deadline - now; `restore_session` retries `OpenClipboard` through the pure `open_retry_delay(attempt, elapsed, max_wait)`: normal restores back off 20/40/80/160 ms, a flush job never sleeps past its budget and makes exactly one try when the budget is tiny. (3) After every `restore_result` that leaves entries stored the book emits `ArmTimer(0)` (the timer computes the wait), so a timer tick swallowed while a restore ran is never lost; `start_session` debug-asserts that no restore is in flight and the hook refuses to paste while `book.inflight()`. (4) A requeued restore is marked `settled` instead of faking an old `last_paste_ms`, so early in the process it does not wait for a second settle period. (5) Over the 128 MiB garbage cap the hook thread zeroes and frees (documented worst case: one memset of at most 64 MiB, a few ms). Tests: deterministic xorshift property tests over 10k-step random event streams (8 seeds; every `restore_result` variant, monotonic time) check conservation (each payload in exactly one place or exited once), newest-only restores, single flight, timer liveness, no give-up outside a flush and at most two stored entries; a second property test runs 300 flushes against an always-busy clipboard and checks that the newest payload present is attempted at least once, everything exits, and the flush ends within budget + pace.

Review round 13 (Windows Lead review of 04ec932):
- **NULL-owner adoption works now.** Every later comparison uses the sequence number read AFTER the close (`post_close`), including `verify_write(None, ..)` / `null_owner_ok`; the in-open number is only recorded in `OwnSeqs` (same D-WIN-16 pattern, no other in-open-vs-after-close compare left).
- **A give-up no longer leaves our text behind.** When a deadline zeroes the user's data, `clear_if_ours` makes one bounded open attempt and empties the clipboard if owner + marker are still ours (the partial state of a failed restore included), so a later Ctrl+V cannot paste our temporary text. (The model's clipboard oracle checks this.)
- **A flush still waits for the settle period.** A `Current` session that is not settled waits `min(settle remaining, budget remaining - FLUSH_ATTEMPT_RESERVE_MS (100 ms))` before the first attempt, so the target can read our text before the user's data returns; the attempt always fits before the deadline. Urgent data and requeued (settled) restores are not delayed.
- **Normal busy restores park forever, on purpose.** They keep retrying every 5 s (500 .. 2000 ms first) so the user's data is never dropped; the tradeoff: while the clipboard stays busy, our temporary text is on it and a manual Ctrl+V pastes the expansion. There is no UI for this; mode-off / quit end it within 5 s / 700 ms.
- **Exact give-up.** The flush retry delay is clamped to the time left, so the give-up happens exactly at the deadline (test with a 130 ms budget and 50 ms pace).
- **Property model clipboard oracle** (`Clip`: empty / user / temp / partial / foreign): a restore never overwrites foreign data (executor answers `Foreign`), never restores an older generation over the text of a newer session, restores a non-urgent first round only `SETTLE_MS` after the last paste (a flush may cut it to the attempt reserve), and our text is never left on the clipboard once nothing is pending.
- **Smaller items.** The snapshot is taken under the lock right before the write (it cannot vanish; a missing one is logged and nothing is written); `TIMER_REFRESH` is not re-armed when the mode is off; the hook-thread zeroing doc says up to 2 x 64 MiB.
- **Narrow text rescue reinstated (owner decision).** If the clipboard is plain text only (CF_TEXT / OEMTEXT / UNICODETEXT / LOCALE plus our flags and marker), the restore is in round 0 and the text equals what we wrote, the restore goes ahead even though the marker is gone (`text_fingerprint`, `plain_text_only`, `text_rescue_allowed`, `restore_action_text`; `keys.log`: `clipboard-text-rescue fired`). Without it a Ditto / RDP / cloud-sync tool that re-sets our text would make the restore look foreign and the user's original clipboard (possibly an image) would be lost. The false positive: if the user copies exactly the same plain text within the settle window, they get their previous clipboard back instead of the new copy. We judge the first worse than the second.

Review round 14:
- **The text rescue reads a foreign clipboard safely.** `clipboard_owner_responsive(50)` runs BEFORE the clipboard is opened (like the capture); if the owner does not answer the rescue is skipped and the restore is abandoned as foreign. With the probe passed, only the format LIST (no data) decides "plain text only", and `GetClipboardData(CF_UNICODETEXT)` is only decoded when `GlobalSize` equals exactly the size of our text (`our_text_bytes(units)`), so a huge or different foreign text is never read.
- **`clear_if_ours` after a give-up** (`should_clear`): empties the clipboard when the owner is ours AND (marker present OR the text equals ours), or when a manager re-set exactly our plain text (owner foreign: the same conditions as the rescue). That includes the partial state of a failed write whose marker never got set. It never touches other content, and the book emits an explicit `ClearIfOurs` action after every give-up.
- **Proof and verdict rules are pure and shared with the model.** `restore_decision(urgent, own, rounds, owner_marker, owner_is_ours, seq, rescue)` and `write_verdict(owner, marker, in_open, post_close, s1, s2)` (NULL owner compares `post_close`) are what the executor calls; the property test drives them over a fake clipboard (owner, sequence number with the +3 close bump and late bumps, marker, plain-text flag, text fingerprint) with events for a failed write, a foreign copy, an identical-plain-text copy, a manager re-setting our text without the marker, and NULL-owner opens. It checks that a restore never overwrites genuinely foreign data, that the rescue only fires in round 0 on non-urgent data, that every restore we just wrote is recognised (adopted) on the window AND NULL-owner paths, and that nothing of ours survives a give-up. Mutations verified: reverting the `post_close` comparison, deleting the `ClearIfOurs` emission, and dropping the flush settle wait each fail a test.
- **Clocks.** The give-up is exact on the book's clock; the real `SetTimer` has about 15 ms granularity, so on Windows it can land up to ~15 ms after the deadline.
- **Quit latency.** `quit` waits up to 600 ms for the settle period before its first attempt (700 ms budget minus the 100 ms attempt reserve): a just-sent paste gets time to be read before the user's data returns. The price is exit latency of up to ~0.7 s when a paste was sent in the last second; mode-off waits at most 1.2 s the same way.


Review round 15:
- **A probe is only valid for the owner it was taken from.** Every `GetClipboardData` on a clipboard that may be foreign-owned (the rescue text read in `restore_session`, the flag values and the data in `capture`) now goes through `read_allowed(probed)`: the owner is probed before the open (`probe_owner` returns who it was and whether it answered), and after the open `GetClipboardOwner()` is compared; `owner_read_ok(probed, owner_now, ours)` (pure) answers `Yes` (our window, no owner, or the probed owner answered), `No` (the probed owner did not answer) or `Reprobe` (the owner changed between probe and open, or nothing was probed). `Reprobe` sends a fresh `WM_NULL` probe with the clipboard open (fine while open); an owner that still does not answer means no read. The rescue is gated by `rescue_gate(urgent, flush, rounds, text_known)`: never for urgent data, never in a flush (its budget is for the attempt), only in round 0.
- **`should_clear` only clears our own window's clipboard** (owner ours AND marker or equal text). The branch that cleared a FOREIGN owner holding identical plain text is gone: it could wipe the user's own identical copy for almost no benefit. `clear_if_ours` reads text only when our own window owns the clipboard.
- **Model.** Foreign owners now have ids and a "render hangs" flag; a copy by a hanging owner can arrive between the probe and the open (owner race), and the fake `GetClipboardData` asserts it is never called for a hanging owner. The model also has a foreign plain text that differs from ours. Mutation verified: making `owner_read_ok` accept a probe of a different owner fails the model.
