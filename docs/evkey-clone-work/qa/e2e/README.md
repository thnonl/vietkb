# VietKB host E2E harness (`run-host-e2e.ps1`)

The whole host gate in one unattended pass: launch the release build, type the corpus into
Notepad / Chrome / VS Code, read the text back, judge it, tear everything down, and restore
`config.toml` byte-for-byte. Windows Sandbox cannot do this (`96-sandbox-feasibility.md`:
`KEYEVENTF_UNICODE` is dropped there), so this is the only place accented typing is graded.

## Run

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File E:\vietkb\docs\evkey-clone-work\qa\e2e\run-host-e2e.ps1
```

| flag | effect |
|---|---|
| *(none)* | full pass, includes the 60 s soak (~10-12 min) |
| `-Fast` | skips the soak (~7-8 min) |
| `-SelfCheck` | no app, no GUI: encoding, UIA/MSAA, readers, verdict arithmetic (incl. `SKIP` and the blocking labels), the chord parser and "no chord literal" scan, the kill-rule matrix and kill-logging order, the owner-quit wait (proved to contain no stop), the method-switch round-trip on a copy of the live config, the row budget, and **every corpus expectation re-pinned through `vkb simulate`**. Run this first; it is meant to make the next host window a one-shot. |
| `-OwnerQuit` | if the installed VietKB (or a leftover `target\` build) is alive, wait up to 120 s for the owner to close it, then abort `HOLD(owner-quit)`. Never kills it. |
| `-ExpectSha` / `-ExpectBytes` | identity of the validated build (defaults: the build this harness was last checked against). A mismatch is recorded in `# binary-id:` and noted, never a gate. |

Output: `results-host.txt` next to the script (`FOCUS` lines are interleaved; `NOTE` lines carry
observations that are not verdicts; `KILL`/`REFUSED` lines record every process stopped or
declined). Exit code 0 only on `SHIP`.

Constraints the script keeps: it **never kills a process it did not launch** (see "The owner's
app is untouchable"), it backs up `%APPDATA%\vietkb\config.toml` and restores it with a byte
compare, it quits the process under test through its own `Thoát hẳn` button by that pid, and it
closes every scratch window/Chrome profile it opened.

## Verdicts

Judged per the round's **Correctness rule** (`91-round-plan-sound-cases.md`, owner 2026-10-30):
"Không gõ được tiếng Việt sau / cũng được, nhưng không được gõ sai" — never emit wrong text. A row
is acceptable when the output is either the composed Vietnamese or the literal raw keystrokes;
it is a failure only when it is neither.

| verdict | meaning (rule) | blocks? |
|---|---|---|
| `PASS` | the pinned composed text appeared (rule 1) | no |
| `PASS(raw)` | the literal raw keystrokes appeared instead (rule 4: also acceptable) | no |
| `QUALITY` | a documented, owner-accepted loss (L1-L7 / the raw zone) and the app produced exactly its pinned text (rule 2: record, do not fail a release) | no |
| `READ-FAIL` | the harness could not read the field: U+FFFC object replacement, or empty after a retry at a longer settle, and then the second reader (clipboard) empty too. **Nothing is known about the product** — the row is unmeasured, and the note lists the readers tried and the bytes read. | no (prints a `CAVEAT` line) |
| `MANUAL` | a row this build cannot drive from a script; the note is a task for the owner with a time estimate | no |
| `SKIP(unbound)` | the live config binds no chord for this group (`hotkeys.<name>` empty or unparsable). Nothing was pressed and nothing is claimed. | no |
| `BLOCKED(redirected)` | the launch landed on the installed build (Tauri single-instance) instead of `target\release`. The run is void; nothing is graded and the owner's app is not touched. | **yes** (summary reads `BLOCKED(redirected)`) |
| `HOLD(owner-quit)` / `HOLD(stale-target)` | the owner's installed app (or a leftover `target\` build this run did not launch) is still in the way. Nothing is killed; the owner closes it and re-runs. | **yes** (summary reads the same label) |
| `FAIL` | anything else, including a harness row about the state of the run (single instance, method switch, quit path, process count, config restore) | **yes** |

**Verdict arithmetic:** `HOLD` when `FAIL > 0`, else `SHIP`; a blocked row is never `SHIP` and
carries its own label (`BLOCKED(redirected)`, `HOLD(owner-quit)`, `HOLD(stale-target)`).
`QUALITY`, `MANUAL`, `SKIP(unbound)` and `READ-FAIL` never block. The summary line reports the
buckets separately:

```
PASS n / PASS(raw) n / QUALITY n / READ-FAIL n / FAIL n / MANUAL n / SKIP n / BLOCKED n
```

A `READ-FAIL` still prints a `CAVEAT` line, because an unmeasured row is not evidence of
correctness either.

## The owner's app is untouchable

Authoritative: `99-agent-guardrails.md` and `00-charter.md` §6 (owner, 2026-10-30):

> **The owner's own running app is off-limits.** The installed build
> (`%LOCALAPPDATA%\VietKB\vietkb.exe`, the daily driver) must never be stopped, killed,
> restarted or replaced by an agent, and no agent may rely on Tauri single-instance
> redirecting into it. An agent may only kill a process **it launched itself** (a binary
> under `E:\vietkb\target\`) and must name the pid+path in its report. If the installed
> instance is alive and a test needs the hook, the run stops and asks the owner to quit it —
> killing it is a defect, not a convenience.

What the harness does with that:

- **PRECONDITIONS** detects the installed instance by path (`Get-Process vietkb` filtered to
  `%LOCALAPPDATA%\VietKB\`), prints pid / path / start time, and aborts with
  `HOLD(owner-quit)` — *"installed VietKB is running, owner must quit it — the script will not
  kill it"*. With `-OwnerQuit` it waits up to 120 s for the owner to close it, then aborts the
  same way. It never terminates it, and if the owner quits it during the wait the script does
  **not** relaunch it afterwards (the summary says so).
- A leftover `target\` build from an earlier run is treated the same way (`HOLD(stale-target)`):
  this run did not launch it, so it is not killed.
- **Only** a process this run launched may be stopped, and a `vietkb` stop must also be under
  `E:\vietkb\target\`. `Test-KillAllowed` is the single decision point; `Stop-Owned` logs
  `KILL <time> pid=<n> path=<p> reason=<why>` (or `REFUSED …`) to `results-host.txt` **and**
  `log.md` before the stop. Nothing in the script addresses `vietkb` by name.
- After launch the running process is checked by pid+path: a process that is not new and not
  under `target\` means Tauri single-instance handed the launch to the installed app. That is
  `BLOCKED(redirected)` — a void run, nothing graded, and the installed app is not touched.
- `TE1` quits the **test** pid only (its own footer button, 5 s); `TE2` counts processes under
  `target\` only, so the owner's app (if it is running) can never fail the teardown.

An aborting run prints its bucket line and exits 7 before touching any target window.



## Chords come from the config, never from this file

No chord value is written into the script. At run time it parses `hotkeys.toggle`,
`restore_word`, `open_settings`, `enable` and `disable` out of the live `config.toml` with the
product's own grammar (`crates/vkb-config/src/hotkey.rs`: case- and order-insensitive,
modifier-only chords allowed, a bare key only for F1..F24), and presses exactly those. The
release orders it exercises follow the binding: the completing key first, each modifier first,
then the whole chord.

This is not cosmetic. The first host run pressed a hardcoded `Ctrl+Shift` for the toggle while the
config binds `Alt+Shift`, so CH1 reported three "product" failures that were the harness's. With
the live config, `Alt+Shift` gives the orders `shift`, `alt`, `both`.

The results file opens with a `# CONFIG` block, one line per binding, so evidence can be checked
against the machine without trusting the script:

```
# CONFIG toggle        raw="Alt+Shift"  parsed=Alt+Shift  mods=alt+shift  key=(none, modifier-only)  ok
# CONFIG restore_word  raw="Ctrl+Shift+Z"  parsed=Ctrl+Shift+Z  mods=ctrl+shift  key=Z (0x5A)  ok
# CONFIG open_settings raw="Ctrl+Alt+S"  parsed=Ctrl+Alt+S  mods=ctrl+alt  key=S (0x53)  ok
# CONFIG enable        raw="F1"  parsed=F1  mods=(none)  key=F1 (0x70)  ok
# CONFIG disable       raw="F2"  parsed=F2  mods=(none)  key=F2 (0x71)  ok
# CONFIG toggle_mode   release
```

An empty binding is not a failure: `cycle_method` is unbound by default, and any group whose chord
is unbound reports `SKIP(unbound)` instead of guessing. `-SelfCheck` fails the run if a chord
literal ever reappears in a press path.

The binary under test is identified in the header too: `# binary:` (path, size, SHA-256) and
`# binary-id:` compared against `-ExpectSha` / `-ExpectBytes` (defaults: the build this harness
was last checked against, 8 390 144 B, `346299C0…13374`). A mismatch is recorded and noted, never
a gate: a rebuilt binary is the normal state of a round, and the report must simply name the
build the rows describe.

## What is `MANUAL`, and why

Only one row, and only when this build refuses to expose the surface:

- **`RT4-tray-menu`** — right-click the tray icon and read the popup menu. The harness tries it:
  right-click the icon rect, wait for the classic popup window (class `#32768`), read the item
  names through UIA (its MSAA proxy) and then a raw MSAA walk, `Esc` to dismiss. The tray menu is
  a Win32 popup owned by the menu crate, not part of the overflow flyout, so on some builds it
  may expose no item text to either reader. When that happens the row is `MANUAL` instead of
  `FAIL`, with this note: *OWNER, ~20 s: right-click the VietKB tray icon and confirm the menu
  reads `Bật/tắt tiếng Việt` (checked = ON), `Kiểu gõ` and `Thoát VietKB`.*
  The **left click** half of the tray route is fully automated and graded on the engine state
  (`RT4-tray-left-click`), so a broken tray click is still a `FAIL`.

Everything else is automated: typing (3 apps), chords (3 release orders each), `F1`/`F2`,
`Alt+Shift`, the UI switch (`TogglePattern` on `Bật tiếng Việt`, by accessible name, state read
back), the quit button, process count and the config byte compare.

Not covered by design (owner checks, listed so nobody mistakes silence for a pass): audibility of
the `go_moc` pair, and the settings window's visual layout.

## How a row is measured

1. **Focus discipline** — before typing, the target window is made foreground and verified
   (`GetForegroundWindow == target`); each re-take is logged as a `FOCUS` line. This is what
   catches a hotkey that also reaches the app (F1 opens Notepad Help / the VS Code palette) and
   was the cause of the empty `E2-on-f1` reads in the first host run.
2. **Clear and verify empty** — `Ctrl+A`, `Delete`, read back; retried three times. Every
   expectation is therefore **absolute** (the text of a freshly cleared field), never a delta.
   A field that will not clear makes the row `READ-FAIL`, not `FAIL`.
3. **Type** through the real VK path, actions (`BS`, `Enter`, `Tab`, `F1`, `F2`, `Ctrl+Shift+Z`)
   as separate `SendInput` calls.
4. **Read**, in this order: UIA `TextPattern` (ValuePattern built in) — for VS Code, `Ctrl+S` and
   the saved file; then the same reader again after a 700 ms settle; then the clipboard
   (`Ctrl+A`, `Ctrl+C`, selection collapsed with `End`). A text containing `U+FFFC` or an empty
   field aborts that reader. All three failing is `READ-FAIL`.
5. **Judge** against the pinned expectation, with one retry at 130 ms (a 55 ms row that fails is
   usually the hook's re-injection racing the read, not a text defect).

## Method (VNI vs Telex)

The corpus carries 19 VNI rows (the owner's method) and 15 Telex rows, and each row declares the
method it must run under — the first host run typed Telex keys while the engine was in VNI mode and
every `T*` row came back `PASS(raw)`, which measured nothing.

The script switches method through the only supported surface that needs no IPC: it quits the app,
rewrites `[input] method` in `config.toml` (one line, every other line byte-identical, read back),
relaunches, and proves the switch with a typed probe (`baajt` → `bật`, and `ba6t5` → `bật` for VNI)
plus a config read-back. That is the `H1-method-switch-*` harness row. The original method is
restored before the chord/toggle/soak blocks, and the final `TE3` row byte-compares the config
against the backup taken before anything ran. If the switch fails, its group is not run and its
rows are reported as `READ-FAIL` with "method switch failed" so the row count stays honest.

## Row budget (3 apps)

| block | rows |
|---|---|
| typing, 34 per app (19 VNI + 15 Telex) | 102 |
| chords: toggle, restore, open — every release order each bound chord has | 11 |
| toggles: RT1 enable/disable, RT2 partial+full toggle, RT3 UI switch, RT4 left-click, RT4 menu | 5 |
| teardown: TE1 quit, TE2 zero processes, TE3 config restored | 3 |
| harness: method switch to the other group, restore | 2 |
| soak (skipped with `-Fast`) | 0-1 |
| **total** | **123** (`-Fast`), 124 full |

The chord count is derived from the bindings (`-SelfCheck` prints `release orders : toggle=… restore=…
open=…`), so rebound chords change it instead of silently skipping orders. `-SelfCheck` prints the
budget and fails if the corpus or the helpers drift, so a miscount shows up before the owner's
desktop is busy.

## Files

| file | role |
|---|---|
| `run-host-e2e.ps1` | the gate (ASCII-only on purpose: every Vietnamese literal is built from codepoints, so a re-save without a BOM cannot mangle an expectation) |
| `results-host.txt` | last run's rows and summary |
| `config.backup.toml` | the config as it was before the run |
| `probe-host.html` / `launch-vietkb.cmd` | generated: the Chrome probe page, the detached launcher |
| `app.log` | stdout of the app; tracing usually does not reach it (GUI subsystem, see `95-e2e-report.md` O2) |
