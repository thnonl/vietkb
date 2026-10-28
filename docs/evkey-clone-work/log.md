# Log

- 2026-10-28 Sprint 0 start. Charter written (`00-charter.md`). Legacy code declared reference-only; owner authorized full deletion.
- 2026-10-28 Dispatched background scouts: legacy repo survey, EVKey capability research, toolchain check.
- 2026-10-28 Env facts recorded (charter §10): Rust 1.98.1 MSVC, Node 24.22.1, WebView2 155,
  no Tauri CLI, no NSIS/WiX yet, git needs `safe.directory`, legacy `target/` = 13.4 GB.
  PowerShell blocks `.ps1` shims → use `npm.cmd`/`npx.cmd`.
- 2026-10-28 BUILD: engine feat/engine 2026-10-28T09:20Z — claimed for vkb-core/vkb-cli sprint 1.
- 2026-10-28 Engine sprint 1: `vkb-core` state machine (Telex/VNI/VIQR, tone placement,
  `qu`/`gi`, `ươ`, uppercase, backspace over raw history, macro, spell, restore), 9 charsets
  with provenance status, `vkb-cli simulate`/`bench`. 131 tests green (debug + release),
  P1 p99 = 0.8 µs / 3.26 M keys·s⁻¹, `Noop` path allocation-free. Notes:
  `70-impl-notes-engine.md`. BUILD: engine released.
