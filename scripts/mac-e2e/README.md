# VietKB macOS real-app E2E harness

Drives the real VietKB `.app` binary against real macOS apps with synthetic CGEvents and reads the result back
(AppleScript / Accessibility). Written for the QA run recorded in `docs/evkey-clone-work/95-e2e-report-mac.md`.

| File | Role |
|---|---|
| `run.sh` | Driver. Owns the process, config and input-source guardrails; builds and runs the harness per phase. |
| `vkbe2e.swift` | Harness: key synthesis, physical-input watcher, focus guard, per-app target openers, case tables, report. |

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
* **Focus guard**: before and after every batch and every ~12 keys inside one, the AX-focused application must be
  the target pid and the topmost normal window must be the exact window id this harness opened (CGWindowList).
  Otherwise nothing more is sent and the case aborts. Chrome's omnibox popup (same pid) is allowed for that target.
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
