# VietKB macOS real-app E2E harness

Drives the real VietKB `.app` binary against real macOS apps with synthetic CGEvents and reads the result back
(AppleScript / Accessibility). Written for the QA run recorded in `docs/evkey-clone-work/95-e2e-report-mac.md`.

| File | Role |
|---|---|
| `run.sh` | Driver. Owns the process, config and input-source guardrails; builds and runs the harness per phase. |
| `vkbe2e.swift` | Harness: key synthesis, physical-input watcher, focus guard, per-app target openers, case tables, report. |
| `appsuite.swift` | App-level suite (CAP-1/2/3/4/14): LaunchServices launch, Dock/activation policy, window z-order and visibility, AX-driven GUI, quit paths. Compiled together with `vkbe2e.swift` (run.sh copies that file to `main.swift`). |
| `build-dev-app.sh` | Builds the dev-channel bundle (`vn.vietkb.app.dev`, config dir `VietKB Dev`) into `target/dev-app/` for the app suite. |
| `site-check.py` | Static site / README / release-notes checker (CAP-19, grep half of CAP-18). Pure file reads. |
| `strings-audit.py` | App string audit: locales vs source, hardcoded-string heuristic, Rust i18n table (CAP-20). Pure file reads. |

## Run

```sh
scripts/mac-e2e/run.sh                      # telex + vni + per-app phases, all targets
scripts/mac-e2e/run.sh --dry                # preflight only: TCC grants, idle, input source, manifest
scripts/mac-e2e/run.sh --phases telex --targets-telex textedit-plain --only TX-B1,TX-B2 --no-extras
```

Options: `--app <VietKB.app>` (default `/Users/nam/Workspace/vietkb/qa-build/VietKB.app`), `--out <dir>`,
`--phases telex,vni,perapp`, `--targets-telex a,b`, `--targets-vni a,b`, `--only <case ids>`, `--no-extras`
(skip the GUI smoke, secure-field and toggle-icon checks).

Targets: `textedit-plain`, `textedit-rich`, `notes`, `stickies`, `chrome-textarea`, `chrome-omnibox`, `vscode`,
`terminal`; extras `gui`, `gui-toggle`, `secure` (hidden-answer dialog).

Output in `--out`: `guardrail.log` (every KILL / LAUNCH / RELAUNCH / config sha), `harness.log`, `results.jsonl`,
`results.md` (tables), `shots/*.png`, `manifest-before/after.txt`, per-phase VietKB logs and config copies.

Requirements: the shell that runs it must hold Accessibility, Input Monitoring (listen), Post-Event and Screen
Recording grants (spawned children inherit them); `swiftc`; Chrome and VS Code installed for those targets.
The harness refuses to start without the grants.

## Guardrails built in

* **Owner idle**: HID idle >= 60 s before anything is touched (`run.sh` waits up to 5 min, then aborts untouched).
* **EVKey**: quit before the run (`osascript quit`, fallback `pkill -x EVKey`), logged as `KILL ... pid path reason`;
  relaunched in a `trap` handler on every exit path (`open -a EVKey`, fallback the `.app` path), logged as `RELAUNCH`.
* **Input source**: switched to ABC if needed, restored in the trap.
* **Config isolation**: VietKB is launched directly (`exec`, not `open`) with `HOME=<tmp>`, so
  `~/Library/Application Support/VietKB` (the owner's) is never read or written. A sha256 manifest of that dir,
  `VietKB Dev`, `~/Library/LaunchAgents`, EVKey's support dir and the selected input sources is taken before and
  after and compared. Only pids launched by the script/harness are killed.
* **Physical-input watcher**: a listen-only session `CGEventTap` flags any key/mouse/scroll event with no source pid
  and neither this harness's marker nor VietKB's in `kCGEventSourceUserData`; the first one aborts the whole run
  (exit 6) and no phase continues. (HID idle cannot be used between batches: synthetic events posted to the
  session tap still reset `HIDIdleTime`; the harness probes this and logs `idle detector NOT usable`.)
* **Focus guard (per key, guardrails section 7)**: before EVERY key (and before the key of a modifier chord, after its
  modifiers are down) and before/after every batch, the AX-focused application must be the target pid and the topmost
  normal window must be the exact window id this harness opened (CGWindowList). The first mismatch aborts the case:
  nothing more is sent and nothing is re-focused. Every `typeScript` call site passes a guard (no empty `check`).
  The script parser refuses `F1`..`F12` and any `CMD+` chord except `A`, `N`, `W`, `L` (keys that can launch or
  focus another app). Chrome's omnibox popup (same pid) is allowed for that target. The check costs a few ms per key,
  so burst (2-4 ms/key) runs are a bit slower than before. One exception: the single Escape that closes the status-item
  menu in GUI-4 is posted directly (only after the menu window was seen; it goes to the menu being tracked).
* **No clipboard**, no Return/Enter outside the hidden-answer dialog this harness owns, no System Settings.
* **Own windows only**: TextEdit documents / Notes notes are identified by id; Stickies and Terminal windows by
  CGWindowID; Chrome and VS Code are separate processes with throw-away `--user-data-dir`.
* **Teardown**: TextEdit docs closed `saving no`; the Notes note is deleted (it lands in *Recently Deleted*; the
  harness never empties that); our Stickies note is emptied and closed; Terminal is a new window running a `.command`
  of ours (`cat > /dev/null`), closed with Ctrl-C and the close button; Chrome/VS Code pids are killed.

## Case syntax

`{BS} {TAB} {LEFT} {RIGHT} {SP} {ESC} {WAIT:ms}`, `{CAPS}` (toggles a synthetic Caps Lock *flag*; the real toggle
cannot be driven), `{SHIFT}` ... `{/SHIFT}` (held), chords `{CTRL+SHIFT}` (modifier tap), `{CTRL+SHIFT+Z}`,
`{CMD+A}`. Upper-case letters press Shift per letter. Speeds: `fast` 8-15 ms/key, `slow` 150-250 ms, `burst` 2-4 ms.
Expected values come from `crates/vkb-core/tests/editing.rs` (`owner_*`), the corpus tests and docs 50/51/88/97.

## Debug subcommands (`vkbe2e <cmd>`, built by `run.sh` into a temp dir)

`preflight`, `inputsource [id]`, `watch <s>` (who generates input events), `front`, `winlist [pid]`,
`axdump <pid> [right]` (click a status item), `stickies-cleanup` (remove a leftover test note), `report`.

## App suite and static checks (CAP-1/2/3/4/14/19/20)

```sh
scripts/mac-e2e/build-dev-app.sh                       # once per build: target/dev-app/"VietKB Dev.app" (vn.vietkb.app.dev)
scripts/mac-e2e/run.sh --phases static                 # site + README + strings: file reads only, no gate, no process
scripts/mac-e2e/run.sh --phases app --idle-wait 1800   # app suite (waits up to 30 min for >= 60 s HID idle, else aborts untouched)
scripts/mac-e2e/run.sh --phases app --app-phases single,quit
```

App phases: `main` (launch, Dock, windows/modals, language, relaunch), `badlang`, `hidden` (start hidden, second launch, SIGTERM),
`single` (owner rule M8 singleton: `open -n`, other HOME, second .app copy, direct exec), `update` (badge; SKIP until the app exposes one),
`quit` (Cmd+Q via AX, Dock Quit via AX, osascript quit; SIGKILL only with `VKB_E2E_SIGKILL=1`), `tap` (opt-in: direct exec that inherits
this shell's grants, so the event tap exists; an exec-launched Regular app leaves a stale Dock tile, so it is not default).

Rules the suite enforces on itself: dev-channel bundle only (identifier must end `.dev`, else refused); never starts while any
instance of that bundle id runs; one dev instance alive at a time; no key, no click, no coordinate on the menu bar (all AX on our pid; the
Dock tile is touched only when its AXURL is our bundle); graceful quit first (AX Quit, then SIGTERM, SIGKILL last resort, logged);
after every phase it asserts no new Dock tile of the dev bundle and no running instance remains (`APP-DOCK-<phase>`); it aborts at the next
step if HID idle drops under 8 s; the owner's VietKB pids are logged before/after. Screenshots: VietKB windows by id and a tile-sized crop of the Dock.
Web content is read after a read-only AX hit test "primes" WebKit's tree; the status-item menu cannot be opened by AXPress (finding D-MAC-34).

### Cases the harness now automates (63 ids; A = full, P = partial)

| Capability | Check ids | Catalog cases |
|---|---|---|
| CAP-1 LaunchServices launch, temp HOME | APP-01, 60..6z | LCY-06 P, LCY-07 A |
| CAP-2 Dock / policy | APP-02, 06, 08, 13, 14, 16, 30, 41, DOCK-* | DCK-01 P, DCK-02 A, DCK-03 A, DCK-05 A, DCK-06 P, DCK-08 P, DOC-13 A |
| CAP-3 z-order / visibility / occlusion | APP-03, 06-*, 07, 08-*, 12, 15, 16, 21 | WIN-01 P, WIN-06 P, WIN-07 P, WIN-08 A, WIN-15 P, WIN-16 P, WIN-17 P, WIN-18 A, WIN-22 P, DOC-10 P |
| CAP-4 GUI by AX (modals, VI/EN, config read-back) | APP-04, 05, 05b, 09, 10, 11, 17..25 | LNG-01 P, LNG-02 P, LNG-03 A, LNG-04 A, LNG-05 A, LNG-06 P, LNG-08 P, LNG-15 P, DOC-05 P, DOC-06 A, DOC-07 A, DOC-08 P, DOC-16 P, I18-01 P, MBR-01 A, MBR-02 / A11-01 (AXPress part only) |
| CAP-14 quit paths | APP-23, 34, 40, 41, 42, 51 | LCY-01 A, LCY-02 A, LCY-03 A, LCY-05 P, LCY-08 P (modal, not file dialog), LCY-12 A |
| CAP-19 site | SITE-01..14 (`site-check.py`) | LNG-20 A, 21 A, 22 A, 23 A, 24 P, 25 P, 28 A, 29 P, 30 A; DOC-01..04 P, 05 P, 06 P, 07 P, 10 P, 12 A, 13 P, 15 P, 22 P, 23 A |
| CAP-20 strings | STR-01..08 (`strings-audit.py`) | LNG-06 P, LNG-07 P, I18-05 A (table; live menu read is blocked) |

Not automatable under guardrails section 6 (tray menu cannot be opened by AX): MBR-02..08, WIN-02/03, DCK-04, status-item halves of WIN-01/06;
WIN-05 (hotkey) needs the granted-grants `tap` mode plus synthetic keys and is not implemented. The theme toggle (M6 Mac) and the update badge
report SKIP until the UI has them. Smoke run: `docs/evkey-clone-work/qa/mac/run-appsuite-smoke/` (dev build of main 240c440).
