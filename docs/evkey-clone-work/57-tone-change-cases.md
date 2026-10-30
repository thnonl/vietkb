# 57 — Tone change and partial deletion (owner report 2026-10-30)

**Owner:** QA. **Layer:** `vkb-core` only, character path (`vkb_core::simulate` / `vkb-cli simulate`) —
headless, no hook, no app, no Notepad/Chrome. **Product code changed:** none.
**Build:** `target\debug\vkb.exe` 1 042 944 B, SHA-256
`3E6B33D3303AD423AEB32A19F0EDEA3C7FC9FE167ECD634D5E765F2FDE66D17D` (HEAD `14bc655` + uncommitted
working tree, i.e. the tree that carries the §A9 amendment). **Harness:** `qa/tone/run.ps1`;
**raw output:** `qa/tone/matrix.txt` (TSV), `qa/tone/repro.txt` (`--actions` traces and engine pointers).

**Report (verbatim):** “nếu tôi muốn đổi dấu của chữ đã viết rồi, hoặc xóa 1 unit chữ có dấu để viết
lại mỗi chữ đó với dấu mới đôi lúc không thành công.”

---

## 1. The two families, operationalised

| family | shape | what “works” means |
|---|---|---|
| **A** | the word is already composed on screen and the user presses a **tone key** (`s f r x j`, VNI `1…5`) again — same key or a different one | the tone changes in place (or stays), the letters keep their marks, the word stays a word |
| **B** | the user **deletes one composed character** (`Backspace`) and types again — the tone key, the deleted letter, a letter with a new mark, an unrelated letter, or Space | the surviving text composes on; the retyped key lands as Vietnamese |

`Backspace` is fed as `KeyInput::backspace()` (vk `0x08`), the same object the hook builds, so the
engine's A7 path (replay, canonical raw rebuild, settle) is the one measured.

## 2. Method

* One `vkb-cli simulate` run per row; `<BS>` in the table is a real `0x08` argument, `<SP>` a real space.
  No row was inferred from another.
* Expected is the **NFC** string; `unicode_nfd` rows are compared after `Normalize(FormC)`, so the
  table states the letter, not the code-unit shape.
* `legacy` is **not selectable** in this build: `vkb simulate --method legacy …` exits
  `Error: unknown method "legacy"; expected one of telex, vni` (`InputMethod` has only `Telex` and
  `Vni`; `Legacy` is a serde sentinel for old config files). The legacy-adjacent dimension that does
  exist is the tone style, covered by `--old-tone-style`.
* Options exercised: `--old-tone-style` (`modern_tone`), `--no-free-tone` (`free_tone`),
  `--no-spell-check`, `--charset unicode|unicode_nfd`.
* **Source of “expected”**, per row in the `source` column: `20-spec.md` §A1 (Telex triggers and the
  escape table), §A2 (VNI triggers, digit escapes, digit collision), §A4 (tone placement), §A5
  (mark/tone are independent fields), §A6 (boundary/commit), §A7 (Backspace), §A9 (spell check);
  `50-test-cases.md` pins where they exist (`E-TE-002 ass→as`, `E-TE-012 aass→aas`, `E-TE-016`,
  `E-BS-014`, `E-TP-002/008`, `E-VN-039`). Where neither pins a shape the row cites the EVKey
  convention it uses: *a tone key whose tone is already in force changes nothing*, and *a plain
  letter is not a trigger — it appends*.

## 3. Matrix

| id | method | charset | options | keystrokes | expected | actual | verdict | source |
|---|---|---|---|---|---|---|---|---|
| A-01 | telex | unicode | - | `baajts` | bất | bất | **PASS** | A4+A9: s replaces nang, sac legal before coda t |
| A-02 | telex | unicode | - | `baajtj` | bật | baatj | **FAIL** | EVKey: the same tone key on a finished word changes nothing; A1 escape table / E-TE-002 pin only the ADJACENT repeat |
| A-03 | telex | unicode | - | `baajtf` | bậtf | bầt | **FAIL** | A1 rule 5 + A9: huyen is illegal before coda t -> trigger literal |
| A-04 | telex | unicode | - | `baajtr` | bậtr | bẩt | **FAIL** | A1 rule 5 + A9: hoi illegal before coda t |
| A-05 | telex | unicode | - | `baajtx` | bậtx | bẫt | **FAIL** | A1 rule 5 + A9: nga illegal before coda t |
| A-06 | telex | unicode | - | `basf` | bà | bà | **PASS** | A4: no coda, huyen legal |
| A-07 | telex | unicode | - | `basr` | bả | bả | **PASS** | A4: no coda, hoi legal |
| A-08 | telex | unicode | - | `basx` | bã | bã | **PASS** | A4: no coda, nga legal |
| A-09 | telex | unicode | - | `basj` | bạ | bạ | **PASS** | A4: no coda, nang legal |
| A-10 | telex | unicode | - | `bass` | bas | bas | **PASS** | A1 escape table / 50-test E-TE-002: adjacent repeat is the documented escape |
| A-11 | telex | unicode | - | `basz` | ba | basz | **FAIL** | A1: z removes the tone |
| A-12 | telex | unicode | - | `az` | az | az | **PASS** | A1: z with no tone present is literal |
| A-13 | telex | unicode | - | `toansz` | toan | toansz | **FAIL** | A1: z removes the tone of a coda word |
| A-14 | telex | unicode | - | `baajz` | bâ | baajz | **FAIL** | A1+A5: z removes the tone, the circumflex stays |
| A-15 | telex | unicode | - | `bans<BS>s` | bas | bas | **PASS** | A7 rebuilds raw bas; the repeated s is the documented ADJACENT escape (A-10 / E-TE-002 analogue) |
| A-16 | telex | unicode | - | `hoangff` | hoangf | hoangf | **PASS** | documented adjacent escape: raw hoangf is what E-TE-002 pins for a double press |
| A-17 | telex | unicode | - | `hoangfj` | hoạng | hoạng | **PASS** | A4: different tone replaces, tone on a of ang |
| A-18 | telex | unicode | - | `hoangfs` | hoáng | hoáng | **PASS** | A4: different tone replaces |
| A-19 | telex | unicode | - | `baajt<SP>f` | bật f | bật f | **PASS** | A6: the space committed the word; f starts a new word with no vowel -> literal |
| A-20 | telex | unicode | - | `baajt<SP>s` | bật s | bật s | **PASS** | A6: same, s is literal in the next word |
| A-21 | telex | unicode | - | `baajt<SP><SP>as` | bật  á | bật  á | **PASS** | A6: the tone key still reaches the new word |
| A-22 | telex | unicode | - | `baajt.f` | bật.f | bật.f | **PASS** | A6: punctuation commits; f is literal in the next word |
| A-23 | telex | unicode | - | `baajt.<BS>s` | bất | bất | **PASS** | A7 step 1: BS after a separator re-opens the word, s changes the tone |
| A-24 | telex | unicode | - | `Baajtf` | Bậtf | Bầt | **FAIL** | A12 + A1 rule 5: case of the base key, trigger still illegal |
| A-25 | telex | unicode | - | `Baajts` | Bất | Bất | **PASS** | A12: capital first letter, legal tone change |
| A-26 | telex | unicode | - | `BAATJf` | BẬTf | BẦT | **FAIL** | A12 + A1 rule 5 |
| A-27 | telex | unicode | - | `BAATJs` | BẤT | BẤT | **PASS** | A12 |
| A-28 | telex | unicode | - | `bas<BS>as` | bá | bá | **PASS** | A7: delete the toned char, retype it with the same tone |
| A-29 | telex | unicode | - | `bas<BS>af` | bà | bà | **PASS** | A7: retype the vowel, then a different tone key |
| A-30 | telex | unicode | - | `bans<BS>f` | bà | bà | **PASS** | A7 step 4: the next key re-places the tone on the surviving word |
| A-31 | telex | unicode | - | `dduowcjf` | đượcf | đườc | **FAIL** | A9: huyen illegal before coda c |
| A-32 | telex | unicode | - | `dduowcjs` | đước | đước | **PASS** | A9: sac legal before coda c |
| A-33 | telex | unicode | - | `kieemsf` | kiềm | kiềm | **PASS** | A9: m is not in {c ch p t}, huyen legal |
| A-34 | telex | unicode | - | `dduowcjj` | dduowcj | dduowcj | **PASS** | documented adjacent escape (raw minus the first trigger) |
| A-35 | vni | unicode | - | `ba65t1` | bất | bất | **PASS** | A2: 1 = sac, legal before t |
| A-36 | vni | unicode | - | `ba65t2` | bật2 | bầt | **FAIL** | A2 rule 5 + A9: huyen illegal before t -> digit literal |
| A-37 | vni | unicode | - | `ba65t3` | bật3 | bẩt | **FAIL** | A2 rule 5 + A9: hoi illegal before t |
| A-38 | vni | unicode | - | `ba65t4` | bật4 | bẫt | **FAIL** | A2 rule 5 + A9: nga illegal before t |
| A-39 | vni | unicode | - | `ba65t5` | bật | ba6t5 | **FAIL** | EVKey: the tone digit already in force changes nothing |
| A-40 | vni | unicode | - | `ba11` | ba1 | ba1 | **PASS** | A2 digit escape: same digit repeated reverts + literal |
| A-41 | vni | unicode | - | `ba12` | bà | bà | **PASS** | A2: 2 = huyen |
| A-42 | vni | unicode | - | `ba15` | bạ | bạ | **PASS** | A2: 5 = nang |
| A-43 | vni | unicode | - | `a60` | â | a | **FAIL** | A2: 0 removes the tone only |
| A-44 | vni | unicode | - | `ba650` | bâ | ba | **FAIL** | A2+A5: 0 removes the tone, the circumflex stays |
| A-45 | vni | unicode | - | `ba65t0` | bât | bat | **FAIL** | A2+A5: same with a coda |
| A-46 | vni | unicode | - | `ba10` | ba | ba | **PASS** | A2: 0 with no mark present leaves the word alone |
| A-47 | vni | unicode | - | `ba65t6` | bật6 | ba5t6 | **FAIL** | A2 rule 1: a digit with no target is literal (the 6 cannot mark an already-circumflexed a) |
| A-48 | vni | unicode | --no-free-tone | `ba65t1` | bất | bất | **PASS** | A2 + free_tone off: different tone still replaces |
| A-49 | vni | unicode | --no-free-tone | `ba65t5` | bật | ba6t5 | **FAIL** | free_tone off must not change the same-tone rule |
| A-50 | telex | unicode | --old-tone-style | `baajts` | bất | bất | **PASS** | tone style does not affect a coda word |
| A-51 | telex | unicode | --no-free-tone | `baajts` | bất | bất | **PASS** | free_tone off: tone change still works |
| A-52 | telex | unicode | --no-spell-check | `baajtf` | bầt | bầt | **PASS** | A9: with spell check off the transform applies (control row) |
| A-53 | telex | unicode_nfd | - | `baajts` | bất | bất | **PASS** | same as A-01 in NFD |
| A-54 | telex | unicode_nfd | - | `baajtj` | bật | baatj | **FAIL** | same as A-02 in NFD |
| A-55 | telex | unicode_nfd | - | `baajtz` | bât | baajtz | **FAIL** | same as A-14 in NFD |
| B-01 | telex | unicode | - | `baajt<BS>` | bậ | bậ | **PASS** | A7 step 3: the press removes t, the tone stays on a |
| B-02 | telex | unicode | - | `baajt<BS>s` | bấ | bấ | **PASS** | A7 step 4: the next key re-places the tone |
| B-03 | telex | unicode | - | `baajt<BS>st` | bất | bất | **PASS** | A7 step 4 + A4 |
| B-04 | telex | unicode | - | `baajt<BS>f` | bầ | bầ | **PASS** | A7 step 4: huyen legal on the surviving open syllable |
| B-05 | telex | unicode | - | `baajt<BS>n` | bận | bận | **PASS** | A7 step 4: an unrelated letter continues the word |
| B-06 | telex | unicode | - | `baajt<BS>b` | bậb | baajb | **FAIL** | A9: a plain letter is not a trigger, it appends |
| B-07 | telex | unicode | - | `baajt<BS>z` | bâ | baajz | **FAIL** | A1: z removes the tone of the surviving word |
| B-08 | telex | unicode | - | `baajt<BS><BS>` | b | b | **PASS** | A7 step 2-3: one displayed character per press (b-a-t: a and its tone are one character) |
| B-09 | telex | unicode | - | `baajt<BS><BS>t` | bt | bt | **PASS** | A7 step 2-3: the second press also removes a |
| B-10 | telex | unicode | - | `baajt<BS><BS>b` | bb | bb | **PASS** | A7: a plain letter continues the two-letter onset |
| B-11 | telex | unicode | - | `Nguyeexn<BS>` | Nguyễ | Nguyễ | **PASS** | A7 step 3: the press removes n, the tone char is one character |
| B-12 | telex | unicode | - | `Nguyeexn<BS>n` | Nguyễn | Nguyễn | **PASS** | A7 step 4: retype the deleted letter |
| B-13 | telex | unicode | - | `Nguyeexn<BS>nn` | Nguyễnn | Nguyeexnn | **FAIL** | A9: a plain letter appends |
| B-14 | telex | unicode | - | `Nguyeexn<BS>e` | Nguyễe | Nguyexe | **FAIL** | A1 escape table applies to a doubled LETTER key (ee), not to a letter typed after a tone key |
| B-15 | telex | unicode | - | `Nguyeexn<BS>ee` | Nguyễee | Nguyexee | **FAIL** | A9: two more letters append |
| B-16 | telex | unicode | - | `hoangf<BS>` | hoàn | hoàn | **PASS** | A7 step 3: the press removes g, tone stays on a |
| B-17 | telex | unicode | - | `hoangf<BS>g` | hoàng | hoàng | **PASS** | A7 step 4: retype the deleted letter |
| B-18 | telex | unicode | - | `hoangf<BS>s` | hoán | hoán | **PASS** | A7 step 4 + A4: sac on the surviving syllable |
| B-19 | telex | unicode | - | `hoangf<BS>f` | hoanf | hoanf | **PASS** | documented adjacent escape: the rebuilt history ends in f, so the second f is the double press |
| B-20 | telex | unicode | - | `hoangf<SP><BS>s` | hoáng | hoáng | **PASS** | A7 step 1 + 4: reopen the committed word, then retone |
| B-21 | telex | unicode | - | `hoangf<SP><BS><BS>s` | hoán | hoán | **PASS** | A7 step 1-4: reopen, delete g, retone |
| B-22 | telex | unicode | - | `hoangf<SP><BS><BS><SP>` | hoàn  | hoàn  | **PASS** | A7 + A6: delete the last char and commit with space |
| B-23 | telex | unicode | - | `ddi<BS>` | đ | đ | **PASS** | A7 step 5: the press removes i, d stays barred |
| B-24 | telex | unicode | - | `ddi<BS>i` | đi | đi | **PASS** | A7 step 4: retype the deleted letter |
| B-25 | telex | unicode | - | `ddi<BS>e` | đe | đe | **PASS** | A9: a plain vowel after d appends |
| B-26 | telex | unicode | - | `ddi<BS>d` | dd | dd | **PASS** | A1 doubled-letter escape: ddd -> dd (documented) |
| B-27 | telex | unicode | - | `dduowcj<BS>` | đượ | đượ | **PASS** | A7 step 5: the press removes c, the horn and tone stay |
| B-28 | telex | unicode | - | `dduowcj<BS>c` | được | được | **PASS** | A7 step 4: retype the deleted letter |
| B-29 | telex | unicode | - | `dduowcj<BS>s` | đướ | đướ | **PASS** | A7 step 4 + A4: sac on o of uo |
| B-30 | telex | unicode | - | `dduowcj<BS>j` | dduowj | dduowj | **PASS** | documented adjacent escape of the rebuilt history dduowj |
| B-31 | telex | unicode | - | `kieems<BS>` | kiế | kiế | **PASS** | A7 step 3 |
| B-32 | telex | unicode | - | `kieems<BS>m` | kiếm | kiếm | **PASS** | A7 step 4: retype the deleted letter |
| B-33 | telex | unicode | - | `kieems<BS>s` | kiees | kiees | **PASS** | documented adjacent escape of the rebuilt history kiees |
| B-34 | telex | unicode | - | `kieems<BS>e` | kiếe | kiese | **FAIL** | A1 doubled-letter escape needs two e KEY presses in a row; here the previous key is the tone s |
| B-35 | telex | unicode | - | `tieengs<BS>` | tiến | tiến | **PASS** | A7 step 2 (pinned by editing.rs) |
| B-36 | telex | unicode | - | `tieengs<BS>g` | tiếng | tiếng | **PASS** | A7 step 4: retype the deleted letter |
| B-37 | telex | unicode | - | `tieengs<BS>s` | tieens | tieens | **PASS** | documented adjacent escape of the rebuilt history tieens (pinned by editing.rs for the reopen route) |
| B-38 | telex | unicode | - | `tieengs<BS>n` | tiếnn | tieensn | **FAIL** | A9: a plain letter appends |
| B-39 | telex | unicode | - | `tieengs<SP><BS>s` | tieengs | tieengs | **PASS** | documented adjacent escape - pinned today by editing.rs::a_tone_trigger_on_a_reopened_toned_word_keeps_the_escape_rule |
| B-40 | telex | unicode | - | `toans<BS>` | toá | toá | **PASS** | A7 step 4: the remaining characters stay as displayed |
| B-41 | telex | unicode | - | `toans<BS>n` | toán | toán | **PASS** | A7 step 4: retype the deleted letter |
| B-42 | telex | unicode | - | `toans<BS>s` | tóa | toás | **FAIL** | A7 step 4: the next key re-runs A4 (oa without coda -> tóa) |
| B-43 | telex | unicode | - | `toans<BS>f` | tòa | toáf | **FAIL** | A7 step 4 + A4 |
| B-44 | telex | unicode | - | `toans<BS>r` | tỏa | toár | **FAIL** | A7 step 4 + A4 |
| B-45 | telex | unicode | - | `hoais<BS>` | hoá | hoá | **PASS** | A7 step 4 (A4 no longer applies to oai without coda) |
| B-46 | telex | unicode | - | `hoais<BS>s` | hóa | hoás | **FAIL** | A7 step 4 + A4 |
| B-47 | telex | unicode | - | `hoais<BS>f` | hòa | hoáf | **FAIL** | A7 step 4 + A4 |
| B-48 | telex | unicode | - | `hoais<BS>i` | hoái | hoái | **PASS** | A7 step 4: retype the deleted letter |
| B-49 | telex | unicode | - | `thuys<BS>s` | thus | thus | **PASS** | documented adjacent escape of the rebuilt history thus |
| B-50 | telex | unicode | - | `thuys<BS>f` | thù | thù | **PASS** | A4: different tone replaces |
| B-51 | telex | unicode | - | `baajt<SP><BS><BS><SP>` | bậ  | bậ  | **PASS** | A7 step 1-4 + A6 |
| B-52 | telex | unicode | - | `baajt<SP><BS><BS>n<SP>` | bận  | bận  | **PASS** | A7 step 1-4 + A6 |
| B-53 | telex | unicode | - | `baajt<SP><BS>s` | bất | bất | **PASS** | A7 step 1: reopen + different tone |
| B-54 | telex | unicode | - | `baajt<SP><BS>j` | bật | baatj | **FAIL** | A7 step 1: reopen + same tone = no change |
| B-55 | vni | unicode | - | `ba65t<BS>` | bậ | bậ | **PASS** | A7 step 3 |
| B-56 | vni | unicode | - | `ba65t<BS>1` | bấ | bấ | **PASS** | A7 step 4: digit re-places the tone |
| B-57 | vni | unicode | - | `ba65t<BS>2` | bầ | bầ | **PASS** | A7 step 4: huyen legal on the surviving open syllable |
| B-58 | vni | unicode | - | `ba65t<BS>t` | bật | bật | **PASS** | A7 step 4: retype the deleted letter |
| B-59 | vni | unicode | - | `ba65t<BS><BS>t` | bt | bt | **PASS** | A7 step 2-3: one displayed character per press |
| B-60 | vni | unicode | - | `ba65t<BS>5` | ba65 | ba65 | **PASS** | documented adjacent digit escape of the rebuilt history ba65 |
| B-61 | vni | unicode | - | `ba65t<BS>0` | bâ | ba | **FAIL** | A2: 0 removes the tone only |
| B-62 | telex | unicode | --no-free-tone | `baajt<BS>s` | bấ | bấ | **PASS** | free_tone off: tone change after a deletion still works |
| B-63 | telex | unicode | --old-tone-style | `baajt<BS>s` | bấ | bấ | **PASS** | old tone style: same |
| B-64 | telex | unicode | --no-spell-check | `baajt<BS>f` | bầ | bầ | **PASS** | spell off: transform applies (control row) |
| B-65 | telex | unicode_nfd | - | `baajt<BS>` | bậ | bậ | **PASS** | same as B-01 in NFD (the press deletes every unit) |
| B-66 | telex | unicode_nfd | - | `kieems<BS>m` | kiếm | kiếm | **PASS** | same as B-32 in NFD |
| B-67 | telex | unicode_nfd | - | `baajt<BS>s` | bấ | bấ | **PASS** | same as B-02 in NFD |
| B-68 | telex | unicode_nfd | - | `Nguyeexn<BS>n` | Nguyễn | Nguyễn | **PASS** | same as B-12 in NFD |

## 4. Counts

| slice | rows | PASS | FAIL |
|---|---|---|---|
| all | 123 | 88 | 35 |
| A — tone re-typed on a word already on screen | 55 | 34 | 21 |
| B — delete one unit, then retype | 68 | 54 | 14 |
| Telex (`unicode` 94 / `unicode_nfd` 7) | 101 | 76 | 25 |
| VNI (`unicode` 22) | 22 | 12 | 10 |
| `unicode_nfd` | 7 | 5 | 2 |
| `--old-tone-style` / `--no-free-tone` / `--no-spell-check` rows | 8 | 7 | 1 |

Failing ids: `A-02…A-05, A-11, A-13, A-14, A-24, A-26, A-31, A-36…A-39, A-43…A-45, A-47, A-49,
A-54, A-55, B-06, B-07, B-13…B-15, B-34, B-38, B-42…B-44, B-46, B-47, B-54, B-61`.

## 5. Narrowest failing shape per distinct failure

One row per root cause; the full row sets are in the defect table of `60-defects.md`.

| defect | narrowest repro | actual | expected | method · charset | rows |
|---|---|---|---|---|---|
| **D-TN-01** the tone key is already in force and a letter sits between the two presses → the whole word is reverted to raw keystrokes and the circumflex is lost | `baajtj` | `baatj` | `bật` | Telex · unicode + NFD · every option set | A-02, A-39, A-49, A-54, B-54 |
| **D-TN-02** Telex `z` (remove the tone) is not a trigger at all | `basz` | `basz` | `ba` | Telex · unicode + NFD | A-11, A-13, A-14, A-55, B-07 |
| **D-TN-03** VNI `0` removes the tone **and** every mark | `a60` | `a` | `â` | VNI · unicode | A-43, A-44, A-45, B-61 |
| **D-TN-04** a tone change to a tone that is illegal before the coda is applied instead of staying literal | `baajtf` | `bầt` | `bậtf` | Telex + VNI · unicode + NFD · incl. `BAATJf` | A-03…A-05, A-24, A-26, A-31, A-36…A-38 |
| **D-TN-05** a trigger with no target (mark digit on an already-marked vowel) reverts the word to raw instead of appending the literal digit | `ba65t6` | `ba5t6` | `bật6` | VNI · unicode | A-47 |
| **D-TN-06** after a Backspace whose display cannot be re-composed from the surviving keys the word freezes as literal characters, so the next tone key is literal | `toans<BS>f` | `toáf` | `tòa` | Telex · unicode | B-42…B-44, B-46, B-47 |
| **D-TN-07** a plain letter that makes the syllable invalid rewrites the whole word to raw keystrokes | `baajt<BS>b` | `baajb` | `bậb` | Telex · unicode | B-06, B-13, B-38 |
| **D-TN-08** a letter whose base equals the last *composed* letter escapes the mark even though the previous keystroke was the tone key | `Nguyeexn<BS>e` | `Nguyexe` | `Nguyễe` | Telex · unicode | B-14, B-15, B-34 |

Two disks of the same root are worth calling out because they are the owner's literal wording:

* **B (delete one unit, then retype a tone)** fails through **D-TN-06**: `toán` `BS` shows `toá` (A7
  step 4 keeps the display) and the very next key must re-place the tone per A4 → `tòa`/`tóa`; the
  engine instead froze `toá` as literal keystrokes and appends `f`/`s`/`r` → `toáf`/`toás`/`toár`.
  `hoài`+BS behaves the same (`hoás`). This is the only shape in the matrix where the word *stops
  composing* after a deletion.
* **A (change the tone of text already written)** fails through **D-TN-01**: with a coda letter
  between the two presses the escape branch of `apply_tone` fires, and because it drops the first
  trigger keystroke the `aa`/`65` mark chain literalizes too. VNI is the harder evidence: §A2 says the
  digit escape applies to a digit “repeated **immediately after** its own transform”, and `ba65t5`
  has `t` in between.

## 6. Documented behaviour the matrix confirms — **not** defects

Kept here so the same shapes are not re-filed:

* **Adjacent same-trigger escape** — `bass → bas` (A-10, pinned by `E-TE-002`), `hoangff → hoangf`
  (A-16), `dduowcjj → dduowcj` (A-34), `hoangf<BS>f → hoanf` (B-19), `dduowcj<BS>j → dduowj` (B-30),
  `kieems<BS>s → kiees` (B-33), `tieengs<BS>s → tieens` (B-37), `tieengs<SP><BS>s → tieengs` (B-39,
  currently pinned as intended by `editing.rs::a_tone_trigger_on_a_reopened_toned_word_keeps_the_escape_rule`),
  `thuys<BS>s → thus` (B-49), `ba65t<BS>5 → ba65` (B-60), `ddi<BS>d → dd` (B-26). In each of these
  the two presses are adjacent **in the rebuilt raw history**, so the second one is the documented
  double press (`20-spec.md` §A1 escape table, §A2 digit escapes). The same text typed in one go
  produces the same result, which is the invariant the implementation keeps.
  These are also the rows the owner is most likely to *feel* as “không thành công”: a word can lose
  its tone display this way. `60-defects.md` D-TN-01 carries the ruling request so the Lead can decide
  whether “adjacent” is the intended boundary — the engine currently applies the escape to
  non-adjacent repeats as well (the four rows marked FAIL).
* **One displayed character per Backspace** — `bật`+BS+BS is `b`, not `bâ` (A7 step 3: the tone dies
  with the character that carries it), and `bật`+BS+BS+`t` is `bt` (B-08/B-09). The brief's assumed
  `bâ` is not reachable in any charset; recorded as a spec clarification, not a defect.
* **A separator commits the word and the tone key reaches the next word** — `bật f`, `bật s`,
  `bật  á`, `bật.f` (A-19…A-22). The tone key starts a new word with no vowel, so it is literal there.
* **Reopen after a separator** — `bật`+`.` or `bật`+space, then BS, re-opens the word and a *different* tone key
  works (`A-23 bất`, `B-20 hoáng`, `B-21 hoán`, `B-53 bất`); only the *same* tone key is affected
  (D-TN-01).
* **`--no-spell-check`** rows behave as the switch says (`A-52`, `B-64`); **`--old-tone-style`** and
  **`--no-free-tone`** do not change any tone-change result (`A-48…A-51`, `B-62`, `B-63`), except the
  two D-TN-01/04 rows that fail with every option set.
* **NFD** deletion removes every code unit of a multi-unit character and the word keeps composing
  (`B-65`, `B-66`, `B-67`, `B-68`); the two NFD failures are the same roots as their NFC twin.

## 7. Scope notes

* Engine-only evidence: the hook, Notepad, Chrome and the injection path are untouched by this pass
  (headless constraint). `vkb-clone-work/95-e2e-report.md` covers those separately. Anything that only
  appears in an app (double `Replace` collapse, IME interference) would need a desktop run.
* Tone *style* for `oa`/`oe`/`uy` in this build is `hòa`/`thúy` for `modern_tone = true`
  (`28-engine-rulings.md`, `50-test-cases.md` `E-VN-039`, `E-TP-008`), i.e. the opposite of the
  `20-spec.md` §A4 T6 sentence. Not part of this report, but it decides the expected strings of the
  `oa` rows here (`A-40`, `B-42…B-48`): they follow the ruling/corpus, not §A4 T6.
* Rows whose expected value is “the word is unchanged” are the ones where a tone key is re-pressed for
  the tone already in force. That convention is stated once, in §2, and used by D-TN-01/06/08.


