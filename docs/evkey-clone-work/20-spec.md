# 20 — Specification (engine, config, UI behavior, acceptance)

**Owner:** BA. **Status:** draft v1 for Gate 0. **Authority:** below `00-charter.md` and `25-contract.md`;
where this spec needs something the contract does not express, it is listed in §B.0 as a
**contract change request (CCR)** — Dev must not implement a CCR until the Lead applies it.

**Repo code citations:** none. The workspace has no product code yet (`crates/` is empty,
verified 2026-10-28), so every behavior below is specified from first principles plus external
evidence. Legacy code on `legacy/pre-rewrite` was not consulted (evidence only, not contract).

**External evidence used**
| Ref | URL | Used for |
|---|---|---|
| E1 | https://github.com/lamquangminh/EVKey | EVKey existence, platforms |
| E2 | https://quantrimang.com/cong-nghe/cach-su-dung-bo-go-tieng-viet-evkey-149861 | verbatim labels: `Khởi động cùng Windows`, `Loại trừ ứng dụng`, `Sửa lỗi dấu gợi ý`, `Hỗ trợ ứng dụng Metro`, `Sử dụng Clipboard để gửi phím`, `Vẫn cho phép gõ tắt khi tắt tiếng Việt`, `chạy với quyền Admin`, `hiển thị hộp thoại khi khởi động`, `Bảng gõ tắt`, exclusion picker (… button + drag-locator + `Thêm`/`Xoá`) |
| E3 | https://hocdohoacaptoc.com/evkey/ | `Sửa lỗi dấu gợi ý 1` (two variants exist) |
| E4 | https://voz.vn/t/khuyen-cao-nen-bo-evkey-va-chuyen-qua-openkey.977921/page-15 | "EVKey có 2 cái Sửa lỗi dấu gợi ý, chọn 1 trong 2" |
| E5 | https://en.wikipedia.org/wiki/VSCII | TCVN3 (VSCII-3) byte table — **verified** against the table on 2026-10-28 |
| E6 | https://en.wikipedia.org/wiki/VISCII | VISCII byte table — **verified** 2026-10-28 |
| E7 | https://gist.github.com/teefan/433e70d4cb33686b1d2791c73b162c58 | VNI Windows sequences (single community source) |
| E8 | https://www.rfc-editor.org/rfc/rfc1456 | VIQR mnemonics and `\` escape |
| E9 | https://en.wikipedia.org/wiki/Telex_(input_method), https://en.wikipedia.org/wiki/VNI | Telex / VNI key assignments |

`UNVERIFIED` = not established from a source; each carries a verification method. QA must turn
each into a pass/fail fact before Gate 1 or the item stays provisional.

---

## A. Engine semantics

### A.0 Data model (shared by A1–A13)

| Term | Definition |
|---|---|
| **raw** | ring of ≤ 32 `RawKey { ch: u8 (ASCII), upper: bool, consumed_as: Role }` — every key accepted into the current word, in order. `Role ∈ {Letter, Tone(t), Mod(m), Escape, Literal}` |
| **word** | the composed output for raw, ≤ 32 `char`s, kept as a fixed array + len |
| **emitted** | the exact string the engine believes is on screen for this word (in output charset units, see A11). Invariant: `emitted == encode(word)` after every action |
| **syllable split** | `word = C1 · V · C2` (initial consonant, vowel cluster, final consonant) recomputed on each key; O(32) worst, O(≤6) typical |
| **flags** | `non_viet` (word is not Vietnamese; all later triggers literal), `sticky_literal[k]` (key k was escaped in this word), `raw_zone` (word follows a URL/code symbol, see A9) |
| **last_word** | snapshot of previous committed word (raw + emitted + commit key) for one-step resume (A7) and manual restore (A10) |

Every key yields exactly one `EngineAction` (contract §3). The engine computes a **minimal diff**:
`p = common_prefix(old_emitted, new_emitted)`; action = `Replace{ delete = units(old) − p, insert = new[p..] }`.
If `delete == 0` and `insert == key char` → `Noop` (let the OS deliver the key; zero alloc).
If `delete == 0` and insert ≠ key char → `Commit(insert)`.

**Vowels:** `a ă â e ê i o ô ơ u ư y`. **Tones:** `ngang(0) sắc(1) huyền(2) hỏi(3) ngã(4) nặng(5)`.
**Initials C1** (longest match): `ngh ng gh gi kh nh ph th tr ch qu b c d đ g h k l m n p r s t v x`
(+ `f j w z` iff `jfwz_as_consonant`). **Finals C2:** `c ch m n ng nh p t` (semivowel finals `i y o u`
live inside V).

### A1. Telex

**Keys** (E9):

| Key | Role | Target selection |
|---|---|---|
| `s f r x j` | tone sắc/huyền/hỏi/ngã/nặng | tone slot chosen by A4 |
| `z` | remove tone | the toned vowel |
| `a e o` (2nd occurrence) | circumflex `â ê ô` | the same letter in V (adjacent or, with `free_marking`, across C2) |
| `w` | breve/horn | see W-rules |
| `d` (2nd) | `đ` | C1 = `d` (adjacent `dd`; non-adjacent `dongd → đong` only with `free_marking`) |

**W-rules** (evaluated in order; first match wins):
1. V contains `uo` (both unmodified, or `uô`/`ưo`/`uơ`) and C1 ≠ `qu` → both become `ươ` (`nguoiw → ngươi`, `truongw → trương`).
2. C1 = `qu` and V contains `o` → `ơ` (`quow → quơ`).
3. V contains `a`/`â` (rightmost) → `ă` (`aaw → ă`: breve replaces circumflex).
4. V contains `o`/`ô` → `ơ`.
5. V contains `u` → `ư`.
6. V is empty (word empty or only C1 letters) and not `jfwz_as_consonant` → insert `ư` (`w → ư`, `tw → tư`, `nhw → như`).
7. Otherwise `w` is literal.
Escape: `w` immediately after a W-transform reverts that transform and emits a literal `w`;
`sticky_literal[w]` is then set (`aww → aw`, `ww → w`, `www → ww`).

**Auto-horn:** `ưo` followed by any of C2 or `i`/`u` becomes `ươ` (`thuwong → thương`, `muwoi → mươi`).
`uơ` without a following final stays (`huơ`, `thuở`).

**Doubled-letter escapes** (all set `sticky_literal[key]`):

| Typed | Result | Rule |
|---|---|---|
| `aa` / `aaa` / `aaaa` | `â` / `aa` / `aaa` | 3rd `a` reverts circumflex + literal |
| `ee`, `oo` same pattern | `ê`/`ee`, `ô`/`oo` | |
| `dd` / `ddd` | `đ` / `dd` | |
| `as` / `ass` / `asss` | `á` / `as` / `ass` | repeating the **same** tone key reverts tone + literal |
| `as` + `f` | `à` | different tone key replaces (not an escape) |
| `az` | `az` | `z` with no tone present is literal |
| `asz` | `a` | `z` removes tone |
| `aw` / `aww` | `ă` / `aw` | |

**A trigger is literal text (no transformation) when ANY holds:**
1. word has no vowel (tone keys, `z`, `w` with `jfwz_as_consonant`): `s`, `str`, `x`.
2. `sticky_literal[key]` is set in this word.
3. `non_viet` is set: word began with a digit; or contains a letter not legal in its slot after
   spell analysis (only when `spell_check = true`); or `raw_zone` is set.
4. Applying it would produce more than one tone in the word (impossible by construction — tone moves instead).
5. `spell_check = true` and the result is invalid (A9), e.g. `f r x` on a C2 ∈ `{c ch p t}` (`tacf → tacf`).
6. Ctrl/Alt/Win is held (never a trigger; see A6).
7. The key is beyond `MAX_WORD`: word is committed and reset first, then the key starts a new word.

**Case:** see A12.

### A2. VNI

Note: the task brief lists `6 ư/ơ`. Standard VNI (E9) is `6 = mũ (â ê ô)`, `7 = móc (ơ ư)`. This spec uses the standard.

| Key | Effect | Target |
|---|---|---|
| `1 2 3 4 5` | sắc huyền hỏi ngã nặng | A4 |
| `6` | `â ê ô` | rightmost `a/e/o` in V |
| `7` | `ơ ư`; `uo → ươ` (C1 ≠ qu) | rightmost `o/u` |
| `8` | `ă` | `a` in V |
| `9` | `đ` | C1 = `d` |
| `0` | remove tone | toned vowel |

**Digit escapes:** same digit repeated immediately after its own transform reverts + literal
digit + `sticky_literal[digit]` (`a11 → a1`, `a111 → a11`, `d99 → d9`).
**Collision rule (user wants digits):**
1. Digit is literal when the word has no target (`2026`, `x2`, `k8` if no vowel).
2. A word whose first key is a digit is `non_viet` (`1a1 → 1a1`, `3g → 3g`).
3. Once any digit was literal in a word, all later digits in that word are literal (`x2a1 → x2a1`:
   `2` literal (no vowel) ⇒ `1` literal too). A digit that applies does not set this flag (`ab1 → áb`).
4. Numpad digits (`VK_NUMPAD0..9`, `vk 0x60–0x69`) are **always** literal and act as word-commit (they are typed numbers).
5. `spell_check = true`: digit literal if the result is not a valid syllable (`abc1 → abc1`).
6. Letters `w f j z` are ordinary letters in VNI.

### A3. VIQR (E8)

| Key | Effect |
|---|---|
| `'` `` ` `` `?` `~` `.` | sắc huyền hỏi ngã nặng |
| `^` | `â ê ô` |
| `(` | `ă` |
| `+` | `ơ ư` (`uo+ → ươ`) |
| `*` | alias of `+` — `UNVERIFIED` that EVKey accepts it (method: type `o*` in EVKey VIQR mode, observe) |
| `dd` | `đ` (as Telex) |
| `-` after `d` | `đ` alias — `UNVERIFIED` (same method with `d-`) |
| `)` | listed in the brief; **no standard role**. `UNVERIFIED`; until verified it is plain punctuation |
| `\` | escape: the next trigger is literal; `\` itself is swallowed (`nha\. → nha.`). `\\` → literal `\` |
| `0` | remove tone — `UNVERIFIED` (Unikey convention); fallback: none |

**Ambiguity rule** (VIQR punctuation doubles as sentence punctuation):
1. A tone/mod trigger applies only if the char immediately before it in the word is a vowel that
   can take it **or** the vowel cluster can take it per A4 (`ok. → ok.`, `nha. → nhạ`).
2. Same trigger twice reverts + literal (`nha.. → nha.`) — this is how users end a sentence after a vowel.
3. A trigger that is literal **commits** the word (it is punctuation). `?`/`.`/`'` used as tone do not commit.
4. Space after a trigger commits normally.

### A4. Tone placement (decision table)

Split: strip C1 by longest match, with two cluster exceptions:
- **qu**: `u` belongs to C1 iff C1 = `q` + `u` and another vowel follows (`qua → C1=qu, V=a`; `qu` alone → V=`u`).
- **gi**: `i` belongs to C1 iff `g` + `i` and another vowel follows (`gia, giữ, giường`); otherwise `i` is V (`gì, gìn, gịt`).

Evaluate rows top-down; first match places the tone. `classic` = `old_tone_style = true` (see §B naming).

| # | Condition on V (after C1) and C2 | Mark goes on | Examples (modern / classic) |
|---|---|---|---|
| T1 | V contains `ơ` (incl. `ươ`, `uơ`, `ươi`, `ươu`) | `ơ` | người, rượu, thuở / same |
| T2 | V contains exactly one of `ă â ê ô ư` | that letter | thuế, khuấy, cứu, cửa, muốn, tiến, khuyến / same |
| T3 | C2 non-empty | last vowel of V | hoàn, toán, khuỳnh, xoòng / same |
| T4 | \|V\| = 1 | it | má, gì, quý, già / same |
| T5 | \|V\| = 3 | middle vowel | hoài, ngoáy, khuỷu, khuya→khuỵa, ngoẻo / same |
| T6 | V ∈ {`oa`, `oe`, `uy`}, C2 empty | modern: **2nd**; classic: **1st** | hoá/hòa, hoè/hòe, thuý/thúy |
| T7 | any other \|V\| = 2, C2 empty (`ai ao au ay eo eu ia iu oi ua ui`) | 1st | tài, cáo, mía, tíu, mùa, túi / same |

Consequences that the corpus must pin:
- Tone **migrates** as letters arrive: classic `hoaf → hòa`, then `n → hoàn` (T6→T3). Modern `hoaf → hoà`, `n → hoàn`.
- `qu`/`gi` make old/new identical: `quys → quý`, `gias → giá` in both styles.
- `uê` hits T2 (`thuế`) in both styles — `uê` is **not** in the T6 set.
- `iu`, `ưu` → T7/T2: `tíu`, `cứu`.
- Incomplete clusters (user hasn't added horn yet) still place by table: `muon` + `s` → `muón` (T3); later `o` circumflex → `muốn` (T2) — tone re-placed each key.
- Only T6 differs between styles.

### A5. Modified letters and mark composition

| Raw | Letter | Telex | VNI | VIQR |
|---|---|---|---|---|
| a | ă | `aw` | `a8` | `a(` |
| a | â | `aa` | `a6` | `a^` |
| e | ê | `ee` | `e6` | `e^` |
| o | ô | `oo` | `o6` | `o^` |
| o | ơ | `ow` | `o7` | `o+` |
| u | ư | `uw`, `w` | `u7` | `u+` |
| d | đ | `dd` | `d9` | `dd` |

Composition: a letter is the triple `(base, mod ∈ {none, breve, circumflex, horn}, tone 0..5)`;
mod and tone are independent fields, so order of entry never matters (`aas`, `asa` both → `ấ`;
`ows`, `osw` → `ớ`). Rendering maps the triple → one NFC code point via a static 12×6 table
(`[[char; 6]; 12]`), then to the target charset (A11). Changing mod keeps tone
(`ấ` + `w` → `ắ`); removing mod via escape keeps tone (`ấ` + `a` → `áa`, sticky).
`đ` carries no tone; `d` is never a tone target.

### A6. Word boundary and commit model

**Current word:** keys accepted since the last boundary, ≤ `MAX_WORD = 32` chars. Letters `a–z`,
`A–Z`, plus method triggers (Telex none extra; VNI digits; VIQR the trigger set) extend it.

| Event | Engine | Hook/backend |
|---|---|---|
| Space, Enter, Tab | macro check (A8) → restore check (A10) → reset. If neither fires: `Noop` | pass key |
| Punctuation `` . , ; : ! ? " ' ( ) [ ] { } < > / \ \| - _ = + * & ^ % $ # @ ~ ` `` not consumed as a trigger | same as space | pass key |
| Macro/restore fires on a commit key | `Replace{delete, insert = expansion}` + key re-delivery (**CCR-4**) | delete, type, then re-send original key as a VK event |
| Arrows, Home, End, PgUp, PgDn, Delete, Insert, Esc | `boundary()`; `Noop` | pass key |
| Any key with Ctrl/Alt/Win held (except configured hotkeys) | `boundary()`; `Noop` | pass key |
| Mouse button down (any) | `boundary()` | backend installs `WH_MOUSE_LL` only for button-down events, or polls caret on next key — `UNVERIFIED` which is cheaper; measure P5 |
| Foreground window / focus change | `boundary()` | backend from `EVENT_SYSTEM_FOREGROUND` (WinEvent hook) |
| `MAX_WORD` exceeded | commit + reset, key starts new word | — |
| Key-up events, repeats of modifiers | ignored (`Noop`) | — |
| Auto-repeat of a letter (`repeat = true`) | processed as a normal key | — |

**Invariants (guard list):**
- **I1 no stray backspace:** `delete ≤ units(emitted)` always. If a computed delete would exceed it
  (state desync), the engine returns `Noop`, resets, and increments a debug counter. Never `Replace`
  with `delete > 0` on the first key of a word.
- **I2 no lost first character:** the first key after any boundary is either `Noop` (passed) or a
  `Commit`/`Replace` whose insert contains that key's output. `Swallow` is never returned for the
  first key of a word.
- **I3 idempotent boundary:** `boundary()` twice == once; it never emits.
- **I4** after `Replace`, `emitted == encode(word)` (checked in debug builds).

### A7. Backspace

Backspace never returns `Replace` in the common case; it lets the app delete one char and resyncs.

1. Buffer empty and `last_word` exists and the previous key was the commit key of `last_word`
   (space only; no caret movement since): restore `last_word` as the current word (one-step resume).
   `Noop` (the app deletes the space). Otherwise buffer empty → `Noop`, no state.
2. Buffer non-empty: `Noop`; engine removes the **last output character** from `word`, then
   rebuilds raw canonically from the remaining `word`:
   letter keys in order, then mod keys appended in canonical form, then the tone key last
   (`tiếng` BS → `tiến` → raw `tieens` (Telex)). Sticky flags are cleared for keys no longer present.
3. If the removed char carried the tone, the tone is gone (`hoá` BS → `ho`).
4. Re-placement after BS is **deferred**: the remaining characters stay exactly as displayed
   (no `Replace` on backspace — avoids surprise edits). The next typed key re-runs A4 on the
   whole word and may move the tone (normal minimal-diff Replace).
5. `ư`/`ơ` cases: `ươ` BS → `ư` (raw `uw`); `người` BS → `ngườ` (raw `nguowf`), BS → `ngư`
   (tone was on `ờ`, lost; raw `nguw`), then `o` → `ngưo`, then `i` → `ngươi` via auto-horn (A1).
6. NFD/VNI Windows/legacy multi-unit output: one Backspace in the target app deletes one **code
   unit**, not one grapheme (`UNVERIFIED` per app; method: type `ệ` in NFD into Notepad, Word,
   Chrome textarea, press BS once, inspect remaining code points). If an app deletes one unit,
   the engine must swallow BS and emit `Replace{delete = units(last char), insert=""}` for
   multi-unit chars. See **CCR-3**.
7. Ctrl+Backspace: boundary + pass (word-delete is the app's semantics).

### A8. Macro (gõ tắt)

| Aspect | Rule |
|---|---|
| Match | **exact whole-token** match at a commit key (A6), never prefix, never mid-token |
| Compared form | first the composed word (`đc` typed `ddc`), then raw ASCII (`ko`) |
| Case | lookup case-insensitive; output case: all-caps token → expansion uppercased (`VN → VIỆT NAM`); first-cap token → expansion first letter uppercased (`Vn → Việt Nam`); else verbatim |
| Precedence | macro > restore (A10) > spell (A9) > Telex/VNI transforms on commit |
| Timing | on commit key only; never on the trigger's last letter |
| Commit key | not swallowed in effect: expansion is inserted, then the commit key is re-delivered (CCR-4) |
| "Inside a longer word" | token must start at a boundary: buffer started after whitespace / punctuation / boundary(). `abtw` never expands `btw`. Buffer that started after a caret move (left context unknown) **does** match — `UNVERIFIED` vs EVKey (method: in Notepad type `x`, click right after it, type `btw␠`, observe whether EVKey expands) |
| Vietnamese off | expand iff `macro_active_when_disabled` (`Vẫn cho phép gõ tắt khi tắt tiếng Việt`, E2); engine keeps a raw buffer in off mode solely for this |
| Limits | trigger 1–32 chars, no whitespace; expansion ≤ 4000 chars, may contain `\n` (typed as Enter) and `\t`; table ≤ 10 000 entries (DoD ≥ 5000) |
| Lookup cost | `HashMap<CaseFoldedKey, Arc<str>>` built on load (CCR-2); O(1) at commit; zero cost per non-commit key |
| Excluded apps | no expansion in apps with `enabled = false` rules |

### A9. Spell check (kiểm tra chính tả)

A syllable is **valid** iff `C1? V C2?` parses with:
- C1 from the initial list (A.0); `k`, `gh`, `ngh` only before `i e ê y`; `c`, `g`, `ng` not before `i e ê`
  (except `gi` cluster).
- V ∈ the closed set: `a ă â e ê i o ô ơ u ư y ai ao au ay âu ây eo êu ia iê iu oa oă oe oi ôi ơi oo ua uâ uê ui uô uơ uy ưa ưi ươ ưu yê oai oay oeo uây uôi uya uyê uyu ươi ươu iêu yêu` (+ incomplete-typing states `uo, ưo, ie, ye, uye` which are valid while typing, invalid at commit unless spell is off).
- C2 legal for V: `ch` only after `a ê i oa uê uy`; `nh` only after `a ê i oa uê uy`; no C2 after V ending in `i y o u` (semivowel) except `uy` (`uych, uynh`) and `oo`.
- Tone: C2 ∈ `{c ch p t}` ⇒ tone ∈ {sắc, nặng}.

Policy:
- **Invalid while typing**, `spell_check = true`: the triggering key is literal (A1 #5). `spell_check = false`: transforms apply regardless.
- **False positives (English, URLs, code):**
  - `raw_zone`: after any of `. / : @ _ \ # = ?` (when the char is passed as punctuation) with **no
    whitespace** before the next letter, the next word is raw (no transforms) until whitespace/Enter/Tab
    (`github.com/user`, `a_b`, `x=y`, `me@host`).
  - A word that begins with a digit is `non_viet`.
  - Uppercase-inside-word after a lowercase letter (`iPhone`, `getX`) → `non_viet` from that key on —
    `UNVERIFIED` that this doesn't hurt real Vietnamese (method: corpus run on 10k-word VN text; expect 0 hits).
  - Everything else is handled by A10 restore at commit.

### A10. Restore wrong word (tự động khôi phục từ gõ sai)

**Wrong** = at commit time, (a) at least one transform was applied in this word AND (b) the composed
word is not a valid syllable per A9 (regardless of `spell_check` setting), OR contains a letter
`f j w z` used as a letter while transforms also happened (`zalo`… only if transformed).

| Mode | Behavior |
|---|---|
| Auto (`restore_wrong_word = true`) | on commit key: `Replace{ delete = units(emitted), insert = raw ASCII }` then commit key (CCR-4). `google␠ → gôgle → google␠`; `windows␠ → ưindows → windows␠`; `tesst → test` is NOT restored (escape already produced valid literal `test`) |
| Auto off | no restore |
| Manual hotkey `restore_word` | if current word non-empty → replace with raw (stay in word, `non_viet` set so following keys stay literal). Else if `last_word` exists and caret not moved since its commit → replace `last_word emitted + commit char` with `raw + commit char`. Else the hotkey is passed to the app (so `Ctrl+Shift+Z` = redo keeps working when there is nothing to restore) |
| Reverse | pressing the hotkey again on a just-restored word re-applies the Vietnamese form (toggle) |

### A11. Charset output

**Letter inventory:** 12 vowels × 6 tones = 72 vowel forms, of which `a e i o u y` untoned are ASCII;
so 60 toned + 6 untoned modified (`ă â ê ô ơ ư`) + `đ` = **67 non-ASCII lowercase**, 67 uppercase = 134.

| Charset (config) | Units per letter | Strategy | Status |
|---|---|---|---|
| `unicode` — Unicode dựng sẵn (NFC) | 1 code point | static table → U+00C0–U+1EF9 | certain |
| `unicode_nfd` — Unicode tổ hợp | 1–3 code points | base + horn U+031B / breve U+0306 / circumflex U+0302 + tone U+0300/0301/0303/0309/0323, canonical order (ccc 216 → 220 → 230): `ệ = e U+0323 U+0302`, `ợ = o U+031B U+0323`, `đ = U+0111` | form chosen by EVKey `UNVERIFIED`: EVKey/Unikey "tổ hợp" may be **CP1258-style** (precomposed `ê` + combining tone) rather than full NFD. Method: type `ệ ợ ắ` in EVKey "Unicode tổ hợp" into Notepad, save UTF-16LE, `Format-Hex`. Default implementation: full NFD; add `unicode_cp1258` if EVKey differs (CCR-5) |
| `tcvn3` — TCVN3 (ABC) | 1 byte | VSCII-3 table (E5); uppercase toned not encodable → emit the lowercase byte (renders capital in `.VnXxxH` fonts) | bytes verified (E5); uppercase fallback behavior `UNVERIFIED` vs EVKey |
| `vni_windows` | 1–2 bytes | base ASCII + one diacritic byte; `ơ ư đ ĩ ị ì í ỉ ỵ` single bytes | sourced from E7 only → `UNVERIFIED` until 2nd source |
| `viscii` | 1 byte, all 134 | full table (E6) incl. C0 positions 0x02 0x05 0x06 0x14 0x19 0x1E | verified (E6); injection of C0 bytes `UNVERIFIED` |
| `viqr` | 1–3 ASCII | `base + (^ ( +) + (' ` ? ~ .)`, `đ = dd` (E8) | certain |
| `bkhcm1` — BK HCM1 | ? | not in DoD | `UNVERIFIED`; method: obtain the table from a published spec or by converting a known string with EVKey's conversion tool and hex-dumping; out of Sprint 1 |
| `vps` — VPS | ? | not in DoD | `UNVERIFIED`; same method |
| `utf8` | 2–3 "bytes" | UTF-8 bytes of NFC, each injected as the CP1252 char of that byte ("UTF-8 literal") | semantics `UNVERIFIED` vs EVKey's list (method: EVKey charset dropdown screenshot) |

**Injection of 8-bit charsets:** `SendInput` with `KEYEVENTF_UNICODE` sends UTF-16. For a byte `b`
the injected code point is `cp1252_to_unicode(b)` (identity for 0xA0–0xFF; 0x80–0x9F map per
CP1252, e.g. 0x80 → U+20AC). VISCII uses 0x80–0x9F, incl. 0x81 0x8D 0x8F 0x90 0x9D that are
**undefined in CP1252** (VISCII `Ắ Ẽ Ố Ồ Ở`... at those slots) → `UNVERIFIED` how legacy apps receive
them; method: type each in WordPad with a VISCII font, check glyph. TCVN3 and VNI Windows use only
0xA1–0xFF and ASCII → safe.

**TCVN3 (VSCII-3) bytes, verified E5:**

| Letter | ngang | huyền | hỏi | ngã | sắc | nặng |
|---|---|---|---|---|---|---|
| a | 61 | B5 | B6 | B7 | B8 | B9 |
| ă | A8 | BB | BC | BD | BE | C6 |
| â | A9 | C7 | C8 | C9 | CA | CB |
| e | 65 | CC | CE | CF | D0 | D1 |
| ê | AA | D2 | D3 | D4 | D5 | D6 |
| i | 69 | D7 | D8 | DC | DD | DE |
| o | 6F | DF | E1 | E2 | E3 | E4 |
| ô | AB | E5 | E6 | E7 | E8 | E9 |
| ơ | AC | EA | EB | EC | ED | EE |
| u | 75 | EF | F1 | F2 | F3 | F4 |
| ư | AD | F5 | F6 | F7 | F8 | F9 |
| y | 79 | FA | FB | FC | FD | FE |
| đ | AE | | | | | |

Uppercase untoned: `Ă A1, Â A2, Ê A3, Ô A4, Ơ A5, Ư A6, Đ A7`. Uppercase toned: none in VSCII-3
(VSCII-2/-1 add some, not used).

**VISCII bytes, verified E6** (modified letters + tones):

| Letter | ngang | huyền | hỏi | ngã | sắc | nặng |
|---|---|---|---|---|---|---|
| ă | E5 | A2 | C6 | C7 | A1 | A3 |
| â | E2 | A5 | A6 | E7 | A4 | A7 |
| ê | EA | AB | AC | AD | AA | AE |
| ô | F4 | B0 | B1 | B2 | AF | B5 |
| ơ | BD | B6 | B7 | DE | BE | FE |
| ư | DF | D7 | D8 | E6 | D1 | F1 |
| đ | F0 | | | | | |
| Ă | C5 | 82 | 02 | 05 | 81 | 83 |
| Â | C2 | 85 | 86 | 06 | 84 | 87 |
| Ê | CA | 8B | 8C | 8D | 8A | 8E |
| Ô | D4 | 90 | 91 | 92 | 8F | 93 |
| Ơ | B4 | 96 | 97 | B3 | 95 | 94 |
| Ư | BF | BB | BC | FF | BA | B9 |
| Đ | D0 | | | | | |

(Plain-vowel toned forms follow the E6 table; Dev transcribes all 134 entries with a unit test that
round-trips every entry against a second independent source, e.g. libiconv `VISCII` → UTF-8.)

**VNI Windows sequences (E7, `UNVERIFIED`):** tone bytes `huyền F8, sắc F9, hỏi FB, ngã F5, nặng EF`
(upper `D8 D9 DB D5 CF`). Circumflex `E2` (`â = 61 E2`, `ê = 65 E2`, `ô = 6F E2`); breve `EA`
(`ă = 61 EA`). Circumflex+tone combined bytes: `ầ E0, ấ E1, ẩ E5, ẫ E3, ậ E4` (same for ê/ô);
breve+tone: `ằ E8, ắ E9, ẳ FA, ẵ FC, ặ EB`. Single-byte letters: `ơ F4, Ơ D4, ư F6, Ư D6, đ F1,
Đ D1, ì EC, í ED, ỉ E6, ĩ F3, ị F2, ỵ EE, Ỵ CE`. Horn + tone = horn byte + tone byte (`ớ = F4 F9`,
`ứ = F6 F9`). Verification: second source (VNI Windows font cmap dump of `VNI-Times`, or EVKey's
"Chuyển mã" tool output hex-dumped) must match all 134 before freeze.

`delete` for multi-unit charsets counts **units sent**, not letters (CCR-3).

### A12. Uppercase / auto-capitalize

- Case of an output letter = case of its **base** keystroke. Triggers never change case:
  `AS → Á`, `As → Á`, `aS → á`; `AA → Â`, `Aa → Â`, `aA → â`; `DD/Dd → Đ`, `dD → đ`; `W → Ư`, `w → ư`.
- CapsLock is already reflected in `KeyInput.ch`; engine never reads CapsLock separately.
- Restore (A10) emits raw with original per-key case.
- Macro case per A8.
- **auto_capitalize** (`tự động viết hoa sau dấu câu`): state `cap_next` set by `. ! ?` followed by
  space(s), by Enter, and on `boundary()` caused by focus change into an empty field — the last one is
  `UNVERIFIED` (cannot know field is empty) → **not** set on focus change. When `cap_next` and the next
  key is a lowercase letter: `Commit(uppercase)` (swallow + type). Cleared by any non-space key.
  Not applied in `raw_zone`, after `...` (ellipsis: 2+ dots) or after a digit-dot (`3.5`).

### A13. Performance contract (algorithm level)

| Operation | Bound | Allocation |
|---|---|---|
| Classify key, push to raw | O(1) | none (fixed `[RawKey; 32]`) |
| Syllable split | O(n), n ≤ 32 but V ≤ 3, C1 ≤ 3, C2 ≤ 2 → effectively O(1) | none |
| Tone placement | table lookup on V (≤ 3) | none |
| Render word → NFC | ≤ 32 table lookups | none (`[char; 32]`) |
| Encode → charset | static `[u16;134]` / `[&[u8];134]` tables | none |
| Diff vs emitted | O(32) | none |
| Build `EngineAction` | — | `Noop`: none. `Commit/Replace`: one `String` ≤ 96 B. To meet P1 "zero heap alloc on steady path", **steady path = `Noop`** (≈ 60 % of keys: plain letters that don't modify). CCR-6 proposes `SmallString<[u8; 64]>` to remove the alloc entirely |
| Macro lookup | O(1) hash, commit keys only | none (`Arc<str>` clone) |
| `set_options` / macro reload | O(entries) | allowed, off the key path |

Corpus pins (QA `50-test-cases.md`, ≥ 800 cases, `vkb-cli simulate`): every row of A1 escape table,
every A4 row × {modern, classic}, tone migration, W-rules 1–7, auto-horn, A7 steps 1–5, A8 case rules,
A9 raw_zone, A10 examples, A11 round-trip for all 134 letters per charset, A12 case table. Bench:
criterion over the corpus (100k iterations), report p50/p99 and `dhat` alloc count = 0 for `Noop`.

---

## B. Config schema review

### B.0 Contract change requests (CCR) found while specifying

| CCR | Problem | Proposal |
|---|---|---|
| CCR-1 | Contract §5 comment `old_tone_style = false # gõ dấu kiểu cũ (oà / uý)` is **inverted**: classic (kiểu cũ) is `òa / úy`; modern (kiểu mới) is `oà / uý` | Fix comment; see B.1 naming |
| CCR-2 | `EngineOptions` is `Copy`; macro table cannot reach the engine | add `fn set_macros(&mut self, table: Arc<MacroTable>)` to `VietnameseEngine` |
| CCR-3 | `Replace.delete_graphemes` — apps delete code units per Backspace; NFD/VNI need unit counts | rename `delete_units: u32` = number of `VK_BACK` to send; doc-comment it |
| CCR-4 | No way to "replace, then let the commit key through" (macro/restore on Enter must send real Enter, not `\n`) | add `pass_through: bool` to `Replace`, or variant `ReplaceThenPass{..}` |
| CCR-5 | `Charset` lacks `bkhcm1`, `vps`, `unicode_cp1258` | add when verified (Sprint 2) |
| CCR-6 | `String` in actions allocates | `SmallString` / `ArrayString<96>` |
| CCR-7 | `EngineOptions` lacks fields in B.2 (`free_marking`, `macro_active_when_disabled`, ...) | extend struct |
| CCR-8 | Default `restore_word = "Ctrl+Shift+Z"` collides with Redo in most apps | keep default (EVKey parity `UNVERIFIED`) but A10 passes it through when nothing to restore; UI shows warning badge |
| CCR-9 | `Hotkey` grammar needs lone-modifier and side-specific keys (`RCtrl`, `Ctrl+Shift` release-trigger) | define `HotkeyKey::None` + `side` flags |

### B.1 Tone style naming

Recommendation: TOML `tone_style = "classic" | "modern"` (`kiểu cũ: òa, úy` / `kiểu mới: oà, uý`),
migrated from `old_tone_style` (true → classic). `EngineOptions.old_tone_style` stays as the internal
bool. Rationale: a bool named "old" reads as deprecated and its comment is already wrong (CCR-1).
Default: **classic** — `UNVERIFIED` against EVKey's shipped default (method: fresh EVKey install,
read the checkbox state).

### B.2 Additions (TOML key · type · default · EVKey label)

Labels marked † are verbatim from E2; ‡ owner-provided in brief, not yet seen in EVKey; § `UNVERIFIED`.

| Section.key | Type | Default | Label / note |
|---|---|---|---|
| `typing.tone_style` | `"classic"\|"modern"` | `"classic"` | replaces `old_tone_style` |
| `typing.free_marking` | bool | true | `bỏ dấu ở cuối từ` ‡ — allow tone/mod keys after C2 (`hoangf`, `tienge`) |
| `typing.auto_horn_uo` | bool | true | `ưo → ươ` auto (A1) § label |
| `typing.macro_active_when_disabled` | bool | false | `Vẫn cho phép gõ tắt khi tắt tiếng Việt` † |
| `typing.macro_case_follow` | bool | true | macro output follows trigger case (A8) § label |
| `typing.raw_zone` | bool | true | URL/code protection (A9) § label |
| `input.send_method` | `"keyboard"\|"clipboard"\|"ime"` | `"keyboard"` | `Sử dụng Clipboard để gửi phím` † maps to `clipboard`; `ime` reserved (TSF, not Sprint 1) |
| `compat.browser_suggestion_fix` | `"off"\|"fix1"\|"fix2"` | `"fix1"` | `Sửa lỗi dấu gợi ý (Browsers, Excel,...)` † (two variants, E3/E4). Mechanism `UNVERIFIED`: fix must dismiss the autocomplete selection before backspaces (candidate: inject a no-op char then delete it; method: Chrome omnibox with history `google.com`, type `goo` + Telex tone) |
| `compat.metro_support` | bool | true | `Hỗ trợ ứng dụng Metro` † |
| `compat.clipboard_apps` | `[string]` | `[]` | per-process force clipboard send |
| `compat.send_delay_us` | u32 | 0 | inter-event delay for slow apps § |
| `apps.rules[].method` / `.charset` / `.send_method` | optional overrides | absent | per-app override (DoD 4) |
| `apps.smart_switch` | bool | false | remember on/off per app — label `UNVERIFIED` (EVKey "chuyển chế độ thông minh"?) |
| `system.run_as_admin` | bool | false | `chạy với quyền Admin` † (needed to hook elevated apps) |
| `system.show_window_on_start` | bool | false | `hiển thị hộp thoại khi khởi động` † — inverse alias of `start_minimized`; keep one: drop this, map label to `!start_minimized` |
| `system.notify_on_toggle` | bool | false | toast "Đã bật/tắt tiếng Việt" |
| `system.tray_click_action` | `"toggle"\|"open_settings"\|"menu"` | `"toggle"` | single left-click |
| `system.tray_double_click_action` | same | `"open_settings"` | |
| `system.check_updates` | bool | false | opt-in (charter non-goal telemetry; update check only) |
| `system.overlay_on_toggle` | bool | true | small V/E overlay near caret 600 ms (state visible) |
| `hotkeys.macro_toggle` | hotkey | `""` | |
| `hotkeys.convert_clipboard` | hotkey | `""` | Sprint 2 conversion tool |
| `ui.sidebar_collapsed` | bool | false | |

Parity check: with B.2 applied, every option in E2 is representable. Not representable and
**out of scope** (state explicitly): EVKey's drag-locator window picker is a UI gesture, not config.

---

## C. UI requirements (behavior only)

### C.0 Global rules
- **Live apply:** every control writes via `patch_settings` debounced 150 ms; no Save/Apply/OK
  buttons anywhere. Success is silent; inline "Đã lưu" fades 1.2 s next to the changed row (optional).
  Persistence failure → non-modal toast `Không lưu được cài đặt: <lý do>` + `Thử lại`; the control
  keeps the new value in memory and shows a warning dot.
- **State always visible:** a persistent status strip (top of every screen) shows: on/off
  (`Tiếng Việt` / `English`), kiểu gõ, bảng mã, and active-app override if any
  (`Đang ở: chrome.exe — đã loại trừ`). Each item is clickable to change in place (popover, not dialog).
- Disabled controls show *why* (tooltip/inline): e.g. macro options disabled when `macro_enabled = false`.
- Every section has `Khôi phục mặc định` (calls `reset_settings(section)`), with 5 s undo toast.
- Keyboard: full Tab order, visible focus ring, `Ctrl+F` focuses search where present, `Ctrl+1..8` switches screens.
- Loading: settings are local; first paint ≤ 300 ms after window show. Skeletons only for
  `list_running_apps` and macro import.

### C.1 Tổng quan (dashboard)
Purpose: one-glance state + the 4 most-used switches.
Controls: big on/off toggle (mirrors hotkey; shows the hotkey `Ctrl + Shift`); kiểu gõ segmented
(Telex / VNI / VIQR); bảng mã select; "Thử gõ" test field (local only, never persisted, cleared on
close — C5) showing live output; hook status (`Đang hoạt động` / `Lỗi hook — Khởi động lại`);
current foreground app + its rule; shortcut chips to Gõ tắt / Loại trừ.
States: hook error → red banner with `Khởi động lại bộ gõ`; elevated foreground app and not admin →
amber note `Ứng dụng đang chạy quyền Admin — bật "chạy với quyền Admin" để gõ được`.

### C.2 Gõ tiếng Việt
Controls: kiểu gõ (radio cards with 1-line cheat sheet each), tone_style (two cards showing
`hòa / thúy` vs `hoà / thuý` live), toggles: kiểm tra chính tả, tự động khôi phục từ gõ sai,
bỏ dấu ở cuối từ, dùng j f w z làm phụ âm, tự động viết hoa sau dấu câu, ưo→ươ, bảo vệ URL/code.
Each toggle has a one-line example rendered from the engine (`convert`-style preview).
Disabled state: none (all independent). Error: n/a.

### C.3 Bảng mã
Controls: charset list (radio list with sample `Tiếng Việt có dấu` rendered in that charset's
escaped form, e.g. hex for 8-bit); compat: Sửa lỗi dấu gợi ý (off/1/2 segmented), Hỗ trợ ứng dụng
Metro, Sử dụng Clipboard để gửi phím, clipboard apps list.
Conversion tool (Sprint 2): two textareas + from/to selects + `Chuyển mã` (`convert_text`) +
`Sao chép`. Error: unsupported pair → inline error under selects.

### C.4 Gõ tắt
Controls: master toggle `Gõ tắt`, `Vẫn cho phép gõ tắt khi tắt tiếng Việt`, search box
(`list_macros(query)`, filter-as-you-type 100 ms debounce), virtualized table (trigger | expansion |
row actions), inline add row at top (trigger field → Tab → expansion → Enter saves via `upsert_macro`),
inline edit on double-click/Enter, multi-select + Delete (`delete_macros`, undo toast 5 s),
`Nhập…` / `Xuất…` (file dialog; `import_macros` returns count → toast `Đã nhập 1 234 mục, bỏ qua 3 trùng`).
Validation: empty trigger, whitespace in trigger, length > 32, duplicate (case-insensitive) →
inline error on the field, row not saved. Expansion > 4000 chars → error.
States: empty (`Chưa có gõ tắt nào` + 3 example rows to add with one click), no search results,
import error (file + line number), disabled (table dimmed when master toggle off, still editable).
Performance: 10 000 rows scroll at 60 fps (virtualization, row height fixed).

### C.5 Loại trừ ứng dụng
Controls: list of rules (icon, process name, path, mode: `Tắt tiếng Việt` / `Bật tiếng Việt` /
`Tuỳ chỉnh` with method/charset overrides), `Thêm ứng dụng` → picker listing `list_running_apps`
(search, icon, name, exe) + `Chọn tệp .exe…` + "pick by clicking a window" (Sprint 2, mirrors EVKey
locator E2). Delete with undo. Smart-switch toggle.
States: empty (explain + suggest common: games, terminals), loading running apps (skeleton),
duplicate rule (focus existing row), access denied reading a process path (show name only).

### C.6 Phím tắt
Rows: Bật/tắt tiếng Việt, Khôi phục từ, Đổi bảng mã, Đổi kiểu gõ, Mở cài đặt, Bật/tắt gõ tắt,
Chuyển mã clipboard. Each row: label, capture field, reset icon, conflict/warning badge.

**Capture field behavior (owner requirement 1):**
1. Idle: shows canonical string as key caps (`Ctrl` `Shift`), or `Chưa đặt` muted.
2. Click / Enter / Space on the field → **capturing**: field highlights, text `Nhấn tổ hợp phím…`;
   the global hook suspends hotkey actions while capturing (so pressing the current toggle doesn't toggle).
3. Live display: as modifiers go down they appear immediately (`Ctrl + Shift + …`); the first
   non-modifier completes the chord; a modifier-only chord completes on **release of all keys**
   with no non-modifier pressed (EVKey `Ctrl+Shift` style).
4. Completed chord → `parse_hotkey` → shown canonical (exact stored string, contract §7) in a
   pending state with ✓ / ✕ mini-buttons.
5. **Enter** or ✓ or **blur** commits (live-applied); **Esc** or ✕ restores the previous value;
   **Backspace**/**Delete** (bare) clears → `Chưa đặt` (for the required toggle row: refuses with inline
   `Phím bật/tắt không được để trống` and restores). Bare Esc/Enter/Backspace/Tab are never captured;
   with a modifier they are (`Ctrl+Backspace` ok).
6. Rejected chords: single printable key without modifier (`A`), Shift+printable only (types text),
   Win+L / Ctrl+Alt+Del / Alt+Tab / Win+* system-reserved → inline error, stays capturing.
7. **Conflict detection:** `check_hotkey_conflict(input, ignore_field)` against other rows → blocking
   error `Trùng với "Đổi kiểu gõ"` with action `Hoán đổi` (swaps both rows). Against a built-in list of
   common app shortcuts (`Ctrl+Z/Y/C/V/X/A/S/F`, `Ctrl+Shift+Z`, `Ctrl+Shift+T`, `Alt+Shift`
   (Windows layout switch), `Ctrl+Space` (CJK IME), `Win+Space`) → amber warning, allowed.
8. Reset icon per row → default value, live, undo toast. `Khôi phục tất cả` at section bottom.
9. Preset menu (▾ on the toggle row): 1 `Ctrl+Shift` (mặc định, EVKey), 2 `Alt+Z` (Unikey quen thuộc),
   3 `Ctrl+`` ` ``, 4 `Ctrl+Alt+Space`, 5 `Right Ctrl` (nhấn nhả, CCR-9). Choosing one = commit.
10. Toggle-on-release semantics: `Ctrl+Shift` fires on release only if no other key was pressed while
    held (`Ctrl+Shift+T` in Chrome never toggles).

### C.7 Hệ thống
`Khởi động cùng Windows` †, start minimized (`hiển thị hộp thoại khi khởi động` inverse), `chạy với quyền
Admin` † (requires restart: shows `Khởi động lại ngay`), notify_on_toggle, toggle sound, overlay on
toggle, tray click / double-click action selects, language (vi/en, live re-render), theme
(system/light/dark, live), check updates (opt-in), `Mở thư mục cấu hình` (`open_config_folder`),
export/import whole config, reset all (confirm inline, undo 10 s).
Errors: autostart registry write failure → inline error with reason.

### C.8 Giới thiệu
`app_info()`: version, arch, build date, hook state, config path (copyable), licences, privacy
statement (`Không ghi lại, không gửi bất kỳ phím gõ nào`), link to repo/issues, `Kiểm tra cập nhật`
(only if opted in).

### C.9 Tray
Icon: `V` (on, accent) / `E` (off, neutral), adapts to light/dark taskbar; tooltip
`VietKB — Tiếng Việt · Telex · Unicode`.
Left click = `tray_click_action`; double-click = `tray_double_click_action`; right click = menu:
```
✓ Bật tiếng Việt            Ctrl+Shift
──────────
Kiểu gõ        ▸  ● Telex  ○ VNI  ○ VIQR
Bảng mã        ▸  ● Unicode dựng sẵn  ○ Unicode tổ hợp  ○ TCVN3 (ABC)  ○ VNI Windows  ○ VIQR  ○ VISCII …
✓ Gõ tắt
✓ Kiểm tra chính tả
Loại trừ "chrome.exe"        (current foreground app, toggles rule)
──────────
Bảng gõ tắt…                  (opens Gõ tắt screen)
Mở cài đặt…
Chuyển mã…                    (Sprint 2)
──────────
Thoát
```
Every menu change applies live and updates the settings window if open.

### C.10 First run
1. Install → first launch → window opens on Tổng quan with a 3-step inline card (not a wizard modal):
   kiểu gõ (Telex preselected), test field, hotkey reminder. `Xong` dismisses permanently.
2. Detect running Unikey/EVKey/OpenKey (`list_running_apps`) → amber banner `Đang có bộ gõ khác chạy
   (UniKey.exe) — tắt nó để tránh gõ sai`.
3. Legacy `settings.ini` import (contract §5) → toast `Đã nhập cài đặt cũ`.
4. Subsequent launches: tray only (start_minimized).

---

## D. Acceptance criteria

Notation: `typed → expected` via `vkb-cli simulate` unless stated. Default options unless stated.

**Telex (A1)**
- AC-1 `tieengs → tiếng`; `vieetj → việt`; `dduwowngf → đường`; `nguoiwf → người`.
- AC-2 Every row of the A1 escape table produces the listed result, incl. sticky (`asss → ass`).
- AC-3 W-rules 1–7 each have ≥ 3 corpus cases; `w → ư`, `tw → tư`, `quow → quơ`, `aaw → ă`.
- AC-4 Tone key with no vowel is literal: `str → str`, `x → x`.
- AC-5 `jfwz_as_consonant = true`: `w → w`, `wa → wa` (no `ư`).
- AC-6 Auto-horn: `thuwong → thương`; `huow → hươ`; `huo + w` without final → `hươ`; `thuowr → thuở`.
- AC-7 `spell_check = true`: `tacf → tacf`; `false`: `tacf → tàc`.

**VNI (A2)**
- AC-8 `tie61ng → tiếng`; `d9u7o7ng2 → đường`; `nguo7i2 → người`.
- AC-9 `a11 → a1`; `d99 → d9`; `2026 → 2026`; `1a1 → 1a1`.
- AC-10 Numpad digits never transform (`a` + Numpad1 → `a1`).
- AC-11 `0` removes tone (`a10 → a`).

**VIQR (A3)**
- AC-12 `vie^.t → việt`; ``ddu+o+`ng → đường``; `a(' → ắ`.
- AC-13 `nha. → nhạ`; `nha.. → nha.`; `ok. → ok.`; `nha\. → nha.`.
- AC-14 Each `UNVERIFIED` VIQR key (`* - ) 0`) has a recorded EVKey observation before Gate 1.

**Tone placement (A4)**
- AC-15 Each T1–T7 row example passes in both styles with the listed difference only in T6.
- AC-16 Migration: classic `hoaf → hòa`, `hoafn → hoàn`; modern `hoaf → hoà`.
- AC-17 `quys → quý` and `gias → giá` in both styles; `gif → gì`; `ginf → gìn`.
- AC-18 `thuees → thuế` in both styles.

**Modified letters (A5)**
- AC-19 Order independence: `aas = asa = ấ`; `ows = osw = ớ`; `ấ + w → ắ`.
- AC-20 All 134 letters reachable in each method (generated test: for every letter, a canonical key sequence per method).

**Boundary / commit (A6)**
- AC-21 After any boundary event in the A6 table, the next key starts a new word (`ab`, click, `s` → `s` literal not tone).
- AC-22 I1: fuzz 1 M random key sequences (letters, BS, commit keys, boundaries): `delete ≤ units(emitted)` always; zero panics.
- AC-23 I2: first key of every word appears in output in 100 % of fuzz cases.
- AC-24 Ctrl+C / Ctrl+V / Alt+Tab pass through untouched and reset the word.
- AC-25 E2E (Windows): 500 words in Notepad, Chrome, VS Code, Excel, Zalo — 0 stray backspaces, 0 lost first chars (QA diff vs expected text).
- AC-26 A word of 33 letters commits at 32 and continues as a new word without loss.

**Backspace (A7)**
- AC-27 `tieengs` BS → `tiến`; then `g` → `tiếng`.
- AC-28 `hoas` BS → `ho`.
- AC-29 `nguoiwf` BS BS → `ngư`; then `oi` → `ngươi`.
- AC-30 One-step resume: `chaof␠` BS → current word is `chào` again; then `x` → `chão`. After a click between, BS does not resume.
- AC-31 BS on empty buffer → `Noop`, no state change.
- AC-32 NFD: one BS removes exactly one composed letter in Notepad and Chrome (after CCR-3).

**Macro (A8)**
- AC-33 `btw␠ → by the way␠`; `abtw␠ → abtw␠`.
- AC-34 `BTW␠ → BY THE WAY␠`; `Btw␠ → By the way␠`.
- AC-35 Trigger `đc` matches typed `ddc␠`.
- AC-36 Macro on Enter in a chat app sends expansion then a real Enter (message sent once) — after CCR-4.
- AC-37 Vietnamese off + `macro_active_when_disabled = true` → expands; `false` → not.
- AC-38 Macro not expanded in an excluded app.
- AC-39 10 000 macros: commit-key latency p99 ≤ 50 µs.
- AC-40 Validation rejects empty, whitespace, > 32-char, duplicate triggers with inline error.

**Spell (A9)**
- AC-41 Valid-set check: 100 % of a 6 000-syllable Vietnamese word list validates; 0 of 200 listed invalid syllables validate.
- AC-42 `github.com/user`, `me@host`, `a_b`, `x=y` produce no transforms after the symbol.
- AC-43 `iPhone` and `getX` unchanged.

**Restore (A10)**
- AC-44 Auto: `google␠ → google␠`; `windows␠ → windows␠`; `tesst␠ → test␠`.
- AC-45 Off: `google␠ → gôgle␠`.
- AC-46 Manual hotkey on current word restores raw; on just-committed word restores previous; with nothing to restore, `Ctrl+Shift+Z` reaches the app (Redo works in Notepad++/VS Code).
- AC-47 Second hotkey press re-applies Vietnamese.

**Charset (A11)**
- AC-48 NFC: all 134 letters equal Unicode NFC of their NFD form (`unicode-normalization` check).
- AC-49 NFD: output is canonical NFD (`is_nfd` true) for all 134; `ệ = 65 0323 0302`.
- AC-50 TCVN3: all 67 lowercase + 7 uppercase untoned match the A11 table byte-for-byte.
- AC-51 VISCII: all 134 match E6.
- AC-52 VNI Windows: all 134 match a second independent source before freeze (else defect).
- AC-53 VIQR: `việt → vie^.t`, `đường → ddu+o+`ng`.
- AC-54 Typing `Tiếng Việt` in Word with `.VnTime` (TCVN3) and `VNI-Times` (VNI) renders correctly.
- AC-55 Each `UNVERIFIED` charset item has a recorded verification result or is marked out-of-scope.

**Case (A12)**
- AC-56 Case table in A12 passes.
- AC-57 `auto_capitalize`: `xin chào. tôi → xin chào. Tôi`; `3.5 kg` unchanged; `...a` unchanged.

**Performance (A13 / P1–P7)**
- AC-58 Criterion: `process_key` p99 ≤ 50 µs; `dhat` shows 0 allocations on `Noop`.
- AC-59 P2 e2e p95 ≤ 8 ms, p99 ≤ 15 ms.
- AC-60 P3 ≤ 20 MB, P4 ≤ 90 MB, P5 ≤ 0.1 %, P6 ≤ 400 ms, P7 ≤ 10 MB — measured per charter.

**Config (B)**
- AC-61 Every key in B.2 round-trips (write → read) with default and non-default values.
- AC-62 Unknown keys survive a write (charter guard).
- AC-63 `old_tone_style = true` in a v1 file migrates to `tone_style = "classic"`.
- AC-64 Each B.2 option changes engine/backend behavior as described (one test each).
- AC-65 `send_method = "clipboard"`: previous clipboard (text + one image + one file list) is restored exactly after each injection.
- AC-66 `browser_suggestion_fix = fix1`: Chrome omnibox with history entry, typing `google` Telex-free + Vietnamese word → no duplicated letters (10/10 trials).
- AC-67 `metro_support`: typing works in Windows Mail / Store app text box.
- AC-68 Per-app override: `excel.exe` charset override = TCVN3 while global = Unicode.
- AC-69 `run_as_admin = false` + elevated foreground app → dashboard shows the admin note.

**UI (C)**
- AC-70 Changing any control persists to `config.toml` within 500 ms without pressing anything else; no Save button exists in the DOM.
- AC-71 The status strip reflects a hotkey toggle within 100 ms while the window is open.
- AC-72 Persistence failure (read-only config file) shows the toast and keeps the UI value.
- AC-73 Hotkey capture: steps 1–10 of C.6 each have a test (focus, live display, Enter, Esc, Backspace, blur, reject, conflict + swap, reset, preset).
- AC-74 While capturing, pressing the current toggle hotkey does not toggle Vietnamese.
- AC-75 `Ctrl+Shift+T` never toggles; `Ctrl+Shift` alone toggles on release.
- AC-76 Gõ tắt: search over 10 000 rows returns in ≤ 50 ms; scroll stays ≥ 55 fps.
- AC-77 Import of a 5 000-line file reports exact counts; malformed line reported with line number.
- AC-78 Loại trừ: adding from the running-apps picker creates a rule effective on next keystroke in that app.
- AC-79 Tray menu changes reflect in the open settings window within 100 ms and vice versa.
- AC-80 Tray icon shows `V`/`E` correctly in light and dark taskbar.
- AC-81 First run shows the inline card once; conflicting IME banner appears when UniKey.exe runs.
- AC-82 Language switch vi↔en re-renders without restart; product terms (kiểu gõ, bảng mã, gõ tắt) remain Vietnamese in the English UI.
- AC-83 Theme switch applies live; all text/background pairs meet WCAG AA (see `40-design-review.md`).
- AC-84 Full keyboard navigation reaches every control; focus ring visible on each.
- AC-85 "Thử gõ" field content is never written to disk or logs (grep config/log dirs after use).
