# 95 — E2E report, macOS port (QA, 2026-10-31)

Build under test: `/Users/nam/Workspace/vietkb/qa-build/VietKB.app`, version 0.2.6, executable sha256
`9a056ae955dc7c3822c0d88ac7bd403a8ec9188643060b2d9e5883daf1f4f759` (16 066 080 B), signed `Identifier=vn.vietkb.app`,
`TeamIdentifier=KNB9B3B3L2`. Launched directly from the shell (`exec`, not `open`).

## Result

| Block | Result |
|---|---|
| Telex, all targets (`run-full-1`) | TextEdit plain 33/33, TextEdit rich 33/33, Notes 8/8, Stickies 8/8, Chrome textarea 8/8, Chrome address bar 3/3, VS Code 8/8, Terminal 8/8 |
| VNI, all targets (`run-full-2`) | TextEdit plain 24/25, rich 24/25, Notes 7/8, Stickies 7/8, Chrome textarea 7/8, VS Code 7/8, Terminal 7/8. The single FAIL everywhere is VN-C1 (**D-MAC-01**) |
| VNI localisation probe (`run-probe-vni-caps`) | TextEdit plain: VN-C1, X1, X2, X3 FAIL, X4 PASS |
| Secure field (hidden-answer dialog), Telex and VNI | 2/2 PASS: typed `dduwowcj Vieejt` / `d9u7o7c5 Vie65t` came back raw |
| Per-app rule (`com.apple.stickies` `enabled=false`), Telex | 2/2 PASS (Stickies raw; TextEdit Vietnamese 33/33 in the same phase) |
| GUI smoke | 5/5 PASS checks, plus **D-MAC-02** (settings footer clipped) |

Stability (10 repetitions in one continuous typing run, per-repetition count): Telex `Tieesng Vieetj raast ddepj.` x10 and `dduwowcj` x10 at 8-15 ms/key,
and `dduwowcj` x10 at 2-4 ms/key (burst): **10/10 in every target** (Terminal 3/3, shorter). Slow typing (150-250 ms/key, sentence x2) PASS.
VNI sentence `Tie6ng1 Vie6t5 ra6t1 d9e5p.` x10 and `d9u7o7c5` x10 (fast and burst): 10/10 everywhere. The D-E2E-01 collapse (`dduwowcj` in Windows Notepad) **does not reproduce on macOS**.

## Environment

macOS 27.0.1 (26A434) arm64; input source ABC (U.S.) at start and end (no switch needed); Accessibility, Input Monitoring (listen),
Post-Event and Screen Recording granted to the shell (`vkbe2e preflight`). Apps: TextEdit, Notes (iCloud account), Stickies, Terminal (owner's, already running;
the harness opened a **new** window), Google Chrome (isolated `--user-data-dir`, `--force-renderer-accessibility`, data: URL page with a textarea and an input),
Visual Studio Code (isolated user-data-dir and extensions-dir, `--disable-extensions`, quick suggestions / suggest / inline off, `accessibilitySupport: on`, new untitled file).
Config under test: temp `HOME`, `config.toml` with `method` telex|vni, `start_with_windows=false`, `start_minimized=false`, `check_updates=false`, rest default
(spell_check on, restore_wrong_word off, modern_tone on). Harness: `scripts/mac-e2e/` (README there).

## How results were read

TextEdit: AppleScript `text of document` of the window id. Notes: AppleScript `plaintext of note id`. Stickies, Chrome, VS Code: AX `AXValue` of the focused element.
Terminal: AX text area of the new window (the `cat > /dev/null` echo; Enter never pressed). Hidden-answer dialog: its returned text. Comparison is NFC-normalised.
Every case clears the target (Cmd+A, Delete / Ctrl+U), types, waits until two reads agree, and checks focus before, during (every ~12 keys) and after.

## Per-case table (PASS/FAIL per target)

`-` = not in that target's smoke subset. Full rows with exact keys, actual text, timing and screenshots: `qa/mac/run-full-1/results.md` (Telex),
`qa/mac/run-full-2/results.md` (VNI, per-app), `qa/mac/run-probe-vni-caps/results.md`. Columns: TextEdit plain, TextEdit rich, Notes, Stickies, Chrome textarea, Chrome address bar, VS Code, Terminal.
Telex rows come from `run-full-1`, VNI rows from `run-full-2`. The S1 rows of `run-full-1` were first scored FAIL by a harness counting bug (actual text equals expected, checked), re-judged by `vkbe2e report`.
The VN-E12 expectation was corrected after the run (one Backspace removes the last letter; `vkb_core::simulate_keys` gives `nghiền`).
Screenshots of Notes windows were removed from the repo (they show the owner's note list); the verdict rows stay.

| case | input | expected | textedit-plain | textedit-rich | notes | stickies | chrome-textarea | chrome-omnibox | vscode | terminal |
|---|---|---|---|---|---|---|---|---|---|---|
| TX-B1 | `Vieejt Nam` | `Việt Nam` | PASS | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| TX-B2 | `dduwowcj` | `được` | PASS | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| TX-B3 | `nguowif` | `người` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| TX-B4 | `hoaf` | `hòa` | PASS | PASS | - | - | - | - | - | - |
| TX-B5 | `quaan` | `quân` | PASS | PASS | - | - | - | - | - | - |
| TX-B6 | `tieengs` | `tiếng` | PASS | PASS | - | - | - | - | - | - |
| TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Viee... (x10)` | `Tiếng Việt rất đẹp. Tiếng Việt` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| TX-S2 | `dduwowcj dduwowcj dduwowcj dduwowcj dduw... (x10)` | `được được được được được được ` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| TX-S3 | `Tieesng Vieetj raast ddepj. Tieesng Viee... (x2)` | `Tiếng Việt rất đẹp. Tiếng Việt` | PASS | PASS | - | - | - | - | - | - |
| TX-S4 | `dduwowcj dduwowcj dduwowcj dduwowcj dduw... (x10)` | `được được được được được được ` | PASS | PASS | - | - | - | - | - | - |
| TX-E1 | `ddeei` | `ddeei` | PASS | PASS | - | - | - | - | - | - |
| TX-E2 | `ddeei{BS}m` | `đêm` | PASS | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| TX-E3a | `ddeei{BS}{BS}{BS}ddeem` | `đêm` | PASS | PASS | - | - | - | - | - | - |
| TX-E3b | `ddeei{BS}{BS}{BS}ddeens` | `đến` | PASS | PASS | - | - | - | - | - | - |
| TX-E4 | `ddeei{BS}{BS}eem` | `đêm` | PASS | PASS | - | - | - | - | - | - |
| TX-E5 | `pass{BS}{BS}{BS}dduwowcj` | `được` | PASS | PASS | - | - | - | - | - | - |
| TX-E6 | `dduwowcjk{BS}` | `được` | PASS | PASS | - | - | - | - | - | - |
| TX-E7 | `gifk{BS}` | `gì` | PASS | PASS | - | - | - | - | - | - |
| TX-E8 | `test{BS}` | `té` | PASS | PASS | - | - | - | - | - | - |
| TX-E9a | `1dduwowcj` | `1được` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| TX-E9b | `3giowf` | `3giờ` | PASS | PASS | - | - | - | - | - | - |
| TX-E10 | `Tha {BS}s` | `Thá` | PASS | PASS | - | - | - | - | - | - |
| TX-E11 | `ddeei {BS}{BS}m` | `đêm` | PASS | PASS | - | - | - | - | - | - |
| TX-E12 | `tieengs{BS}{BS}{BS}` | `ti` | PASS | PASS | - | - | - | - | - | - |
| TX-E13 | `tieengs1` | `tiếng1` | PASS | PASS | - | - | - | - | - | - |
| TX-E14 | `tieeng{BS}{BS}{BS}eeng` | `tiêng` | PASS | PASS | - | - | - | - | - | - |
| TX-C1 | `VIEETJ` | `VIỆT` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| TX-C2 | `{SHIFT}vieetj{/SHIFT}` | `VIỆT` | PASS | PASS | - | - | - | - | - | - |
| TX-C3 | `DDUWOWCJ` | `ĐƯỢC` | PASS | PASS | - | - | - | - | - | - |
| TX-C4 | `{CAPS}vieejt{CAPS}` | `VIỆT` | PASS | PASS | - | - | - | - | - | - |
| TX-H1 | `{CTRL+SHIFT}tieengs {CTRL+SHIFT}tieengs` | `tieengs tiếng` | PASS | PASS | - | - | - | - | - | - |
| TX-H2 | `vieejt{CTRL+SHIFT+Z}` | `vieejt` | PASS | PASS | - | - | - | - | - | - |
| TX-H3 | `vie{LEFT}{RIGHT}et` | `vieet` | PASS | PASS | - | - | - | - | - | - |
| VN-B1 | `Vie65t Nam` | `Việt Nam` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| VN-B2 | `d9u7o7c5` | `được` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| VN-B3 | `ngu7o7i2` | `người` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| VN-B4 | `hoa2` | `hòa` | PASS | PASS | - | - | - | - | - | - |
| VN-B5 | `qua6n` | `quân` | PASS | PASS | - | - | - | - | - | - |
| VN-B6 | `tie6ng1` | `tiếng` | PASS | PASS | - | - | - | - | - | - |
| VN-S1 | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6... (x10)` | `Tiếng Việt rất đẹp. Tiếng Việt` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| VN-S2 | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7... (x10)` | `được được được được được được ` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| VN-S3 | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6... (x2)` | `Tiếng Việt rất đẹp. Tiếng Việt` | PASS | PASS | - | - | - | - | - | - |
| VN-S4 | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7... (x10)` | `được được được được được được ` | PASS | PASS | - | - | - | - | - | - |
| VN-E1 | `d9e6i` | `d9e6i` | PASS | PASS | - | - | - | - | - | - |
| VN-E2 | `d9e6i{BS}m` | `đêm` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| VN-E3a | `d9e6i{BS}{BS}{BS}d9e6m` | `đêm` | PASS | PASS | - | - | - | - | - | - |
| VN-E3b | `d9e6i{BS}{BS}{BS}d9e6n1` | `đến` | PASS | PASS | - | - | - | - | - | - |
| VN-E4 | `d9e6i{BS}{BS}e6m` | `đêm` | PASS | PASS | - | - | - | - | - | - |
| VN-E8 | `d9e6i{BS}` | `đê` | PASS | PASS | - | - | - | - | - | - |
| VN-E9 | `2d9e6m` | `2đêm` | PASS | PASS | PASS | PASS | PASS | - | PASS | PASS |
| VN-E10 | `Tha {BS}1` | `Thá` | PASS | PASS | - | - | - | - | - | - |
| VN-E11 | `d9e6i {BS}{BS}m` | `đêm` | PASS | PASS | - | - | - | - | - | - |
| VN-E12 | `nghie6ng2{BS}` | `nghiền` | PASS | PASS | - | - | - | - | - | - |
| VN-C1 | `VIE6T5` | `VIỆT` | **FAIL** | **FAIL** | **FAIL** | **FAIL** | **FAIL** | - | **FAIL** | **FAIL** |
| VN-C4 | `{CAPS}vie6t5{CAPS}` | `VIỆT` | PASS | PASS | - | - | - | - | - | - |
| VN-H1 | `{CTRL+SHIFT}tie6ng1 {CTRL+SHIFT}tie6ng1` | `tie6ng1 tiếng` | PASS | PASS | - | - | - | - | - | - |
| VN-H2 | `vie6t5{CTRL+SHIFT+Z}` | `vie6t5` | PASS | PASS | - | - | - | - | - | - |
| VN-H3 | `vie{LEFT}{RIGHT}6t5` | `vie6t5` | PASS | PASS | - | - | - | - | - | - |
| VN-X1 | `VIE6T` | `VIÊT` | **FAIL** | - | - | - | - | - | - | - |
| VN-X2 | `VIE6` | `VIÊ` | **FAIL** | - | - | - | - | - | - | - |
| VN-X3 | `VIE5` | `VIẸ` | **FAIL** | - | - | - | - | - | - | - |
| VN-X4 | `Vie6T` | `ViêT` | PASS | - | - | - | - | - | - | - |


Caps Lock rows (TX-C4, VN-C4) drive a synthetic Caps Lock **flag** on the key events, not the real toggle (cannot be driven from software); the app handled the flag correctly.
Toggle (TX-H1/VN-H1), restore word `Ctrl+Shift+Z` (TX-H2/VN-H2) and arrow-key caret reset (TX-H3/VN-H3) are TextEdit-only rows and passed in both TextEdit variants.

## GUI smoke (`qa/mac/run-full-1/shots/`)

| Item | Result | Evidence |
|---|---|---|
| Settings window opens at launch (`start_minimized=false`), title "Cài đặt VietKB", native traffic lights | PASS | `gui-settings-window.png` |
| Accessibility banner not shown (permission granted); AX text of the window has no Accessibility/Trợ năng text | PASS | same, `gui-settings-ax.txt` |
| Menu-bar item present | PASS | `gui-statusitem-on.png` (not a CGWindow on macOS 27; found through `AXExtrasMenuBar`) |
| Menu opens on click: Bật/tắt tiếng Việt (checked), Kiểu gõ >, Mở cài đặt…, Phím tắt…, Giới thiệu, Thoát VietKB | PASS | `gui-statusitem-menu.png` |
| `Ctrl+Shift` toggles the icon V -> E -> V; text typed while E stays raw (`tieengs`) | PASS | `gui-statusitem-V-before-toggle.png`, `gui-statusitem-E-after-toggle.png` (the last V image was byte-identical to the first and was dropped) |
| Settings footer row visible | **FAIL (D-MAC-02)** | `gui-settings-window.png` |
| Toggle sound | not verifiable headless; the toggle ran twice per phase with no crash (owner to listen) | — |

## Defects (appended to `60-defects.md`)

- **D-MAC-01** (P2) VNI all-caps word + digit stays raw in every target (the engine itself is correct).
- **D-MAC-02** (P2) settings window footer clipped.
- D-MAC-03 (INFO) AppleScript quit of EVKey is ignored; `pkill -x EVKey` needed.

## Guardrail log

Calibration runs (11:57-12:46 local) used the same driver with `--only` subsets and quit/relaunched EVKey each time:
pid 78420 killed 11:57:55 (relaunched as 95569), then 95569 -> 96076 -> 96657 -> 97839 -> 99893 (12:14:06). pid 99893 was **found gone at 12:23** without a kill by
this QA (cause unknown, no crash report); relaunched manually with `open -a EVKey` at 12:25:52 (pid 2698). Final runs (`run-full-2` guardrail log excerpt; `run-full-1` and the probe have the same pattern):

```
12:36:11 PRE running before the run: TextEdit pid 3478
12:36:11 PRE running before the run: Terminal pid 49785
12:36:11 PRE running before the run: EVKey pid 6383
12:36:20 OWNER FILES manifest saved (manifest-before.txt)
12:36:35 HID idle 60s at gate
12:36:35 KILL 12:36:35 pid=6383 path=/private/var/folders/pf/n265g3x10s9bs25hm164t8sr0000gn/T/AppTranslocation/4B349CB4-9458-480A-8D67-73900037A6C0/d/EVKey.app/Contents/MacOS/EVKey reason=EVKey is a V
12:36:45 KILL fallback pkill -x EVKey
12:36:46 PHASE vni method=vni
12:36:46 LAUNCH 12:36:46 pid=6668 path=/Users/nam/Workspace/vietkb/qa-build/VietKB.app/Contents/MacOS/vietkb HOME=/tmp/vkbe2e.X6yz6y/home-vni
12:42:05 KILL 12:42:05 pid=6668 path=/Users/nam/Workspace/vietkb/qa-build/VietKB.app/Contents/MacOS/vietkb reason=phase vni finished (launched by this script)
12:42:06 PHASE perapp method=telex
12:42:06 LAUNCH 12:42:06 pid=8985 path=/Users/nam/Workspace/vietkb/qa-build/VietKB.app/Contents/MacOS/vietkb HOME=/tmp/vkbe2e.X6yz6y/home-perapp
12:44:12 KILL 12:44:12 pid=8985 path=/Users/nam/Workspace/vietkb/qa-build/VietKB.app/Contents/MacOS/vietkb reason=phase perapp finished (launched by this script)
12:44:14 OWNER FILES manifest identical before/after (sha256 e551db5452d5ec35...)
12:44:14 real VietKB config dir absent (never created): sha before=after=ABSENT
12:44:17 RELAUNCH 12:44:17 EVKey now pid=9418 (was pid=6383, path=/private/var/folders/pf/n265g3x10s9bs25hm164t8sr0000gn/T/AppTranslocation/4B349CB4-9458-480A-8D67-73900037A6C0/d/EVKey.app/Contents/Ma
```

`run-full-1`: EVKey pid 2698 killed 12:29:23, relaunched 12:36:00 (pid 6383). `run-full-2`: pid 6383 killed, relaunched pid 9418 at 12:44:17. Probe run: 9418 killed 12:45:56, relaunched 12:46:40 (pid 10299). EVKey is running at the end of this work (pid 10299).

- Idle gate: >= 60 s before the first synthetic event of each run (the driver waits up to 5 min, else aborts untouched). Then the physical-input watcher guards: `seen=1866 ours=1866 vk=0 otherSoftware={} flag=none` for the VNI + per-app run; no physical event in any run. HID idle cannot be re-checked between batches (our own session-tap events reset `HIDIdleTime`), so the watcher is the guard. Unexplained short HID-idle resets with no CG event were seen while nothing of ours was running; they only delayed the gate.
- Focus guard never tripped in the final runs. In calibration it correctly refused to type when the password dialog was not frontmost, and when Chrome's omnibox popup took the top spot (relaxed for that target only).
- Owner files: the manifest (sha256 of `~/Library/Application Support/VietKB`, `VietKB Dev`, `~/Library/LaunchAgents`, EVKey's support dir, selected input sources) was identical before/after every run. The real VietKB config dir was **never created** (VietKB ran with `HOME=/tmp/...`; test config sha256 `6a1434eb...dc52` telex, `3492d027...2fd` VNI, `69e451bf...d150` per-app). The manifest files themselves were deleted from the repo because they list the owner's LaunchAgents.
- Input source untouched (ABC).
- Notes: one test note per run created and deleted; it sits in *Recently Deleted* (not permanently deleted, by rule); the account is iCloud, so the note synced briefly.
- Stickies: the owner's two saved stickies were untouched. One calibration run left an empty test note (a harness focus-query bug, since fixed); it was closed in the app afterwards (`stickies-cleanup`, matched by its document id) and Stickies deleted its file. Launching Stickies rewrote its `.SavedStickiesState`.
- TextEdit, Notes, Stickies were not running before; all quit after the runs. Chrome and VS Code pids killed by the harness (logged). Terminal: new window closed with Ctrl-C + its close button; the `.command` file removed. `/tmp/vkbe2e.*` removed.
- Clipboard, System Settings and the owner's windows: not touched.

## Not run / caveats

- Real Caps Lock toggle, hardware key timing, layouts other than ABC, the macOS Vietnamese system IM (off by design).
- Safari, other Electron apps, Terminal "Secure Keyboard Entry", Gatekeeper and updater paths.
- The VNI and per-app phases of `run-full-1` aborted at the harness idle gate (own Telex events reset HID idle; fixed with `SKIP_IDLE_GATE`) and were rerun as `run-full-2`; no results from the aborted phases.
- Per-app rule tested only for `enabled=false`; `method` rules and `remember_per_app` not exercised. `app_exclude` is unbound on macOS (decision M2).
- Terminal uses 3 repetitions, Chrome address bar only B1/B2/E2 (fresh profile, no inline autocomplete seen).
