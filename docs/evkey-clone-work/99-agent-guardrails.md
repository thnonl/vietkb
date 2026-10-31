# 99 — Agent guardrails on the owner's machine

**Status:** authoritative for every agent in this repo. Read before running anything that
touches processes, input, windows or user files.
**Why this file exists:** on 2026-10-30 agents stopped the owner's running VietKB (the input
method they type with, `%LOCALAPPDATA%\VietKB\vietkb.exe`) to free the hook or the target
exe, and one E2E run let Tauri single-instance redirect into it. The owner lost their IME
mid-sentence, repeatedly. That is worse than a failed test.

## 1. The owner's running app is not yours

| Path | Rule |
|---|---|
| `%LOCALAPPDATA%\VietKB\vietkb.exe` (installed, daily driver) | **Never** kill, stop, restart, replace, rename or overwrite. **Never** relaunch it. |
| `E:\vietkb\target\**\vietkb.exe` (built by an agent) | May be killed **only** if the agent launched it, and the pid+path must appear in the report. |
| Anything else (Chrome, edge, explorer, the owner's editors) | Off-limits, always. |

A kill of a process you did not launch is a **defect**, even if the test then passes.

## 2. Not even "just for the test"

- If the installed instance is alive and a test needs the global hook or the same exe path,
  **stop and ask the owner to quit it**. A script may wait (`-OwnerQuit`, countdown) but must
  never terminate it.
- Tauri single-instance means a second launch focuses the installed app instead of starting
  the build under test. Treat "the run opened the installed app" as a **blocked run**, never
  as a result.
- Never disable the owner's autostart to avoid a conflict; report it instead.

## 3. Input is lent, not taken

- **Default is zero synthetic input.** No agent types into this desktop unless the owner has
  opened a test window for it. "It was only a short run" is not a justification: a stray `ư`
  landing in the owner's text is the reported defect (2026-10-30), not a side effect.
- Synthesising keyboard/mouse input is allowed only inside an owner-granted window, and only
  for the shortest path that answers the question.
- **Never type into whatever window happens to be focused.** Every batch must be preceded by
  `GetForegroundWindow` == the HWND the agent itself created, must be followed by the same
  check, and must be abandoned (nothing sent) when they differ — including mid-sequence, so a
  delete+insert pair can never be split across two windows.
- Refuse to start while the owner is active: require `GetLastInputInfo` to show ≥ 5 s idle, and
  stop immediately if a physical key arrives during the run.
- Never leave a hook installed: if you launched `vietkb.exe`, quit it before you finish.
- If the owner says they are using the machine, stop within one command: finish the current
  call, do not start another, and report what was running.
- Diagnostic instrumenting (e.g. a read-only key logger) is also input-surface work: it needs
  the same permission and must be removed and its log deleted when the question is answered.

## 4. Leave no trace

- `config.toml` restored byte-identical (SHA + `Compare-Object`), or say why not.
- Windows sound scheme, DPI, focus, cursor position, clipboard: unchanged.
- Any file you created outside the repo (`%TEMP%`) is listed in the report.
- Log one line per process you stopped: `KILL <time> pid=<n> path=<p> reason=<why>`.

## 5. Escalation

If a task cannot be done without breaking one of these rules, the correct output is a short
report saying which rule blocks it and what the owner must do — not a workaround.

## 6. macOS addenda (2026-10-31)

- The owner may be running their own VietKB build (any path, e.g. `target/**/VietKB.app`). Never
  kill, quit or relaunch it. Tauri single-instance keys on the bundle identifier, so a normal
  build launched by an agent is **forwarded into the owner's instance** (it may pop the owner's
  settings window). While the owner's instance runs, agents launch only the `dev-channel` build
  (`vn.vietkb.app.dev`, `tauri.dev.conf.json`). A forwarded launch is a blocked run.
- Synthetic input or clicks only after ≥ 60 s HID idle (not 5 s), and only inside windows the
  agent opened. Never click the menu bar or status items by coordinate (another app's or the
  owner's VietKB item may be there); reach the agent's own status item only via its pid / AX.
- EVKey: relaunch it after a run only if it was running before the run.
- Close every document the agent opened (track them by window id when opening); report any you
  could not identify instead of leaving them.
