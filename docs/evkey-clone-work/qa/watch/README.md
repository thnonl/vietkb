# qa/watch — not used — policy fix instead (99-agent-guardrails.md §3)

This directory held a passive `WH_KEYBOARD_LL` watcher (`keystroke-watch.ps1`,
`summarise.ps1`, `watch.cmd`) built to prove where stray `ư` characters came from.

**Abandoned 2026-10-30, owner clarification:** the stray characters came from agents'
debug/test runs injecting input into whatever window was focused — not from a defect in
the app. The fix is policy (`99-agent-guardrails.md` §3 "Input is lent, not taken"),
not instrumentation. The scripts were deleted; nothing here is runnable.

No keyboard hook was ever installed by this work and no process was left running.
