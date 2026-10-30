
---

# 60 — E2E gate defects, 0.2.4 candidate (2026-10-30)

**Owner:** QA · **Build:** `target\release\vietkb.exe` 8 380 928 B, SHA-256
`0B566EE499A2F1E9DDCE23498C7EEA03DF0C7FF255B0032FE42209D503A743EF`.
**Scope:** independent E2E gate (`95-e2e-report.md`): hook against Notepad / Chrome / VS Code,
toggle routes, UI, budgets, 30-min soak. No product code changed by this pass.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-E2E-01 | P1 (blocking) | default injection silently corrupts text in Windows 11 Notepad | Notepad (Win11 26H2), Vietnamese ON, default `typing.send_standard = false`; type `ba6t5/ta8t1` (VNI for `bật/tắt`) at 55–700 ms | `bật/tắt` | `bật/ttt` (also `bật.ttt`, `bậttt`, `bật/`, `bật\tttt`): separator and second-syllable letters/digits lost, stray `t`s remain. `keys.log` shows the engine emitted the correct `Replace{delete:3,insert:"bật"}` / pass / `Replace{delete:2,insert:"tâ"}` / `Replace{delete:3,insert:"tật"}` | `95-e2e-report.md` §A (Notepad 11/19), `qa/e2e95/typing-notepad.tsv`, `qa/e2e95/app.log`; identical keystrokes PASS in Chrome (18/19) and VS Code (17/19); `send_standard = true` makes 5/5 sequences correct at 120/250/400/700 ms | OPEN |
| D-E2E-02 | P1 (blocking) | `restore_word` hotkey `Ctrl+Shift+Z` never fires | type `ba6t5` (shows `bật`), press `Ctrl+Shift+Z` | current word replaced by its raw keystrokes `ba6t5` (`20-spec.md` "Manual hotkey `restore_word`", AC-46) | unchanged `bật` in Chrome, VS Code and Notepad; `keys.log`: `vk=0x5A mods=SC--- stage=shortcut action=none decision={swallow:false}` — the chord is treated as an app shortcut before the hotkey table | `qa/e2e95/typing-*.tsv` row `C1-restore-word`; `%APPDATA%\vietkb\keys.log` | OPEN |
| D-E2E-03 | P2 | `F1` does not re-enable Vietnamese in VS Code | VS Code editor: `F2` (typing literal — correct), then `F1`, then type `ba6t5` | `bật` | `ba6t5` (stays disabled); Notepad and Chrome re-enable correctly with the same keys | `qa/e2e95/typing-vscode.tsv` row `E2-on-f1` | OPEN |
| D-E2E-04 | P3 | main settings panel is 478 × 459 with no tabs (charter C9 deviation) | open the settings window, read `GetClientRect` + UIA tree | charter C9: 720 × 560, at most 4 tabs | client 478 × 459 at dpi 96, one scrolling page, no `Tab`/`TabItem` in the UIA tree; `Nâng cao` / `Phím tắt` / `Bảng gõ tắt` are separate windows | `qa/e2e95/ui-1-main.png`, `qa/e2e95/ui-results.txt` | OPEN (Lead to amend C9 or schedule work) |

**Observations, not defects**

- `https://vietkb.dev/tải` stays literal (`…/ta3i`) in all three apps — matches the frozen `raw_zone` ruling (`20-spec.md` A9, `special_cases.rs::url_email_and_code_shapes_stay_literal`); the gate brief expected a transform. Contract decision for the Lead.
- `RUST_LOG=vkb=debug` yields **no** tracing output in the GUI process (launcher markers only): the `sound.rs:43` "toggle sound could not be played" line is unreachable in the field, so sound success/failure cannot be diagnosed from logs.
- Tray **menu item** `Bật/tắt tiếng Việt` is not exposed to UIA (only the icon button is); the icon's left-click toggle and its state tooltip are verified, the menu item is not.
- P6 first cold-cache run 433 ms (over the 400 ms budget once); 391/387 ms after.
