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


