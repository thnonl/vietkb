# Mac QA brief: settings UI direction B (branch `ui-macos-b`)

For the Mac Lead session. Read the global `team-workflow` skill and `docs/team-profile.md` first;
follow them (desktop-qa-lock, owner idle, never replace `/Applications/VietKB.app`, QA build in
`qa-build/VietKB.app`, re-signed with the project cert, one app instance).

## Goal
Real-app UI QA on macOS of the new single settings window, then fix Mac-only bugs on a
`mac-ui-b` branch rebased on `ui-macos-b` (shared-code changes go back to the Windows Lead).
Owner approval for this QA: "Còn về QA trên Mac, hãy tự tạo 1 session để chạy QA, nếu cần fix bug
thì follow team workflow." (2026-11-09). Commit, push, merge still need the owner's word per action.

## Build under test
`git fetch && git checkout ui-macos-b` (head 9590a51 or newer; the window is being narrowed to
480 px in a follow-up commit, so pull the latest before starting). Build the QA app per the team
profile (release, `--features custom-protocol`).

## Spec
`design/macos-style/SPEC.md` "B final" sections 1-12 (11 = acceptance checks, 12 = owner tweaks),
mockup `design/macos-style/B-final.html`. Owner rulings: direction B, logo-blue accent (fills
#2876D2 white text, switches #3D8BE0 / #5AA2EE), contrast reasonable, controls 24 px, soft focus
halo on text fields, window narrow (480), height follows content capped at 560.

## Mac-specific rows
- Overlay title bar 28 px, traffic lights placed correctly, no custom caption controls.
- Window staging: no flash, no jump; opens at final height. Accessibility banner present at first
  show when permissions are missing (no 80 px growth after show).
- Tab switch and Gõ tắt list growth resize the window (fit.ts, App.svelte observers).
- Secure-input hint rendered (36 px).
- Light and dark (System Settings appearance and the in-app theme button).
- Cmd+1..5, Cmd+W (app stays in the menu bar), arrows between toolbar tabs, focus rings.
- Menu-bar menu items open the one window on the right tab; second launch shows the window.
- `MAC-REVIEW:` comments in `crates/vkb-app/src/main.rs` (hold_until_ready staging only the main
  window, show_aux emit before the window exists, refocus_modal removal) and
  `ui/src/lib/permissions.svelte.ts`: review and test each.
- Mac build compiles: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## Output
`design/macos-style/qa/qa-report-mac.md` (row | expected | actual | PASS/FAIL/NOT RUN | shot),
screenshots per tab and theme. Send the Windows Lead (session "Giao diện phong cách macOS") a short
status with counts and each FAIL, and any shared-code fix needed.
