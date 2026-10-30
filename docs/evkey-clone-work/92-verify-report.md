# 92 — QA re-verification of D-SC-01…05 (stream B, second pass)

**Date:** 2026-10-30 · **Owner:** QA · **Repo:** `E:\vietkb`, branch `main` (HEAD `14bc655`
+ uncommitted A9/chained-opener change) · **Method:** every number below is a fresh
measurement of the real engine; none of it is taken from the Dev fix note.

**Scope:** the five engine defect rows in `60-defects.md` (§ "Engine special cases, round
2026-10-30"), the 78-case matrix in `51-special-cases.md`, the `20-spec.md` §A9 amendment
(2026-10-30, incl. **§2b chained openers**), and the guard list of `00-charter.md` §9 as
extended by the amendment's §5/§8.

**Verdict: 5 / 5 defects FIXED + VERIFIED. 0 regressions in the pinned set. 2 new
under-protection shapes found (D-SC-06, D-SC-07, P3, outside the amendment's declared
losses).** `cargo test --workspace` is **blocked by an unrelated, uncommitted change in
`vkb-windows/src/sound.rs`** (§5.3) — the engine crates are green.

> **Update 2026-10-30 (third pass, §9):** the `sound.rs` blocker is gone (workspace exit 0), the
> two raw-zone state leaks found after this report are verified fixed inside `Engine`, and this
> report's own `github . LEFT cof` expectation was corrected to `github.cò` — see §9.

| what | result |
|---|---|
| 78-case matrix (`matrix()`) | **78 PASS / 0 FAIL** (exit 0) |
| `vkb-cli` reproduction table | **73 / 73** engine rows as expected (1 row needed `--` for `--opt`; CLI arg parsing, not the engine) |
| QA harness (`tests/qa_92_verify.rs`, new) | **5 tests / 0 failed**, 16 new-shape rows PASS |
| `cargo test -p vkb-core` | exit 0 — **207 passed / 0 failed / 0 ignored** |
| `cargo test -p vkb-core --test no_alloc` | exit 0 — 4 passed |
| `cargo test --workspace` | **exit 101 — unrelated compile error** in `vkb-windows/src/sound.rs:207` (`E0015`); workspace minus `vkb-windows`/`vkb-app` = exit 0 |
| P1 bench `process_key` | **p99 0.8 µs** (budget 50 µs), p50 0.2 µs, max 33.5 µs, 3.18 M keys/s → PASS |

No product code was edited by this pass. Files added: `crates/vkb-core/tests/qa_92_verify.rs`
(test only, 5 tests) and `docs/evkey-clone-work/qa/verify/` (evidence + the CLI driver).

---

## 1. Defect verdicts

Each row was re-run from the exact original repro, by command line and (where the CLI cannot
express the keys: numpad, Backspace, arrow) through the engine harness.

### D-SC-01 — `/` between two Vietnamese words — **FIXED + VERIFIED (2026-10-30)**

```
$ cargo run -q -p vkb-cli -- simulate --method vni   "ba65t/ta8t1"
bật/tắt
$ cargo run -q -p vkb-cli -- simulate --method telex "baajt/tawts"
bật/tắt
```

Siblings, measured (all `exit 0`):

| keys | result | required |
|---|---|---|
| `/ta8t1` | `/tắt` | `/tắt` |
| `ba65t//ta8t1` | `bật//tắt` | `bật//tắt` |
| `ba65t /ta8t1` | `bật /tắt` | `bật /tắt` |
| `ba65t/ta8t1/ba65t` | `bật/tắt/bật` | `bật/tắt/bật` |
| `Ba65t/Ta8t1` | `Bật/Tắt` | `Bật/Tắt` |
| numpad `ba65t/ta8t1` (VK_NUMPAD0+) | `bật/tắt` | `bật/tắt` |
| `ba65t/a8` `a61` `d9` `o7` `u7` | `bật/ă` `bật/ấ` `bật/đ` `bật/ơ` `bật/ư` | same |

**Audit-repro correction (now measured):** the row's Telex repro `baajt/taats` spells
`tất`, not `tắt` — `baajt/taats` → `bật/tất` (correct Telex). The `tắt` repro is
`baajt/tawts` → `bật/tắt`, which is what SC-29 now pins. `60-defects.md` is updated.

### D-SC-02 — `.` / `?` as a sentence end — **FIXED + VERIFIED (2026-10-30)**

```
$ cargo run -q -p vkb-cli -- simulate --method vni "ba65t.ta8t1"   ->  bật.tắt
$ cargo run -q -p vkb-cli -- simulate --method vni "ba65t?ta8t1"   ->  bật?tắt
```

### D-SC-03 — `_`, `@`, `:` — **FIXED + VERIFIED (2026-10-30)**

```
ba65t_ta8t1 -> bật_tắt      ba65t@ta8t1 -> bật@tắt      ba65t:ta8t1 -> bật:tắt
```

### D-SC-04 — `=`, `#`, `\` — **FIXED + VERIFIED (2026-10-30)**

```
ba65t=ta8t1 -> bật=tắt      ba65t#ta8t1 -> bật#tắt      ba65t\ta8t1 -> bật\tắt
```

### D-SC-05 — chained opener after `:` (`C:\Users\nam` in Telex) — **FIXED + VERIFIED (2026-10-30)**

```
$ cargo run -q -p vkb-cli -- simulate --method telex "C:\Users\nam"     ->  C:\Users\nam
$ cargo run -q -p vkb-cli -- simulate --method vni   "C:\Users\nam"     ->  C:\Users\nam
```
The filed "actual" was `C:\Úe\nam` (Telex). Both methods now hold the path, and VNI holds
it for the pre-existing reason (no VNI trigger inside `Users`).

---

## 2. §2b chained openers

Commands are `cargo run -q -p vkb-cli -- simulate --method <m> "<keys>"`; both methods were
run for every row and produced identical text unless noted.

| keys | measured (telex) | measured (vni) | verdict |
|---|---|---|---|
| `C:\Users\nam` | `C:\Users\nam` | `C:\Users\nam` | PASS |
| `C:/Users/nam` | `C:/Users/nam` | `C:/Users/nam` | PASS |
| `C:\\server\share` | `C:\\server\share` | `C:\\server\share` | PASS |
| `https://x/y/z` | `https://x/y/z` | `https://x/y/z` | PASS |
| `http://a.b/c` | `http://a.b/c` | `http://a.b/c` | PASS |

Negative controls (must **not** chain — the `/` after `.`/`_` sees an unarmed predecessor):

```
xyz./tieesng      -> xyz./tiếng          (expected xyz./tiếng)
baajt:_/tieesng   -> bật:_/tiếng         (expected bật:_/tiếng)
```

Still composing (must not regress):

```
baajt/taawts -> bật/tắt      baajt//taawts -> bật//tắt      baajt /taawts -> bật /tắt
/taawts      -> /tắt         a/b           -> a/b           2/3            -> 2/3
baajt.http   -> bật.http     baajt./taawts -> bật./tắt      baajt:_/taawts -> bật:_/tắt
```

`bật` + `:` + `/` does **not** chain (`ba65t:/ta8t1` → `bật:/tắt`): the left token before the
`:` is non-ASCII, so the first opener never armed — the §2b chain cannot reach a Vietnamese
pair, as the spec claims.

---

## 3. Guard list (`00-charter.md` §9, as extended by the amendment)

| keys | measured | verdict |
|---|---|---|
| `github.cof` | `github.cof` | PASS |
| `a@b.com` | `a@b.com` | PASS |
| `file_name` | `file_name` | PASS |
| `x=cof` | `x=cof` | PASS |
| `me@cof` | `me@cof` | PASS |
| `a.` | `a.` | PASS |
| `a. tieengs` | `a. tiếng` | PASS |
| `2/3` | `2/3` | PASS |
| `1.5`, `3.5` | `1.5`, `3.5` | PASS |
| `%20` | `%20` | PASS |
| `--opt` | `--opt` (needs `--` for cargo: `simulate --method vni -- --opt`) | PASS |
| `...` | `...` | PASS |
| `a...cof` | `a...cò` (ellipsis never arms — pinned behaviour) | PASS |
| `3.cof` | `3.cò` (digit-dot never arms — pinned) | PASS |
| `Tha` `/` `BS` `s` | `Thá` | PASS (`run_screen`) |
| `ko/` (macro `ko→không`) | `không/` | PASS |
| `ba65t/ko/` | `bật/không/` | PASS |
| `macro_only` `ko/` | `không/` | PASS |
| `tiengs` | `tíeng` — **pre-existing, not a regression** (see §7.6) | note |

`tests/raw_zone.rs` (all 5 tests) and `special_cases.rs::url_email_and_code_shapes_stay_literal`
are unchanged and green; `special_cases.rs --include-ignored` reports **11 passed / 0 ignored**,
i.e. no defect repro is left `#[ignore]`d.

---

## 4. How the numbers were produced

| evidence | command |
|---|---|
| `qa/verify/matrix.txt` | `cargo test -p vkb-core --test special_cases matrix -- --nocapture` |
| `qa/verify/cli.txt` | `powershell -File docs/evkey-clone-work/qa/verify/run-cli.ps1` (74 real `vkb-cli simulate` runs, command line + stdout + exit code per row) |
| `qa/verify/harness.txt` | `cargo test -p vkb-core --test qa_92_verify -- --nocapture --test-threads=1` |
| `qa/verify/test-core.txt` | `cargo test -p vkb-core` |
| `qa/verify/no_alloc.txt` | `cargo test -p vkb-core --test no_alloc` |
| `qa/verify/test-workspace.txt` | `cargo test --workspace` (blocked, §5.3) |
| `qa/verify/test-workspace-partial.txt` | `cargo test --workspace --exclude vkb-windows --exclude vkb-app` |
| `qa/verify/bench.txt` | `cargo run -q --release -p vkb-cli -- bench` |

The new harness file `crates/vkb-core/tests/qa_92_verify.rs` covers the three things the CLI
cannot express — numpad `VK_NUMPAD0+`, `Backspace`, caret-mover VKs — plus an arm/no-arm probe
matrix. It asserts only documented outcomes; the probe table is printed.

---

## 5. Suite + performance

### 5.1 `cargo test -p vkb-core` — exit 0
207 passed / 0 failed / 0 ignored (19 test binaries; was 202 before this pass — the 5 new
`qa_92_verify` tests are the delta).

### 5.2 `no_alloc` — exit 0
`test result: ok. 4 passed; 0 failed` — the arming path still allocates nothing
(`composed_plain_ascii` walks the existing letter buffer).

### 5.3 `cargo test --workspace` — **exit 101, external blocker**

```
error[E0015]: cannot call non-const method `SoundPack::index` in statics
   --> crates\vkb-windows\src\sound.rs:207:65
207 | static CURRENT_PACK: AtomicU8 = AtomicU8::new(SoundPack::System.index() as u8);
error: could not compile `vkb-windows` (lib) due to 1 previous error
```

`crates/vkb-windows/src/sound.rs` is **uncommitted (+575 lines, sound-pack work, not the
engine fix)**; it does not compile on the current toolchain. Nothing in `vkb-core` is
involved. `cargo test --workspace --exclude vkb-windows --exclude vkb-app` → **exit 0, all
green**. Recorded so the round's workspace gate is not silently claimed.

### 5.4 P1 bench — PASS

```
keys: 100000   method: telex
p50: 200 ns   p95: 700 ns   p99: 800 ns (budget 50000 ns)   max: 33500 ns
throughput: 3182271 keys/sec
P1 (p99 ≤ 50 µs): PASS
```

p99 0.8 µs against a 50 µs budget — the predicate adds no measurable cost (Dev measured
0.8–1.2 µs before/after; this run is inside that band).

---

## 6. New failures found while hunting

Both are **under-protection** (the fix narrows arming more than the amendment declares) and
both are **new** relative to pre-fix behaviour, where any opener with a non-empty word armed
(and even an empty word armed — that is exactly the pre-fix `SC-31 /ta8t1` failure). Severity
P3 each; filed as `D-SC-06`/`D-SC-07` in `60-defects.md` for the Lead's ruling, **not**
blocking D-SC-01…05.

### D-SC-06 (NEW) — a single ASCII letter left of `. / ?` never arms

`spell::is_valid_syllable` accepts a lone onset (`spell.rs::parse_nucleus_and_coda("")` is
`true`), so `a`…`z` count as a "valid syllable" and the amendment's syllable test keeps the
zone off. Measured (`telex`):

```
c/users        -> c/úe          (pre-fix: c/users)
x/users        -> x/úe          (pre-fix: x/users)
x/cof          -> x/cò          (pre-fix: x/cof)
i/cof          -> i/cò
x/tieesng      -> x/tiếng
a/cof          -> a/cò
```

Only the right-hand word's triggers change the text, so `a/b`, `x/y`, `m/c` are unaffected —
but a lowercase drive prefix is a real path shape. `C:/users`, `c:/users` and `C:\Users` are
protected (chain), so this bites only the colon-less `c/users` form. Not covered by L1–L5.

### D-SC-07 (NEW) — Telex digit-run parity: `10/users` composes, VNI keeps it literal

In Telex a second consecutive digit is a **boundary** (`method::digit_joins_word` is false
once the word ends in a digit), so the word resets and the opener sees an empty left token —
which §2 forbids arming on. Even-length digit runs therefore lose `/` protection; VNI keeps
every digit in the word (digits are triggers there) and stays literal. Measured:

| keys | telex | vni |
|---|---|---|
| `10/users` | `10/úe` | `10/users` |
| `10/cof` | `10/cò` | `10/cof` |
| `21/cof` | `21/cò` | `21/cof` |
| `2024/cof` | `2024/cò` | `2024/cof` |
| `v1/tieesng` | `v1/tiếng` | — |
| `1/cof`, `100/cof`, `102/cof` | `1/cof`, `100/cof`, `102/cof` (odd runs arm) | literal |

So the same keystrokes produce different text per method — a consistency defect even where
the text is not otherwise wrong. Root cause is pre-existing digit-boundary handling, newly
**visible** because the empty-left term now refuses to arm; the fix itself is per spec.

---

## 7. Remaining risks — every arm/no-arm asymmetry measured

The probe (`qa/92_verify.rs::arm_asymmetry_probe`, printed in `qa/verify/harness.txt`) types
`<left><opener>tawts` and reports **C** when the right word composed (`tắt`) and **L** when it
stayed literal (`tawts`) — i.e. L = the zone armed. `left` is **keystrokes**, not displayed
text (passing a precomposed `bật` is not a valid keystroke sequence and produces a boundary).

```
left        .    /    ?    _    @    :    =    #    \
a           C    C    C    L    L    L    L    L    L
ab          L    L    L    L    L    L    L    L    L
ban         C    C    C    L    L    L    L    L    L
cof         C    C    C    C    C    C    C    C    C
2           C    L    L    L    L    L    L    L    L
github      L    L    L    L    L    L    L    L    L
x           C    C    C    L    L    L    L    L    L
baajt       C    C    C    C    C    C    C    C    C
Baajt       C    C    C    C    C    C    C    C    C
dduwowngf   C    C    C    C    C    C    C    C    C
file        L    L    L    L    L    L    L    L    L
me          C    C    C    L    L    L    L    L    L
```

Chain probes: `C:\`→L, `C:/`→L, `http:/`→L, `a:/`→L, `abc://`→L (all protected).
Negative controls: `xyz./`→C, `x_/`→C, `bật:/`→C.

Asymmetries, each measured and named:

1. **`.`/`/`/`?` vs the code openers.** For `. / ?` a valid left syllable keeps composing
   (`ban.`, `a/`, `x/`, `me?`), while `_ @ : = # \` arm on the same left token (`a_`, `a@`,
   `a:`). By design (§2), but it means `a_đường` is literal and `a.đường` is not.
2. **Single letters count as syllables** → the whole `. / ?` column is C for `a`…`z`
   (D-SC-06).
3. **Digits arm only the non-dot openers**: `2.` composes (digit-dot exemption), `2/`, `2?`,
   `2_`, `2@`, `2:` arm.
4. **Digit runs of even length (Telex) never arm** (D-SC-07) while VNI always does.
5. **A plain-ASCII token that renders Vietnamese stops arming entirely** — `cof` renders
   `cò`, so `_ @ : = # \` all compose (`cof_taawts` → `cò_tắt`); only tokens that *stay*
   ASCII protect (`file_name`, `github.cof`).
6. **Valid-syllable-looking code tokens lose `. / ?`** — L3: `ban.cof` → `ban.cò` (pinned),
   and measured `git.cof` → `git.cò` (real-world repo name), `v1.2.3/tag` still literal
   because the `/` left is `3`.
7. **L1 reproduced** — a second opener outside the chain set has an empty left:
   `x=@cof` → `x=@cò`, `a./cof` → `a./cò`.
8. **L2 reproduced** — plain-ASCII code-looking left suppresses a Vietnamese right:
   `abc/d9uo7ng2` → `abc/d9uo7ng2`, `#hashtag/tieesng` → `#hashtag/tieesng`.
9. **L3 reproduced** — `ban.cof` → `ban.cò`, `git.cof` → `git.cò`.
10. **L4 is structural** — no lookahead; not behaviourally measurable, and the amendment's
    rejected alternative (provisional arm + retroactive disarm) is not implemented.
11. **L5 reproduced** — `a:/ta8t1` → `a:/ta8t1`, `abc://tieesng` → `abc://tieesng`; the
    chain only fires after a plain-ASCII left token, so `bật:/tắt` still composes and
    `cof:/tieesng` composes (because `cof` renders `cò`, not ASCII).
12. **Backspace kills the zone** — `C:\` + BS + `Users` → `C:Úe` (documented escape hatch),
    `Tha` `/` BS `s` → `Thá`. But continuing a word after a BS-reopen can **jump to the raw
    keystrokes** when composition gives up: `bật.` BS `cof` → `baajtcof`,
    `github.` BS `tawts` → `githubtawts`. This is the pre-existing `backspace()` literal
    fallback (`engine.rs:316-330`), identical before and after the fix (Backspace clears
    `raw_pending` on both sides of the change).
13. **Caret movers leave `raw_pending` set inside `vkb-core`** — `github` `.` `VK_LEFT`
    `cof` → `github.cof` through `Engine` alone, because a `ch: None` key returns `Noop`
    without touching the flag. Production is safe: `vkb-windows/src/state.rs:349` calls
    `reset_engine()` on every `is_caret_mover` VK before the key reaches the engine, and with
    that reset the same sequence gives `github.cò`. Any new caller that drives `Engine`
    directly must call `VietnameseEngine::reset()` on caret moves. Unchanged by this fix
    (the flag survived arrows before too), but now the *only* stale-flag risk left.
14. **`?` vs `!` remains asymmetric** (O-3, pre-existing): `!` is not an opener, so
    `bật!tắt` composes either way; `?` is and is now syllable-gated.
15. **`tiengs` → `tíeng`, not `tiếng`** — pre-existing and outside the changed code
    (no opener key, so `boundary_key` never runs; the diff touches only `boundary_key`, its
    helpers and tests). A4 row T3 would place the tone on the last vowel (`tiéng`), so this
    is worth a separate look, but it is **not** a regression of this fix and not one of the
    guards that changed.

---

## 8. What is safe to claim

- D-SC-01…05 are fixed on the measured build; the exact original repros now produce the
  expected text, in both methods, on the char path and the VK path, with numpad digits, and
  with punctuation on either side of an ordinary Vietnamese word pair.
- The declared losses L1–L5 all reproduce as written; none of them is reachable between two
  Vietnamese words.
- Two under-protection shapes (single-letter left token, Telex even-length digit runs) are
  new relative to pre-fix and are **not** covered by L1–L5; they need either a spec term or a
  ruling to accept them (`D-SC-06`, `D-SC-07`).
- The workspace gate cannot be claimed this round: `vkb-windows/src/sound.rs` (uncommitted,
  sound work) does not compile. `vkb-core` and the rest of the workspace are green.
  *(Superseded on the third pass — §9: workspace exit 0, 361 passed / 5 ignored.)*

---

## 9. Engine nits re-verification (2026-10-30, third pass, `dev-rawzone-nits`)

The reviewer found two state leaks after §7.13 and Dev fixed them (`engine.rs`: `set_options`
and `set_macro_only` clear `raw_pending`; the `ch: None` early return clears it) plus four tests
in `raw_zone.rs`. Everything below is a fresh run on that build; no product code was touched by
this pass. File added: `crates/vkb-core/tests/qa_93_engine_nits.rs` (own harness, 4 tests).

| item | verdict |
|---|---|
| changed assertion `qa_92_verify.rs:180` (`github . LEFT cof (engine only)`): `github.cof` → `github.cò` | **LEGITIMATE — matches production; the old value pinned the leak** |
| Leak A — `set_options` / `set_macro_only` after an armed opener | **FIXED + VERIFIED** |
| Leak B — a `ch: None` caret/command key after an armed opener | **FIXED + VERIFIED** inside `Engine`; the hook's `reset_engine` is now belt-and-braces |
| 78-case matrix (`matrix()`) | **78 PASS / 0 FAIL** (exit 0) |
| `cargo test -p vkb-core` | exit 0 — **215 passed / 0 failed / 0 ignored** |
| `cargo test --workspace` | exit 0 — **361 passed / 0 failed / 5 ignored** (the §5.3 blocker is gone) |
| `cargo test -p vkb-core --test no_alloc` | exit 0 — 4 passed |
| P1 bench `process_key` | **p99 0.8 µs** (p50 0.2, max 16.6 µs, 3.10 M keys/s, budget 50 µs) → PASS |
| guards `00-charter.md` §9 + the five D-SC fixes | no regression (spot rows §9.3) |

### 9.1 The changed expectation is correct

`github.` arms legitimately (left text `github` is plain ASCII and not an A9 syllable). `VK_LEFT`
carries `ch: None`, and spec §7 gives the armed flag a one-key life that ends at a caret move —
`reset()` is exactly what production calls at `state.rs:350-352` — so the `cof` typed after the
caret move is composed by the current method → `cò`.

**In one sentence: `github.cò` is the correct engine-only output because a caret move ends the
armed window exactly as `reset()` does in production, and the old `github.cof` pinned the
engine-only leak that the hook's `reset_engine()` had been masking** (pre-fix measurement:
`github` `.` `VK_LEFT` `cof` → `github.cof`, §7.13 above). `VK_HOME` takes the same `ch: None`
path and gives the same text (§9.2).

### 9.2 Both leaks, independently reproduced

Own harness: `KeyInput::char` / `KeyInput::key(vk, None)` straight into `Engine::process_key`,
none of Dev's `raw_zone.rs` helpers. Fixed build, Telex default:

| sequence | output |
|---|---|
| control: `github` `.` `cof` (no switch, no caret) | `github.cof` |
| `github` `.` `set_options(raw_zone = false)` `cof` | `github.cò` |
| `github` `.` `set_macro_only(true)` `set_macro_only(false)` `cof` | `github.cò` |
| `github` `.` `VK_LEFT (ch: None)` `cof` | `github.cò` |
| `github` `.` `VK_HOME (ch: None)` `cof` | `github.cò` |
| `github` `.` each of `VK_RIGHT VK_UP VK_DOWN VK_END VK_PRIOR VK_NEXT VK_DELETE VK_F1` `cof` | `github.cò` (8 / 8) |

**Why removing the fix would flip the last rows back to `github.cof` — reasoning check, not a
measurement on this build:** `word_key` consumes `raw_pending` (`engine.rs:228-234`) and never
reads `options.raw_zone`; the flag alone sets `literal_from = raw.len()`. Before the fix both
routes left it `true` (a config switch did not touch it; the `ch: None` arm returned `Noop`
without clearing), so the same `cof` would stay literal — that is the control row above. The
toggle half is code-read only; the caret half is also the pre-fix measurement in §7.13.

### 9.3 Spot checks that did not regress

```
vni   ba65t/ta8t1   -> bật/tắt        telex baajt/tawts  -> bật/tắt
vni   C:\Users\nam  -> C:\Users\nam   telex C:\Users\nam -> C:\Users\nam
vni   github.cof    -> github.cof     telex github.cof   -> github.cof
vni   ba65t.ta8t1 -> bật.tắt   ba65t?ta8t1 -> bật?tắt   ba65t_ta8t1 -> bật_tắt
vni   ba65t@ta8t1 -> bật@tắt   ba65t:ta8t1 -> bật:tắt   ba65t=ta8t1 -> bật=tắt
vni   ba65t#ta8t1 -> bật#tắt   ba65t\ta8t1 -> bật\tắt
vni   /ta8t1 -> /tắt   ba65t//ta8t1 -> bật//tắt   telex a. tieengs -> a. tiếng
telex 3.cof -> 3.cò    vni file_name -> file_name  vni me@cof -> me@cof
telex x=@cof -> x=@cò (L1 still as declared)
```

All exit 0. `raw_zone.rs` now runs 9 tests (4 new) — all green. `00-charter.md` §9 untouched:
no new dependency (core stays OS-free), no new config key, no action change.

**Residual boundary, not a reopen:** `Ctrl`/`Alt`/`Win` chords and key-up still return before
the clear (a chord is not text). Production resets on shortcut chords (`state.rs:322-331`) and
on non-text keys (`:355-362`), so there is no user-visible path; a caller driving `Engine`
directly must still `reset()` on chords, as §7 states. Not fileable against this fix.

**`60-defects.md`: nothing reopens** — no D-SC or D-SND row is affected by these engine changes.
