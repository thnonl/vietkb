# 91 — Round plan: toggle sounds + special-case typing audit

**Lead:** main session · **Date:** 2026-10-30 · **Owner request (verbatim intent):**
1. Create a few audio sounds usable as the on/off (bật/tắt) sound of the input method.
2. Audit special cases around typing, starting from a reported bug:
   typing `/` then Vietnamese no longer composes —
   expected `bật/tắt`, got `bật/ta8t1` (VNI digits left unconverted after the slash).

Authoritative project rules: `00-charter.md` (Lead never edits product code; Dev children own
`crates/` and `ui/`; QA is independent; one build slot; artifact ownership table in §7).

## Scope cut — 2026-10-30 (owner ruling, sound stream only)

Owner: "Tôi chỉ muốn có âm thanh khi bật tắt bộ gõ, tôi không yêu cầu thêm config hay UI mới nào cả."

**Rule now: the on/off toggle is audible and ON differs from OFF. That is the whole feature.**

- **A1 / A2 / A3 collapse into one item:** *one embedded pair (`go_moc`), no config, no UI.* A1 is the
  reduced `22-sound-spec.md`; A2/A3 are the same landing (`assets/sounds/go_moc/{on,off}.wav` +
  `crates/vkb-windows/src/sound.rs`), with no `sound_pack` key, no picker row, no preview command.
- **Cut** (built 2026-10-30 01:06–01:29, then removed — do not re-add): the six packs + `system` choice,
  `[system] sound_pack` + `SoundPack`, `scheme_silent`/`schemeSilent`, `preview_sound` /
  `api.previewSound`, the `Kiểu âm thanh` row with hints and `Bật`/`Tắt` buttons, the 15 copy strings,
  `design/sound-data.json`, `design/sound-preview.txt`, and the `Nâng cao` height 388 → 424.
- **Why:** the owner asked only "make the toggle audible", so the picker/config/preview surface has no
  requester; every extra surface is dead weight in the panel (C9) and in the exe (P7). One pair is 8 468 B
  versus 52 834 B for six.
- **Kept:** `system.notification_sound` as the only sound switch; `Injection::Beep` → `play_error()` as the
  invalid-keystroke error cue (defect fix, deliberately asymmetric to the three toggle routes).
- Detail, measurements and the archived copy table: `22-sound-spec.md` §9.

## Pipeline for this round

| Step | Owner | Artifact |
|---|---|---|
| A1 sound spec (**collapsed** — reduced, see Scope cut) | BA | `22-sound-spec.md` |
| A2 generate the `go_moc` pair (**collapsed** into A3) | Dev | `assets/sounds/go_moc/{on,off}.wav` |
| A3 wire playback, no config, no UI (**collapsed**) | Dev | `crates/vkb-windows/src/sound.rs`, toggle callers |
| B1 reproduce `/` bug on the real engine, enumerate sibling cases | QA | `51-special-cases.md` |
| B2 fix engine/punctuation boundary defects | Dev | `crates/vkb-core` (+`vkb-windows` if hook-side) |
| B3 independent re-verification of both streams | QA | `92-verify-report.md` |
| Final gate | Lead + `cavecrew-reviewer` | review receipt in `log.md` |

## Report contract (all children)
- Deliverable = the artifact file + a compressed receipt: what changed, exact `file:line`,
  commands run with exit status, counts, failing case ids. No transcripts, no narration,
  no full logs (truncate to the relevant lines and say what was cut).
- Every claim about behavior carries a command + observed output, not a reading of the code.
- Model/provider/reasoning level: read from your own session metadata; pass the same string
  to every child you spawn.
- Language: artifacts and agent messages in English; product strings stay Vietnamese verbatim.
- Never edit a file outside your role's scope. Never delete a test to make a suite green.

## Release policy (owner, 2026-10-30)

**No version bump and no release until full end-to-end testing is green.** The owner's words:
"Khi nào test đầy đủ tính cả end to end thì hãy bump + release version mới."

`VERSION` / `Cargo.toml` / `tauri.conf.json` / `package.json` stay at the current number through
every intermediate build. The unit/workspace test suites and the engine matrix are necessary but
**not sufficient**: they do not install the app, do not run the hook against a real foreground
window, and cannot hear a sound. The release gate is:

1. Release build + install (or portable drop) on this machine from a clean `target/`.
2. Hook E2E in Notepad, Chrome and one code editor: the golden paragraph corpus, the `/` family
   (`bật/tắt` VNI + Telex), `C:\Users\nam`, backspace/recompute, restore-word, macro, one
   non-Unicode charset; measured against expected text, not "looks right".
3. Toggle E2E on all three routes (hotkey `Alt+Shift`, `F1`/`F2`, UI switch, tray): state flips,
   icon follows, the **`go_moc` pair** is what plays and ON is audibly higher than OFF
   (instrumented probe + owner listens).
4. UI E2E (**no sound row** — scope cut 2026-10-30): `Nâng cao` → `Hệ thống` shows the
   `Âm thanh khi bật/tắt` master switch and nothing else; `Nâng cao` is 560 × 388 with no clipped
   row and no scrollbar (the 424 growth that existed only for the picker reverts); `[system]` in
   `config.toml` has no `sound_pack`; panel still fits 720 × 560 with no scrolling.
5. Budgets P1–P7 re-measured on the release build; P7 installer size included.
6. 30-minute soak typing in 5 apps, no crash (DoD §8.10).
7. Only then: bump the version (Lead), changelog, tag, release artifact.

## Correctness rule (owner, 2026-10-30)

Owner: "Không gõ được tiếng Việt sau / cũng được, nhưng không được gõ sai."

This is the round's hard invariant and it outranks composition quality:

1. **Never emit wrong text.** Text appearing in the app must be either the composed Vietnamese, or the
   literal raw keystrokes. A state like `bật/ttt` is never acceptable; `bật/ta8t1` is.
2. A missing transform (Vietnamese does not compose after `/`, a word stays literal, the raw zone
   over-protects) is a **quality** issue: record it, do not fail a release over it.
3. Any path where the injection layer cannot guarantee what it typed should degrade to raw keystrokes for
   that word instead of a partial delete/insert.
4. Applies to E2E evidence: a row is PASS when output ∈ {composed, raw keystrokes}; it is FAIL only when
   output is neither.

## Escape hatch for a hard defect (owner, 2026-10-30)

Owner on the Notepad corruption (`D-E2E-01`, `bật/tắt` → `bật/ttt` with the default injection path):
"Nếu không xử lí được cho case bật/tắt thì ghi nhận lại rồi bỏ qua đi."

Rule while the fix unit runs:
- The fix must be **provable and low risk**. If the mechanism cannot be explained with evidence, or the
  only fix is a wide rewrite of the injection layer, stop: record `D-E2E-01` as a known issue (exact repro,
  affected apps, the `send_standard = true` workaround, evidence paths), do not ship a speculative change.
- A default change to `send_standard = true` counts as "handled" only with: the mechanism explained, all
  five apps re-tested, P1/P2 re-measured, and a migration note for existing config files.
- Either way the round continues: the item is logged, the release report names it, and no other gate item is
  waived. `D-E2E-02` (restore-word hotkey) has no such waiver — it stays blocking.

## Definition of done for this round
1. One (1) sound asset ships — the `go_moc` on/off pair, embedded, spec-compliant by duration / peak /
   sample-format measurement, played on every toggle route (hotkey, UI switch, tray) with ON ≠ OFF.
   No `sound_pack` config key, no picker, no preview command (scope cut 2026-10-30).
2. `bật/tắt` in VNI produces exactly `bật/tắt` through the engine, with a regression test;
   the documented sibling cases (`/`, `.`, `,`, `-`, `_`, digits, `@`, `:`, brackets,
   apostrophe, reopen-after-separator, macro trigger with punctuation) are each classified
   pass or defect on measured output.
3. No engine purity regression; `cargo test --workspace` green; performance budgets P1
   unaffected (`cargo bench -p vkb-core -- process_key` before/after numbers recorded).
