# macOS handoff: bring VietKB to the Mac

Brief from the Windows Lead session ("Fix bugs gõ"), sent on the owner's request, received 2026-10-31.
Mac Lead: Opus 5.5 at medium effort. Team models unchanged (Dev Sonnet, QA Haiku/Sonnet, reviewer caveman:cavecrew-reviewer). Reply to the owner in Vietnamese, terse. Ask the owner before any visible GUI/feature decision.

Goal: a macOS build of VietKB, tested and debugged by the same agent team as the Windows build, so the next release (0.2.7 or 0.3.0) ships Windows and macOS together.

## Start
```bash
git clone https://github.com/thnonl/vietkb.git ~/vietkb
cd ~/vietkb
```
Toolchain: Xcode Command Line Tools, Rust stable (`rustup`), Node 20, then `npm --prefix ui ci`.
Read first: `docs/evkey-clone-work/00-charter.md`, `20-spec.md`, `25-contract.md`, `99-agent-guardrails.md`, `60-defects.md`, `crates/vkb-macos/src/lib.rs`, `crates/vkb-windows/` (reference backend).

## Current state (2026-10-31, Windows 0.2.6)
- `vkb-core` (engine), `vkb-config`, `vkb-platform` (injection planning) are platform-neutral and tested: `cargo test --workspace` passes on Windows, including `crates/vkb-core/tests/editing_scenarios.rs` (~300 000 generated edit sequences, Telex and VNI).
- `crates/vkb-macos` is a stub: every function is `todo!()` or returns `false`/`None`. `vkb-app` depends on it under `cfg(target_os = "macos")`.
- `.github/workflows/release.yml` builds only the Windows NSIS installer; `ci.yml` runs only on `windows-latest`.

## Team (same as Windows)
- Lead (you): plans, decides, briefs agents, merges, writes docs. Never edits `crates/` or `ui/` directly.
- Dev = Sonnet subagent in its own git worktree; writes code and tests, commits on its branch.
- QA = subagent (Haiku for mechanical runs, Sonnet for test design); writes tests, runs real-app checks, mostly read-only.
- Reviewer = `caveman:cavecrew-reviewer`; one review pass before every merge; a BLOCK must be resolved and re-reviewed.
- Commit or push only when the owner asks. No attribution lines in commits.

## Work plan
1. Build on macOS: `cargo build --workspace` and `cargo test --workspace` pass on macOS (fix `cfg(windows)` assumptions in shared crates and the app). Engine tests pass unchanged.
2. Backend in `crates/vkb-macos`:
   - `CGEventTap` on a dedicated run-loop thread, feeding `vkb-core::Engine` like `vkb-windows/src/state.rs` (key down/up, modifiers, Backspace, boundaries, caret-move resets).
   - Injection: `CGEventPost` + `CGEventKeyboardSetUnicodeString` for text, real key codes for Backspace; reuse `vkb-platform` planning (`plan.rs`, `keep_unchanged_head`); tag injected events (event source user data) so the tap ignores them.
   - Accessibility: `AXIsProcessTrusted`, prompt via `AXIsProcessTrustedWithOptions` on first enable, clear UI when missing.
   - Secure input: `IsSecureEventInputEnabled` → pass everything through.
   - Toggle hotkeys, restore-word hotkey, per-app state (`NSWorkspace` frontmost bundle), autostart (LaunchAgent or `SMAppService`), sounds, menu-bar-only app.
   - Mac-native GUI and features (owner, 2026-10-31): GUI and feature set may change to fit macOS — menu-bar extra instead of tray, hotkeys that do not clash with macOS shortcuts (no `Alt+Shift`/`F1`/`F2` defaults), System Settings–style layout, hide or replace settings meaningless on macOS (legacy charsets, Windows-only injection options). Keep the shared engine and config schema; record every Mac-specific decision in a new `docs/evkey-clone-work/27-mac-decisions.md`; ask the owner on anything visible.
3. Packaging: Tauri `.app` + `.dmg` (universal, or arm64 + x64), updater support. Add a `macos-latest` job to `ci.yml` and a macOS build job to `release.yml` uploading the `.dmg` next to the Windows installer. Unsigned first; document Gatekeeper steps in release notes.
4. QA on the real Mac, same depth as Windows:
   - `cargo test -p vkb-core --test editing_scenarios`.
   - Real-app E2E: TextEdit, Safari or Chrome, VS Code (isolated: `--disable-extensions`, own user-data dir, suggestions off), Notes, Terminal, a password field (must stay raw).
   - Owner-style editing: type, delete fully, retype; delete back to the first consonant and continue; wrong English word then add tone; digit before a word; reopen across a space; Shift capitals, Caps Lock; Telex and VNI.
   - Log every defect in `60-defects.md` (repro, expected, actual, evidence, status).
5. Release only when the owner says so.

## Guardrails on the owner's Mac
- EVKey (or another Vietnamese IME) running: quit it before tests (`osascript -e 'quit app "EVKey"'` or `pkill -x EVKey`), relaunch after the run if it was running. Owner approved. Log every process quit/relaunched.
- macOS built-in Vietnamese input source: switch to ABC (U.S.) for the run, restore afterwards.
- Touch nothing else of the owner's; restore any config byte-identical.
- Synthetic input only after ≥60 s owner idle; abort on physical input; keep focus on agent-opened test windows; close VS Code welcome/sign-in/extension tabs that steal focus.
- Accessibility and Input Monitoring prompts are the owner's to approve; ask, never script System Settings.
