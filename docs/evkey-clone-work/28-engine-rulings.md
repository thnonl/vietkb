# 28 — Rulings on QA engine defects (Lead, 2026-10-28)

Source: `80-verify-report.md` part 1 and the QA report. 522/551 rows pass; 7 defects open.

## Accepted — engine bugs to fix

| id | sev | defect | required behaviour |
|---|---|---|---|
| D1 | P0 | `đ` acts as a tone target, so the bar and the tone cancel (`ddepj` → `dep`) | `đ` never receives a tone. `ddepj` → `đẹp`. |
| D2 | P1 | a standalone `w` does not produce `ư` | with `jfwz_as_consonant` off and Telex, a lone `w` at a syllable start is `ư`; with the option on, `w` stays a letter. Both pinned by tests. |
| D3 | P1 | repeating the mark/tone key keeps the transform instead of reverting to the raw keys | the second press of the same trigger returns the literal keystrokes (`asss` → `ass`, `dddd` → `ddd`). This is the documented escape rule in `20-spec.md` A1. |
| D3b | P2 | a non-doubling mark key half-transforms (`await` → `ăait`) | if the sequence cannot become a valid transform, the whole word reverts to raw; never a partial transform. |
| D5 | P2 | the spell validator accepts `ngi` + vowel (and similar) | the validator must reject an onset that cannot take a nucleus (`ngi`, `kgh`, …). Invalid syllables pass through as raw text. |
| D7 | P1 | `typing.free_tone` is a dead toggle | add `free_tone: bool` to `EngineOptions`, default `true`, and honour it: when on, the mark may be typed after the vowel run or at the end of the word; when off, the mark attaches to the current vowel position only. |
| D6 | P2 | no English/dictionary layer (`test` → `tét`) | **no dictionary in this cycle** — see ruling below. |

## Rejected / re-scoped

- **D4 `restore_wrong_word` gated on `spell_check` — NOT a defect.** EVKey's own help text says the feature only works together with the spell checker, and the UI already greys the row out with `Cần bật Kiểm tra chính tả`. The 4 corpus rows must assert the dependency instead: with `spell_check = false`, restore does nothing. QA updates `50-test-cases.md`.
- **D6 — no dictionary layer.** Rationale: a Vietnamese/English dictionary costs megabytes and a licence we do not have, and EVKey itself relies on syllable validity, not a lexicon. The fix lands through D5: once the validator is strict, `test` is not a valid Vietnamese syllable, so it passes through untouched. If a user explicitly wants a word transformed, the spell-check toggle is the escape hatch.
  - Acceptance for D6: with defaults, `test`, `chrome`, `github`, `select` pass through unchanged; with `spell_check` off, Vietnamese transforms apply as before.

## Contract changes approved here

- `EngineOptions::free_tone: bool` (default `true`), mapped from `TypingConfig::free_tone`.
- `parse_hotkey` grammar eventually needs side-specific modifiers (`Ctrl phải`) — **deferred**, the preset chip for `Ctrl phải` stays dropped until then.

## Also to fix (docs, Lead)

`25-contract.md` §3 still carries the pre-correction tone-style examples. Correct them to
`modern_tone = true → hòa`, `false → hoà`.

## Binary-confirmed rules (REA + Ghidra 12.1.4, 2026-10-29)

Target: `EVKey64.exe`, sha256 `69274b44…`. Full decompile at `reference/evkey-decomp/`
(2,300 functions, one `.c` per address). Nothing below is inferred from black-box probing —
EVKey ignores `SendInput`-injected keys (`LLKHF_INJECTED`), so probing is worthless here.

Evidence chain:
- `0x14002fcf0`:17 — `SetWindowsHookExW(0xd, FUN_14002dc90, …)` → LL hook proc is `FUN_14002dc90`.
- `0x14002dc90` builds a key-event object (`FUN_14002d640`) and calls `FUN_14000a590` = per-key dispatcher.
- `0x14000a590`: vk `0x08` → `FUN_140036830` → `FUN_14003cb50` (engine backspace);
  every other key → `FUN_1400367e0` → `FUN_14003c8a0` (engine process-char).
- Engine object base `DAT_1400ac490`: letter array at `+0xc50`, last-letter index `+0x1c`
  (`-1` = empty), delete count `+0x14`. Outputs: delete count `DAT_1400ac468`,
  text `DAT_1400ab460`, text byte length `DAT_1400ac46c`.

**B1 — Backspace deletes exactly one displayed character.** `FUN_14003cb50` pops exactly one
letter entry (`[+0x1c] -= 1` after re-composing that letter via `FUN_14003ba90`), writes
`[+0x14]` to the delete-count output, and returns `false` when `[+0x1c] < 0`
(`0x14003cb50`:43-79). One tone-marked letter is one letter entry, so BS never leaves a bare
base vowel behind: `tiếng` → `tiến` → `tiế` → `ti`.

**B2 — EVKey passes the key through when there is nothing to rewrite.** `0x14000a590`:40-48:
it swallows and injects (8 × `delete count`, then the text buffer) only when
`delete count > 1` or the text buffer is non-empty; otherwise it returns 0 and lets the
physical Backspace reach the app. So the app's own Backspace semantics apply, and our
`Replace { delete_units }` with `delete_units = 1` matches for Unicode output.

**B3 — A word-boundary key resets the word.** `FUN_140039740` (word-break class) sets
`[+0x1c] = -1`, `[+0x185] = -1`, and — unless the char is CR `0xd` — clears `+0xc44` and
`[+0x189]` (`0x140039740`:87-94). A Backspace arriving after a word boundary therefore hits
the `[+0x1c] < 0` early-return in `FUN_14003cb50`.

Charset branches still present in the engine (`[+8]+4`, `[+8]+0x14` option fields) are not
modelled: we ship Unicode only.

**B4 — EVKey does *not* reopen the word after a space.** Two independent traces of the
decompiled tree agree: after a word-break key the engine holds `[+0x1c] = -1`
(`0x140039740`:84-105), the next Backspace hits the early return in `FUN_14003cb50`
(`0x14003cb50`:28-33) and the dispatcher passes the key through
(`0x14000a590`:33-46, 70-89). Nothing in the EVKey layer reads the focused control's text
back to rebuild the word: the only `GetGUIThreadInfo` use (`0x1400154f0`:21-39) resolves a
window handle and never retrieves text; `+0xc48` is a remembered character code used for
punctuation/space classification (`0x14003b5b0`:51-76), not a word cache; and
`DAT_1400ac468` / `DAT_1400ac46c` / `DAT_1400ab460` are pending delete/output bookkeeping
for the current keystroke, cleared by the next engine call.

Consequence: `Tha` → space → BS → `s` produces **`Thas`** in EVKey, not `Thá`. The reopen
behaviour in `vkb-core` (`CommittedWord`, space only) is therefore a **VietKB extra**, not a
clone requirement — see D8.

## D8 — Reopen after space: keep or drop

Our engine reopens the just-committed word when the boundary space is deleted and a letter
follows (`tiếng…` spec case: `Tha` + space + BS + `s` → `Thá`). EVKey does not do this (B4).
Options: keep as an intentional VietKB extra (documented, differs from EVKey), or drop it for
byte-for-byte fidelity. Owner decision pending; until then the implemented behaviour and the
`editing.rs` cases that assert it stay as they are.

## Accepted nuance (QA, 2026-10-28)

`COMPAT_RESTORE_RAW` — the fallback that emits raw keystrokes when a transform cannot apply
(`of`, `aww`, …) — runs **before** the spell-check gate, so it restores even with
`spell_check = false`. That is correct and is not the `restore_wrong_word` feature: it is the
"no valid transform, so type what was typed" path, which must never depend on the spell
checker. `E-RW-005` / `E-RW-007` therefore stay as they are.

