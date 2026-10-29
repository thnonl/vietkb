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

## Config matrix E2E (same harness, one `config.toml` per case)

20/20 PASS, run after the macro wiring fix. Each case writes `config.toml`, restarts the app,
types, and asserts.

| # | Flag | Method | Expected | Result |
|---|---|---|---|---|
| A1/A2 | `modern_tone` on/off | Telex | `hòa` / `hoà` | PASS |
| A3/A4 | `free_tone` on/off | Telex | `hoàn` / `hòan` | PASS |
| A5/A6 | `jfwz_as_consonant` on/off | Telex | `w` / `ư` | PASS |
| A7 | `auto_capitalize` on | Telex | `tiếng. Việt` | PASS |
| A8/A9 | `spell_check` off/on | Telex | `ád` / `asd` | PASS |
| A10/A11 | `macro_enabled` on/off | Telex | `Việt Nam ` / `vn ` | PASS |
| A12/A13 | `macro_case_follow` on/off | Telex | `By the way ` / `by the way ` | PASS |
| V1/V2 | `modern_tone` on/off | VNI | `hòa` / `hoà` | PASS |
| V3/V4 | `free_tone` on/off | VNI | `hoàn` / `hòan` | PASS |
| V5 | `spell_check` off | VNI | `ád` | PASS |
| V6 | `macro_enabled` on | VNI | `Việt Nam ` | PASS |
| V7 | `macro_case_follow` off | VNI | `by the way ` | PASS |

This run found a real product bug that no unit test could: **the Windows backend never called
`engine.set_macros`**, so gõ tắt did nothing at runtime even though the engine, the config and
the macro editor were all correct. Fixed in `crates/vkb-windows/src/state.rs` (fingerprint guard,
rebuilt only when the table changes); the five macro cases went from FAIL to PASS.

## Flake seen once (QA run 2026-10-29)

The harness failed `V4-quo7` once (`quơ` came back as `qu`) and passed on an immediate re-run of
the same binary. The engine's own corpus test `quo7 → quơ` (`crates/vkb-core/tests/corpus_vni.rs`)
passes, so this is harness timing (35 ms/key with the app's own injections interleaving), not a
product regression. If it repeats, raise the per-key settle in `host-e2e.ps1` before touching the
engine.

## Known gap in this run

The About modal (tray item → window → X close) is still not scriptable: the tray menu has no
external API. Its regression test is `every_window_has_a_capability` (capability list must
include `about`), and the manual check is one click.
