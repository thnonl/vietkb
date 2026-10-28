# Log

- 2026-10-28 Sprint 0 start. Charter written (`00-charter.md`). Legacy code declared reference-only; owner authorized full deletion.
- 2026-10-28 Dispatched background scouts: legacy repo survey, EVKey capability research, toolchain check.
- 2026-10-28 Env facts recorded (charter §10): Rust 1.98.1 MSVC, Node 24.22.1, WebView2 155,
  no Tauri CLI, no NSIS/WiX yet, git needs `safe.directory`, legacy `target/` = 13.4 GB.
  PowerShell blocks `.ps1` shims → use `npm.cmd`/`npx.cmd`.
- 2026-10-28 BA: `20-spec.md` v1 (A engine, B config + 9 CCRs, C UI, D AC-1…AC-85) and `40-design-review.md` pass 1 (3 directions, rec. A "Lặng") written. CCR-1…9 await Lead.
- 2026-10-28 Legacy preserved: tag `legacy-v0`, branch `legacy/pre-rewrite`, 23 files copied to
  `reference/legacy/` (tests + core + hook + gui). Old tree deleted, 14.4 GB freed.
- 2026-10-28 EVKey inspected (bytes only, never executed): `SetWindowsHookExW` + `SendInput` +
  clipboard fallback, `ActivateKeyboardLayout`/`LoadKeyboardLayout` present (source of the
  unwanted-US-layout bug), **no** `RegisterHotKey`, no IMM. Settings: Win32 dialog, 3 tabs
  (Cơ bản/Phím tắt/Hệ thống), defaults F1/F2/F6/F8/F9/F11. Repo has no engine source → clean room.
- 2026-10-28 Scaffold committed (`13521e4`, 66 files). `cargo test -p vkb-core -p vkb-config`
  = 12 tests green, clippy clean, config round-trips unknown keys and empty-string unbinding.
- 2026-10-28 Sprint 1 dispatched: worktrees `wt-engine` (feat/engine), `wt-windows` (feat/windows),
  `wt-ui` (feat/ui) — all on DeepSeek V4.1 Flash max. QA started on `50-test-cases.md`.
- 2026-10-28 BA spec + 3 directions delivered (`20-spec.md` 8.5k words, `40-design-review.md`).
  Contract amended by Lead: `modern_tone` semantics corrected (kiểu mới = `hoà`/`thuý`, BA was
  right, the earlier comment was backwards), `Replace.delete_units` (UTF-16 units = Backspaces),
  new `ReplaceThenPass` for macro-on-Enter, `set_macros(MacroTable)` on the engine trait.
- 2026-10-28 Owner rejected the first UI direction pass as "chưa đẹp". Dedicated designer pass
  started (Opus 5.5 high) writing screenshot-ready mocks to `design/`; `scripts/shoot-design.ps1`
  renders them with headless Edge at 1040×720 for visual review.
- 2026-10-28 QA delivered `50-test-cases.md`: 551 engine rows, 48 UI tests, P1–P7 protocol.
  5 charsets remain UNVERIFIED (`tcvn3`, `vni_windows`, `viscii`, `bk_hcm1`, `vps`).
- 2026-10-28 Owner rejected Lead hand-editing product code during integration; charter now forbids it
  (Lead writes docs only, dev children own `crates/` and `ui/`).
- 2026-10-28 Three streams merged: `cargo test --workspace` = 151 tests green. Tone-style semantics
  corrected twice — final: `modern_tone = true` → `hòa`/`hóa` (tone on the first vowel), old →
  `hoà`/`hoá`. Design pills and COPY.md updated to match.
- 2026-10-28 Runtime smoke: app panicked at startup (`plugins.updater` missing in tauri.conf) and was
  handed to a dev child. UI build: 131 KB JS / 24.6 KB CSS gzipped to 46 KB / 5.5 KB.



- 2026-10-28 wt-windows: pure layer done in `vkb-platform` (injection planning, hotkey matcher,
  per-app rule resolution) + `vkb-windows` backend (WH_KEYBOARD_LL on a dedicated thread with
  install timeout and watchdog, batched `SendInput`, in-hook hotkeys, foreground cache, HKCU
  autostart, opt-in clipboard). Tests green; details in `70-impl-notes-windows.md`. No full
  build run, so no BUILD slot claimed.
- 2026-10-28 wt-ui: settings shell + 8 screens built (Svelte 5, strict TS, hand-written
  tokens); live-apply `patch_settings`, hotkey capture field, charset converter, macro
  import/export, running-app picker, tray with method/charset submenus. Notes:
  `70-impl-notes-ui.md`.
- 2026-10-28 wt-ui **contract change proposals (additive only, Lead to approve)**:
  1) `SettingsView` gains `apps: { rememberPerApp: boolean }`;
  2) new command `set_app_rule_overrides(process, charset: string|null, method: string|null)`
  because the frozen `set_app_rule` cannot carry per-app overrides. No frozen name or
  payload changed.
- 2026-10-28 **Env gotcha for everyone using node here**: the session exports
  `NODE_OPTIONS=--import=<headroom hook shim>`, which makes npm's network calls die with
  ECONNREFUSED and makes installs hang. Clear `NODE_OPTIONS` (and `NODE_EXTRA_CA_CERTS`) for
  npm/vite child processes: `$env:NODE_OPTIONS=$null; npm.cmd install --prefer-offline
  --no-audit --no-fund`. First install took 14 s after that fix (49 packages).
- 2026-10-28 BUILD: ui 2026-10-28 19:25 (wt-ui) — `npm.cmd run build` + `svelte-check`,
  `cargo check -p vkb-app`, `cargo test -p vkb-app` (10/10). Slot released.
  `ui/dist` = 158,723 bytes; 0 errors / 0 warnings. Visual review done on the built
  `dist` in both themes; notes in `70-impl-notes-ui.md`.
- 2026-10-28 wt-ui **C9/C10 scope change applied** (owner): sidebar/dashboard shell deleted
  (10 files), panel is now 720 × 560 frameless, 4 tabs — Gõ · Gõ tắt · Phím tắt · Hệ thống.
  Charset screen/convert tool and About removed (C10: separate windows later); Hệ thống
  absorbs app exceptions as a 3-row list + Quản lý dialog. Design handover for 720 × 560
  still pending (existing `design/DESIGN-SPEC.md` is the 1040 × 720 round).
  Blocked on vkb-config: `typing.auto_horn_uo`, `typing.raw_zone` (and a distinct
  `typing.free_marking`); those rows render disabled with the missing key named.
- 2026-10-28 wt-ui: **design handover applied** (`design/HANDOVER.md` + `DESIGN-SPEC.md` v3).
  Tokens + control kit ported to `tokens.css`/`kit.css`; 11 components; four tabs at
  720 × 560 frameless; tauri.conf set to 720×560 / min 680×520 / not resizable;
  corrections 1–5 applied. Still blocked on vkb-config: `typing.auto_horn_uo`,
  `typing.raw_zone`, `typing.macro_case_follow`, `compat.send_delay_us`,
  `hotkeys.cycle_charset`, `hotkeys.cycle_method`, `hotkeys.toggle_mode`.
- 2026-10-28 BUILD: ui 2026-10-28 (wt-ui) — `svelte-check` 0/0, `npm run build` pass,
  `cargo check -p vkb-app` pass. `ui/dist` = 151,771 B (43.65 KB gzip JS, 5.35 KB CSS);
  720 × 560 fit verified on all four tabs; 680 × 520 clips the Gõ test strip (window is
  fixed at 720 × 560). Slot released.
- 2026-10-28 wt-ui: **final copy pass** — COPY.md + `27-copy-decisions.md` applied
  verbatim (off state `Đã tắt tiếng Việt`, `Đặt lại…`, `Giữ nguyên email, địa chỉ web`,
  `Cần mở lại VietKB`). Schema additions in this worktree: `typing.auto_horn_uo`,
  `typing.raw_zone`, `typing.macro_case_follow`, `typing.send_delay_us`,
  `hotkeys.toggle_mode`, `hotkeys.cycle_charset`, `hotkeys.cycle_method`; hotkey defaults
  now Ctrl+Shift / Ctrl+Shift+Z / Ctrl+Alt+S; new `restart_app`; tray copy updated.
  `Nâng cao` = 9 options (40 px short rows to keep C9's no-scroll rule). Deviations:
  `Ctrl phải` preset omitted (no side-specific modifier in the hotkey grammar), import
  toast lacks the skipped-duplicate count, update check has no channel.
- 2026-10-28 BUILD: ui 2026-10-28 (wt-ui) — `svelte-check` 0/0, `npm run build` pass,
  `cargo test -p vkb-config` 9/9, `cargo test -p vkb-app` 11/11, `cargo check -p vkb-app`
  pass. `ui/dist` = 162,482 B; 720 × 560 fit on all four tabs plus the 9-row Nâng cao;
  125/150 % scale wrappers show no reflow; 680 × 520 clips the Thử gõ strip (window is
  fixed). Slot released.
