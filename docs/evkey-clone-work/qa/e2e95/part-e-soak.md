**Window:** 03:30:15 → 04:02:02 (31 min 47 s wall, 10 rounds × 3 min). **Targets rotated:** Chrome probe, VS Code scratch editor, Notepad scratch, a Windows Terminal console, and the app's own settings WebView. **Keystrokes:** ≈ 35 060 (79-char Vietnamese sentence × 4 bursts × ~110 loops), with `F2`/`F1` toggles every third round and `m01 ` macro use on even rounds.

| Round | Target | App alive | app / total mem | Hook canary (`ba6t5` in Chrome) | WARN/ERROR lines |
|---|---|---|---|---|---|
| 1 | chrome | yes | 25.1 / 353.8 MB | TRANSFORMS | 0 |
| 2 | vscode | yes | 25.2 / 355.1 MB | TRANSFORMS | 0 |
| 3 | notepad | yes | 25.4 / 356.5 MB | TRANSFORMS | 0 |
| 4 | console | yes | 25.4 / 356.7 MB | TRANSFORMS | 0 |
| 5 | settings webview | yes | 25.5 / 367.4 MB | TRANSFORMS | 0 |
| 6 | chrome | yes | 25.4 / 369.2 MB | TRANSFORMS | 0 |
| 7 | vscode | yes | 25.5 / 368.8 MB | TRANSFORMS | 0 |
| 8 | notepad | yes | 25.4 / 368.6 MB | TRANSFORMS | 0 |
| 9 | console | yes | 25.5 / 368.5 MB | TRANSFORMS | 0 |
| 10 | settings webview | yes | 25.5 / 370.5 MB | TRANSFORMS | 0 |
| end | tray only (windows closed) | yes | **2.6 MB** (4.1 MB two minutes later) | — | 0 |

**Soak findings**

- **No crash, no hang, no dead hook.** The app process was the same PID for the whole window; every round's canary keystroke was transformed, including the last one after 30 minutes of continuous input.
- **Memory is flat, not climbing.** The app process stayed within 25.1–25.5 MB for 30 minutes (0.4 MB spread); the total stayed under the P4 400 MB ceiling even with the settings WebView open (peak 370.5 MB) and collapsed to 2.6 MB once the windows were closed — the "destroy, don't hide" behaviour P3/P4 require.
- **No WARN/ERROR storm — but this check cannot fail on this build.** `app.log` contains no tracing output at all (see O2), so a zero count is not evidence. Judge the soak on the alive/transform/memory rows, not on that column.
- One transient dialog appeared, and it was ours: Notepad's `Keep unsaved changes?` for the QA scratch file that the harness recreates between rounds; `Repair-Notepad` dismissed it (round 3 log line). Not an app fault.
- The Notepad target kept losing text exactly as D-E2E-01 describes during the soak; it never affected the other four targets or the hook's liveness.
