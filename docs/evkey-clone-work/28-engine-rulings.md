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

**B3 — Only class 3 resets the word.** `FUN_140039740` handles class 3 (Enter and the other
control characters) by setting `[+0x1c] = -1`, `[+0x185] = -1`, and — unless the char is CR
`0xd` — clearing `+0xc44` and `[+0x189]` (`0x140039740`:87-94). A Backspace arriving after such
a key then hits the `[+0x1c] < 0` early return in `FUN_14003cb50`.

Charset branches still present in the engine (`[+8]+4`, `[+8]+0x14` option fields) are not
modelled: we ship Unicode only.

**B4 — EVKey *does* re-open the word after a separator, and the separator set is class 1.**
The class table built by `FUN_140036b10` fills `0..32` with class 3, then writes the 33-byte
list at `DAT_1400a28e8` — which contains space `0x20` together with every ASCII punctuation
character — as class **1**, and Tab `0x09` as class 1 as well. So space is *not* class 3.

Class 1 goes to `FUN_14003d480`, which appends a boundary entry (`*puVar1 = 1`) to the letter
array, sets the last-letter index to it, and returns 0 so the app types the character itself
(`0x14003d480`:46-60, 72-87). The word is not cleared. On Backspace, `FUN_14003cb50` pops that
boundary entry (`0x14003cb50`:75-79) and the dispatcher lets the physical Backspace delete the
separator (`0x14000a590`:40-47) — after which the previous word is the tail of the array
again, so the next key composes onto it. That is the whole reopen mechanism: no text is ever
read back from the focused control, and no extra "last word" cache exists.

Measured class-1 set: space, Tab, and `! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ `` { | } ~`.

Consequence: `Tha` + space + BS + `s` = `Thá` in EVKey, and the same holds for Tab and for
every punctuation character above. Enter (and other control characters) does not reopen.

## D8 — Reopen: required behaviour, now matched to the binary

`vkb-core` reopens the just-committed word through `CommittedWord` + `is_reopening_separator`,
which uses the class-1 set measured in B4 (space, Tab, ASCII punctuation) and refuses to store
an empty word. Owner ruling (2026-10-29): keep it — it is EVKey's behaviour, verified with VNI
tone digits (`Tha` + space + BS + `1` = `Thá`), and it must also hold for VNI modifier digits
(`6`…`9`) and for Telex letters. An earlier revision of this document claimed EVKey does not
reopen; that came from reading the `0..32` class fill and missing that the punctuation list is
written afterwards and reclaims space.

**Update (2026-11-02, owner: "Làm luôn đi"): reopen is a line history, like EVKey.** The
one-word `reopen` slot and the `parked` slot are gone. `vkb-core` keeps a bounded history
(`crates/vkb-core/src/line.rs`, `LineHistory`): a ring of 256 raw keystrokes plus a ring of 32
headers, each either a committed word (keys, `literal_from`, a hash of the on-screen text, a
pinned tone slot) or one separator mark per separator character the app typed. Every class-1
separator and digit after a word pushes the word and then a mark; a bare separator pushes a
mark. Backspace on an empty word pops a mark (the app deletes that separator) and, if the next
header is a word, rebuilds that word from its keys with the normal compose path and makes it the
current word, but only if the rebuilt text hashes to what was on screen; otherwise the line is
forgotten. Entries past the cap drop oldest-first and are beyond reach.

The line is forgotten (`clear`) by: Enter and control characters (class 3), Escape, any key with
no character except a bare modifier (arrows, Home/End, Delete, F-keys), Ctrl/Alt/Win chords,
`reset()`, `set_options` method or charset change, `boundary()`, a macro commit, a word-cap
commit, the inner-capital split, Ctrl+Shift+Z when it rewrites text, a D10 one-word-back
rewrite, a code-shape (`.`/`->`/`::`) rewrite, and any other character the history cannot
describe. Tests: `tests/line_history.rs` (owner cases, clearing, cap), and the seeded
`tests/line_reopen_generated.rs` (2,000 generated cases, Telex and VNI, oracle = fresh typing).

## D9 — Raw zone arms on the left token, not on the punctuation (owner ruling, 2026-10-30)

`20-spec.md` §A9 specified "any opener ⇒ the next word is literal". Measured on the real engine, that
made every Vietnamese word after `/ . ? _ @ : = # \` literal (`bật/tắt` → `bật/ta8t1`, `D-SC-01…04`).
Owner ruling: `/` between two Vietnamese words must not disable Vietnamese for the rest of the word.

Ruling: an opener only arms the raw zone when the token on its **left** is visibly not Vietnamese —
non-empty, plain ASCII on screen (no composed letter, no tone), and — for the three characters a
Vietnamese typist also puts between words (`. / ?`) — not a valid A9 syllable. An empty left token
(start of line, after whitespace, after a previous opener) never arms, which is what makes
`/tắt`, `bật//tắt`, `bật /tắt` compose. The decision is made on the left token's **final** text for
that boundary key (A10 restore / macro output included), at the end of `boundary_key`, and consumed
by the next `word_key` exactly as before. Full predicate, decision tables, precedence and test
contract: `20-spec.md` §A9 amendment 2026-10-30.

Consequences accepted in the same ruling (pinned by tests, none in the owner's list): a plain-ASCII
code-looking left token still
suppresses a Vietnamese right word (`abc/đường`), and a left token that happens to be a valid
syllable loses protection for `. / ?` (`ban.cof` → `ban.cò`). Backspace still deletes the opener and
kills the armed flag (`engine.rs:278`), so `Tha` `/` BS `s` = `Thá` is unchanged.

**Correction (2026-10-30, after §2b):** the "a run of openers loses protection" consequence no longer
holds — §2b chains `\ /` after `:` `\` `/`, so `http://cof` stays `http://cof` (measured; it is no
longer a declared loss). The remaining losses are L2–L5 in `20-spec.md` §9.

## Accepted nuance (QA, 2026-10-28)

`COMPAT_RESTORE_RAW` — the fallback that emits raw keystrokes when a transform cannot apply
(`of`, `aww`, …) — runs **before** the spell-check gate, so it restores even with
`spell_check = false`. That is correct and is not the `restore_wrong_word` feature: it is the
"no valid transform, so type what was typed" path, which must never depend on the spell
checker. `E-RW-005` / `E-RW-007` therefore stay as they are.


## D10 — Auto-restore English and code words (owner, 2026-11-01, for 0.2.8)

Owner: "Tôi muốn app thêm feature tự nhận biết tiếng Anh hoặc code để khi cách ra là sẽ chuyển lại thành tiếng Anh hoặc code. Làm ngay ở phiên bản tiếp theo là 0.2.8." Answers to the design questions:

- **Ambiguous words keep Vietnamese.** At a word break, the raw keystrokes are put back only when they spell an English word AND the composed text is not a common Vietnamese word. So `user`→`user`, `there` stays `thể`, `cos` stays `có`, `mix` stays `mĩ` (⌃⇧Z / Ctrl+Shift+Z still restores by hand).
- **Code is recognised by shape** (`user_id`, `getValue`, `foo.bar`, `item2`, `a->b`) and always keeps the typed keys.
- **Setting:** new option in Advanced, "Tự nhận biết tiếng Anh và code", default ON for new installs and for upgrades from 0.2.7.
- Owner: the feature must be showcased on GitHub Pages and in the hero video.

Probe on 0.2.7 (Telex, spell check on): already raw — set test case window google install string class import export; wrong today — user→ủe, use→úe, were→wể, here→hể, there→thể, user_id→ủe_id.

**D10 amendment (owner, 2026-11-01, same day):** "Đối với những chữ vừa có tiếng Việt vừa có tiếng Anh hãy xem xét trong ngữ cảnh đó nó nên là chữ gì. Ví dụ như \"Tôi hay mix nhạc chung với tiếng ồn\" thì mix là đúng chứ không phải mĩ." → ambiguous words are decided by context, not always Vietnamese. At the word break the engine weighs the composed Vietnamese word against the English word using the words already typed before it (e.g. `hay` + `mĩ` is an unlikely Vietnamese pair → `mix`; `nhà` + `có` → `có`; after an English word → English). With no usable context (sentence start), Vietnamese wins. Replaces the first bullet above.
Owner example 2 (same day): "Ví dụ như chữ \"tuổi teen\" thì teen đúng hơn là tên chẳng hạn." → `tuổi` + `tên` is an unlikely pair → `teen`. Note `teen` composes through the double-letter rule (ee→ê), not a tone key.

**D10 amendment 2 (owner, 2026-11-01):** "Hãy dựa vào cả ngữ cảnh, app đang dùng, ... tất cả ngữ cảnh sẽ ảnh hưởng đến sự lựa chọn output cuối cùng. Ví dụ như gõ if mà ở đầu câu thì nó vẫn là if, vì tiếng Việt ít mà có chữ ì ở đầu câu, đặng biệt là khi dùng trong các code idie app." → the choice is a score over ALL context signals, not a fixed default:
- word frequencies (how common the Vietnamese word is vs the English word), including position: at a sentence start a rare Vietnamese word (`ì`) loses to a common English word (`if`). This replaces "with no usable context, Vietnamese wins".
- neighbouring words already typed (Vietnamese pair likelihood, English/code before it).
- the app in front: code editors, IDEs and terminals (VS Code, Cursor, Xcode, JetBrains, Sublime, Vim, Terminal, iTerm, Windows Terminal, PowerShell, cmd, Notepad++ …) lean strongly to English/code; chat/document apps stay neutral.

**D10 amendment 3 (owner, 2026-11-01):** "Nếu có thể chuyển được ngay sang tiếng Anh khi đang gõ ví nhận biết chắc chắn như từ order, why, twitter,... chẳng hạn thì chuyển luôn, không cần phải chờ cách hoặc dấu chấm câu." → when, mid-word, the letters typed so far can no longer be any Vietnamese word and are certainly English (`ord…`, `wh…`, `tw…`), put the raw keys back immediately and stop composing for the rest of that word; do not wait for the space or punctuation. Uncertain words still wait for the word break.

**D10 performance (owner, 2026-11-01):** "Function này tôi nghĩ là khá ảnh hưởng đến performance, nên hãy tối ưu nhất có thể để khi gõ không bị delay quá." → charter P1 still binds with the feature ON, in every app profile: `process_key` p99 ≤ 50 µs, zero heap allocation on the steady path, word-break and mid-word checks included. Tables are built at compile time or once at startup off the hook thread, never on a keystroke. Report before/after numbers from `vkb bench`.

**D10 test requirement (owner, 2026-11-01):** "Thêm case gõ 1 đoạn thật dài tiếng Việt có nhiều từ tiếng Anh bồi hoặc có trích câu tiếng Anh hoặc có trích code, chủ đề về công nghệ sẽ có nhiều từ phải dùng tiếng Anh để test xem có bị leak memory không." → a long Vietnamese tech passage (loanwords, quoted English, inline code) is a fixture: engine output pinned, 1 000-loop soak with zero steady-path allocation and bounded buffers, and real-app typing QA compares process memory before/after.

**D10 amendment 4 (owner, 2026-11-01):** asked whether VietKB may correct the word just finished once the NEXT word is known ('Mix thuật' → 'Mĩ thuật', 'Max speed' keeps 'Max', 'Box công' → 'Bõ công'), owner chose "Cho sửa 1 từ trước": at the space after the next word, VietKB may rewrite exactly ONE previous word, only when confident; never after a caret move, Backspace, focus/app change, mouse click or a ⌃⇧Z on that word.

**D10 scope (owner, 2026-11-01):** "chỉ cần sửa 1 từ phía trước là đủ rồi. không cần phải cover quá nhiều đâu." → the one-word-back correction stays at exactly one word. Scope is frozen: from here only fix cases where the feature makes Vietnamese worse, or where it damages something that was right before; further English/code coverage goes to the backlog, not 0.2.8.

**D10 priority (owner, 2026-11-01):** "Dù hỗ trợ tự chuyển thành tiếng Anh nhưng vẫn ưu tiên gõ ra tiếng Việt, vì thế nếu có case nào nghi ngờ hoặc chưa đủ score thì luôn output tiếng Việt." → Vietnamese is the default output; English/code only on clear evidence. Any doubtful or below-threshold case outputs Vietnamese, including the one-word-back rewrite.

**D10 rule (owner, 2026-11-01):** "Làm gì có từ tiếng việt nào mà bắt đầu từ kí tự F bao giờ mà Fix bị sai được, chữ W, Z ở đầu cũng vậy." → a word starting with f, w, z or j (when those are consonants, the default) is never Vietnamese: it is output exactly as typed from the first key (Fix, Facebook, Wifi, Zalo, Zoom, JSON). With `jfwz_as_consonant` off, a leading w is ư and the rule does not apply to it.

**Correction (owner, same day):** "Tôi nhầm, W vẫn nên xem xét tiếng Việt chứ không hoàn toàn là tiếng Anh" → only f, z, j at a word start are never Vietnamese. A leading w goes through the normal rules (Telex `w` = ư: ưa, ước, ừ); doubt → Vietnamese.

**D10 (owner, 2026-11-01):** "the car ra the cả là sai, làm gì có từ tiếng Việt nào như thế." → after a strong English determiner/possessive (the, an, my, your, his, her, its, our, their, this, these, those, every, each, some, any), a listed English word stays English: that is clear evidence, not doubt. Non-determiners (you, in, will) keep the doubt → Vietnamese rule ("thank you cả nhà").

**D10 (owner, 2026-11-01):** "chữ í trong tiếng Việt rất hiếm có, thường thì hay ở cuối câu thôi." → `is` followed by a space is English "is"; only at a sentence end (. ! ? … Enter, closing quote/bracket) does it compose to "í".

**D10 (owner, 2026-11-02, after 0.2.8, relayed from the Windows session):** "Tiếng Việt không có từ rôm, nên gõ room phải ra room chứ." → a listed English word whose Telex composition is not a real stand-alone Vietnamese word (`room`→rôm, also candidates `soon`→sôn, `door`→dổ, `more`→mỏe, `see`→sê) comes out as typed at the word break, in any context ("meeting room", "phòng room", "room 5"). A composition that is a real word stays Vietnamese (max→mã, box→bõ, mix→mĩ, six→sĩ, tax→tã, gas→gá, bus→bú, moon→môn alone). The compounds `rôm rả` / `rôm sảy` must still be typeable (one-word-back phrase list). Target 0.2.9.

**D10 (owner, 2026-11-02, relayed from the Windows session):** "sau từ tiếng Anh mà room có nghĩa thì luôn lấy room, còn lại thì luôn là tiếng Việt. Cả engine nên xử lí theo kiểu này cho toàn bộ từ có từ trước là tiếng Anh chứ." → engine-wide context rule: when the previous word (same line) came out as English (a listed English word or code shape left as typed, not a raw typo), a current word whose keys spell a listed English word is output as typed, even if it also composes a real Vietnamese word, **only when the pair is a meaningful English phrase** ("meeting room", "mail box", "booking room"). Owner follow-up, same day: "file max không có nghĩa." → "file max" gives "file mã". The meaning test is an English bigram/phrase signal (pair attested in an English corpus); no signal → Vietnamese. Words whose keys are not a listed English word keep composing ("meeting xong", "team bỏ"). With no English word before it, ambiguous real Vietnamese words stay Vietnamese; the stand-alone rule above (room/soon/door/more/see) still applies. Target 0.2.9.

**D10 (owner, 2026-11-02, relayed from the Windows session):** "từ room có thể ghép với từ sau hoặc từ trước như thế nào để loại trừ thì phải loại trừ để hiện tiếng Việt cho đúng. Những từ nghi ngờ đều phải làm thế." → every ambiguous word (keys spell a listed English word AND compose a real Vietnamese syllable) is decided by its neighbours on BOTH sides: an attested Vietnamese pair with the previous or next word → Vietnamese ("rôm sảy", "rôm rả", "cá trê", "mã nguồn"); an attested English pair → English ("meeting room", "room service", "mail box"); neither → that word's default. The next-word side reuses the existing one-word-back correction (still exactly one word back, same cancel rules: Backspace, caret move, Enter, focus change, ⌃⇧Z); its trigger widens from the fixed phrase list to these attested pairs. This supersedes the "scope frozen" note for ambiguous words only. Pending owner: a word with both a Vietnamese and an English pair (proposed default: Vietnamese, per the priority ruling). Target 0.2.9.

**D10 (owner, 2026-11-02, relayed from the Windows session):** "1. cho phép 2. giữ như hiện tại" → (1) the Tatoeba eng/vie and Common Voice vi sentence files may be downloaded again to build the pair tables (raw files stay out of the repo); (2) English function words typed alone (this, as, or, if, has, us, there, these, too …) keep the 0.2.8 behaviour: Vietnamese by default, English only with English context on the left. The next-word side of the two-sided rule does not apply to them.

**D10 hand list (Dev, 2026-11-02, 0.2.9, applies the owner rule above to every syllable):** the stand-alone rule decides by the composed syllable at any SCOWL level (`level == 1` is gone). A plausible syllable is real when the corpus attests it (count > 0) or it is listed in `crates/vkb-core/data/real_syllables.rs` (165 keys; `// U` = doubtful, kept Vietnamese; `// P` = pinned by a test or the VI test list). Each plausible count-zero syllable that is not in that list is in the review `crates/vkb-core/tests/fixtures/d10_not_vietnamese.txt` (595 keys). The test `d10_review.rs` enumerates the Telex compositions of every listed word and of every suffix inflection (`-s`, `-es`, `-ed`, `-er`, `-ing`, `-ers`, with the stem's final `e` dropped or kept) and fails if a plausible count-zero syllable appears in neither list. Compositions the structural check rejects are English by that check, not by the list. Compound-only keys (`trê`, `tủn`, `kên`, `chủn`, `vảy`) are not standalone: alone, `tree`, `turn`, `keen`, `churn`, `vary` stay English, and their compounds keep their rules (`cá trê`, `tủn mủn`, `kên kên`, `vảy cá`, `vảy nến`, `vảy rồng`, `ngắn chủn`, `chủn ngủn`, and the left-side partners `nổi/mọc/bị rôm`, `đánh/cạo vảy`). Inflections are decided the same way; the Telex literal `uss`/`ass` is never an inflection. Owner review of the U list is pending. Also (2026-11-02, after review): an English pair never undoes a common Vietnamese reading (`tooi sex in` → `tôi sẽ in`; the one-word-back rewrite and the pair clause need `vi_count < 100`); the explicit list `max speed`, `six pack` is still trusted.
