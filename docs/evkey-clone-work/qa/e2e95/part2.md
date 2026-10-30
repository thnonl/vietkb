
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
| C6 | `Giới thiệu…` | the button lives **inside `Nâng cao`**, not on the main panel | see note |
| C7 | Main panel | **478 × 459** client (dpi 96), single scrolling page, **no tabs**; the charter's C9 target is 720 × 560 with at most 4 tabs | **deviation — D-E2E-04** |
| C8 | WebView console errors | not reachable from outside the WebView in this pass (no devtools port); no error toast or broken control observed | not measured |

**C6 note:** the brief lists `Giới thiệu` as a main-window dialog; on this build it is the third footer button of `Nâng cao` (`Nâng cao` screenshot). It was not opened in this pass — no About content was verified. `About` reports 0.2.3 as expected.

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

`<SOAK>`

## F. State left behind

| Item | Value |
|---|---|
| App | stopped; no `vietkb.exe` running |
| Config | restored from `qa/e2e95/config.backup.toml`; SHA-256 `58EB422E87C456553C868DE2FE61BD882F5E786498BBED782B34E3E3806EC704`; `Compare-Object` against the pre-gate copy **empty** |
| Other state changed by QA | scratch files under `qa/e2e95/` (`scratch-np.txt`, `scratch-vsc.txt`), the isolated Chrome profile in `%TEMP%\vkbe2e-chrome2`, `keys.log` (opt-in trace, left as evidence), `%APPDATA%\vietkb\app.log`-equivalent under `qa/e2e95/app.log` |
| Installed 0.2.3 | untouched |

