# 98 — Verify round close (QA, 2026-10-30)

**Scope:** independent QA of the round's fixes at `main` `b418c21` (working tree clean at start).
**Desktop:** untouched. No `vietkb.exe` launched, no process stopped, no keyboard or mouse input,
no focus or window moved, no `--live-hooks`. Every run is the engine CLI, the headless
`vkb-cli simulate`, the unit/integration tests in default mode, or the in-process zero-focus
harness (`hooks_installed=0`, `global_events_sent=0`).
**Raw outputs:** `docs/evkey-clone-work/qa/verify-close/raw/` (one file per step, plus
`exit-codes.txt`). Cargo steps are scripted in `docs/evkey-clone-work/qa/verify-close/run-cargo.sh`.

## 1. Checks

| # | Check | Command | Exit | Counts | Result |
|---|---|---|---|---|---|
| 1 | Workspace tests | `cargo test --workspace` | 0 | 32 binaries: **428 passed / 0 failed / 5 ignored** | PASS (matches dev 428/0/5) |
| 1b | Workspace tests after QA's own added test (see §3) | `cargo test --workspace` | 0 | **429 passed / 0 failed / 5 ignored** | PASS |
| 2 | vkb-windows with harness feature | `cargo test -p vkb-windows --features e2e-harness` | 0 | lib 95 passed / 1 ignored; `hook_integration` 1 passed / 2 ignored; **96 passed / 0 failed / 3 ignored** | PASS |
| 3a | no_alloc | `cargo test -p vkb-core --test no_alloc` | 0 | **4 passed / 0 failed** | PASS (4/4) |
| 3b | raw_zone | `cargo test -p vkb-core --test raw_zone` | 0 | **9 passed / 0 failed** | PASS |
| 3c | tone_change | `cargo test -p vkb-core --test tone_change` | 0 | **29 passed / 0 failed** | PASS |
| 4 | Tone matrix (debug `vkb.exe`, sha256 `19782BB9…4241F`, 1 050 112 B) | `powershell -NoProfile -ExecutionPolicy Bypass -File docs/evkey-clone-work/qa/tone/run.ps1` | 0 | **123 PASS / 0 FAIL** (baseline before round 104/19; prior QA run 88/35) | PASS. See §4 on the execution-policy note |
| 5a | Corpus oracle (release `vkb.exe`) | `powershell -NoProfile -ExecutionPolicy Bypass -File docs/evkey-clone-work/qa/e2e/check-corpus.ps1` | 0 | 10 paragraphs telex PASS, 10 vni PASS; `PARAGRAPH_FAILURES=0` | PASS |
| 5b | Edge-case oracle (release `vkb.exe`) | `powershell -NoProfile -ExecutionPolicy Bypass -File docs/evkey-clone-work/qa/e2e/check-edge-cases.ps1` | 0 | `EDGE_PASS=95 EDGE_FAIL=0` | PASS |
| 6 | In-process zero-focus harness (default mode) | `cargo run -p vkb-windows --features e2e-harness --example zero_focus_e2e --release` | 0 | **PASS=254 FAIL=0**; control `caught`; `mode=in-process`; `hooks_installed=0`; `global_events_sent=0`; events_fed 30 468, injected_captured 39 604, passed 26 922, swallowed 3 546; "no focus interaction" | PASS. Output file `target/zero-focus-e2e.txt` (copy in raw/). Harness statement "MODE in-process: no hooks installed, no SendInput, zero global events" confirmed |
| 7 | P1 bench (release `vkb-cli`) | `target/release/vkb.exe bench --keys 1000000 --method telex` and `… --method vni` | 0 / 0 | telex p99 **1.000 µs**, p50 0.300, max 321.8 µs, 2.35 M keys/s; vni p99 **0.600 µs**, p50 0.300, max 767.8 µs, 3.12 M keys/s. Budget p99 ≤ 50 µs | PASS (max outliers are not the budgeted metric; noted in §5) |
| build | Debug and release `vkb-cli` build | `cargo build -p vkb-cli`, `cargo build -p vkb-cli --release` | 0 / 0 | — | PASS |

Test-binary `ignored` items were not run (they are `#[ignore]` by design): `hook_integration`
(two tests that install a global keyboard hook and SendInput a key), `clipboard` round-trip
(clobbers the clipboard), `update` tests (network / fake release env). `zero_focus_e2e --live-hooks`
was not run.

### Execution-policy note (step 4)
Plain `powershell -NoProfile -File docs/evkey-clone-work/qa/tone/run.ps1` is refused on this
machine ("running scripts is disabled on this system", exit 1). A bash-mangled path first
returned exit 127. The run that counts used a process-scoped `-ExecutionPolicy Bypass`; no
machine or user policy was changed. The same flag is needed for every `.ps1` in `qa/e2e`.
Attempts are all recorded in `exit-codes.txt`.

## 2. Spot checks (code and tests)

| Item | Where | Status |
|---|---|---|
| D-TN-02 Telex `z` removes the tone | `crates/vkb-core/tests/tone_change.rs` `d_tn_02_*` (4 tests: removes tone, keeps circumflex, literal without tone, retype after z) | covered, pass |
| D-TN-03 VNI `0` removes the tone only | `tone_change.rs` `d_tn_03_*` (keeps marks; literal digit with nothing to remove) | covered, pass |
| D-TN-04 tone change obeys the A9 coda check | `tone_change.rs` `d_tn_04_*` (4 tests: illegal change literal, fresh-word sibling, legal changes apply, spell-check off applies) | covered, pass |
| D-TN-05 digit with no target is literal | `tone_change.rs` `d_tn_05_*` (2 tests) | covered, pass |
| D-TN-01/06/07/08 (already fixed in `dev-tone-build`) | `tone_change.rs` `d_tn_01_*`, `d_tn_06_*`, `d_tn_07_*`, `d_tn_08_*` | covered, pass |
| D-E2E-03 trigger swallow (down and up of F1/F2 swallowed, release-mode latch) | `crates/vkb-windows/src/state.rs` tests `release_mode_trigger_down_and_up_are_swallowed`, `auto_repeat_follows_the_bit_not_the_latch`, `bit_is_set_only_when_the_key_down_was_swallowed`, `apply_config_clears_the_swallowed_bits` | covered, pass |
| Stale trigger bits do not swallow a fresh key-down | `state.rs` `stale_bit_is_cleared_by_a_fresh_key_down`, `fresh_down_passes_when_both_the_down_and_trigger_bits_are_stale` | covered, pass |
| Negative control for D-E2E-03 | harness `CONTROL PASS hotkey/f1-enable-composes/telex/uni` (`caught`) | pass |
| Bare Shift no longer ends a word (`aS` → `á`, `DD` → `Đ`) | code `state.rs` ~L443 (`ch.is_none() && is_modifier` → PASS, no reset). Unit test: **added by QA**, `state::tests::bare_shift_between_letters_does_not_end_the_word` (see §3) | covered, pass |
| Same, end to end in-process | harness shifts every capital (`zero_focus_e2e.rs` `char_syns`: `tap_vk(vk, shift)` on uppercase); edge rows T20 `VIEETJ`→`VIỆT`, T44 `AAS`→`Ấ` pass in both modes | covered in-process, pass |
| Mouse button-down ends the word | `state.rs` test `mouse_button_down_ends_the_word` (line ~1440); harness case `mouse-click-ends-word` (telex and vni, std mode, PASS) | covered in-process, pass. Real WH_MOUSE_LL not exercised (see §6) |

## 3. QA changes

- **One test added** to `crates/vkb-windows/src/state.rs` (test module only, end of file):
  `bare_shift_between_letters_does_not_end_the_word` (aS → á; space; DD → Đ). Result: PASS
  (`raw/10-new-bare-shift-test.txt`), and the full workspace rerun is 429 / 0 / 5
  (`raw/11-cargo-test-workspace-after-qa-test.txt`). This is the only product-tree file touched.
  It was not run against the pre-fix code (product code must not be reverted for a check). Its
  failure mode is reasoned from the pre-fix behaviour: a bare Shift reset the engine, so the
  shifted `S` would be literal and `swallow` would be false.
- Docs only otherwise: this file, `60-defects.md`, `log.md`, `qa/verify-close/**`, and the
  refreshed `qa/tone/matrix.{txt,md}` and `qa/e2e/{corpus,edge}-results.txt` written by the scripts.

## 4. Failing or suspicious

Nothing failed. Counts match the dev report (428/0/5 workspace, 254/0 harness, 123/0 tone).

Observations, none of them blocking:
1. The execution policy on this machine blocks the `.ps1` oracles and the tone matrix unless
   `-ExecutionPolicy Bypass` is passed (see §1). Worth a line in `qa/README` for the next runner.
2. Bench max outliers (321.8 µs telex, 767.8 µs vni) are well above the p99 budget. The budget
   is p99 (≤ 50 µs), so this is not a failure, but a single 0.7 ms outlier per million keys is
   worth watching in the host window.
3. Temp files written by the oracle scripts outside the repo, listed as required by guardrail §4:
   `C:\Users\namth\AppData\Local\Temp\vkb-sim.txt`, `…\vkb-sim.err`, `…\vkb-edge.txt`,
   `…\vkb-edge.err`. They are overwritten on every run and contain no user text.
4. The bare-Shift fix is covered end to end only by the in-process harness (which types capitals
   with a shift wrap) and the new unit test. The headless CLI oracles type no Shift, so they do
   not cover it.

## 5. Needs host window (cannot be verified without the desktop)

These need an owner-approved test window (guardrail §3). None was attempted here.

1. Real-app Unicode transport: Notepad, VS Code and Chrome with the D-E2E-01 scancode-ASCII
   replacement. The engine and packet builders are verified in-process; the OS text stack is not.
   Also the residual three-consecutive-non-ASCII collapse (for example `được`) in Notepad.
2. The real `WH_MOUSE_LL` thread delivering clicks, and the mouse-button-down word boundary in a
   real app (for example `aa` then click then `a`).
3. The real low-level hook ordering (H_low, backend, H_high sandwich) with `--live-hooks`. The
   in-process harness installs no hooks.
4. OS delivery of the D-E2E-02 restore prefix (key-ups for the chord modifiers before
   `restore_word` replays, `build_restore`). Unit-tested as a replay of the INPUT array only.
5. D-E2E-03 real VS Code check: F2 then F1 re-enables Vietnamese and `ba6t5` gives `bật`.
   The status line in `60-defects.md` says the same.
6. D-E2E-05 real-app check: VIEETJ, AAS and DD typed with Shift in Notepad, VS Code and Chrome.
7. Restore key `Ctrl+Shift+Z` in real apps (it is also redo in many editors, see `state.rs` ~L603).
8. Sound audibility of the toggle (`play_toggle`, `SND_MEMORY`). Not auditable headless.
9. Installer and updater. The P7 installer was not built in this round; updater tests are
   network-only and ignored.
10. Tray menu item, focus behaviour and the settings panel at other DPI settings.
11. The hook integration tests that install a real global hook (`hook_integration`, ignored).

## 6. Leave no trace

- No process started or stopped by QA. `tasklist` showed no `vietkb` or `vkb` process at the
  end of the run. The installed VietKB was not running at the time of the check.
- No config file touched. No `Cargo.lock` change. No version bump.
- Files created outside the repo: the four temp files in §4.3.
