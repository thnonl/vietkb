# Impl notes — engine (`vkb-core`, `vkb-cli`)

Owner: engine dev, branch `feat/engine`. Scope: `crates/vkb-core/**`, `crates/vkb-cli/**`.
Contract (`25-contract.md`) and `vkb-config` untouched.

---

## 1. Shape of the engine

The word is a **pure function of the raw keystrokes**. `method::compose` replays the
buffer from key 0 on every `process_key` and produces `Composed { letters, tone, literal,
escaped }`; `engine::Engine` renders that to the target charset and diffs it against the
text it last put on screen, which yields one of:

| situation | action |
|---|---|
| the key just appends its own character (the steady path) | `Noop` — the app types it |
| the word gained one character that differs | `Commit(that char)` |
| the word was rewritten (tone moved, mark applied) | `Replace { delete, insert }` |
| the key was consumed and nothing visible changed | `Swallow` |

Why replay instead of incremental mutation:

- **backspace over the raw keystroke history** is free and always correct
  (`tieengs` → `tiếng`, ⌫ → `tiêng`, ⌫ → `tiên`, ⌫ → `tiê`);
- **the tone re-places itself as the syllable grows** (`hoà` + `n` → `hoàn`), which an
  append-only engine gets wrong;
- `restore_wrong_word` and the `Phục hồi từ` hotkey are the same function of the raw
  buffer.

Cost is O(32) per key with everything in inline `SmallVec`s — measured well under the
P1 budget (§6). `Noop` returns **before** any composition work for ignored keys
(key-up, Ctrl/Alt/Win chords, non-text keys) so that path provably does not allocate.

`delete_graphemes` is in **charset units**: Unicode scalar values for the Unicode
charsets, bytes for the legacy 8-bit ones. For NFC (the default) that is one per
grapheme. The count is always exactly what the engine emitted, so
"delete N, insert S" is self-consistent for the backend. Documented on the enum.

## 2. Tone placement (`tone.rs`)

One tone per word, position recomputed at render time:

1. a vowel carrying a modification mark wins; of several, the last (`ươ` → `ơ`);
2. three-vowel nuclei take the middle vowel (`oai`, `uyu`, `uyê`, `ươi`);
3. `oa`, `oe`, `uy` take the last vowel when a coda follows, and — modern style only —
   the first when the syllable ends in the vowel (`hòa`/`hoà`, `thủy`/`thuỳ`);
4. otherwise the first vowel (`thái`, `mùa`, `gửi`);
5. `qu` and `gi` are onsets: their `u`/`i` is skipped when another vowel follows
   (`quá`, `già`, `giữ`) but kept when it is the only vowel (`gì`).

Rule 1 is what makes `thuật`, `tuệ`, `cuộc` place correctly without special cases.

## 3. Method semantics

**Telex** — `s f r x j` tones, `w` (breve/horn, `uow` marks both), doubled
`a e o` (circumflex), doubled `d` (đ). Tones escape when typed twice: the accent drops
and the key becomes literal (`ass` → `as`), matching the reference implementation. A
mark key that cannot apply is appended literally and keeps the word
(`aww` → `ăw`, `aaa` → `âa`, `ddd` → `đd`).

**VNI** — `1..5` tones, `6` circumflex, `7` horn, `8` breve, `9` đ, `0` strips marks and
tone. Digits only join a word when the word is empty (numeric run) or when the raw prefix
is e-mail shaped (`pndquyens203@…`).

**VIQR** — `^ ( +` modifiers, `' \` ? ~ .` tones, `dd` or `d-` for đ, `\` escapes the
next character. The escaped character is typed by the engine (`Commit`) and *ends* the
word, so `ngày\.` produces `ngày.`.

**`f j w z` as consonants** — treated as consonants in the onset only (no vowel yet in
the word), which is what makes `fa`, `jazz`, `zalo` typeable. Two shapes are kept from
the reference implementation as an explicit compat shim: raw `of` and `oj` stay literal.

## 4. Spell check and restore

`spell_check` is a **structural** test (onset + nucleus + coda, `spell.rs`), backtracked
over the onset so `gi` + `ương` parses as an onset plus `ương`. As soon as the composed
word stops being a well-formed syllable, the word is shown as the **raw keystrokes** —
which is what `Kiểm tra chính tả` means, and what makes `google`, `windows`, `install`
and `Microsoft` typeable without a dictionary. `restore_wrong_word` then keeps that raw
form at the boundary instead of re-deriving it.

The cost is the documented EVKey behaviour: a typo also reverts the accents
(`tieengk` shows `tieengk`, `nghiễngq` shows `nghieengxq`) until the offending key is
deleted. Keeping the accented text instead would need the dictionary EVKey has.

Known gaps:

- no dictionary, so no `hỏi`/`ngã` word-sense check and no English-word list (EVKey has
  both);
- `oo` is deliberately **not** a valid nucleus: `xoong`/`boong` (loanwords) stay raw.
  Including it makes `google` show `gô` for its first three keys before the word reverts
  to raw, which looks broken;
- `restore_wrong_word` keeps the reference implementation's shape list
  (`of oj awf owf uwf aww oww uww`) — a real fix needs a dictionary;
- `free_tone` (`Đặt dấu tự do`) has no `EngineOptions` field, so legacy "type the tone
  first, place it later" behaviour is not selectable; the placement is always the smart
  one.

## 5. Charsets (`charset.rs`)

Nine charsets, each carrying a `TableStatus` and a provenance note:

| charset | status | source |
|---|---|---|
| `unicode` (NFC), `utf8` | Verified | 146-letter table pinned to Unicode names in `tests/letter_table.rs` |
| `unicode_nfd` | Verified | canonical decomposition, mark before tone (U+0302/0306/031B then U+0301…) |
| `viscii` | Verified | RFC 1456 Table 1, cross-checked against the VietUnicode chart |
| `viqr` | Verified | RFC 1456 Table 2 |
| `vps` | Documented | VietUnicode VPS chart and the Wikipedia `VPSKeys` table agree on all 73 letters, both cases |
| `tcvn3` (VSCII-3) | Documented | 73 lowercase + 7 uppercase are VSCII-3 proper; uppercase *toned* letters use the chart's `uppercase base + lowercase tone byte` pair (`À` = `41 B5`) |
| `vni_windows` | Documented | VietUnicode VNI byte pairs agree with the Wikipedia combining table; not yet validated against a VNI font |
| `bk_hcm1` | **Unverified** | no primary source found → emitted as the VNI Windows layout, assumption pinned by a test |

Per case the table holds 73 letters: the 67 that every legacy charset encodes (60 toned
vowels + `ă â ê ô ơ ư đ`) plus the six plain vowels that happen to be ASCII. The 67 were
cross-checked three ways: the reference implementation's TCVN3 table, the VSCII chart and
the VietUnicode chart agree byte for byte. Every single-byte table is asserted injective.

`EngineAction` cannot carry raw bytes, so legacy charsets put each byte in a `String` as
`char::from(byte)` (the Latin-1 view the reference implementation used: `tiếng` → `tiÕng`
in TCVN3). `Charset::encode(&str) -> Vec<u8>` gives the real bytes for the convert tool;
the Tauri `convert_text` command must use it. Boundary characters the user types pass
through the app untouched, so VIQR text typed live does not get `\.` escaping — only
converted text does.

## 6. Performance (charter P1)

`cargo run -p vkb-cli --release -- bench --keys 100000` on this machine
(`rustc 1.98.1`, Windows 11 26300, x64), corpus = words with `ươ`, `qu`, `gi` and a
coda, so every key either appends or re-writes the word:

| metric | value |
|---|---|
| p50 | 200 ns |
| p95 | 700 ns |
| p99 | **800 ns** (budget 50 000 ns) |
| max | 203 µs (single first-touch outlier; the 100k-sample p99 is 0.8 µs) |
| throughput | 3.26 M keys/sec |

`cargo bench -p vkb-core --bench engine` (criterion) reports the steady-state times per
benchmark id; see the sprint report for the numbers of this run.

`tests/no_alloc.rs` wraps the system allocator in a counter and fails if any of these
paths allocates:

- 64 × (key-up, Ctrl chord, Alt chord, non-text key) → `Noop`;
- `t`,`a`,`n`,space — the steady append plus boundary path;
- VNI `0` with nothing to strip (`Swallow`).

It also asserts that a `Replace` allocates exactly once (its insert `String`), which
proves the counter is live rather than silently broken. Everything else lives in inline
`SmallVec`s (`raw: [char; 32]`, `letters: [Letter; 32]`, rendered text: `[char; 128]`).

## 7. Additions to the public API (no contract change needed)

`25-contract.md` §3 freezes the `EngineAction` enum, `EngineOptions`, the trait and
`KeyInput`. Everything below is *additive* on the concrete `Engine` type or in new
modules, so no other worktree is affected:

- `Engine::set_macros(MacroTable)`, `upsert_macro`, `macros()`, `restore_word()`,
  `current_text()`, `buffer()`;
- `vkb_core::MacroTable` (sorted, case-insensitive, 5000+ entries, no allocation on
  lookup) — the macro table has to live somewhere and `EngineOptions` only carries the
  on/off flag;
- `vkb_core::simulate(options, input)`, `simulate_keys`, `apply_action` — the harness the
  corpus and `vkb-cli simulate` share;
- `Charset::encode`, `status`, `status_note`, `is_multibyte`; `charset::TableStatus`;
- `KeyInput::key(vk, ch)`, `backspace()`, `enter()`, `space()` and the `VK_*` constants;
- `letters`, `tone`, `spell`, `method`, `charset`, `macros` are `pub` modules so QA can
  assert on the tables.

**Proposals to the Lead** (not applied):

1. `EngineOptions` would benefit from `free_tone: bool` and a `charset_convert` helper on
   the config side; today `free_tone` exists in config but has no engine field.
2. The macro table should be pushed to the engine from `vkb-app`/`vkb-config` via the new
   `set_macros`; no trait change is required, but the platform layer must call it when
   config changes.
3. `EngineAction::Replace`'s `delete_graphemes: u32` is fine, but the backend must
   interpret it as "charset units" for the legacy charsets (see §1). This is a semantic
   note, not a signature change.

## 8. Deliberate deviations from the reference implementation

The reference is evidence, not a contract. Where it and the documented behaviour differ,
the documented behaviour won and the case is pinned by a test:

| case | reference | here | why |
|---|---|---|---|
| VNI `8` | routed through the Telex `w` rule, so `thuong82` → `thường` | `8` is the breve; `thu7o7ng2` → `thường` | VNI documents `8` as breve, `7` as horn; the old behaviour produced `ơ` for `8` |
| invalid syllable | silently transformed | reverts to raw keystrokes | `Kiểm tra chính tả`, and the only way to type English without a dictionary |
| English words | hardcoded 250-word dictionary | structural spell check | no dictionary in scope; same visible result for the corpus |
| `w` alone | literal `w` | literal `w` | UniKey's `w` → `ư` shortcut is not in the evidence; QA should decide |

## 9. Corpus

`crates/vkb-core/tests/`:

- `corpus_telex.rs`, `corpus_vni.rs`, `corpus_viqr.rs` — the reference cases ported to the
  frozen API plus the sprint-1 additions (escapes, boundaries, digits, e-mail shapes,
  uppercase propagation, old tone style);
- `tone_placement.rs` — the placement matrix (`qu`/`gi`, `ươ`, `oai`, `uyê`, two-vowel,
  coda movement, old vs modern);
- `editing.rs` — backspace over raw history, restore-word, boundaries, auto-capitalise,
  `MAX_WORD`, macros;
- `charset.rs` — exact byte sequences for `tiếng`/`Việt`/`đường` in all nine charsets, the
  full 146-code-point coverage matrix, table status;
- `letter_table.rs` — all 146 letters against their Unicode code points;
- `spell_restore.rs`, `no_alloc.rs`, `paragraphs.rs` (golden paragraphs, per word).

No I/O, no sleeps; the whole suite runs in well under a second.

## §10 D10 — auto-restore English and code (0.2.8)

Setting `typing.auto_restore_english` (`EngineOptions::auto_restore_english`, default **on**, also
for a 0.2.7 config without the key). Independent of `spell_check` and `restore_wrong_word`.
`app_kind` (`Neutral` | `Code`) is set by the platform layer on a focus change
(`appkind::classify_process` on Windows, `classify_bundle` on macOS; one pure table).

**At a word break** (space, Tab, Enter, punctuation, digit) `auto_restore` runs. Only a word whose
composed text differs from the typed keys is a candidate.

1. *English.* The typed letters (lowercase) must be in `data/english.txt` (about 1,600 words, only
   those Telex changes; SCOWL ≤ 60 + tech + loanwords + the doubled-tone-key class below). One score
   decides (`lexicon::english_beats_vietnamese`, decades of log10, positive = English):
   English frequency by SCOWL level (−2.8 … −5.0) minus the Vietnamese candidate's corpus frequency
   (a syllable seen < 4 times counts 0.3), plus a left-context term, plus the app:

   | left | rule |
   |---|---|
   | Start (line / sentence start) | protected words stay. Neutral app: flips only when the Vietnamese reading is absent from the corpus list and the English word is common (`if is or of us`); `Chaos Lens Hair Seen Chars` stay `Cháo Lén Hải Sên Chá`, `Mix`/`Max` stay `Mĩ`/`Mã`. Code app: −1.0, +3.5, weak −3 |
   | Code (after `= ( [ { < > + * % & \| !`) | +3.0, code app +3.5; protected words only for a programming word in a code app |
   | Clause (after `, ; :`) | −1.0; code app +1.0; weak −3 |
   | English or code | +3.0; code app +3.5 |
   | Vietnamese word | **a missing pair is neutral**; a pair seen in the corpus pulls toward Vietnamese; base −2.5; loanword +1.5; code app +0.8; a known English collocation (`hay mix`, `tuổi teen`, `nhóm/kiểu/cách teen`, `thích mix`; 6 hashes in `lexicon.rs`) +3.5 and beats protection |

   **Protected words** (Vietnamese syllable seen ≥ 100 times: `có tên nó thể sẽ ít cả lén …`) flip only
   on positive English evidence: a known collocation, or a code context (left is a code operator or
   bracket: `x = cos`, `foo(max`) in a code app with a programming word (tag `tech` in
   `english.txt`). At a sentence or line start they stay Vietnamese **in a code app too** (a comment
   may start with `Có`); `cos(x)` and `Math.cos` stay code through the shape rules (`(` glued to a
   word, `.` chain). After a Vietnamese word, after an English word, after a comma, at a sentence
   start they stay Vietnamese (`Mình teen` is not
   a thing; `test có xanh`, `hook sẽ chạy`, `staging ít nhất` keep their Vietnamese). A weak English
   form (plural/inflected, level > 2, not a loanword) never beats a real Vietnamese word after
   English either (`test mô phỏng` ≠ `moo`).

   `Left` is stored as numbers (`PrevWord { kind, count, hash }`): the previous word is English when
   it was restored, froze as raw, or is a function word; Vietnamese when its syllable key is in
   `vi.txt`. Only a plain space keeps the context; Enter, other punctuation, Backspace over the
   space, focus change, `boundary()`, `set_options` and macros cut it.

2. *Doubled tone key.* English words typed with `ss ff rr xx jj` (`pass`, `message`, `error`,
   `coffee`, `possible`) are swallowed by the Telex undo (`pas`, `mesage`). `english.txt` also lists
   those words (SCOWL size ≤ 20, plus `escape-allow.txt`); the engine restores them **mid-word**, as
   soon as the typed keys are such a word (not after a Backspace edit), and at the word break.
   Not restored, so the undo idiom still types a literal: `ass`→`as`, `iss`→`is`, `lass`→`las`
   (a collapsed form that is a common English word, or on `escape-deny.txt`: `bass` is pinned by
   E-TE-002).

3. *Code shapes always keep the typed keys.* "Could this be a Vietnamese word" is decided by the
   **spelling**, not by a corpus count (`spell::is_plausible_syllable`: well-formed, real rime, a
   Vietnamese onset (not `f j w z`), a stop coda only with sắc or nặng) or a corpus count > 0:
   - `_`, `@` or `(` right after a composed word: restored unless the word is plausibly Vietnamese and
     not an English word (`is_empty`, `as_str` restore; `ngỗng_to`, `bật_` stay).
   - a digit after a word of ≥ 3 keys that is not plausibly Vietnamese (`utf8`, `h264`; `Phường1`,
     `Quận3` stay). Telex only: in VNI a digit is a tone key.
   - `.`, `->`, `::` need lookahead (`PendingCode`): the word stays on screen and the next key decides;
     a letter right after a complete separator confirms code, one correction rewrites word and
     separator to the typed keys and the new word is literal (`foo.bar`, `std::io`, `a->b`).
     `.` counts only after a word that is not plausibly Vietnamese (`ngỗng.Con`, `gỏi.Có` stay).
   - a path: a token that *starts* with `/`, `~`, `./`, `C:\` or contains `://` (not a slash glued to
     a word: `xanh/đỏ/tím/vàng`, `anh/chị` are lists). Inside it a word that is not a common (seen ≥ 20)
     plausible Vietnamese word keeps the typed keys (`cd /usr/bin`, `~/Downloads`, `C:\Users`).
   - an inner capital after a lowercase letter: if the part before is a whole plausible Vietnamese
     word of ≥ 2 letters that is not an English word, and the capital is not a Telex key, the capital
     starts a new word (`ViệtNam`); in that new token capitals are never keys. A capital that is a
     key (`NguyeenX`: Shift+X tone) composes as in 0.2.7. Otherwise the word freezes as typed
     (`getValue`, `useEffect`, `HttpClient`, and every one-letter prefix: `iOS`, `macOS`, `eBay`,
     `iCloud`, `iMac`, `eKYC`).
   - `!` is sentence punctuation (a context cut like `.`); only `!=` is code, because `=` is.
   - the word after a code separator is literal, even with `raw_zone` off.

4. *Mid-word.* `spell::is_feasible_prefix` judges plain letters: onset + a prefix of one of 103
   attested rimes. It runs in the spell-check block of `compose_word`: the word goes raw on the first
   key that cannot be Vietnamese (`ord` of `order`). `feasibility_never_trips_on_a_real_syllable`
   types all 2,600 corpus syllables in Telex and VNI (tone at the end and after the nucleus) and
   compares with the feature off. Only with `spell_check` on.

5. *Undo.* An automatic correction (word break or mid-word) is kept until the next typed key. The
   restore hotkey (`can_restore`/`restore_word`) puts the Vietnamese word back, and again the English.
   A mid-word correction taken back leaves that word alone (`auto_suppressed`).

**VNI.** English letters are never altered in VNI, so the English rule is inert; digits are tone keys.
`tie61ng`, `to6i`, `co1` are unaffected.

**Lists.** `english.txt` (12 KB), `vi.txt` (22 KB), `pairs.bin` (21 KB), rime prefixes (112 `u32`).
Sources and licences: `THIRD_PARTY_NOTICES.md` at the repo root (bundled with the app) and
`crates/vkb-core/data/THIRD_PARTY.md`. Regenerate with `scripts/gen_wordlists.py`.

**Cost.** Everything is `static`; lookups are binary searches at the word break only, no allocation
(`no_alloc.rs`, and `tech_mixed_leak.rs`: the 2,833-word passage ×1,000 through one engine, no
steady-path allocation, identical output every loop, identical buffer capacities). `vkb bench`
(release): p99 about 1.5 µs against the 50 µs budget. Soak: `vkb bench --text-file <keystrokes>
--loops N --report-rss` prints peak and current RSS at start, after loop 1 and at the end (flat
from 10 to 3,000 loops).

**Long passage fixture.** `crates/vkb-core/tests/data/tech_mixed_vi.src.txt` (the text, expected output),
`tech_mixed_vi.txt` (Telex keystrokes), `tech_mixed_vi_vni.txt` (VNI), one paragraph per line, generated
by `scripts/gen_tech_fixture.py`; run by `tests/tech_mixed.rs` in both app profiles and both methods.
No pinned exceptions: all four profiles match the source text.

**Known false positives / negatives.** Loanwords after a Vietnamese word the corpora never saw
(`teen` after a rare word) can flip when the Vietnamese reading is rare; English words typed with a doubled tone key that are
not in the list stay damaged; sentences typed without a space after `.` whose first word is a rare
non-Vietnamese-looking name.
