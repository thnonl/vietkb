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
