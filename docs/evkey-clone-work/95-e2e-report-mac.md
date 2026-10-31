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

## Final regression — 2026-10-31

Build: `qa-build/VietKB.app` v0.2.6, sha256 `d9684596…226eb59` (not rebuilt). Harness `scripts/mac-e2e/run.sh` (all phases, extras on). Evidence: `qa/mac/run-final/` (`results.md`, `results.jsonl`, `guardrail.log`, `harness.log`, `shots/`). Run exit code 0, 13:36–13:51 (+07). Idle gate passed first try (HID idle 438 s); no retry needed.

Per-target results (249 cases, 249 PASS, 0 FAIL, 0 skipped):

| Target | Telex | VNI |
|---|---|---|
| textedit-plain | 66 PASS | 29 PASS |
| textedit-rich | 33 PASS | 29 PASS |
| notes | 8 PASS | 8 PASS |
| stickies | 10 PASS | 8 PASS |
| chrome-textarea | 8 PASS | 8 PASS |
| chrome-omnibox | 3 PASS | n/a (not in VNI set) |
| vscode | 8 PASS | 8 PASS |
| terminal | 8 PASS | 8 PASS |
| secure (password dialog) | 1 PASS | 1 PASS |
| gui (settings, status item, menu, toggle icon) | 5 PASS | n/a |

Per-app phase (`textedit-plain`, `stickies`; Stickies rule `enabled=false`) ran and exited 0. Its rows are not tagged separately in `results.jsonl`, so they are counted in the Telex rows above.

Stability (x10 repetitions, all reps pass): 40 of 40 stability rows pass, `repsPass == reps` on every row. Terminal uses 3 reps by harness design. Slow (x2) and burst (x10) variants included.

Guardrails:
- EVKey: running before the run (pid 26744). Quit by the harness at 13:36:58 with a `KILL` log line, fallback `pkill` at 13:37:09. Relaunched by the trap at 13:51:08, now pid 36203 (running).
- VietKB: launched directly with a temp HOME, one process per phase, each killed and logged. No VietKB process remains.
- Owner files: manifest identical before and after. The real `~/Library/Application Support/VietKB` was never created.
- Input source: ABC at start and end; no change.
- Physical-input watcher: no abort (exit 6 never occurred).
- Pre-existing TextEdit (pid 19829) and Terminal (pid 49785) were running before the run and were not started or killed by it. The harness reports TextEdit as still running afterward, as expected.
- Temp dir `/tmp/vkbe2e.*` removed.

Failures: none. No D-MAC entry added. No phase was skipped.

Lead note (2026-10-31): the Notes screenshots of this run showed the owner's note list and were deleted before commit. Harness follow-up: screenshot only the note body (or hide the sidebar/list) for the Notes target.


## 0.2.7 RC — 2026-10-31

Builds: dev-channel bundle `target/dev-app/VietKB Dev.app` (`vn.vietkb.app.dev`, 0.2.7), built with `scripts/mac-e2e/build-dev-app.sh` and re-signed with the local Apple Development identity (team KNB9B3B3L2). Typing, first app suite, rerun and static ran on **aa5e71f** (bin sha256 `81630aa8…88b1`); the app suite, static and the theme screenshots were repeated on **HEAD a337357** (M9 title bar, M10 parity, release notes). Evidence `qa/mac/run-0.2.7-rc/` (`static/`, `typing/`, `app/`, `app-rerun*/`, `app-head/`, `static-head/`, `typing-retry-*`, `shots/`, `shots-aa5e71f/`). Harness edits made for this run: `run.sh` picks the `VietKB Dev` config dir for a dev bundle; `appsuite.swift` has the M7 theme phase `theme` (APP-80..83), a polling update-badge check (needs `check_updates = true`, the fake feed answers locally) and screenshots of the window only.

| Area | Result |
|---|---|
| Static: site-check | aa5e71f 15/20 (SITE-08/06, 08/07, 08/10, 12, 13 FAIL); HEAD 18/20 (SITE-09 new, SITE-13 remains) |
| Static: strings-audit | 6/8 then 7/8 (STR-02 `common.add` unused remains; STR-06 hit was an HTML comment) |
| Typing, Telex + VNI, all targets (aa5e71f) | 247 PASS, 2 FAIL of 249: TX-E13 textedit-rich (D-MAC-42, 4/4 reproducible), VN-H1 textedit-plain (failed once, 3/3 PASS on retry) |
| Stability (x10, slow x2, burst x10 rows) | all repetition rows PASS (no stability row failed) |
| App: launch, Dock/policy, singleton check, windows | PASS (APP-01..04, 13..16) |
| App: z-order of every modal from main | macros, hotkeys, advanced, About-from-Advanced PASS; hotkeys and advanced failed once in the first run (extra window, D-MAC-47), PASS in 4 later runs |
| App: modals from the menu | NOT TESTABLE: the status-item menu cannot be opened by AX (APP-05b, D-MAC-34 reproduced); no coordinate clicks on the menu bar by rule |
| App: VI/EN | PASS (toggle in 5 windows, EN in 194 ms, no leaks, no clipping, 10 round trips, EN persists over restart) |
| App: theme (M7) | PASS: default `system` follows the OS (dark), one click stores explicit `light`, Reset all returns `system`. Live OS appearance change not tested (system setting) |
| App: update badge (M8) | settings-window button PASS on fake feed (`Update available — open About`); menu-bar badge not AX-readable: manual |
| App: quit paths | PASS (Quit button, Cmd+Q with a modal open, Dock-tile Quit, osascript quit, SIGTERM); SIGKILL skipped by design |
| Singleton (M8) | PASS: `open -n`, other HOME, second .app copy, direct exec -> 1 process, 1 status item, 0 taps, 1 Dock tile, settings window shown, second pid exits |
| Hidden start (APP-30, APP-60) | FAIL as written; cause is the dev bundle having no Accessibility/Input Monitoring grant, so D-MAC-15 shows the window (D-MAC-45, harness expectation) |
| hand-edited language (APP-25) | FAIL, D-MAC-36 reproduced |
| Aria labels (APP-11) | FAIL, D-MAC-35 reproduced |

Screenshots of the theme toggle (window only): `qa/mac/run-0.2.7-rc/shots/theme-toggle-1-system-default.png`, `-2-explicit-light.png`, `-3-en-light.png`, `-4-after-reset.png`, `-5-update-button.png` (HEAD layout); the aa5e71f layout is in `shots-aa5e71f/`.

Defects: verified (D-MAC-02, 04, 05, 15, 16, 17, 22, 25, 26, 28, 30 (window half), 31 (sequential only), 37, 40); still open and reproduced (34, 35, 36, 38 partial, 39); new D-MAC-42..47 (42 TX-E13 rich P1 undecided, 46 guardrail gap with the owner's live instance P1).

Guardrails:
- Owner's VietKB pid 36608 (prod bundle) was never signalled, quit or launched into; it was running through the whole run.
- Idle gate: HID idle was 1409 s at the typing start and above 200 s before each app run; no retry needed; no physical-input abort (exit 6 never occurred).
- Typing phases ran the dev bundle exec'd with a temp HOME (own config under `VietKB Dev`). The owner's `~/Library/Application Support/VietKB/config.toml` changed hash during the perapp phase (D-MAC-46); `VietKB Dev` in the real HOME was never created; the app-suite manifests were identical before and after.
- One dev instance at a time; every instance quit by AX Quit or osascript, logged in each `guardrail.log`. Dock: a stale tile of the dead dev bundle appeared after the quit phase in both full runs and after the update phase once (D-MAC-44); the final state of every run was checked clean (0 tiles, 0 dev processes), the last check after the HEAD run.
- EVKey was not running before or after: nothing quit, nothing relaunched. Input source ABC unchanged. TextEdit pid 19829 and Terminal pid 49785 pre-existed and were not touched (the harness's own documents were closed). Screenshots only of VietKB windows, harness-owned test windows and Dock tile crops; manifests stay gitignored.


## 0.2.7 RC2 typing re-run (owner VietKB quit) — 2026-10-31

Product code a93bc52 (main b214377). Dev-channel bundle rebuilt with `build-dev-app.sh` and re-signed with the Apple Development identity (team KNB9B3B3L2), bin sha256 `ee08a8ad…d1ad`. Evidence `qa/mac/run-0.2.7-rc2-typing/` (`owner-instance.log`, `typing/`, `typing-noextras/`, `toggle-diag-1..2/`, `typing-e13-1..5/`, `typing-vnh1-1..5/`). Harness edit: `vkbe2e.swift` takes no screenshot for the Notes target (no list screenshots); manifests gitignored.

| Run | Result |
|---|---|
| Full run with extras (telex, vni, perapp) | INVALID after GUI-5: 145 PASS, 100 FAIL, 1 ABORT; GUI-5 failed and left the engine OFF for the rest of the telex phase (cascade; VNI and perapp phases fine). New D-MAC-48, not reproduced |
| Full run, no extras, all targets, telex + vni + perapp, stability rows (x10, slow, burst) | **242 PASS, 0 FAIL** |
| gui-toggle (GUI-5) alone, 2 runs | 2/2 PASS (3 rows each incl. gui) |
| TX-E13 textedit-rich `tieengs1` x5 (+ 1 duplicate row, +1 in the full run) | **6/6 PASS** (`tiếng1`) |
| VN-H1 textedit-plain x5 (+ 1 in the full run) | **5/5 PASS** |

Defects: D-MAC-42 CLOSED (it was the owner's live instance), D-MAC-46 mitigated by M11, D-MAC-48 new (watch). typing-e13-2 holds two rows because the orphaned run of the interrupted driver and its replacement wrote to the same dir.

Owner instance log (all in `owner-instance.log`):
- Idle gate: HID idle 3611 s on the first try.
- `QUIT 17:02:49 pid=36608 path=.../target/universal-apple-darwin/release/bundle/macos/VietKB.app/Contents/MacOS/vietkb`; osascript quit returned "User canceled (-128)" (the app declined), alive after 10 s, `kill -TERM` at 17:02:59, gone at 17:03:04. No SIGKILL.
- `RELAUNCH 17:48:31 pid=38269` from the same path via `open`, after the binary sha256 was re-verified unchanged (`1b023bba…49ba2`). Downtime 45 min (longer than planned: I replaced the driver mid-run to add the no-extras and toggle reruns; the first driver was SIGKILLed, not the app).
- Owner `config.toml` sha256 `faad8a0f…9513` before, after quit, and after relaunch: unchanged.
- EVKey was not running before or after; input source ABC unchanged; no leftover dev process; one dev instance at a time.
