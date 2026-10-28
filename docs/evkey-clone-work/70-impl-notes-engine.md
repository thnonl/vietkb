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
