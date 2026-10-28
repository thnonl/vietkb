# 80 — Verification report, part 1: engine

**Date:** 2026-10-28 · **Owner:** QA · **Scope:** `50-test-cases.md` §1 (555 engine rows)
and §2 (charset round-trips), headless only. **GUI half (§3, §2.10 `convert_text`,
tray/hook behaviour) is not covered here** — it waits for the UI rebuild.
**No product code was modified.** Only test code + this report + the corpus corrections
listed in §3 below (plus the D4 rows rewritten after the Lead ruling — §2 and §4).

## 1. How the check runs

New integration test: `crates/vkb-core/tests/spec_51_engine.rs` (7 tests).

It parses §1 out of `docs/evkey-clone-work/50-test-cases.md` **at run time**, so the markdown
table is the single source of truth — a corrected row is picked up without touching Rust.
Options come from the §0 baseline (`charset=unicode`, `modern_tone=true`, `free_tone=true`,
`jfwz_as_consonant=false`, `spell_check=false`, `restore_wrong_word=false`,
`macro_enabled=false`, `auto_capitalize=false`) with the four category deviations
(`spell-check`/`english-passthrough` → spell on, `macro` → macro on + the 7-row §1.14 table,
`restore-word` → restore on, `tone-old-style` → `modern_tone=false`) and the six per-row
overrides stated in the rows themselves (`E-TT-061` jfwz on, `E-MC-016` macro off,
`E-RW-013…016` spell check on — D4).
`<sp> <bs> <ent> <tab> <esc> <bt> <empty>` are expanded into real `KeyInput`s and the produced
text is accumulated with the same `apply_action` a backend would use.

```powershell
cargo test -p vkb-core --test spec_51_engine -- --nocapture        # report, never red on mismatch
$env:VKB_SPEC_STRICT=1; cargo test -p vkb-core --test spec_51_engine   # mismatch = test failure
```

The test always asserts harness integrity (555 rows parsed, unique ids, per-category counts
equal to the `### … — N rows` headings) and writes `target/qa/spec51-results.tsv` with one
line per row. Behavioural mismatches are printed per row and only fail the run in strict mode.

**Tone-style correction:** §1 already agrees with the settled rule (`modern_tone=true` →
first vowel of an open `oa`/`oe`/`uy`: `hòa`, `hóa`, `thúy`, `thủy`; old style → second vowel).
**0 rows had to be changed for it.** The inverted statement is in a *contract comment*, not in
the corpus — see §7.

## 2. Result

**555 rows · 555 pass · 0 fail** — green in strict mode too
(`$env:VKB_SPEC_STRICT=1; cargo test -p vkb-core --test spec_51_engine` → 7 tests pass, exit 0).
History: 551 · 522 · 29 at the first run (7 engine defects), 551 · 547 · 4 after the engine fixes
(only the four D4 rows), 555 · 555 · 0 now that D4 is re-scoped as a corpus error.

| category | total | pass | fail |
|---|---:|---:|---:|
| telex-basic | 60 | 60 | 0 |
| telex-tone | 62 | 62 | 0 |
| telex-escape | 18 | 18 | 0 |
| telex-w | 20 | 20 | 0 |
| vni | 60 | 60 | 0 |
| viqr | 40 | 40 | 0 |
| tone-placement | 59 | 59 | 0 |
| tone-old-style | 24 | 24 | 0 |
| modified-letter | 28 | 28 | 0 |
| qu-gi | 21 | 21 | 0 |
| uppercase | 26 | 26 | 0 |
| backspace | 22 | 22 | 0 |
| boundary-commit | 20 | 20 | 0 |
| macro | 20 | 20 | 0 |
| spell-check | 15 | 15 | 0 |
| restore-word | 16 | 16 | 0 |
| english-passthrough | 26 | 26 | 0 |
| isolated | 18 | 18 | 0 |
| **TOTAL** | **555** | **555** | **0** |

No unexplained rows.

**D4 corpus rewrite (Lead ruling `28-engine-rulings.md` §Rejected).** `restore_wrong_word` is not an
independent switch: it restores only together with `spell_check` (the UI greys the row out with
`Cần bật Kiểm tra chính tả`). The four rows that asserted "restore with spell check off" were
therefore wrong corpus rows, not engine bugs. They keep their keystrokes and now assert the
dependency at `spell_check=false` (the transform stays), and four new rows `E-RW-013…016` assert the
same keystrokes at `spell_check=true` where the raw word must come back — all four pass:

| row | keystrokes | `spell_check=false` (rewritten) | `spell_check=true` (new) |
|---|---|---|---|
| 002 / 013 | `asdf<sp>` | `àd<sp>` | `asdf<sp>` |
| 004 / 014 | `nghieengxq<sp>` | `nghiễngq<sp>` | `nghieengxq<sp>` |
| 006 / 015 | `vieetjq<sp>` | `việtq<sp>` | `vieetjq<sp>` |
| 012 / 016 | `tieengsq<sp>` | `tiếngq<sp>` | `tieengsq<sp>` |

Every `spell_check=false` expectation above was taken from a run, not assumed. One nuance the code
keeps: `should_restore` (`engine.rs:264-273`) short-circuits on the fixed compat set
`COMPAT_RESTORE_RAW` (`of`, `aww`, `oww`, `uww`, …) *before* the `spell_check` test, so
`E-RW-005/007` still restore with spell check off; only the spell-validity path is gated.

## 3. Corpus bugs corrected (11 §1 rows, 2 §2 rows, 1 §2 string)

Each was a factual error in the row, not an engine question: the keystrokes or the expected
string could not produce each other under *any* implementation.

| row | was | now | why |
|---|---|---|---|
| E-TB-040 | input `lasm` | input `lafm` | `s` = sắc → `lám`; the row's own intent is huyền |
| E-TB-054 | input `quocs` | input `quoocs` | `ô` needs the doubled `oo` |
| E-TP-014 | input `tuyets` | input `tuyeets` | `ê` needs `ee` |
| E-TP-056 | input `khuays` | input `khuaays` | `â` needs `aa` |
| E-TP-057 | input `quays` | input `quaays` | `â` needs `aa` |
| E-QG-012 | input `quyens` | input `quyeens` | `ê` needs `ee` |
| E-SC-008 | input `trannj` | input `traanj` | `trận` has `â`; `nn` taps nothing |
| E-UC-015 | expected `ĐƯỜNG` | expected `ĐƯƠNG` | input `DDUWOWNG` carries no tone key |
| E-BS-007 | expected `đươ` | expected `đươn` | two backspaces from `dduwowngf` leave `dduwown` |
| E-BS-019 | expected `nghieng` | expected `nghiêng` | removing the VNI `2` keeps `ê` (the row says so) |
| E-MC-015 | expected `vn<sp>` | expected `v<sp>` | backspace removes the `n` from the screen too |
| CS-VQ-07 / CS-VQ-14 (§2.4) | `đ`=`d-`, `Đ`=`D-` | `dd`, `DD` | RFC 1456 Table 2: *d bar = `dd` (repeated d)* |
| §2.4 S1 string | `D-u+o+<bt>ng … ho+u+o+ng` | `DDu+o+<bt>ng … hu+o+ng` | wrong `đ` spelling **and** `hương` is `h u+ o+`, not `h o+ u+ o+` |
| §2.7 + matrix | VISCII *all 14 UNVERIFIED* | letters verified vs RFC 1456 Table 1 | QA diffed them; they match (§5) |

Also updated the `why` cells of the rows whose keystrokes changed so the reason matches the row.
`d-`/`D-` remain valid VIQR *input* (the engine accepts both), which is why §1.6/§1.12 still type
them; only the *output* charset table was wrong.

## 4. Engine defect log — all closed

Ordering by the severity recorded at report time. Each defect now has a named test pinning the
settled behaviour, and every §1 row it explained passes. `cargo test -p vkb-core --test
engine_rulings_28 --test spell_restore` → 7 + 8 tests pass. D4 was re-scoped as a corpus error by
the Lead (`28-engine-rulings.md` §Rejected) and has no engine change.

### D1 — `đ` is treated as a tone target; the tone and the bar vanish (P0, 5 rows) — **CLOSED**

Pinned by `engine_rulings_28::d1_d_bar_never_takes_the_tone`; `E-TB-039`, `E-TP-049`, `E-ML-014`,
`E-VN-046`, `E-VN-057` pass.

`E-TB-039 ddepj → dep`, `E-TP-049 ddoanf → doan`, `E-ML-014 ddos → do`,
`E-VN-046 d9a4 → da`, `E-VN-057 d9ung1 → dung` (expected `đẹp / đoàn / đó / đã / đúng`).

`vkb-core/src/letters.rs:206` maps `('d', Mark::Bar) → Some(12)` and `is_vowel()`
(`letters.rs:121`) returns `true` for it. `tone::target` (`tone.rs:51-58`) therefore sees `đ` as a
*marked vowel* and, by the "last modified vowel wins" rule, hands it the tone; `compose`
(`letters.rs:216`) correctly refuses `đ` + tone and the renderer falls back to the bare `d`
(`engine.rs:123`). Effect: any word where `đ` is the last marked letter loses **both** the bar
and the tone — i.e. most `đ` + plain-vowel words. `đường` survives only because `ơ` is marked
later. Fix: exclude `D_KIND` from `is_vowel()` (or from `tone::target`'s candidate list).

### D2 — Telex `w` never produces `ư` without a preceding `a/o/u` (P1, 4 rows) — **CLOSED**

Pinned by `engine_rulings_28::d2_standalone_w_is_u_horn_unless_jfwz_are_consonants` (both option
states); `E-TW-001/002/016/018` pass.

`E-TW-001 w → w`, `E-TW-016 w<sp> → w<sp>`, `E-TW-018 tw → tw`, `E-TW-002 ww → ww`
(expected `ư`, `ư<sp>`, `tư`, `w`).

`method.rs:250-272`: `apply_w` only *marks* an existing target; with none, `w_target()` returns
`None` and the key is emitted literally. Telex defines standalone `w` as `ư` whenever
`jfwz_as_consonant=false` (the §0 setting for every §1 row). Fix: when no target exists and
`jfwz_as_consonant` is off, push `ư` (`Letter::new('u', Mark::Horn)`); when it is on, keep
today's literal `w` (that is what `E-TT-061`/default-config rows expect).

### D3 — repeating a mark/tone trigger keeps the transform instead of reverting to raw (P1, 12 rows) — **CLOSED**

Pinned by `engine_rulings_28::d3_repeated_trigger_escapes_to_the_raw_keys` and
`corpus_telex::repeated_mark_key_escapes_to_the_literal_keystrokes`.

`E-TE-007 aaa → âa` · `008 eee → êe` · `009 ooo → ôo` · `010 ddd → đd` · `012 aass → âs` ·
`014 thuowngff → thươngf` · `018 a66 → â6` · `E-TW-007 aww → ăw` · `008 oww → ơw` ·
`009 uww → ưw` · `E-VQ-037 a^^ → â^` · `E-VQ-038 d-- → đ-`
(expected `aa, ee, oo, dd, aas, thuowngf, a6, aw, ow, uw, a^, d-`).

`method.rs` (`apply_tone`, `apply_w`, `mark_last`) keeps the letters already transformed, clears
only the tone, sets `escaped`, and appends the trigger literally. §1.3's rule is different and
self-consistent across all three methods: **when the key that produced the current tone/mark is
typed again, the word is shown as the raw keystrokes without that key** (`aaa`→`aa`,
`aass`→`aas`, `thuowngff`→`thuowngf`, `a66`→`a6`, `d--`→`d-`). Without it the user can never
type a literal `aa`, `dd`, `aw`, `a^`…, which is the whole point of the escape. `E-TE-007..016`
and `E-TE-001..006` are mutually consistent under the spec rule, so this is one defect, not seven.

### D3b — a mark key that cannot double leaves the word half-transformed (P2, 1 row) — **CLOSED**

Pinned by `engine_rulings_28::d3b_failed_mark_key_never_half_transforms`; `E-EN-012 await` passes.

`E-EN-012 await → ăait` (expected `await`).

`method.rs:222-227`: when `a` follows `ă` it cannot double, so the code sets `escaped` and pushes a
literal `a` **on top of the transformed prefix**. The spec's escape model would revert the word to
its raw keystrokes and keep it literal; with `spell_check=true` the raw word then also survives
the boundary. Same code path as D3, different trigger condition (non-doubling key rather than a
repeat), hence a separate entry.

### D4 — `restore_wrong_word` gated behind `spell_check` — **REJECTED, not a defect** (was P1, 4 rows)

Lead ruling (`28-engine-rulings.md` §Rejected): EVKey's help text says the feature only works
together with the spell checker, and the UI greys the row out (`Cần bật Kiểm tra chính tả`). The
four rows asserted a behaviour the product deliberately does not have, so QA fixed the corpus, not
the engine — the gate in `engine.rs:271-272` stays.

* `E-RW-002/004/006/012` now run `spell_check=false` and expect the transform that stays on screen
  (`àd / nghiễngq / việtq / tiếngq`), each value taken from a run.
* `E-RW-013/014/015/016` (new, same order) run `spell_check=true` + `restore_wrong_word=true` on the
  same keystrokes and expect the raw word back (`asdf / nghieengxq / vieetjq / tieengsq`) — **all
  four pass**, so the restore path itself is sound and no new defect was opened.
* Nuance kept by the code: `should_restore` (`engine.rs:264-273`) short-circuits on the fixed
  `COMPAT_RESTORE_RAW` set (`of`, `aww`, `oww`, `uww`, …) *before* the `spell_check` test, which is
  why `E-RW-005/007` still restore with spell check off; only the spell-validity path is gated.

Pinned by the corpus rows themselves; `spell_restore.rs` still covers the option pair
(`restore_wrong_word_rewrites_the_word_at_the_boundary`,
`restore_wrong_word_off_keeps_the_transformation`).

### D5 — spell validator accepts `ngi` + vowel (P2, 1 row) — **CLOSED**

Pinned by `engine_rulings_28::d5_impossible_onsets_are_invalid`; `E-SC-015 ngieengs` passes.

`spell.rs:66-72` used to try onset `ng` and then parse `iê` as the nucleus, missing the rule
"`g`/`ng` before `i e ê` must be `gh`/`ngh`". The validator now rejects onset/nucleus pairs that
cannot combine (`ngi`, `kgh`, …).

### D6 — spell check cannot reject English words that are valid syllables (P2, 2 rows) — **CLOSED**

Pinned by `engine_rulings_28::d6_english_passthrough_with_defaults`; `E-SC-011 test`,
`E-EN-003 test` pass.

Lead ruling: **no dictionary layer this cycle** (`28-engine-rulings.md`). `tét` is a well-formed
syllable, so the fix came through D5 — with the stricter validator the default configuration leaves
`test`, `chrome`, `github`, `select` untouched, and turning spell check off restores the old
transforming behaviour.

### D7 — the `Đặt dấu tự do` (`free_tone`) toggle is dead (P1, not covered by §1) — **CLOSED**

Pinned by `engine_rulings_28::d7_free_tone_is_honoured`.

`EngineOptions::free_tone: bool` now exists (default `true`, contract §3 change approved in
`28-engine-rulings.md`) and is mapped from `TypingConfig::free_tone`; the engine honours it instead
of always behaving as `true`.

## 5. §2 charset round-trips

What the engine alone can settle — `Charset::encode` exists, **there is no `decode` in
`vkb-core`**, so R1 (`decode(encode(x))`) and R2 (`encode(decode(b))`) can only be exercised
through `vkb-app`'s convert tool (GUI half).

| charset | letters vs §2 table | encode(S1) | 1 byte/char | status |
|---|---|---|---|---|
| `unicode` | 14/14 codepoints match (§2.1, S1 code points too) | 18 chars → 18 | n/a (UTF-16 in memory) | pass |
| `unicode_nfd` | 14/14 base+mark match; `đ` stays U+0111; mark before tone (`ế` = `65 302 301`, `ờ` = `6F 31B 300`) | 18 → 27 chars | n/a | pass |
| `utf8` | 14/14 byte sequences match (§2.3) | `S1.as_bytes()` | n/a (2–3 bytes) | pass |
| `viqr` | 14/14 after the `đ`=`dd` correction; `Ắ`=`A('` etc. | `DDu+o+`ng ve^` que^ hu+o+ng` | yes (ASCII) | pass |
| `tcvn3` | matches §2.5's "believed" bytes exactly (A8 A9 AA AB AC AD AE / A1…A7) | `A7 AD EA 6E 67 20 76 D2 20 71 75 AA 20 68 AD AC 6E 67` | yes for S1 | **UNVERIFIED** — no authoritative source in repo |
| `vni_windows` | matches no external table QA could check | `D1 F6 F4 F8 6E 67 20 76 65 E0 20 71 75 65 E2 20 68 F6 F4 6E 67` (21 B) | no (2 B/letter) | **UNVERIFIED** |
| `viscii` | **14/14 + ờ/ề match RFC 1456 Table 1** | `D0 DF B6 6E 67 20 76 AB 20 71 75 EA 20 68 DF BD 6E 67` | yes | letters **VERIFIED**, full table not |
| `bk_hcm1` | *is* the VNI Windows table (documented assumption) | identical to `vni_windows` | no | **UNVERIFIED** |
| `vps` | no authoritative source in repo | `F1 DC A9 6E 67 20 76 8A 20 71 75 EA 20 68 DC D6 6E 67` | yes | **UNVERIFIED** |

R3 (no silent loss): `Charset::encode` substitutes `?` for an unmapped non-ASCII letter. For all
nine charsets on S1 and on the 14 letters there is **no `?`** — no loss. But note the *engine
output* path (`charset::push_encoded`) passes an unmapped character through unchanged instead of
using `?`, so `EngineAction` text and `encode()` can disagree for a letter a legacy table lacks;
harmless today (all nine tables cover the 73+73 letters) but it will bite the first partial table.

Bookkeeping conflict: the engine labels `tcvn3`, `vni_windows`, `vps` as `TableStatus::Documented`
while §2 calls them **UNVERIFIED**; `bk_hcm1` is `Unverified` in both. One authority has to win.

## 6. Performance (headless, P1 = p99 ≤ 50 µs)

Not run under the §4.0 isolation protocol (no Lead gate announcement; other sessions may have been
active) — **indicative numbers, not the gate measurement.**

`cargo run -p vkb-cli --release -- bench --keys 100000`, three consecutive runs:

| run | p50 | p95 | p99 | max | throughput | P1 |
|---|---:|---:|---:|---:|---:|---|
| 1 | 0.20 µs | 0.70 µs | **0.80 µs** | 73.9 µs | 3 527 461 keys/s | PASS |
| 2 | 0.20 µs | 0.70 µs | **0.80 µs** | 160.1 µs | 3 447 824 keys/s | PASS |
| 3 | 0.20 µs | 0.70 µs | **0.80 µs** | 114.0 µs | 3 430 108 keys/s | PASS |

p99 is **62× under the 50 µs budget**; even `max` (a preemption outlier, not engine work) stays
within 1–3× of the budget. `p50 = 200 ns` is dominated by `Noop` appends; `p95 = p99 = 700/800 ns`
are the rewrite keys (`Replace`).

`cargo bench -p vkb-core --bench engine` (criterion 0.5, mean per *iteration*; per-key in parens):

| bench | keys/iter | mean | ≈ per key |
|---|---:|---:|---:|
| `telex_word` (`tieengs`) | 7 | 2.735 µs | 391 ns |
| `telex_long_word` (`nghieengx`) | 9 | 1.755 µs | 195 ns |
| `telex_rewrite_every_key` | 8 | 0.612 µs | 76 ns |
| `vni_word` (`tie6ng1`) | 7 | 2.695 µs | 385 ns |
| `noop_chord` (Ctrl+S) | 1 | **0.00095 µs** (0.9 ns) | 0.9 ns |
| `boundary_commit` | 8 | 2.618 µs | 327 ns |

P1 holds with a wide margin; the `Noop` path is allocation-free and ~1 ns as the contract demands.

## 7. Doc conflicts noticed (not changed here)

* `25-contract.md:76` — `modern_tone` doc says "true = kiểu mới (`hoà`, `thuý`)" i.e. the
  *examples are swapped*; line 150 of the same file and `vkb-core`'s `EngineOptions` doc have it
  right (`true` = `hòa`, `thúy`). Contract change, needs Lead approval.
* `crates/vkb-core/tests/corpus_telex.rs:113` — test named
  `modern_tone_places_the_tone_on_the_second_vowel` actually asserts the **old** style. Name only;
  behaviour is correct.

## 8. Still unverified

1. **§3 (UI/UX, 48 rows)** and the tray/hook end-to-end behaviour — GUI pass.
2. **`convert_text` / R1 / R2 for all nine charsets** — no decoder in `vkb-core`; needs the app's
   convert tool (§2.10 matrix).
3. **`tcvn3`, `vni_windows`, `bk_hcm1`, `vps` byte tables** — no authoritative source compared;
   `viscii` is verified for the 14 letters + `ờ`/`ề` only.
4. **Real EVKey oracle.** D3/D3b now encode the settled escape rule and are pinned by tests, but no
   one has confirmed the rule against `EVKey64.exe`; until then it is "engine vs spec", not
   "engine vs EVKey".
5. **`free_tone=false` beyond the engine** — `engine_rulings_28::d7_free_tone_is_honoured` covers the
   option in `vkb-core`; the UI path from the checkbox to `TypingConfig` is still GUI half.
6. **Method-specific performance** for VIQR (bench covers Telex + VNI only).
