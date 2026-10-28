# Plan — VietKB rewrite (simplified Scrum)

Sprint length: one gate each. No ceremony beyond the artifacts in the charter.
Owner sees progress in this file and `log.md`.

---

## Sprint 0 — Foundation (Lead + BA)

- [ ] 0.1 Preserve legacy: tag `legacy-v0`, branch `legacy/pre-rewrite`, keep the working
      tree clean. Nothing from it is merged later.
- [ ] 0.2 Wipe the old workspace, scaffold the frozen crate layout from the charter §4.
- [ ] 0.3 `10-feature-inventory.md`: EVKey capability table with evidence + acceptance
      criterion per row.
- [ ] 0.4 `20-spec.md`: engine semantics, config schema, hotkey model, UI design system
      (tokens, type scale, spacing, radii, states, motion), per-screen specs.
- [ ] 0.5 `40-design-review.md`: 2–3 UI directions with an offline HTML preview; owner picks.
- [ ] 0.6 `50-test-cases.md`: QA baseline, written against the spec, not against code.

**Gate 0:** owner approves direction; contract compiles (`cargo check` on all crates with
`todo!()` bodies); no behavior implemented yet.

## Sprint 1 — Engine + Windows injector + shell UI (3 worktrees in parallel)

| Worktree | Delivers |
|---|---|
| `wt-engine` | `vkb-core` full: Telex/VNI/VIQR, tone placement, charsets, macro, spell, restore-word. Golden corpus ≥ 800 cases. Criterion bench meeting P1. `vkb-cli simulate`. |
| `wt-windows` | `vkb-windows`: low-level hook, `SendInput` injector with batched replacement, hotkeys, active-app probe, per-app rules, autostart. Crash-safe shutdown of the hook. |
| `wt-ui` | `vkb-app` + `ui/`: Tauri 2 shell, tray, single instance, settings screens (Gõ, Bảng mã, Gõ tắt, Loại trừ, Phím tắt, Giới thiệu), typed command surface, live-apply config, hotkey capture field. |

**Gate 1:** typing works in Notepad / Chrome / VS Code / Excel / Zalo; P1–P6 measured;
QA verification round on the real build.

## Sprint 2 — Parity depth

- Macro import/export, charset conversion tool (chuyển đổi bảng mã cho văn bản có sẵn).
- Per-app picker that lists running processes instead of typing exe names.
- Spell check + auto-restore polish against the corpus.
- Tray: quick charset/method switcher, notification toast.
- Installer: NSIS + MSI, per-user install, uninstall leaves config, optional autostart.

**Gate 2:** installer round-trip on a clean profile; 30-minute soak; P3–P7 re-measured.

## Sprint 3 — macOS backend (design now, build later)

- `vkb-macos`: CGEventTap, Accessibility permission flow, menu-bar app, per-app by bundle id.
- Universal binary packaging. Not in the current shipping scope; code must at least compile
  behind `cfg(target_os = "macos")` and be CI-checked.

## Sprint 4 — Release

- Code signing, update checker (opt-in), docs, website download page, release checklist.

---

## Parallelism rules

- The three Sprint 1 worktrees only touch their own crates. Shared needs (`EngineAction`
  variants, config fields, Tauri command names) require a contract change: proposed in
  `log.md`, approved by the Lead, applied to the contract by the Lead or by the owning
  worktree, then merged to all three before continuing.
- Contract files that are frozen during Sprint 1:
  `crates/vkb-platform/src/lib.rs` (traits), `crates/vkb-config/src/schema.rs`,
  `crates/vkb-app/src/commands.rs` signatures.
- Each worktree runs its own `cargo check -p <crate>` freely; full builds claim the slot.
