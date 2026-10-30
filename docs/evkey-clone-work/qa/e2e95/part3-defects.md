## Defects filed by this gate (also added to `60-defects.md`)

### D-E2E-01 — default injection silently corrupts text in Windows 11 Notepad — **BLOCKING**

- **Where:** `crates/vkb-windows/src/input.rs` unicode-packet path (`VK_PACKET`, `send_standard = false`, the shipped default).
- **Repro:** Notepad (Win11 26H2, the shipped `Notepad` app), Vietnamese ON, type `ba6t5/ta8t1` at 55, 120, 250, 400 or 700 ms → the window ends with `bật/ttt` (also seen: `bật.ttt`, `bậttt`, `bật/`, `bật\tttt`). The separator and the second syllable's letters/digits are lost and stray `t`s remain.
- **Evidence that the engine is innocent:** `keys.log` for the same run shows the correct decisions — `Replace{delete:3,insert:"bật"}`, `/` pass, `t` pass, `a` pass, `Replace{delete:2,insert:"tâ"}`, `t` pass, `Replace{delete:3,insert:"tật"}` — so the loss happens between `SendInput` and the Notepad RichEdit document.
- **Scope:** 8 of 19 Notepad rows (`A1`, `A2`, `A3`, `C1`, `C2`, `D1`, `D2`, and `A8` is unrelated); the identical keystrokes pass in Chrome (18/19) and VS Code (17/19).
- **Workaround proven in this gate:** set `[typing] send_standard = true` — the same five sequences then produce `bật/tật` at 120/250/400/700 ms and when typed in three segments (5/5 clean). Config was restored afterwards.
- **User impact:** Vietnamese typing in Notepad — a release-gate app and the default scratchpad — silently mangles words. No error, no log line.

### D-E2E-02 — `restore_word` hotkey (`Ctrl+Shift+Z`) never fires — **BLOCKING for AC-46 / DoD 5**

- **Repro:** type `ba6t5` (engine shows `bật`), press `Ctrl+Shift+Z` → text unchanged in Chrome, VS Code and Notepad (`act=[bật]`, expected `ba6t5`).
- **Evidence:** `keys.log` for that chord is `vk=0x5A mods=SC--- stage=shortcut action=none decision={swallow:false,injection:None}` — the hook classifies Ctrl+Shift+Z as an application shortcut and passes it before the hotkey table is consulted; `Action::RestoreWord` is never dispatched although the current word is non-empty (`20-spec.md` "Manual hotkey `restore_word`" requires the replace, and only passes the chord when there is nothing to restore).
- **Impact:** the only manual way to recover a wrong word is dead, and with `restore_wrong_word = false` in the owner's config there is no automatic recovery either.

### D-E2E-03 — `F1` does not re-enable Vietnamese in VS Code — **HIGH**

- **Repro:** VS Code editor → `F2` (typing becomes literal, correct) → `F1` → typing stays literal (`ba6t5`), while the same two keystrokes behave correctly in Notepad and Chrome.
- **Hypothesis (untriaged):** VS Code's own `F1` (Command Palette) path; the hook still swallows the key but the re-enable does not land. Needs a dev repro; not reproduced outside VS Code.

### D-E2E-04 — main settings panel is 478 × 459 with no tabs — **LOW / contract deviation**

- **Measured:** client 478 × 459 at dpi 96 (screenshot `qa/e2e95/ui-1-main.png`), one scrolling page; the UIA tree contains no `Tab`/`TabItem`. Charter C9 asks for 720 × 560 and at most 4 tabs.
- Not a functional break (all controls work), but the shipped panel does not match the charter; the Lead must either amend C9 or schedule the UI work.

### Observations (not defects)

- **O1 — `https://vietkb.dev/tải` stays literal** in all three apps. Matches the frozen `raw_zone` ruling (`20-spec.md` A9, `special_cases.rs::url_email_and_code_shapes_stay_literal`); the brief's expected string contradicts it. Contract decision for the Lead.
- **O2 — `RUST_LOG=vkb=debug` produces no tracing output in the GUI process** (`app.log` holds only the launcher markers). A Windows GUI-subsystem binary has no stdout for `tracing_subscriber::fmt()`, so the "failure is silent" debug line in `sound.rs:43` is unreachable in the field — support cannot distinguish "played" from "failed". Consider a file/OutputDebugString sink.
- **O3 — tray route not automated:** the Win11 overflow flyout exposed no `VietKB` element to UIA in this pass; the tray menu item and icon state are therefore **unverified** (the installed 0.2.3 tray was not running to compare).
- **O4 — P6 first (cold-cache) run 433 ms** vs 391/387 ms after; 8 % over the 400 ms budget once.
- **O5 — D1/D2 in Notepad also lose the second word**, consistent with D-E2E-01; no separate defect.
