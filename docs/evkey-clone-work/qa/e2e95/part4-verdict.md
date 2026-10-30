## Gate verdict — item by item

| Gate item | What was required | Result | Verdict |
|---|---|---|---|
| Preconditions | binary identity, no stale process, config backed up, install path chosen | all met; brief's hash is 62 chars (typo, see note) | PASS |
| A1 Typing — Notepad | the 11-case list + BS/restore/macro/boundary/off | 11/19 rows correct; text silently corrupted on 8 | **FAIL — D-E2E-01** |
| A2 Typing — Chrome | same list | 17/19 (the two: URL raw-zone by spec, restore-word) | PASS (with D-E2E-02) |
| A3 Typing — VS Code | same list | 15/19 (URL, restore-word, F1 not re-enabling, Tab→space) | PASS (with D-E2E-02/03) |
| B Toggle — hotkeys (`Alt+Shift`, `F1`, `F2`) | state flips, sound, icon | all four flips measured by typed text | PASS |
| B Toggle — UI switch | state flips, sound | On→Off→On with `TogglePattern`, confirmed by typed text | PASS |
| B Toggle — tray `Bật/tắt tiếng Việt` | state flips, icon reflects state, sound | icon left-click toggles both ways; icon name carries the state; the **menu item** is not UIA-reachable → unverified | PASS (icon) / UNVERIFIED (menu item) |
| B Sound path | no sound-error line while a scheme-less machine would produce one | no error line, but the GUI process emits no tracing output at all → the check cannot discriminate | **INCONCLUSIVE** (audibility is the owner's check) |
| C UI — `Nâng cao` 560×388, no clipping, no sound row, no pack select | measured | 558×387 client @96 dpi (=560×388 CSS), footer + last row inside, no picker, master switch only | PASS |
| C UI — `Phím tắt`, `Bảng gõ tắt`, `Giới thiệu` open/close | | hotkeys and macros open/close; About lives inside `Nâng cao` and was opened/closed in the follow-up | PASS |
| C UI — main window tabs/controls | charter C9 | 478×459, no tabs (`D-E2E-04`); controls work | **deviation** |
| D P1 | p99 ≤ 50 µs | 0.8 µs | PASS |
| D P3 | ≤ 20 MB tray-only after 10 min | 8.4 MB total, 0 WebView children | PASS |
| D P4 | app ≤ 40 MB, total ≤ 400 MB, drop ≤ 3 s | 27.9 MB / 347.5 MB / 343 MB dropped in 3 s | PASS |
| D P5 | ≤ 0.1 % idle CPU | 0.016 % over 10 min (1 h extrapolated) | PASS |
| D P6 | ≤ 400 ms cold start → hook ready | 391 ms median, 433 ms first run | PASS (flagged) |
| D P7 | ≤ 10 MB installer | exe 7.99 MiB; installer not built this pass | PASS (exe) / UNMEASURED (installer) |
| E Soak | 30 min, 5 apps, no crash, hook alive, memory stable, no WARN/ERROR storm | 10/10 rounds alive, hook transforming every round, memory flat at 25 MB, no crash | PASS |

**Release policy (`91` §"Release policy") cross-check:** item 1 (install/portable) done; item 2 (hook E2E in Notepad/Chrome/editor) **FAILS on Notepad**; item 3 (toggles: state + icon + sound) state/icon pass, sound inconclusive; item 4 (UI: no pack row, `Nâng cao` 560×388) passes; item 5 (P1–P7) passes on measured values, installer size unmeasured; item 6 (soak) passes on liveness and memory; its WARN/ERROR sub-check is uninformative because the build emits no tracing output (O2); item 7 (bump) therefore blocked.

**HOLD** — blocking items: **D-E2E-01** (Notepad text corruption with the shipped default injection; workaround `send_standard = true` proven) and **D-E2E-02** (`restore_word` hotkey dead). D-E2E-03 (F1 in VS Code) should be triaged in the same pass. Nothing else blocks: budgets are met, the UI gate items pass, and the 30-minute soak ran 30 minutes without a crash and without the hook dying.

