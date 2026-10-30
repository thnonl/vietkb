# 97 — Host E2E, 0.2.4 candidate (QA, 2026-10-30, times UTC)

**Status: PARTIAL, ABORTED.** The last rerun (10:06:04 to 10:08:56) ran Notepad and the settings window, then aborted in the VS Code block at `type-B2` (10:08:52) on the guard "physical input after our last injection". Nothing was sent to a window the guard did not own. VS Code (D) and Chrome (A/B in Chrome) were NOT run. Results below use the screenshots in `qa\host-e2e\shots\`, because the harness verdicts were corrupted (see §4).

## Run history
- Run 1 (09:51): BLOCKED before input (EVKey64 running). No changes.
- Run 2 (10:00 start): ABORTED at first batch by the physical-input guard (owner activity). Teardown clean.
- Run 3 (10:06 start, after owner's 60 s idle gate, 77.5 s idle at start): ABORTED at VS Code `type-B2`. Teardown clean. Items below are from this run.

## 1. Environment
| Item | Value |
|---|---|
| Build | `E:\vietkb\target\release\vietkb.exe` 8 412 672 B, SHA-256 `E2189678BF5EDA8675BA31654B3AC0F244F3C6B3289E881B2A182618EEFC1D02` — matches, re-checked after run |
| Installer | `…\bundle\nsis\VietKB_0.2.4_x64-setup.exe` SHA-256 `5E37703BACD51C0DE5ED51574ED4403954DD10655E4955C6C96C11D3ABA652E4`; FileVersion `0.2.4` — matches |
| Monitors | DISPLAY2 primary 2048×1152 (0,0) 100 %; DISPLAY1 secondary 1920×1080 at (2560,0) 100 %, LogPixelsX 96 |
| Processes (pre-flight) | no EVKey, Unikey, installed VietKB, AutoHotkey or PowerToys. Dev `cargo`/`rustc` running (ignored per brief). No hook process. |
| Config | `%APPDATA%\VietKB\config.toml` SHA-256 `58EB422E…C704` before and after — **identical**. Run used `method = "telex"` (restored). |
| HKCU `Run\VietKB` | original `"E:\vietkb\target\debug\deps\vkb_windows-703be73b8891cfe5.exe" --autostart`. The candidate rewrote it to `…\target\release\vietkb.exe --autostart` on launch; QA restored it. Final value byte-identical to original (`-ceq`). |
| Clipboard | had text; text restored (HTML/other formats not restored) |

## 2. Per-item table (screen-verified)
| Item | Status | Keys → expected → actual (screen) |
|---|---|---|
| Pre-flight | PASS | idle 77.5 s; no owner IME or installed VietKB |
| A1 Notepad `dduwowcj` | PASS | → `được` → `được` |
| A2 Notepad `tieesng vieetj` | PASS | → `tiếng việt` → `tiếng việt` |
| A3 Notepad `nghieeng` | PASS | → `nghiêng` → `nghiêng` |
| A4 Notepad sentence `Tieesng Vieetj raast ddepj.` | FAIL | → `Tiếng Việt rất đẹp.` → `tiếng Việt rất eep.` (leading `T` lowercase, `dd`/`ệ` lost) |
| A5 Notepad `dduwowcj` ×10 | FAIL | → `được` ×10 → 3 of 10 correct; 7 garbled (`ơợc`/`ượợc`-type collapse). **Reproduces the open D-E2E-01 residual (three consecutive non-ASCII characters) in Notepad on 0.2.4.** |
| A — VS Code, Chrome | NOT RUN | aborted at VS Code `type-B2` |
| B1 Notepad `VIEETJ` (Shift per letter) | FAIL | → `VIỆT` → `việt` (leading V and I lost their Shift) |
| B2 Notepad `Vieejt` | FAIL | → `Việt` → `việt` (leading V lowercase) |
| B3 Notepad `AAS DD` | PASS | → `Ấ Đ` → `Ấ Đ` |
| C Notepad click ends word (`xyz⏎vie`, click end of line 1, `et`) | PASS | → `xyzet` / `vie` → `xyzet` / `vie`, no merge into `vie` |
| D VS Code F2, F1, `baatj` → `bật` | NOT RUN | aborted |
| E Notepad `vieejt` + Ctrl+Shift+Z | PASS | → `vieejt` → `vieejt` |
| F Other LL hook processes | RECORDED | none at pre-flight |
| G Toggle sound | SKIP | owner check (as before) |
| H1 Ctrl+Alt+S opens settings | PASS | settings window opened (Telex, Alt+Shift, "Khởi động cùng Windows" on, "Chạy ẩn" on) |
| H2 settings on DISPLAY1 | PASS (screen) | screenshot shows the whole window on DISPLAY1 (≈x 3282–3758, y 310–765). **Harness rect readout (4091,383,4709,968) disagrees with the screenshot — harness bug, not verified.** |
| H3 tray screenshot | CAPTURED | `shots\H-tray-primary.png`; icon presence not auto-checked, owner to view |
| I Installer hash and version | PASS | hash matches; FileVersion 0.2.4; installer not run |
| J Ignored tests | SKIPPED | listed only: `hook_installs_fires_hotkey_and_uninstalls`, `install_is_idempotent_and_stop_is_safe`, `clipboard::tests::text_round_trip_restores_previous_contents` (clobbers clipboard, not run) |

Observation: after Ctrl+Alt+S, Notepad showed its Alt-key accelerator overlay (menu letters). The Alt key reaches Notepad; not a VietKB defect in itself, but worth knowing for the settings-hotkey check.

## 3. Abort and teardown
- Abort (run 3): `physical input after our last injection, before type-B2-VS Code`, 10:08:52, about 45 s into the VS Code block. The key-hook watcher saw no physical key, and the cursor did not move. Most likely cause (unverified): `GetLastInputInfo` also counts input injected by the candidate's own re-injection of transformed keys, and the 50 ms tolerance is too small, so the guard produced a false positive. A fix is needed before another run: detect physical input with the LL keyboard and mouse hooks (checking the injected flags), not `GetLastInputInfo`.
- Teardown (launched by QA only; lines in `log.md`): Code scratch pids 26524, 44356, 25880, 9144 (name blank for 9144, process had already exited); Notepad pid 52976; candidate `vietkb.exe` pid 44820. Post-check: no candidate or scratch process left.
- Monitor used in run 1 (task b6jumrgxh) held `run-log.txt` open and was stopped at 10:10.

## 4. Harness defects (QA, to fix before the next run)
1. `L` wrote to the pipeline, so every `ReadField` return value had log lines mixed into it. The RESULT `actual` fields for A1 to A5, B1, B2, C1, E1 are therefore unreliable. Fixed in the script (`Write-Host`); the verdicts above come from the screenshots.
2. A `tail -f` on `run-log.txt` (run 1 monitor) caused write failures in run 2. Do not hold the log open during a run.
3. The settings-window rect readout in H2 did not match the screen (see table).
4. The physical-input guard uses `GetLastInputInfo` (see §3).

## 5. Next step (owner)
Items A4/A5/B1/B2 are real failures on Notepad and need a decision: they are the open D-E2E-01 residual and a Shift-on-first-letter pattern, and I did not verify the Shift one beyond the screenshot. The remaining items (VS Code D, Chrome A/B, and the reference to hook-based idle) need one more run once the guard is replaced.
