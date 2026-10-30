# 51 — Special-case typing audit (stream B1)

**Date:** 2026-10-30 · **Owner of this artifact:** QA · **Repo:** `E:\vietkb`, branch `main`
**Round:** `91-round-plan-sound-cases.md` step B1 — reproduce the reported `/` defect on the real
engine and enumerate its sibling cases.
**Status:** 78 measured cases · **56 PASS / 22 FAIL** · **one root cause**, four defect rows
`D-SC-01…04` in `60-defects.md`.
**No product code was changed by this pass.** The only files this pass created are
`crates/vkb-core/tests/special_cases.rs` (test-only) and this document plus its evidence.

---

## 1. The report, reproduced

Owner report (verbatim intent): typing `/` then Vietnamese stops composing — expected `bật/tắt`,
got `bật/ta8t1`.

```
$ cargo run -q -p vkb-cli -- simulate --method vni "ba65t/ta8t1"
bật/ta8t1
```

`b a 6 5 t` → `bật` (correct), `/` → separator (correct), `t a 8 t 1` → **`ta8t1` instead of `tắt`**.
Confirmed, and it is **not VNI-specific**: Telex loses the same word.

```
$ cargo run -q -p vkb-cli -- simulate --method telex "baajt/taats"
bật/taats
```

Control: the identical word pair with a space instead of `/` passes.

```
$ cargo run -q -p vkb-cli -- simulate --method vni "ba65t ta8t1"
bật tắt
```

---

## 2. How every row below was measured

All of it runs the **real engine**, never a reading of the code:

| what | how |
|---|---|
| char path | `vkb_core::simulate(EngineOptions { method, .. }, keys)` — one `KeyInput::char` per keystroke |
| vk path | `vkb_core::simulate_keys` with `KeyInput::key(vk, Some(ch))`: `VK_A…VK_Z`, `VK_0…VK_9`, `VK_OEM_2` = `/`, `VK_OEM_PERIOD` = `.`, `VK_NUMPAD0+` for keypad digits, `0x31 + '!'` for Shift+1 |
| macros / macro-only / Backspace | `vkb_core::Engine` driven directly with `set_macros` / `set_macro_only`, actions applied through `apply_action` |
| command-line cross-check | `cargo run -q -p vkb-cli -- simulate --method <telex\|vni> [--actions] "<keys>"` |

Harness: `crates/vkb-core/tests/special_cases.rs`.

```
cargo test -p vkb-core --test special_cases matrix -- --nocapture   # prints the table below
```

Raw output of that run is `qa/special/matrix.txt`; the CLI transcript with the action trace is
`qa/special/repro.txt`.

---

## 3. Case table

`expected` is the text a Vietnamese typist should get. For the punctuation openers this is the
owner's requirement (`91-round-plan-sound-cases.md`, DoD 2: `bật/tắt` in VNI produces exactly
`bật/tắt`); `20-spec.md` §A9 currently says the opposite — see §4.

| case id | method | keystrokes | expected | actual | verdict |
|---|---|---|---|---|---|
| SC-01 | VNI | `ba65t ta8t1` | `bật tắt` | `bật tắt` | PASS |
| SC-02 | VNI | `ba65t/ta8t1` | `bật/tắt` | `bật/ta8t1` | **FAIL** |
| SC-03 | VNI | `ba65t.ta8t1` | `bật.tắt` | `bật.ta8t1` | **FAIL** |
| SC-04 | VNI | `ba65t,ta8t1` | `bật,tắt` | `bật,tắt` | PASS |
| SC-05 | VNI | `ba65t-ta8t1` | `bật-tắt` | `bật-tắt` | PASS |
| SC-06 | VNI | `ba65t_ta8t1` | `bật_tắt` | `bật_ta8t1` | **FAIL** |
| SC-07 | VNI | `ba65t:ta8t1` | `bật:tắt` | `bật:ta8t1` | **FAIL** |
| SC-08 | VNI | `ba65t@ta8t1` | `bật@tắt` | `bật@ta8t1` | **FAIL** |
| SC-09 | VNI | `ba65t;ta8t1` | `bật;tắt` | `bật;tắt` | PASS |
| SC-10 | VNI | `ba65t(ta8t1` | `bật(tắt` | `bật(tắt` | PASS |
| SC-11 | VNI | `ba65t)ta8t1` | `bật)tắt` | `bật)tắt` | PASS |
| SC-12 | VNI | `ba65t[ta8t1` | `bật[tắt` | `bật[tắt` | PASS |
| SC-13 | VNI | `ba65t]ta8t1` | `bật]tắt` | `bật]tắt` | PASS |
| SC-14 | VNI | `ba65t"ta8t1` | `bật"tắt` | `bật"tắt` | PASS |
| SC-15 | VNI | `ba65t'ta8t1` | `bật'tắt` | `bật'tắt` | PASS |
| SC-16 | VNI | `ba65t+ta8t1` | `bật+tắt` | `bật+tắt` | PASS |
| SC-17 | VNI | `ba65t=ta8t1` | `bật=tắt` | `bật=ta8t1` | **FAIL** |
| SC-18 | VNI | `ba65t*ta8t1` | `bật*tắt` | `bật*tắt` | PASS |
| SC-19 | VNI | `ba65t&ta8t1` | `bật&tắt` | `bật&tắt` | PASS |
| SC-20 | VNI | `ba65t%ta8t1` | `bật%tắt` | `bật%tắt` | PASS |
| SC-21 | VNI | `ba65t#ta8t1` | `bật#tắt` | `bật#ta8t1` | **FAIL** |
| SC-22 | VNI | `ba65t!ta8t1` | `bật!tắt` | `bật!tắt` | PASS |
| SC-23 | VNI | `ba65t?ta8t1` | `bật?tắt` | `bật?ta8t1` | **FAIL** |
| SC-24 | VNI | `ba65t~ta8t1` | `bật~tắt` | `bật~tắt` | PASS |
| SC-25 | VNI | `` ba65t`ta8t1 `` | `` bật`tắt `` | `` bật`tắt `` | PASS |
| SC-26 | VNI | `ba65t\|ta8t1` | `bật\|tắt` | `bật\|tắt` | PASS |
| SC-27 | VNI | `ba65t\ta8t1` | `bật\tắt` | `bật\ta8t1` | **FAIL** |
| SC-28 | VNI | `ba65t/ta8t1` (same as SC-02) | `bật/tắt` | `bật/ta8t1` | **FAIL** |
| SC-29 | Telex | `baajt/taats` | `bật/tắt` | `bật/taats` | **FAIL** |
| SC-30 | VNI | `a/b` | `a/b` | `a/b` | PASS |
| SC-31 | VNI | `/ta8t1` (leading `/`) | `/tắt` | `/ta8t1` | **FAIL** |
| SC-32 | VNI | `ba65t//ta8t1` | `bật//tắt` | `bật//ta8t1` | **FAIL** |
| SC-33 | VNI | `ba65t /ta8t1` | `bật /tắt` | `bật /ta8t1` | **FAIL** |
| SC-34 | VNI | `ba65t/ ta8t1` | `bật/ tắt` | `bật/ tắt` | PASS |
| SC-35 | VNI | `http://a.b/c` | `http://a.b/c` | `http://a.b/c` | PASS |
| SC-36 | VNI | `a@b.com` | `a@b.com` | `a@b.com` | PASS |
| SC-37 | Telex | `github.cof` | `github.cof` | `github.cof` | PASS |
| SC-38 | Telex | `a. tieengs` | `a. tiếng` | `a. tiếng` | PASS |
| SC-39 | Telex | `3.cof` | `3.cò` | `3.cò` | PASS |
| SC-40 | VNI | `-ta8t1` | `-tắt` | `-tắt` | PASS |
| SC-41 | VNI | `--opt` | `--opt` | `--opt` | PASS |
| SC-42 | VNI | `file_name` | `file_name` | `file_name` | PASS |
| SC-43 | VNI | `2/3` | `2/3` | `2/3` | PASS |
| SC-44 | VNI | `1.5` | `1.5` | `1.5` | PASS |
| SC-45 | VNI | `%20` | `%20` | `%20` | PASS |
| SC-46 | VNI, `raw_zone = false` | `ba65t/ta8t1` | `bật/tắt` | `bật/tắt` | PASS |
| SC-47 | VNI, `raw_zone = false` | `ba65t.ta8t1` | `bật.tắt` | `bật.tắt` | PASS |
| SC-48 | VNI, `raw_zone = false` | `ba65t_ta8t1` | `bật_tắt` | `bật_tắt` | PASS |
| SC-49 | VNI | `t8` (digit, no target) | `t8` | `t8` | PASS |
| SC-50 | VNI | `ba65t t8` | `bật t8` | `bật t8` | PASS |
| SC-51 | VNI | `t8 t8` | `t8 t8` | `t8 t8` | PASS |
| SC-52 | VNI | `ba65t.t8` | `bật.t8` | `bật.t8` | PASS |
| SC-53 | VNI | `a8` | `ă` | `ă` | PASS |
| SC-54 | VNI | `ba65t/a8` | `bật/ă` | `bật/a8` | **FAIL** |
| SC-55 | VNI | `a61` | `ấ` | `ấ` | PASS |
| SC-56 | VNI | `ba65t/a61` | `bật/ấ` | `bật/a61` | **FAIL** |
| SC-57 | VNI | `d9` | `đ` | `đ` | PASS |
| SC-58 | VNI | `ba65t/d9` | `bật/đ` | `bật/d9` | **FAIL** |
| SC-59 | VNI | `o7` | `ơ` | `ơ` | PASS |
| SC-60 | VNI | `ba65t/o7` | `bật/ơ` | `bật/o7` | **FAIL** |
| SC-61 | VNI | `u7` | `ư` | `ư` | PASS |
| SC-62 | VNI | `ba65t/u7` | `bật/ư` | `bật/u7` | **FAIL** |
| SC-63 | VNI + keypad | `ba65t/ta8t1` (numpad digits, `VK_NUMPAD0+`) | `bật/tắt` | `bật/ta8t1` | **FAIL** |
| SC-64 | VNI + keypad | `ta8t1` (numpad digits) | `tắt` | `tắt` | PASS |
| SC-65 | VNI + vk | `ba65t!ta8t1` (`VK_1` + `!`) | `bật!tắt` | `bật!tắt` | PASS |
| SC-66 | VNI + vk | `ba65t!ta8t1` (`0x31` + `!`) | `bật!tắt` | `bật!tắt` | PASS |
| SC-67 | Telex | `Tha` `/` `BS` `s` | `Thá` | `Thá` | PASS |
| SC-68 | VNI + macro `ko→không` | `ko ` | `không ` | `không ` | PASS |
| SC-69 | VNI + macro `ko→không` | `ba65t/ko ` | `bật/không ` | `bật/không ` | PASS |
| SC-70 | VNI + macro `bt/→bật tắt` | `bt/ ` (trigger contains `/`) | `bt/ ` (trigger cannot match) | `bt/ ` | PASS¹ |
| SC-71 | VNI + `macro_only` | `ko/` | `không/` | `không/` | PASS |
| SC-72 | VNI | `ba65t/ta8t1` + `BS` | `bật/ta8t` | `bật/ta8t` | PASS |
| SC-73 | VNI | `ba65t/ta8t1/ba65t` | `bật/tắt/bật` | `bật/ta8t1/ba65t` | **FAIL** |
| SC-74 | VNI | `ba65t/ta8t1` `Enter` `ta8t1` | `bật/ta8t1`⏎`tắt` | same | PASS |
| SC-75 | VNI | `Ba65t/Ta8t1` | `Bật/Tắt` | `Bật/Ta8t1` | **FAIL** |
| SC-76 | VNI + macro `ko→không` | `ko/` | `không/` | `không/` | PASS |
| SC-77 | VNI + macro `ko→không` | `ba65t/ko/` | `bật/không/` | `bật/không/` | PASS |
| SC-78 | VNI | `ba65t$ta8t1` (`$` = Shift+4) | `bật$tắt` | `bật$tắt` | PASS |

¹ SC-70 is a **limitation, not a defect** — see §6.

Counts: **78 rows · 56 PASS · 22 FAIL**. The 22 FAILs are
`SC-02, 03, 06, 07, 08, 17, 21, 23, 27, 28, 29, 31, 32, 33, 54, 56, 58, 60, 62, 63, 73, 75`.

---

## 4. Root cause — which layer

**The pure engine (`vkb-core`), specifically its `raw_zone` detector. The `vkb-windows` hook is not
involved.**

Evidence, in order:

1. **It reproduces with zero OS code.** `cargo run -q -p vkb-cli -- simulate --method vni
   "ba65t/ta8t1"` → `bật/ta8t1`, and the same through `vkb_core::simulate` in the test harness.
2. **The hook cannot be involved.** `grep`-level fact: `crates/vkb-windows/src/` contains no
   Vietnamese rule (only key tables, `SendInput`, hotkeys); it forwards `KeyInput` and applies
   `EngineAction`. Measurement: the char path and the real-virtual-key path produce byte-identical
   output (`special_cases.rs::vk_path_matches_char_path`, PASS), including for the failing input
   (SC-65/66, SC-63). So the hook's mapping is neutral here.
3. **The failure is a deliberate feature firing too broadly.** Action trace for the exact report
   (`--actions`):

   ```
   'b' -> Noop
   'a' -> Noop
   '6' -> Replace { delete_units: 2, insert: "bâ" }
   '5' -> Replace { delete_units: 2, insert: "bậ" }
   't' -> Noop
   '/' -> Noop                      <-- boundary; arms `raw_pending`
   't' -> Noop                      <-- word starts literal: `literal_from = 0`
   'a' -> Noop
   '8' -> Noop
   't' -> Noop
   '1' -> Noop
   -- text: bật/ta8t1
   -- raw:  ta8t1
   ```

   Every keystroke of the second word is `Noop`: the engine has frozen the word as literal text, so
   the VNI digits never become a mark or a tone. Telex loses the same word the same way.
4. **Turning the feature off fixes it and nothing else changes.** With
   `EngineOptions { raw_zone: false, .. }` (`EngineOptions` `raw_zone`, default `true`),
   `ba65t/ta8t1` → `bật/tắt`, `ba65t.ta8t1` → `bật.tắt`, `ba65t_ta8t1` → `bật_tắt`
   (SC-46/47/48, PASS).

Mechanism, for the fix owner: `Engine::boundary_key` (`crates/vkb-core/src/engine.rs:421-435`) sets
`raw_pending` for the opener set `matches!(ch, '.' | '/' | ':' | '@' | '_' | '\\' | '#' | '=' | '?')`
— every ASCII character that appears in a URL, an e-mail address or a code line. The next `word_key`
(`engine.rs:224-232`) then sets `literal_from = 0`, and `method::compose` with `literal_from = 0`
stops transforming for the **whole word**. Whitespace disarms it (SC-01/34), and Backspace removes
the opener and re-opens the word (SC-67/72) — so the state itself is well-behaved; the *classification
of the opener* is what is wrong.

**This is a spec-versus-expectation conflict, not a coding slip.** `20-spec.md` §A9 says:

> `raw_zone`: after any of `. / : @ _ \ # = ?` (when the char is passed as punctuation) with **no
> whitespace** before the next letter, the next word is raw (no transforms) until whitespace/Enter/Tab
> (`github.com/user`, `a_b`, `x=y`, `me@host`).

So the observed output is exactly what the spec asks for, and `crates/vkb-core/tests/raw_zone.rs`
pins three of these shapes. But the owner's requirement for this round (§DoD 2) and everyday
Vietnamese (`bật/tắt`, `và/hoặc`, `km/h`, `câu. Tiếp`) both say the following word must still
compose. The classifier therefore needs a narrower trigger (e.g. only when the token on the left
looks like a host/path/e-mail/code identifier, not after any occurrence of these characters), and
§A9 must be amended with it. **QA does not choose the rule; four measured defect rows record the
cost of the current one.**

Two facts that matter for the fix:
- Raw-zone protection **must survive** the change: `http://a.b/c` (SC-35), `a@b.com` (SC-36),
  `github.cof` (SC-37), `file_name` (SC-42), `--opt` (SC-41), `2/3` (SC-43), `1.5` (SC-44),
  `%20` (SC-45) all pass today, and `tests/raw_zone.rs` +
  `special_cases.rs::url_email_and_code_shapes_stay_literal` pin them.
- `raw_zone` has **no UI control**: `ui/src/lib/bindings.ts:25` marks it `// reserved, not
  implemented`, and `docs/evkey-clone-work/87-feature-matrix.md` lists it as reserved. It is on by
  default (`vkb-config/src/schema.rs:124`), is passed to the engine
  (`crates/vkb-app/src/commands.rs:213`), and can only be turned off by editing
  `typing.raw_zone = false` in `config.toml`. **Measured consequence:** a user cannot work around
  this defect from the settings window.

---

## 5. Defect rows (detail lives in `60-defects.md`)

| id | opener(s) | severity | representative case ids |
|---|---|---|---|
| D-SC-01 | `/` — the owner report | P1 | SC-02, 28, 29, 31, 32, 33, 54, 56, 58, 60, 62, 63, 73, 75 |
| D-SC-02 | `.` `?` (sentence end, no space) | P2 | SC-03, 23 |
| D-SC-03 | `_` `@` `:` | P3 | SC-06, 07, 08 |
| D-SC-04 | `=` `#` `\` | P3 | SC-17, 21, 27 |

All four share the single cause in §4, so a fix that narrows the opener classifier should clear all
of them at once; they are filed separately because the severity and the acceptable narrowing differ
per character (`/` and `.` occur in ordinary Vietnamese prose; `=`/`#`/`\` only in code).

---

## 6. Other findings (measured, not classified as defects)

| # | finding | measured evidence | reading |
|---|---|---|---|
| O-1 | A macro trigger that contains punctuation can never match | macro `bt/ → bật tắt`, keys `bt/ ` → `bt/ ` (SC-70) | By design: triggers are matched against the *word* raw buffer, and punctuation ends the word. Consistent with `20-spec.md` §A1 (macro on a word boundary). Not a defect, but it means no `gõ tắt` entry can end in a symbol — worth a copy/spec note. |
| O-2 | Macro expansion **does** work after a punctuation boundary | `ko/` → `không/` (SC-76), `ba65t/ko/` → `bật/không/` (SC-77), `macro_only` + `ko/` → `không/` (SC-71) | Good: a fix to §4 must not disturb this. |
| O-3 | `?` and `!` are asymmetric | `ba65t?ta8t1` fails (SC-23) while `ba65t!ta8t1` passes (SC-22, SC-65, SC-66) | Both are sentence enders and both are Shift+igits on a US layout (`?` = Shift+`/`, `!` = Shift+`1`); the opener list contains `?` because it is in the URL query-string set, not because it ends a sentence. |
| O-4 | Keypad digits and main-row digits are equivalent (including for the defect) | SC-63 vs SC-02, SC-64 vs SC-53 | The engine keys off `ch`, never `vk`; a hook-side fix cannot help. |
| O-5 | A raw-zone word is still editable normally | `ba65t/ta8t1` + `BS` → `bật/ta8t` (SC-72); `Tha` `/` `BS` `s` → `Thá` (SC-67) | The boundary/reopen state machine is correct; only the transform suppression is wrong. |
| O-6 | The raw zone starts over after Enter | `ba65t/ta8t1` `Enter` `ta8t1` → `bật/ta8t1`⏎`tắt` (SC-74) | Scope is one word, as specced. |
| O-7 | Stale doc comment on the option | `crates/vkb-core/src/lib.rs:264`: "Reserved: URL/code raw zone (spec §A9) is not implemented yet; the value is ignored" | The zone **is** implemented (`engine.rs:429-432`) and the value **is** honoured (SC-46/47/48 differ from SC-02/03/06). One-line doc fix, not a behaviour defect. |

---

## 7. Re-running this audit

```powershell
cd E:\vietkb

# the table (78 rows, PASS/FAIL) — this is what section 3 was generated from
cargo test -p vkb-core --test special_cases matrix -- --nocapture

# the four open defects, as failing assertions (ignored by default)
cargo test -p vkb-core --test special_cases -- --include-ignored

# one-off reproductions through the CLI
cargo run -q -p vkb-cli -- simulate --method vni  "ba65t/ta8t1"
cargo run -q -p vkb-cli -- simulate --method telex "baajt/taats"
cargo run -q -p vkb-cli -- simulate --method vni --actions "ba65t/ta8t1"
```

`cargo test -p vkb-core` on the audit branch: **exit 0 — 188 tests, 0 failed, 5 ignored**
(all 5 are in `special_cases.rs`: the four defect repros plus one action-trace diagnostic helper).
The repros are `#[ignore]`d so the suite stays green while the defects are open; deleting the
attribute is all a fix needs to make them permanent regressions — the measured expectations they
assert are section 3's `expected` column.
