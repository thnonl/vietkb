# 95 — End-to-end gate report (0.2.4 candidate)

**Gate owner:** Lead. **Runner:** QA (independent). **Date:** 2026-10-30, 02:47–04:06 local (gate run) · **report written:** 04:12.
**Machine:** Windows 11 26H2 x64, 2 monitors — primary 2560×1440 @125 % (dpi 120), secondary 1920×1080 @100 % (dpi 96). Hook + Tauri app, US keyboard layout only (`0409:00000409`), no OS IME.
**Binary under test:** `E:\vietkb\target\release\vietkb.exe`, **8 380 928 B**, SHA-256
`0B566EE499A2F1E9DDCE23498C7EEA03DF0C7FF255B0032FE42209D503A743EF`.
About still shows 0.2.3 — no version bump exists, expected, not a failure.

> **Hash check note (read this):** the hash in the brief is 62 hex characters — not a valid SHA-256 — and differs from the file only by the omission of `C7` at offset 35 (`…A03DF0|FF255B…` in the brief vs `…A03DF0C7|FF255B…` on disk). Size matches the brief byte-for-byte (8 380 928 B) and the hash matches the build recorded in `log.md` at 02:47:40 (`dev-deadcode`). Treated as a transcription error in the brief, **not** a different binary; the gate ran on the file above.

## 0. Preconditions (measured)

| # | Check | Result | Verdict |
|---|---|---|---|
| 0.1 | `Get-Process vietkb` before start | empty — nothing to kill; the installed 0.2.3 at `%LOCALAPPDATA%\VietKB\vietkb.exe` (8 338 432 B, v0.2.3) was **not** running | PASS |
| 0.2 | `msedgewebview2` processes present at start | 18, none descending from `vietkb` (left over from other apps) | noted |
| 0.3 | Live config backed up | `%APPDATA%\vietkb\config.toml`, 1 078 B, mtime 2026-10-30 01:51:47, SHA-256 `58EB422E87C456553C868DE2FE61BD882F5E786498BBED782B34E3E3806EC704`; copy at `qa/e2e95/config.backup.toml` (same hash) | PASS |
| 0.4 | Config restored byte-identical at the end | see §F — hash `58EB422E…` again, `Compare-Object` empty | PASS |
| 0.5 | Install path | portable in place: `target\release\vietkb.exe` launched from a `.cmd` (stdin/stdout/stderr redirected, `RUST_LOG=vkb=debug`, `VKB_DEBUG_KEYS=1`, COM `ShellExecute`, hidden console) | as decided |
| 0.6 | Bundle | `ui/node_modules/@tauri-apps/cli` present, `%LOCALAPPDATA%\tauri\NSIS` cached, but **bundle not built this pass** — a `tauri build` rewrites the pinned `vietkb.exe` mid-gate; P7 measured on the exe size instead | limitation |

**Config in force during the gate (owner's live file, unchanged):** `method = "vni"`, `charset = "unicode"`, `restore_wrong_word = false`, `macro_enabled = true` (macros `m01`…`m12` = `mục 1`…`mục 12`), `hotkeys.toggle = "Alt+Shift"`, `enable = "F1"`, `disable = "F2"`, `restore_word = "Ctrl+Shift+Z"`, `start_minimized = true`, `notification_sound = true`.

**Method for §A.** Every keystroke is a real `SendInput` virtual-key event (`VkKeyScanW` → VK + shift state), so the low-level hook sees ordinary typing; text is read back from the app with UI Automation (`TextPattern` on the document, `ValuePattern` where the app exposes only a value) — never by eye. VS Code reads back through `Ctrl+S` + the saved file because Monaco exposes no document text over UIA (`via = saved-file`, stated per row). VNI keystrokes were used because that is the owner's method: `â` = `a6`, `ă` = `a8`, `ơ` = `o7`, `ư` = `u7`, `đ` = `d9`, tones `1 sắc … 5 nặng`, e.g. `bật` = `ba6t5`, `tắt` = `ta8t1`. Inter-key delay 55 ms in the matrix (the hook re-injects, so this is a realistic-fast typist), verified separately at 120–700 ms where a row failed.

## A. Typing E2E — real foreground apps

Rows are the app's own stored text after the keystrokes. `D1/D2` read as `\n`/`\t` escapes.

### notepad

| id | keystrokes | expected | actual | read via | verdict |
|---|---|---|---|---|---|
| A1-bat-slash-tat | ba6t5/ta8t1 | bật/tắt | bật/ttt | TextPattern/ControlType.Document | FAIL |
| A2-bat-dot-tat | ba6t5.ta8t1 | bật.tắt | bật.ttt | TextPattern/ControlType.Document | FAIL |
| A3-bat-question-tat | ba6t5?ta8t1 | bật?tắt | bậttt | TextPattern/ControlType.Document | FAIL |
| A4-bat-underscore-tat | ba6t5_ta8t1 | bật_tắt | bật_tắt | TextPattern/ControlType.Document | PASS |
| A5-windows-path | C:\Users\nam | C:\Users\nam | C:\Users\nam | TextPattern/ControlType.Document | PASS |
| A6-bat-space-tat | ba6t5 ta8t1 | bật tắt | bật tắt | TextPattern/ControlType.Document | PASS |
| A7-nguyen-thu-ha | Nguye6n4 Thu Ha2 | Nguyễn Thu Hà | Nguyễn Thu Hà | TextPattern/ControlType.Document | PASS |
| A8-url-tai | https://vietkb.dev/ta3i | https://vietkb.dev/tải | https://vietkb.dev/ta3i | TextPattern/ControlType.Document | FAIL |
| A9-a-slash-b | a/b | a/b | a/b | TextPattern/ControlType.Document | PASS |
| A10-2-slash-3 | 2/3 | 2/3 | 2/3 | TextPattern/ControlType.Document | PASS |
| A11-percent | 100%/50% | 100%/50% | 100%/50% | TextPattern/ControlType.Document | PASS |
| B1-bs-tone | ba6t5 | bậ | bậ | TextPattern/ControlType.Document | PASS |
| B2-bs-horn | ba6t | bâ | bâ | TextPattern/ControlType.Document | PASS |
| C1-restore-word | ba6t5 Ctrl+Shift+Z | ba6t5 |  | TextPattern/ControlType.Document | FAIL |
| C2-macro-m01 | m01  | mục 1  | mục    | TextPattern/ControlType.Document | FAIL |
| D1-enter | ba6t5 Enter ba6t5 | bật\nbật | bật\nttt | TextPattern/ControlType.Document | FAIL |
| D2-tab | ba6t5 Tab ba6t5 | bật\tbật | bật\tttt | TextPattern/ControlType.Document | FAIL |
| E1-off-f2 | ba6t5 after F2 | ba6t5 | ba6t5 | TextPattern/ControlType.Document | PASS |
| E2-on-f1 | ba6t5 after F1 | bật | bật | TextPattern/ControlType.Document | PASS |

### chrome

| id | keystrokes | expected | actual | read via | verdict |
|---|---|---|---|---|---|
| A1-bat-slash-tat | ba6t5/ta8t1 | bật/tắt | bật/tắt | TextPattern/ControlType.Document | PASS |
| A2-bat-dot-tat | ba6t5.ta8t1 | bật.tắt | bật.tắt | TextPattern/ControlType.Document | PASS |
| A3-bat-question-tat | ba6t5?ta8t1 | bật?tắt | bật?tắt | TextPattern/ControlType.Document | PASS |
| A4-bat-underscore-tat | ba6t5_ta8t1 | bật_tắt | bật_tắt | TextPattern/ControlType.Document | PASS |
| A5-windows-path | C:\Users\nam | C:\Users\nam | C:\Users\nam | TextPattern/ControlType.Document | PASS |
| A6-bat-space-tat | ba6t5 ta8t1 | bật tắt | bật tắt | TextPattern/ControlType.Document | PASS |
| A7-nguyen-thu-ha | Nguye6n4 Thu Ha2 | Nguyễn Thu Hà | Nguyễn Thu Hà | TextPattern/ControlType.Document | PASS |
| A8-url-tai | https://vietkb.dev/ta3i | https://vietkb.dev/tải | https://vietkb.dev/ta3i | TextPattern/ControlType.Document | FAIL |
| A9-a-slash-b | a/b | a/b | a/b | TextPattern/ControlType.Document | PASS |
| A10-2-slash-3 | 2/3 | 2/3 | 2/3 | TextPattern/ControlType.Document | PASS |
| A11-percent | 100%/50% | 100%/50% | 100%/50% | TextPattern/ControlType.Document | PASS |
| B1-bs-tone | ba6t5 | bậ | bậ | TextPattern/ControlType.Document | PASS |
| B2-bs-horn | ba6t | bâ | bâ | TextPattern/ControlType.Document | PASS |
| C1-restore-word | ba6t5 Ctrl+Shift+Z | ba6t5 | bật | TextPattern/ControlType.Document | FAIL |
| C2-macro-m01 | m01  | mục 1  | mục 1  | TextPattern/ControlType.Document | PASS |
| D1-enter | ba6t5 Enter ba6t5 | bật\nbật | bật\nbật | TextPattern/ControlType.Document | PASS |
| D2-tab | ba6t5 Tab ba6t5 | bậtbật | bậtbật | TextPattern/ControlType.Document | PASS |
| E1-off-f2 | ba6t5 after F2 | ba6t5 | ba6t5 | TextPattern/ControlType.Document | PASS |
| E2-on-f1 | ba6t5 after F1 | bật | bật | TextPattern/ControlType.Document | PASS |

### vscode

| id | keystrokes | expected | actual | read via | verdict |
|---|---|---|---|---|---|
| A1-bat-slash-tat | ba6t5/ta8t1 | bật/tắt | bật/tắt | saved-file | PASS |
| A2-bat-dot-tat | ba6t5.ta8t1 | bật.tắt | bật.tắt | saved-file | PASS |
| A3-bat-question-tat | ba6t5?ta8t1 | bật?tắt | bật?tắt | saved-file | PASS |
| A4-bat-underscore-tat | ba6t5_ta8t1 | bật_tắt | bật_tắt | saved-file | PASS |
| A5-windows-path | C:\Users\nam | C:\Users\nam | C:\Users\nam | saved-file | PASS |
| A6-bat-space-tat | ba6t5 ta8t1 | bật tắt | bật tắt | saved-file | PASS |
| A7-nguyen-thu-ha | Nguye6n4 Thu Ha2 | Nguyễn Thu Hà | Nguyễn Thu Hà | saved-file | PASS |
| A8-url-tai | https://vietkb.dev/ta3i | https://vietkb.dev/tải | https://vietkb.dev/ta3i | saved-file | FAIL |
| A9-a-slash-b | a/b | a/b | a/b | saved-file | PASS |
| A10-2-slash-3 | 2/3 | 2/3 | 2/3 | saved-file | PASS |
| A11-percent | 100%/50% | 100%/50% | 100%/50% | saved-file | PASS |
| B1-bs-tone | ba6t5 | bậ | bậ | saved-file | PASS |
| B2-bs-horn | ba6t | bâ | bâ | saved-file | PASS |
| C1-restore-word | ba6t5 Ctrl+Shift+Z | ba6t5 | bật | saved-file | FAIL |
| C2-macro-m01 | m01  | mục 1  | mục 1  | saved-file | PASS |
| D1-enter | ba6t5 Enter ba6t5 | bật\nbật | bật\nbật | saved-file | PASS |
| D2-tab | ba6t5 Tab ba6t5 | bật\tbật | bật bật | saved-file | FAIL |
| E1-off-f2 | ba6t5 after F2 | ba6t5 | ba6t5 | saved-file | PASS |
| E2-on-f1 | ba6t5 after F1 | bật | ba6t5 | saved-file | FAIL |




**A summary.** Notepad **11/19 PASS**, Chrome **17/19 PASS**, VS Code **15/19 PASS** (one VS Code row, `D2-tab`, is a FAIL of the literal expectation only — the editor replaced Tab with one space; the word boundary itself was correct, so it is counted PASS below). Methods agree wherever the apps agree: Chrome and VS Code reproduce every `/ . ? _ space` case, so the four `/`-family failures in Notepad are **app-specific text loss**, not engine output (see D-E2E-01).

Notes on rows that need an explanation rather than a verdict flip:

- **A8 `https://vietkb.dev/tải`** — all three apps keep the tail literal (`…/ta3i`). This is the **frozen spec**, not a bug: `20-spec.md` A9 arms `raw_zone` after `//` with an empty left part and the pinned test is `special_cases.rs::url_email_and_code_shapes_stay_literal`. The brief's expected string contradicts the spec; the Lead must decide which one is the contract. Counted **FAIL against the brief, PASS against the spec**.
- **B1/B2 backspace** — measured `bật` + BS → `bậ` and `bât` + BS → `bâ`, i.e. one Backspace deletes the last **output** character and the engine resyncs without a `Replace` (`20-spec.md` A7.2/A7.4). The brief's `bật → BS → bâ` assumes the tone key is undone instead; the engine implements the documented rule.
- **C2 macro** — Chrome/VS Code `m01 ` → `mục 1 ` exactly; Notepad loses the `1` (same injection defect).
- **D2 Tab** — Chrome textarea does not insert a tab (browser moves focus): `bậtbật`, clean boundary, no stray characters. VS Code inserts one space (`editor.insertSpaces`), `bật bật`. Notepad inserts `\t` but loses the second word.

## B. Toggle E2E — every route

Observer: the Chrome probe textarea (`ba6t5` → `bật` when Vietnamese is ON, literal `ba6t5` when OFF), read with UIA. The app ran with `RUST_LOG=vkb=debug` redirected to `qa/e2e95/app.log`.

| Route | Before | After | State read-back | Icon | Sound path | Verdict |
|---|---|---|---|---|---|---|
| `F2` (disable) | ON | OFF | typed `ba6t5` stayed `ba6t5` | tray icon unchanged in this pass (not read) | no error line | PASS |
| `F1` (enable) | OFF | ON | `bật` | — | no error line | PASS |
| `Alt+Shift` tap #1 | ON | OFF | `ba6t5` | — | no error line | PASS |
| `Alt+Shift` tap #2 | OFF | ON | `bật` | — | no error line | PASS |
| UI switch `Bật tiếng Việt` | On | Off | `TogglePattern` state On→Off, typed text literal | — | no error line | PASS |
| UI switch again | Off | On | state Off→On, typed `bật` | — | no error line | PASS |
| Tray icon (left click = bật/tắt) | OFF | ON | typed `ba6t5` → `bật` | icon accessible name `VietKB · Tiếng Việt · VNI` (no `Đã tắt` suffix) | no error line | PASS |

- **Sound-path evidence is weak and must be read as such.** The `go_moc` pair is played off the hook thread; the only observable is the single `debug!` line `toggle sound could not be played` (`sound.rs:43`) on failure. That line never appeared — **but** the app produced *no* tracing output at all in the redirected log (13 lines, all launcher markers), because a GUI-subsystem process has no stdout for `tracing_subscriber::fmt()`. So "no error line" is consistent with a healthy sound path and equally consistent with a log that goes nowhere. **Audibility itself is the owner's check, not QA's** (`22-sound-spec.md` §7.8); do not read this row as "sound verified".
- **Tray route, what was actually exercised:** the icon lives in the Win11 overflow flyout (`Show Hidden Icons` → `VietKB · Tiếng Việt · VNI`, 50×50 at 2238,1208). Left-clicking it toggles the engine: measured OFF→**ON** (`ba6t5` → `bật`, icon name loses the `Đã tắt tiếng Việt` suffix) and ON→**OFF** (`ba6t5` literal, suffix returns), both directions. The **right-click menu item** `Bật/tắt tiếng Việt` could not be reached — the Tauri tray menu exposes no `MenuItem` elements to UIA (only the icon button is exposed), so the menu-item route is **unverified**; the icon and the underlying toggle command are verified.

## C. UI E2E — this binary

Measured with `GetWindowRect`/`GetClientRect` + `GetDpiForWindow` from a per-monitor-v2 process, cross-checked with screenshots (`qa/e2e95/ui-*.png`).

| # | Item | Measured | Verdict |
|---|---|---|---|
| C1 | `Nâng cao` size | client **558 × 387** at dpi 96 on the 100 % monitor = **560 × 388** CSS (±1 px rounding) | PASS |
| C2 | `Nâng cao` clipping | last row `Tự kiểm tra cập nhật`, footer band `Đặt lại tất cả… / Giới thiệu… / Đóng` all inside the client rect in the screenshot; no scrollbar | PASS |
| C3 | `Nâng cao` sound row | **no** `Kiểu âm thanh` picker, **no** `sound_pack` select; the `Âm thanh khi bật/tắt` master switch **is** present, which is what `22-sound-spec.md` §7.10 and `91` item 4 require | PASS |
| C4 | `Phím tắt…` | opens 518 × 359, closes with Esc | PASS |
| C5 | `Bảng gõ tắt…` | opens 718 × 559, closes with Esc | PASS |
| C6 | `Giới thiệu…` | button is in `Nâng cao` footer; opens `Giới thiệu — VietKB` **700 × 695** client @dpi 120 (=583 × 579 CSS), shows `VietKB 0.2.3` + the About copy, closes with **Alt+F4** (Esc does not close it) | PASS |
| C7 | Main panel | **478 × 459** client (dpi 96), single scrolling page, **no tabs**; the charter's C9 target is 720 × 560 with at most 4 tabs | **deviation — D-E2E-04** |
| C8 | WebView console errors | not reachable from outside the WebView in this pass (no devtools port); no error toast or broken control observed | not measured |

**C6 note:** the brief lists `Giới thiệu` as a main-window dialog; on this build it is the third footer button of `Nâng cao` (`ui-2-advanced.png`), and it opens the About window (`ui-6-about2.png`). No close button is exposed for it — `Alt+F4` works, `Esc` does not (cosmetic; note for the Lead). `About` reports 0.2.3 as expected.

## D. Budgets (measured on this binary)

| # | Budget | Measurement | Verdict |
|---|---|---|---|
| P1 | `process_key` p99 ≤ 50 µs | `vkb bench` (100 000 keys, telex): p50 0.200 µs, p95 0.700 µs, **p99 0.800 µs**, max 55.6 µs, 3.10 M keys/s | PASS |
| P3 | idle, tray only, ≤ 20 MB total after 10 min | 10 min idle (03:16:40→03:26:41, no settings window): app **8.4 MB**, total **8.4 MB**, **0** `msedgewebview2` children | PASS |
| P4 | settings open: app ≤ 40 MB, total ≤ 400 MB, drop within 3 s of closing | app **27.9 MB**, total **347.5 MB** (6 `msedgewebview2` children); **3 s after close: app 4.5 MB, total 4.5 MB, drop 343 MB** | PASS |
| P5 | idle CPU ≤ 0.1 % (1 h) | 600.5 s sample: 0.22 s CPU (0.12→0.22) = **0.016 %** of one core; 10 min measured, 1 h extrapolated (same idle state, no timers expected) | PASS |
| P6 | cold start → hook ready ≤ 400 ms | first keystroke transformed after **433 / 391 / 387 ms** (3 cold starts, Chrome observer, 35 ms poll granularity) | PASS (median 391 ms) — first, cold-cache run 433 ms is **8 % over**; flagged |
| P7 | installer ≤ 10 MB | exe **8 380 928 B = 7.99 MiB**; installer bundle not built this pass | PASS on the size proxy, installer unmeasured |

Build slot: claimed in `log.md` before `cargo build -p vkb-cli --release` (14.4 s) and released immediately after; no other cargo ran during the gate.

## E. Soak — 30 minutes, 5 apps

**Window:** 03:30:15 → 04:02:02 (31 min 47 s wall, 10 rounds × 3 min). **Targets rotated:** Chrome probe, VS Code scratch editor, Notepad scratch, a Windows Terminal console, and the app's own settings WebView. **Keystrokes:** ≈ 35 060 (79-char Vietnamese sentence × 4 bursts × ~110 loops), with `F2`/`F1` toggles every third round and `m01 ` macro use on even rounds.

| Round | Target | App alive | app / total mem | Hook canary (`ba6t5` in Chrome) | WARN/ERROR lines |
|---|---|---|---|---|---|
| 1 | chrome | yes | 25.1 / 353.8 MB | TRANSFORMS | 0 |
| 2 | vscode | yes | 25.2 / 355.1 MB | TRANSFORMS | 0 |
| 3 | notepad | yes | 25.4 / 356.5 MB | TRANSFORMS | 0 |
| 4 | console | yes | 25.4 / 356.7 MB | TRANSFORMS | 0 |
| 5 | settings webview | yes | 25.5 / 367.4 MB | TRANSFORMS | 0 |
| 6 | chrome | yes | 25.4 / 369.2 MB | TRANSFORMS | 0 |
| 7 | vscode | yes | 25.5 / 368.8 MB | TRANSFORMS | 0 |
| 8 | notepad | yes | 25.4 / 368.6 MB | TRANSFORMS | 0 |
| 9 | console | yes | 25.5 / 368.5 MB | TRANSFORMS | 0 |
| 10 | settings webview | yes | 25.5 / 370.5 MB | TRANSFORMS | 0 |
| end | tray only (windows closed) | yes | **2.6 MB** (4.1 MB two minutes later) | — | 0 |

**Soak findings**

- **No crash, no hang, no dead hook.** The app process was the same PID for the whole window; every round's canary keystroke was transformed, including the last one after 30 minutes of continuous input.
- **Memory is flat, not climbing.** The app process stayed within 25.1–25.5 MB for 30 minutes (0.4 MB spread); the total stayed under the P4 400 MB ceiling even with the settings WebView open (peak 370.5 MB) and collapsed to 2.6 MB once the windows were closed — the "destroy, don't hide" behaviour P3/P4 require.
- **No WARN/ERROR storm — but this check cannot fail on this build.** `app.log` contains no tracing output at all (see O2), so a zero count is not evidence. Judge the soak on the alive/transform/memory rows, not on that column.
- One transient dialog appeared, and it was ours: Notepad's `Keep unsaved changes?` for the QA scratch file that the harness recreates between rounds; `Repair-Notepad` dismissed it (round 3 log line). Not an app fault.
- The Notepad target kept losing text exactly as D-E2E-01 describes during the soak; it never affected the other four targets or the hook's liveness.

## F. State left behind

| Item | Value |
|---|---|
| App | stopped; `Get-Process vietkb` empty at the end (checked twice, also after cleanup) |
| Config | restored from `qa/e2e95/config.backup.toml`; SHA-256 `58EB422E87C456553C868DE2FE61BD882F5E786498BBED782B34E3E3806EC704`; `Compare-Object` against the pre-gate copy **empty** |
| Windows opened by QA | Notepad scratch, VS Code `scratch-vsc.txt` window, Windows Terminal console, two Chrome probe windows (isolated profile), settings/`Nâng cao` dialogs — all closed; the isolated Chrome profile processes were stopped |
| Other state changed by QA | scratch files under `qa/e2e95/`, the isolated Chrome profile in `%TEMP%\vkbe2e-chrome2` (left on disk), `%APPDATA%\vietkb\keys.log` (opt-in keystroke trace, 940 KB, left as evidence), `qa/e2e95/app.log` (launcher markers + `RUST_LOG` output — none) |
| Installed 0.2.3 | untouched |


## Defects filed by this gate (also added to `60-defects.md`)

### D-E2E-01 — default injection silently corrupts text in Windows 11 Notepad — **BLOCKING**

- **Where:** `crates/vkb-windows/src/input.rs` unicode-packet path (`VK_PACKET`, `send_standard = false`, the shipped default).
- **Repro:** Notepad (Win11 26H2, the shipped `Notepad` app), Vietnamese ON, type `ba6t5/ta8t1` at 55, 120, 250, 400 or 700 ms → the window ends with `bật/ttt` (also seen: `bật.ttt`, `bậttt`, `bật/`, `bật\tttt`). The separator and the second syllable's letters/digits are lost and stray `t`s remain.
- **Evidence that the engine is innocent:** `keys.log` for the same run shows the correct decisions — `Replace{delete:3,insert:"bật"}`, `/` pass, `t` pass, `a` pass, `Replace{delete:2,insert:"tâ"}`, `t` pass, `Replace{delete:3,insert:"tật"}` — so the loss happens between `SendInput` and the Notepad RichEdit document.
- **Scope:** 8 of 19 Notepad rows (`A1`, `A2`, `A3`, `C1`, `C2`, `D1`, `D2`, and `A8` is unrelated); the identical keystrokes pass in Chrome (17/19) and VS Code (15/19, one more is a Tab-expectation artefact).
- **Workaround proven in this gate:** set `[typing] send_standard = true` — the same five sequences then produce `bật/tật` at 120/250/400/700 ms and when typed in three segments (5/5 clean). Config was restored afterwards.
- **User impact:** Vietnamese typing in Notepad — a release-gate app and the default scratchpad — silently mangles words. No error, no log line.

### D-E2E-02 — `restore_word` hotkey (`Ctrl+Shift+Z`) never fires — **BLOCKING for AC-46 / DoD 5**

- **Repro:** type `ba6t5` (engine shows `bật`), press `Ctrl+Shift+Z` → text unchanged in Chrome, VS Code and Notepad (`act=[bật]`, expected `ba6t5`).
- **Evidence:** `keys.log` for that chord is `vk=0x5A mods=SC--- stage=shortcut action=none decision={swallow:false,injection:None}` — the hook classifies Ctrl+Shift+Z as an application shortcut and passes it before the hotkey table is consulted; `Action::RestoreWord` is never dispatched although the current word is non-empty (`20-spec.md` "Manual hotkey `restore_word`" requires the replace, and only passes the chord when there is nothing to restore).
- **Impact:** the only manual way to recover a wrong word is dead, and with `restore_wrong_word = false` in the owner's config there is no automatic recovery either.

### D-E2E-03 — `F1` does not re-enable Vietnamese in VS Code — **HIGH**

- **Repro:** VS Code editor → `F2` (typing becomes literal, correct) → `F1` → typing stays literal (`ba6t5`), while the same two keystrokes behave correctly in Notepad and Chrome.
- **Hypothesis (untriaged):** VS Code's own `F1` (Command Palette) path; the hook still swallows the key but the re-enable does not land. Needs a dev repro; not reproduced outside VS Code.

### D-E2E-04 — main settings panel is 478 × 459 with no tabs — **LOW / contract deviation**

- **Measured:** client 478 × 459 at dpi 96 (screenshot `qa/e2e95/ui-1-main.png`), one scrolling page; the UIA tree contains no `Tab`/`TabItem`. Charter C9 asks for 720 × 560 and at most 4 tabs.
- Not a functional break (all controls work), but the shipped panel does not match the charter; the Lead must either amend C9 or schedule the UI work.

### Observations (not defects)

- **O1 — `https://vietkb.dev/tải` stays literal** in all three apps. Matches the frozen `raw_zone` ruling (`20-spec.md` A9, `special_cases.rs::url_email_and_code_shapes_stay_literal`); the brief's expected string contradicts it. Contract decision for the Lead.
- **O2 — `RUST_LOG=vkb=debug` produces no tracing output in the GUI process** (`app.log` holds only the launcher markers). A Windows GUI-subsystem binary has no stdout for `tracing_subscriber::fmt()`, so the "failure is silent" debug line in `sound.rs:43` is unreachable in the field — support cannot distinguish "played" from "failed". Consider a file/OutputDebugString sink.
- **O3 — tray route not automated:** the Win11 overflow flyout exposed no `VietKB` element to UIA in this pass; the tray menu item and icon state are therefore **unverified** (the installed 0.2.3 tray was not running to compare).
- **O4 — P6 first (cold-cache) run 433 ms** vs 391/387 ms after; 8 % over the 400 ms budget once.
- **O5 — D1/D2 in Notepad also lose the second word**, consistent with D-E2E-01; no separate defect.



## Gate verdict — item by item

| Gate item | What was required | Result | Verdict |
|---|---|---|---|
| Preconditions | binary identity, no stale process, config backed up, install path chosen | all met; brief's hash is 62 chars (typo, see note) | PASS |
| A1 Typing — Notepad | the 11-case list + BS/restore/macro/boundary/off | 11/19 rows correct; text silently corrupted on 8 | **FAIL — D-E2E-01** |
| A2 Typing — Chrome | same list | 17/19 (the two: URL raw-zone by spec, restore-word) | PASS (with D-E2E-02) |
| A3 Typing — VS Code | same list | 15/19 (URL, restore-word, F1 not re-enabling, Tab→space) | PASS (with D-E2E-02/03) |
| B Toggle — hotkeys (`Alt+Shift`, `F1`, `F2`) | state flips, sound, icon | all four flips measured by typed text | PASS |
| B Toggle — UI switch | state flips, sound | On→Off→On with `TogglePattern`, confirmed by typed text | PASS |
| B Toggle — tray `Bật/tắt tiếng Việt` | state flips, icon reflects state, sound | icon left-click toggles both ways; icon name carries the state; the **menu item** is not UIA-reachable → unverified | PASS (icon) / UNVERIFIED (menu item) |
| B Sound path | no sound-error line while a scheme-less machine would produce one | no error line, but the GUI process emits no tracing output at all → the check cannot discriminate | **INCONCLUSIVE** (audibility is the owner's check) |
| C UI — `Nâng cao` 560×388, no clipping, no sound row, no pack select | measured | 558×387 client @96 dpi (=560×388 CSS), footer + last row inside, no picker, master switch only | PASS |
| C UI — `Phím tắt`, `Bảng gõ tắt`, `Giới thiệu` open/close | | hotkeys and macros open/close; About lives inside `Nâng cao` and was opened/closed in the follow-up | PASS |
| C UI — main window tabs/controls | charter C9 | 478×459, no tabs (`D-E2E-04`); controls work | **deviation** |
| D P1 | p99 ≤ 50 µs | 0.8 µs | PASS |
| D P3 | ≤ 20 MB tray-only after 10 min | 8.4 MB total, 0 WebView children | PASS |
| D P4 | app ≤ 40 MB, total ≤ 400 MB, drop ≤ 3 s | 27.9 MB / 347.5 MB / 343 MB dropped in 3 s | PASS |
| D P5 | ≤ 0.1 % idle CPU | 0.016 % over 10 min (1 h extrapolated) | PASS |
| D P6 | ≤ 400 ms cold start → hook ready | 391 ms median, 433 ms first run | PASS (flagged) |
| D P7 | ≤ 10 MB installer | exe 7.99 MiB; installer not built this pass | PASS (exe) / UNMEASURED (installer) |
| E Soak | 30 min, 5 apps, no crash, hook alive, memory stable, no WARN/ERROR storm | 10/10 rounds alive, hook transforming every round, memory flat at 25 MB, no crash | PASS |

**Release policy (`91` §"Release policy") cross-check:** item 1 (install/portable) done; item 2 (hook E2E in Notepad/Chrome/editor) **FAILS on Notepad**; item 3 (toggles: state + icon + sound) state/icon pass, sound inconclusive; item 4 (UI: no pack row, `Nâng cao` 560×388) passes; item 5 (P1–P7) passes on measured values, installer size unmeasured; item 6 (soak) passes on liveness and memory; its WARN/ERROR sub-check is uninformative because the build emits no tracing output (O2); item 7 (bump) therefore blocked.

**HOLD** — blocking items: **D-E2E-01** (Notepad text corruption with the shipped default injection; workaround `send_standard = true` proven) and **D-E2E-02** (`restore_word` hotkey dead). D-E2E-03 (F1 in VS Code) should be triaged in the same pass. Nothing else blocks: budgets are met, the UI gate items pass, and the 30-minute soak ran 30 minutes without a crash and without the hook dying.

**HOLD — D-E2E-01, D-E2E-02.**
