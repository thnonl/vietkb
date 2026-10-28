# Charter — VietKB (EVKey clone, Rust + Tauri 2)

**Status:** authoritative. Every agent reads this before touching anything.
**Owner:** the human user. Taste and scope decisions are theirs.
**Repo:** `E:\vietkb` (Windows dev machine).

---

## 1. Mission

Build a Vietnamese input method that reaches **full EVKey feature parity** while being
faster, smaller and far easier to configure — then ship it for **Windows first**, with a
macOS backend designed in but implemented later.

The existing code in this repo is **legacy reference only** and is being replaced.
Nothing in it is authoritative; its behavior is evidence, not a contract.

Non-goals for the current cycle: Linux backends, cloud sync, telemetry of any kind,
keystroke content leaving the machine.

## 2. Product constraints (settled)

| # | Constraint |
|---|---|
| C1 | Rust for every engine/backend path. No C++ beyond a thin `SendInput`/hook FFI layer. |
| C2 | Tauri 2 for the settings UI. WebView2 on Windows, WKWebView on macOS. |
| C3 | Engine is pure: no OS calls, no I/O, no threads, fully unit-testable. |
| C4 | No clipboard-based text injection in the default path. Clipboard is an explicit opt-in fallback for apps where `SendInput` fails, and it must restore the previous clipboard content. |
| C5 | No keystroke content is ever logged, persisted or transmitted. Crash logs are local and opt-in. |
| C6 | Config is a human-readable TOML file with a schema version and forward migrations. |
| C7 | UI language: Vietnamese default, English available. All *product* strings stay Vietnamese-verbatim where they are EVKey terminology (kiểu gõ, bảng mã, gõ tắt, …). |
| C8 | Windows 10 1809+ and Windows 11, x64 first. arm64 build target exists but ships later. |
| C9 | **The settings window is a quick-settings panel, not a dashboard.** Default size 720 × 560 (frameless, the status strip doubles as the title bar), minimum 680 × 520, not resizable above what the content needs. At most **4 tabs**, no sidebar, no home screen. The screen the user lands on is the one with the typing options; everything else is one tab away. No scrolling on any tab at 100 % DPI. |
| C10 | Anything that needs more room than the panel (bảng gõ tắt 5000 dòng, công cụ chuyển mã văn bản) opens as a **separate window or dialog**, never by growing the main panel. |

## 3. Performance budgets (hard gates, measured on this machine)

| Metric | Budget | How measured |
|---|---|---|
| P1 Engine `process_key` latency | p99 ≤ 50 µs, zero heap alloc on the steady path | `cargo bench` in `vkb-core`, criterion, 100k iterations |
| P2 End-to-end keystroke → visible char | p95 ≤ 8 ms, p99 ≤ 15 ms | instrumented build, 500 words into Notepad + Chrome, timestamps from hook to `SendInput` return |
| P3 Idle memory, GUI closed, tray only | ≤ 20 MB RSS | Task Manager / `Get-Process` after 10 min idle |
| P4 Idle memory, settings window open | ≤ 90 MB RSS | same |
| P5 CPU while idle 1 h | ≤ 0.1 % average | same |
| P6 Cold start to hook-ready | ≤ 400 ms | process start → first successful hook install |
| P7 Installer size | ≤ 10 MB | built msi/nsis artifact |

A change that breaks a budget is a defect, not a tradeoff. Measure before and after.

## 4. Architecture (frozen contract)

```
crates/
  vkb-core/        pure engine: telex, vni, viqr, tone, vowel, charset, macro, spell, restore
  vkb-config/      schema (TOML), defaults, migration, hotkey parsing, per-app rules
  vkb-platform/    PlatformBackend + EngineSink traits, AppInfo, no OS code
  vkb-windows/     WH_KEYBOARD_LL hook, SendInput injector, hotkeys, active-app probe, autostart
  vkb-macos/       CGEventTap + Accessibility (skeleton, compiles, unimplemented)
  vkb-app/         Tauri 2 host: commands, tray, single-instance, updater, window mgmt
  vkb-cli/         `simulate` and `bench` subcommands for headless testing
ui/                Vite + TypeScript + Svelte 5, hand-written CSS design tokens
```

Rules:
- `vkb-core` must not depend on any crate that touches the OS. Enforced by CI check.
- `vkb-windows`/`vkb-macos` contain **no Vietnamese logic**. They translate key events in
  and `EngineAction` out.
- The Tauri layer only orchestrates: it never contains typing rules.
- The frontend talks to Rust exclusively through the typed `@tauri-apps/api` command
  surface defined in `crates/vkb-app/src/commands.rs`. No `invoke("raw_string")` anywhere.

## 5. Team

| Role | Agent | Model | Owns | Must not |
|---|---|---|---|---|
| Lead | main session | `opencode-go/deepseek-v4.1-flash` | charter, contract, gates, final report, integration | write product code beyond scaffolding |
| BA | `general` | `anthropic/claude-opus-5-5#medium` | feature inventory, spec, plan, acceptance criteria | edit product code |
| Dev | `general` | `opencode-go/deepseek-v4.1-flash#max` | all implementation in its worktree | change the contract, change design values |
| QA | `general` | `opencode-go/deepseek-v4.1-flash#low` | test cases, defect log, independent verification | edit product code (may add tests) |
| Design | `general` | `anthropic/claude-opus-5-5#medium` | visual directions, tokens, screenshot-ready mocks | decide taste alone |
| Copy | `general` | `anthropic/claude-opus-5-5#medium` | every Vietnamese UI string, terms, states | edit code or layout |

**Reasoning level rule (owner, 2026-10-28):** Opus runs at **`medium`**. `high`/`xhigh` only when
the owner asks for it by name — the cost is not justified for routine work. DeepSeek devs stay at
`max` (they are cheap), QA at `low`.


Scrum, simplified: Sprint 0 = charter + inventory + spec + contract freeze.
Sprint 1+ = 3 parallel dev worktrees, one demo gate per sprint, QA verifies after design
converges. One `review` pass by `cavecrew-reviewer` before each merge.

**Editing rule (owner, 2026-10-28 — enforced after a violation):** the Lead **never hand-edits
product code**, not even mechanical merge-conflict resolution, a one-line rename, or a "quick fix".
Every change under `crates/` and `ui/` goes to a dev child; the Lead writes only `docs/`,
`design/` and the charter/contract, and runs read-only verification (`git status`, `cargo test`,
`cargo check`) plus the commit itself. If a dev child is unavailable, the Lead stops and asks
instead of editing.

Model rule: read the model from the dispatching session's metadata at dispatch time.
Never hardcode a model string across turns. Children inherit the same rule for their children.

## 6. Worktrees and shared resources

Max **3** worktrees, each a long-lived stream:

| Worktree | Branch | Scope |
|---|---|---|
| `wt-engine` | `feat/engine` | `vkb-core`, `vkb-config`, `vkb-cli` + tests/benches |
| `wt-windows` | `feat/windows` | `vkb-platform`, `vkb-windows`, `vkb-macos`, packaging |
| `wt-ui` | `feat/ui` | `crates/vkb-app`, `ui/`, tray, autostart, updater |

Protocol:
- A worktree owns only its scope. A cross-scope change goes through the Lead as a
  contract change, never as a side edit.
- Build slots: only one `cargo build`/`tauri build` runs at a time per machine.
  Claim it in `docs/evkey-clone-work/log.md` (`BUILD: <agent> <branch> <ts>`), release
  when done. Concurrent cargo on the same `target/` corrupts artifacts.
- Dev server ports: `wt-ui` 1420, `wt-engine` 1421, `wt-windows` 1422.
- Signing/packaging and any installer run belongs to the Lead gate only.
- Test devices: this machine is the only Windows target. Input-hook testing requires a
  real foreground app; nobody runs the app while QA is measuring. QA owns the window.

## 7. Artifacts (all under `docs/evkey-clone-work/`)

| File | Owner |
|---|---|
| `00-charter.md` | Lead |
| `10-feature-inventory.md` | BA |
| `20-spec.md` | BA |
| `30-plan.md` | BA → Lead approves |
| `40-design-review.md` | BA/Design |
| `50-test-cases.md` | QA |
| `60-defects.md` | QA opens, Dev notes fixes, QA closes |
| `70-impl-notes.md` | Dev |
| `80-verify-report.md` | QA |
| `log.md` | everyone, 1–3 lines per milestone |

## 8. Definition of Done (Sprint 1, Windows)

1. Telex, VNI, VIQR produce correct Vietnamese for the golden corpus (≥ 800 cases,
   including `ươ`, `uy`, `oa`, `oe`, `qu`, `gi`, tone-shift rules, backspace, uppercase,
   punctuation-triggered commit).
2. Bảng mã: Unicode dựng sẵn (NFC), Unicode tổ hợp (NFD), TCVN3 (ABC), VNI Windows, VIQR.
3. Gõ tắt (macro) with a searchable table UI, import/export, max ≥ 5000 entries.
4. Per-app rules: exclude list and per-app override, matched by process name.
5. Hotkey rebinding UI with capture-field, conflict detection, and reset-to-default.
6. Tray menu: bật/tắt, kiểu gõ, bảng mã, mở cài đặt, thoát. Icon reflects state.
7. Settings persist across restart; config migration from v0 legacy INI if present.
8. All performance budgets P1–P7 measured and met.
9. Windows installer builds and installs cleanly on a machine without Rust.
10. No crash in a 30-minute soak with continuous typing in 5 apps.

## 9. Guard list (must not regress)

- Engine purity (no OS crate in `vkb-core` dependency tree).
- Hotkey defaults: `Ctrl+Shift` toggle, EVKey-compatible behavior.
- Text injection never leaves a stray backspace when the action is a no-op.
- Clipboard, if used at all, is restored exactly, including non-text formats ignored.
- Config file never loses unknown keys on write (forward compatibility).

## 10. Environment facts (measured 2026-10-28)

| Fact | Value |
|---|---|
| OS | Windows 11 build `10.0.26300` (26H2), x64 |
| Rust | `rustc 1.98.1`, toolchain `stable-x86_64-pc-windows-msvc` |
| Installed targets | `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu` |
| Node | `v24.22.1` |
| WebView2 | runtime `155.0.4283.62` present |
| Tauri CLI | **not installed** — use `npm.cmd install` + `npm.cmd run tauri` or add `@tauri-apps/cli` as a devDependency |
| Bundlers | NSIS and WiX **not** on PATH; Tauri downloads NSIS on first bundle (network needed) |
| Git | `2.55.0.windows.5`; repo needs `safe.directory` (dubious-ownership error) before any git command works |
| Disk | E: free ≈ 208 GB; legacy `target/` is 13.4 GB and will be deleted |
| macOS builds | **cannot** be produced on this machine (no Apple linker/SDK). macOS work is compile-checked only, shipped from a Mac later |

**PowerShell gotcha:** `.ps1` shims are blocked by execution policy. Always call
`npm.cmd`, `npx.cmd`, `node.exe` — never the bare `npm`/`npx` aliases.

Legacy code lives on branch `legacy/pre-rewrite` (tag `legacy-v0`). Do not merge it.
