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

Opt-in only (`typing.use_clipboard`). Pure rules live in `vkb_platform::clipboard_plan` (tested on every host); `crates/vkb-windows/src/clipboard.rs` is the Win32 side.

- **Threading.** The hook/injector only calls `clipboard::submit`, which enqueues a job. One `vkb-clipboard` worker owns every clipboard call (retrying `OpenClipboard`, delayed-render owners). Jobs run in order, so consecutive expansions share one snapshot. Residual risk: a physical key typed in the few ms before the worker sends the batch can overtake it; the default Unicode path does not have this and remains the recommendation.
- **Snapshot.** Pass 1 lists ids and names only; `snapshot_verdict` decides before any data is read. HGLOBAL formats (text, DIB/DIBV5, HDROP, locale, PNG/HTML/any registered name) are copied byte for byte (32 MiB per format, 64 MiB total). CF_BITMAP/CF_PALETTE/CF_DSP* are never dereferenced; they are rebuilt by Windows from the DIB, so a bitmap without a DIB, metafiles, private/GDI ranges, unnamed registered ids, `DataObject`, a hung clipboard owner, an unreadable or oversized format all make the snapshot "lossy". Lossy => the text is typed with Unicode `SendInput`, the clipboard is untouched.
- **Paste.** Only after `SetClipboardData` succeeded. One `SendInput`: Backspaces, held Shift up, LCtrl down, V down, V up, LCtrl up, Shift down again. Scan codes 0x1D / 0x2F, injected tag kept.
- **Restore.** No signal exists for "the target consumed the paste", so: wait until the queue has been idle 400 ms (`SETTLE_MS`; each new job restarts it), then restore only if `GetClipboardSequenceNumber` equals the value taken right after our write (checked with the clipboard open). If anyone else wrote meanwhile the snapshot is dropped, never restored over their data; a later job re-snapshots the new content.
- **Privacy.** Our temporary text and the restored copy get `CanIncludeInClipboardHistory`, `CanUploadToCloudClipboard` (DWORD 0) and `ExcludeClipboardContentFromMonitorProcessing`, so Win+V and cloud sync skip them. Flag formats found in the user's data are not copied (ours replace them).
- **Panics.** Every job runs under `catch_unwind`, the clipboard is closed by an RAII guard, every API result is checked. The release profile is `panic = "abort"`, so in release the guarantee is "no unchecked call", not unwinding.
- **Limits / not verified.** `OpenClipboard(NULL)` is used (no owner window); the docs warn SetClipboardData may then fail for delayed data, we only set immediate data and check the result. Not run on a live Windows desktop here; the `#[ignore]`d round-trip test in `clipboard.rs` and a manual bitmap/PNG/Win+V check are still to do.
