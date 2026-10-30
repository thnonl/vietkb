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

---

## Build 0D39D42F (rebuilt candidate, D-E2E-06 fix, main 32e6f06): run v2, 11:10–11:12 UTC

**Status: PARTIAL, ABORTED in VS Code `rows` batch (11:12:05).** Notepad ran in full; VS Code and Chrome did not finish. Teardown clean.

### Build and environment
| Item | Value |
|---|---|
| exe | `E:\vietkb\target\release\vietkb.exe` SHA-256 `0D39D42F5F66CA4CE40D22068FA31DE940E32D811DD5F393C79FD775FADFEB92` — matches; FileVersion 0.2.4 |
| installer | `…\bundle\nsis\VietKB_0.2.4_x64-setup.exe` SHA-256 `CD45BD84E26DC3A46A4DF51CD96465A31932DEBE310741797AAE190D3812DCBC` — matches; FileVersion 0.2.4; not run |
| Monitors | DISPLAY2 primary; DISPLAY1 secondary 1920×1080 at (2560,0), used for test windows |
| Pre-flight | no EVKey, Unikey or installed VietKB; no AutoHotkey or PowerToys; `cargo`/`rustc` ignored |
| Idle gate | 61.25 s ≥ 60 s, before the first batch (GetLastInputInfo used only here) |
| Physical-input detection | our WH_KEYBOARD_LL (flags 0x10) and WH_MOUSE_LL (flags 0x01) only; 1.5 s grace around our own SetCursorPos. No physical event recorded before the abort. |
| Caps Lock | off at start; turned on only for B6 (`{CAPS_ON}`) and off again; verified off at teardown |
| Config | `method = "telex"` for the run; restored, SHA `58EB422E…C704`, identical |
| Run key | candidate rewrote it on launch; restored; final value byte-identical to original `…\target\debug\deps\vkb_windows-703be73b8891cfe5.exe --autostart` |
| Cursor / clipboard / focus | cursor restored to 931,1067; clipboard text restored; foreground restored |

### Results (verdicts from the clipboard copy after the batch; screenshot `shots2\Notepad-rows.png` agrees)
| Row | Notepad | VS Code | Chrome |
|---|---|---|---|
| A1 `dduwowcj` → `được` | PASS | NOT RUN | NOT RUN |
| A2 `tieesng vieetj` → `tiếng việt` | PASS | NOT RUN | NOT RUN |
| A3 `nghieeng` → `nghiêng` | PASS | NOT RUN | NOT RUN |
| A4 sentence → `Tiếng Việt rất đẹp.` | PASS | NOT RUN | NOT RUN |
| A5 `dduwowcj` ×10 | PASS, failures 0/10 | NOT RUN | NOT RUN |
| B1 `VIEETJ` (Shift per letter) → `VIỆT` | PASS | NOT RUN | NOT RUN |
| B2 `Vieejt` → `Việt` | PASS | NOT RUN | NOT RUN |
| B4 Shift held through `VIEETJ` → `VIỆT` | PASS | NOT RUN | NOT RUN |
| B5 `duong` + Shift+J → (recorded) | RECORD: `dụong` (tone on `u`, not `ơ`; not packet garbling) | NOT RUN | NOT RUN |
| B6 Caps Lock on `vieejt` → `VIỆT` | PASS | NOT RUN | NOT RUN |
| Shift stuck after each B row (GetAsyncKeyState LSHIFT/RSHIFT/SHIFT) | PASS, all up after B1, B2, B4, B5, B6 | NOT RUN | NOT RUN |
| C click ends word (`xyz⏎vie`, click line 1, `et`) → `xyzet` / `vie` | PASS (Notepad) | — | — |
| D F2, F1, `baatj` → `bật` | — | NOT RUN (aborted) | — |
| E `vieejt` + Ctrl+Shift+Z → `vieejt` | PASS (Notepad) | — | — |

B5 detail: the typed keys were `duong` then Shift+J. The expected `dượng` was not produced; Notepad got `dụong`. The text is clean (no packet garbling), but the tone landed on `u` instead of `ơ`. Recorded for the engine owner.

Abort: `foreground changed mid-batch rows-VS Code` at 11:12:05. No physical key or mouse event was recorded (the abort reason is the foreground check, not the watchers). Probable cause, unverified: VS Code's own window activation during its startup, after our Force. The VS Code section did not produce a result.

H (VietKB): H1 PASS, settings window opened with Ctrl+Alt+S (Telex, Alt+Shift, start with Windows and run hidden on). H2 PASS by screenshot: the window is fully on DISPLAY1 (screen x ≈3282–3758). The harness rect readout again disagreed, so the readout is a harness bug. H3 tray captured (`shots2\H-tray-primary-run2.png`), icon not auto-checked.

Notepad screenshot: `shots2\Notepad-rows.png`. Settings screenshot: `shots2\H-settings-display1-run2.png`.

### Items
| Item | Status |
|---|---|
| I Installer hash and version | PASS (hash matches, FileVersion 0.2.4, not run) |
| J ignored tests | SKIPPED (need dedicated desktop) |
| F other hooks | none running |
| G toggle sound | SKIP (owner check) |

### Teardown (lines in log.md)
- KILL Code pids 47668, 2932, 3780, 45248; notepad 51592; vietkb 19236 (owned rebuilt target build).
- Candidate processes left: 0. Config restored identical. Run key byte-identical. Caps Lock off. Cursor and clipboard restored.

### Harness notes for the next run
1. The VS Code abort is a foreground-check abort, not a physical-input abort. Either wait for VS Code to finish starting before the first Force, or allow one re-focus within the batch.
2. The settings-window rect readout does not match the screen; the harness should read the rect of the Tauri top-level window, not a child.
3. Chrome and D (VS Code F1/F2) still need a run.

---

## Build 0D39D42F: run v3, VS Code and Chrome rows (11:16–11:17 UTC, owner-approved)

**Status: COMPLETE for the requested rows. VS Code output is wrong in the first rows (a product finding, not a harness artefact: the screenshot shows the same text).** Teardown clean.

### Isolation and focus
- VS Code: separate instance, `--new-window --disable-extensions --user-data-dir <scratch>\vscode-ud --extensions-dir <scratch>\vscode-ext`, pre-created `User\settings.json` with the owner's seven settings. **Dropped and unsupported in this build (not in CLI help or app code):** `--disable-workspace-trust`, `--skip-welcome`, `--skip-release-notes`; covered by the settings file. No GitLens tab opened; the owner's 14 VS Code processes were unchanged before and after.
- Settle: VS Code title stable 3.1 s, foreground ours on attempt 1. Chrome title stable 3.1 s, foreground ours on attempt 1. **No FOCUS-TAKEN event in this run** (the earlier abort is consistent with the extension tabs).
- Chrome: fresh `--user-data-dir`, `--no-first-run --no-default-browser-check`.

### Results
| Row | VS Code | Chrome |
|---|---|---|
| A1 `dduwowcj` → `được` | FAIL: line absent | PASS |
| A2 `tieesng vieetj` → `tiếng việt` | FAIL: shows `ếng việt` | PASS |
| A3 `nghieeng` → `nghiêng` | PASS (content) | PASS |
| A4 sentence → `Tiếng Việt rất đẹp.` | PASS (content) | PASS |
| A5 `dduwowcj` ×10 | content 10/10 correct; harness index verdict FAIL (line shift) | PASS, 10/10, 0 failures |
| B1 `VIEETJ` → `VIỆT` | FAIL: merged into `ViệtViệt` | PASS |
| B2 `Vieejt` → `Việt` | FAIL: merged | PASS |
| B4 Shift held through `VIEETJ` → `VIỆT` | PASS (content) | PASS |
| B6 Caps on `vieejt` → `VIỆT` | PASS (content) | PASS |
| E `vieejt` + Ctrl+Shift+Z → `vieejt` | PASS (content) | PASS |
| Shift stuck after B rows | PASS: all up after B1, B2, B4, B6 | PASS: all up |
| D F2, F1, `baatj` → `bật`, palette must not open | PASS by screenshot: palette not open, `bật` typed in editor. Harness index verdict FAIL (line shift). | — |

"Content" means the expected text is present in the screenshot at a different line number. The harness reads by line index, and the VS Code merges shifted every later index, so the harness verdicts for VS Code are unreliable at row level. Screenshots: `shots3\VS Code-rows.png`, `shots3\vscode-D-after-F1.png`, `shots3\Chrome-rows.png`.

### Product finding (VS Code)
In VS Code, on this build, the first two rows are damaged: the first `được` line disappears, `tiếng việt` loses `ti`, and the B1/B2 lines merge. Chrome, with the same keys, has no such damage. Root cause not investigated; likely the whole-word rewrite path in VS Code (the D-E2E-06 area). Needs a dev look before the VS Code result is accepted.

### Teardown
- KILL: chrome pids 18900, 27836, 29292; Code pids 50096, 4944, 44356; candidate pid 53000.
- Candidate processes left: 0. Scratch profiles `vscode-ud`, `vscode-ext`, `chrome-ud` deleted (exists-after=False).
- Caps Lock off. Cursor restored. Config SHA `58EB422E…C704` identical. Run key byte-identical to original `…\target\debug\deps\vkb_windows-703be73b8891cfe5.exe --autostart`. Clipboard text restored. Foreground restored.

---

## Build 0D39D42F: run v4, VS Code (suggestions off) — ABORTED before rows (11:24–11:25 UTC)

**Status: no VS Code rows were run.** The run stopped at the focus sentinel. The sentinel did land in the editor, so this was a harness bug (my check), not a focus failure. The one approved rerun is used; the fixed script is ready but needs a go-ahead.

### What ran
- Suggestion settings added to the isolated `settings.json` (quickSuggestions off, wordBasedSuggestions off, suggestOnTriggerCharacters false, acceptSuggestionOnEnter off, parameterHints off, inlineSuggest off, auto-closing brackets/quotes never, chat command center off, secondary sidebar hidden, Copilot off, chat AI features disabled).
- VS Code window on DISPLAY1 at 2660,100,3428,660 (onDISPLAY1=True). Settle: title stable 3.1 s, foreground ours on attempt 1, no focus taken.
- Screenshot `shots4\vscode-v4-settled.png`: editor empty, Chat panel not open (the settings did keep it closed), status shows "Ln 1, Col 1", only the benign "All installed extensions are temporarily disabled" notice.
- Click into the editor and sentinel `x` typed. Readback `"\nx"`: the x was in the editor, on line 2. Cause: `Set-Content -Value ''` wrote a blank line, and the click below the content put the caret on line 2.

### Fix for the next run (already in host-e2e4.ps1)
1. The scratch file is written with `[IO.File]::WriteAllText($fileV,'')` (one empty line, no extra newline).
2. The sentinel check trims surrounding newlines before comparing with `x`.

### Teardown (clean)
- KILL lines in `log.md` for the v4 VS Code instance and the candidate pid 20124 (logged by the run).
- Candidate processes left: 0. Scratch profiles `vscode-ud` and `vscode-ext` deleted.
- Caps Lock off, cursor restored to 118,885, config SHA `58EB422E…C704` identical, Run key byte-identical to original, clipboard restored, foreground restored, owner VS Code processes 14 before and 14 after.

---

## Build 0D39D42F: run v5, VS Code (suggestions off, fixed harness) — ABORTED before rows (11:26–11:27 UTC)

**Status: no VS Code rows were run. B1/B2 merge with suggestions off is still UNANSWERED.**

### What ran
- Same isolated profile and suggestion-off settings as v4. Hashes matched, idle gate 61.6 s, VS Code window on DISPLAY1 at 2660,100,3428,660 (onDISPLAY1=True), settled 3.1 s on attempt 1, no focus taken.
- Click into the editor at 2852,296 (OK). Sentinel `x` sent (OK). The whole-document copy batch sent (OK), but the clipboard did not change within 4 s, so the readback recorded `<clipboard-no-change>`. Recorded as a harness check; per the coordinator's rule the rows were allowed to continue.
- Between 11:27:31 and 11:27:36 the mouse watcher recorded a non-injected mouse move (WM_MOUSEMOVE, no LLMHF_INJECTED). Per the hard rules this is physical input and the run aborted at the next batch (sentinel-delete, 11:27:36.15). The move was about 10 s after my own click, outside the 1.5 s grace window. Most likely the owner moving the pointer; not verified.

### Teardown (clean)
- KILL lines for the v5 VS Code instance and the candidate pid 40948 (in log.md). Candidate processes left: 0.
- Scratch profiles `vscode-ud`, `vscode-ext` deleted.
- Caps Lock off; cursor restored to 3375,810; config SHA `58EB422E…C704` identical; Run key byte-identical to original; foreground restored; owner VS Code processes 14 before and 14 after.

### Open
- The clipboard copy did not update in this run. Whether that is the harness or VS Code needs a look before any further VS Code run.
- B1/B2 merge with suggestions off has not been tested. Earlier evidence (run v3 screenshot, suggestions on) showed merges; v4 and v5 never reached the rows.

---

## Build 0D39D42F: run v6, VS Code (suggestions off, final) — ABORTED before the click (11:29 UTC)

**Status: no keys, click or rows were sent. B1/B2 merge with suggestions off is still UNANSWERED.**

- Hashes matched. Idle gate 65.6 s. VS Code window on DISPLAY1 (2660,100,3428,660), settled 3.1 s, foreground ours.
- During the 60 s idle wait before the click, the mouse watcher recorded a non-injected mouse move at 11:29:38 (msg 0x200). Per the hard rules this is physical input and the run stopped before any injection. The owner appears to be using the machine.
- Teardown clean: candidate processes left 0; scratch profiles deleted; Caps Lock off; cursor restored; config identical; Run key byte-identical; foreground restored; owner VS Code processes 14 before and 14 after. KILL lines for the v6 VS Code instance and candidate pid 49324 are in log.md.
- Note: a 60 s quiet period immediately before the click cannot be met while the owner is at the machine. Either the owner steps away for the run, or the run is scheduled for a quiet period.

---

## Build 0D39D42F: run v7, VS Code — NOT STARTED (EVKey64 running)

- Pre-flight check before launch: EVKey64 is running again (pid 42488, `D:\Software\EVKey\EVKey64.exe`), the owner's IME. Per the hard rules, QA does not stop it and does not start the run.
- Done before stopping: the isolated v7 `settings.json` written to `scratchpad\host-e2e7\vscode-ud\User\settings.json` (full list from the owner, 28 entries; `globalStorage` not pre-created). QN2 helper extended (non-injected physical-input timestamp, PhysIdle, top-level window enumeration for the sign-in/Welcome cleanup). Neither was run against the desktop.
- Nothing launched, nothing stopped, config and Run key untouched (no launch happened). No KILL lines for this attempt.
- B1/B2 merge with suggestions off: still UNANSWERED.
- To continue: the owner quits EVKey64, then gives a go-ahead. The v7 orchestrator is not written yet (it will need the settings path above and the cleanup loop).

---

## Build 0D39D42F: run v7, VS Code (suggestions off, isolated profile) — COMPLETE (11:33–11:35 UTC)

**Result: all rows PASS. B1 and B2 do NOT merge with suggestions off.**

- Pre-flight: EVKey64, Unikey and installed VietKB not running (owner had quit EVKey64). Parse check and dry-run of host-e2e7.ps1 passed before the run.
- Idle waits: 60.3 s before start, 74.6 s before the click (both within the 15 min limit). Mouse guard not triggered.
- Cleanup: no extra top-level windows appeared (no sign-in or Welcome window to close). Title stable, clean in about 4 s.
- Sentinel: x confirmed by pixel diff (17 px) within 1 s.
- Rows (verdict from one Ctrl+A, Ctrl+C in our window; screenshot agrees):
  - A1 dduwowcj → `được`: PASS (line 1)
  - A2 tieesng vieetj → `tiếng việt`: PASS (line 2)
  - B1 VIEETJ → `VIỆT`: PASS (line 3)
  - B2 Vieejt → `Việt`: PASS (line 4)
  - B4 Shift held through VIEETJ → `VIỆT`: PASS (line 5)
  - D F2, F1, baatj → `bật`: PASS (line 6); palette did not open (screenshot vscode-v7-D-after-F1.png)
  - E vieejt + Ctrl+Shift+Z → `vieejt`: PASS (line 7)
- Shift stuck after B1, B2, B4: none (all up, caps off).
- Teardown: candidate left 0; VS Code instance killed (KILL lines in log.md); scratch profiles deleted; Caps Lock off; config identical; Run key byte-identical; clipboard and focus restored; owner VS Code processes 14 before and 14 after.
- Screenshots: shots7\vscode-v7-final.png, shots7\vscode-v7-D-after-F1.png.
