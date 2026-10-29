# 88 — E2E report: dev build, Telex + VNI (2026-10-29)

## Result

12/12 PASS on the host desktop, dev channel, against `target-dev\release\vietkb.exe` built from
`a775f58`. Compile-time evidence is `crates/vkb-core` + `feature_matrix*`; this is the run that
proves the whole chain: hook → engine → `SendInput` → a real edit control.

| # | Method | Input | Expected | Actual | Result |
|---|---|---|---|---|---|
| T1 | Telex | `tieengs` | `tiếng` | `tiếng` | PASS |
| T2 | Telex | `tieengs` + BS×3 | `ti` | `ti` | PASS |
| T3 | Telex | `Tha` + space + BS + `s` | `Thá` | `Thá` | PASS |
| T4 | Telex | `thuowng` | `thương` | `thương` | PASS |
| T5 | Telex | `af` | `à` | `à` | PASS |
| T6 | Telex | F2 then `tieengs` | `tieengs` | `tieengs` | PASS |
| T7 | Telex | F1 then `tieengs` | `tiếng` | `tiếng` | PASS |
| V1 | VNI | `Tha` + space + BS + `1` | `Thá` | `Thá` | PASS |
| V2 | VNI | `thu7ong` | `thương` | `thương` | PASS |
| V3 | VNI | `thuo7ng` | `thương` | `thương` | PASS |
| V4 | VNI | `quo7` | `quơ` | `quơ` | PASS |
| V5 | VNI | `tie6ng1` | `tiếng` | `tiếng` | PASS |

Profile isolation: `%APPDATA%\VietKB` file list and `config.toml` SHA256
(`CBB17F…A0DDA472`) are identical before and after the run — the dev channel wrote only to
`%APPDATA%\VietKB Dev`.

## How it ran

- `E:\vietkb\sandbox-share\e2e\host-e2e.ps1` (copy kept at `docs/evkey-clone-work/qa/e2e/`).
- A WinForms window with one multiline `TextBox` is the target; the dev app is launched with
  `VKB_DEBUG_KEYS=1`, so `%APPDATA%\VietKB Dev\keys.log` records every hook decision.
- Keys are typed with `[Windows.Forms.SendKeys]::SendWait`, 60 ms apart; the harness reads
  `$tb.Text` after each case.
- Between cases the harness sends **ESC** (class-3 reset, `28-engine-rulings.md` B3) and then
  clears the control. Without the ESC the engine's word buffer still held the previous case's
  word and re-injected it (`Thas` + `thuowng`), which is a harness defect, not a product one.
- Telex phase runs with `config.toml` pinned to `method = "telex"`, VNI phase rewrites it to
  `method = "vni"` and restarts the app.

## Why not Windows Sandbox

Windows Sandbox cannot deliver `KEYEVENTF_UNICODE` injections, so no IME can be exercised there.
Probe (`sandbox-share/e2e/probe-unicode.ps1`, no VietKB running): a `SendInput` unicode packet
returns `down=1 up=1` and the focused `TextBox` stays empty, for both `0x00EA` and `0x1EC7`.
`SendKeys` does land, which is why the sandbox harness showed raw keystrokes only.

Direct evidence that the app itself is correct there: `keys.log` shows
`stage=engine-swallow action=Replace{delete:3,insert:"tiê"} … send=12/12 lasterror=0 attempts=1`
and the injected packets reach the hook (`stage=own-injection`), they are simply not delivered
to the target by the sandbox session.

Sandbox remains the right place for installer, uninstall, autostart and profile-isolation
checks (those passed in `install-diag.txt`); typing E2E belongs on the real desktop.

## Known gap in this run

The About modal (tray item → window → X close) is still not scriptable: the tray menu has no
external API. Its regression test is `every_window_has_a_capability` (capability list must
include `about`), and the manual check is one click.
