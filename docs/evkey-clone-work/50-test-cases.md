# Test corpus & acceptance tests — VietKB (Sprint 1)

**Owner:** QA. **Status:** baseline written against `00-charter.md`, `25-contract.md`,
`30-plan.md` and Vietnamese orthography — **not** against any implementation.
**Purpose:** the golden corpus the Dev must satisfy and QA verifies independently.

Nothing here is measured yet. §4 is a protocol, not a result.

---

## 0. Conventions used by every table

**Notation in `input (raw keystrokes)`**

| Token | Meaning |
|---|---|
| `<sp>` | Space |
| `<bs>` | Backspace |
| `<ent>` | Enter |
| `<tab>` | Tab |
| `<esc>` | Escape |
| `<bt>` | Literal backtick `` ` `` (VIQR huyền key); written as a token so it never collides with Markdown |
| `\` | VIQR literal escape (literal next char) |
| `<empty>` | No text produced by the engine (visible output unchanged) |

**Engine options for §1 rows.** Unless the row's `why it matters` cell says otherwise:

| Option | Value | Reason |
|---|---|---|
| `charset` | `unicode` (NFC) | isolates keyspace logic from charset tables (§2) |
| `modern_tone` | `true` | "Đặt dấu kiểu mới" — `hòa`, `thúy`, `lóa` |
| `free_tone` | `true` | tone key may be typed before the end of the syllable |
| `jfwz_as_consonant` | `false` | keeps `f`/`j`/`w`/`z` available as tone/mark keys, which is what the tone rows pin |
| `spell_check` | `false` | pins pure transform semantics; §1 categories `spell-check` and `english-passthrough` set it `true` |
| `restore_wrong_word` | `false` | except category `restore-word` |
| `macro_enabled` | `false` | except category `macro`, whose table is fixed in §1.14 |
| `auto_capitalize` | `false` | |
| `method` | per row (`telex` / `vni` / `viqr`) | |

Projection model for every §1 row: a word under composition is the span since the last
boundary; the row's `expected output` is the **entire visible text** after the last
listed keystroke. Spaces and punctuation are boundaries (`boundary()`), so `a,b` is two
words and output equals input.

Row ids are stable — quote them in defect reports.

---

## 1. Engine corpus

### 1.1 telex-basic — 60 rows

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-TB-001 | telex | `Tieengs Vieetj` | `Tiếng Việt` | telex-basic | canonical two-word sentence; both words tone-at-end, initial caps preserved |
| E-TB-002 | telex | `tieengs` | `tiếng` | telex-basic | `ee`→ê, tone applied to ê, not to i |
| E-TB-003 | telex | `vieetj` | `việt` | telex-basic | nặng lands on ê inside a closed syllable |
| E-TB-004 | telex | `dduwowngf` | `đường` | telex-basic | `dd`+`uw`+`ow` in one syllable; tone on ơ |
| E-TB-005 | telex | `thuyeenf` | `thuyền` | telex-basic | ê steals the tone from the `uy` cluster |
| E-TB-006 | telex | `quas` | `quá` | telex-basic | `u` of `qu` is onset, never a tone target |
| E-TB-007 | telex | `giaf` | `già` | telex-basic | `i` of `gi` is onset, never a tone target |
| E-TB-008 | telex | `thasi` | `thái` | telex-basic | tone typed before the last vowel; tone must re-target after `i` arrives |
| E-TB-009 | telex | `nguyeenx` | `nguyễn` | telex-basic | ngã on ê inside `uyê` |
| E-TB-010 | telex | `toans` | `toán` | telex-basic | `oa` + coda → tone on `a` in both tone styles |
| E-TB-011 | telex | `caanf` | `cần` | telex-basic | `aa`→â then huyền |
| E-TB-012 | telex | `ddi` | `đi` | telex-basic | `dd`→đ with no vowel mark |
| E-TB-013 | telex | `hocj` | `học` | telex-basic | nặng on a short closed syllable |
| E-TB-014 | telex | `sinh` | `sinh` | telex-basic | no-transform word must be emitted unchanged |
| E-TB-015 | telex | `tieenf` | `tiền` | telex-basic | huyền on ê |
| E-TB-016 | telex | `tieens` | `tiến` | telex-basic | sắc on ê |
| E-TB-017 | telex | `ddaats` | `đất` | telex-basic | â + coda t + sắc |
| E-TB-018 | telex | `nuwowcs` | `nước` | telex-basic | ư + ơ + coda c, tone on ơ |
| E-TB-019 | telex | `chaof` | `chào` | telex-basic | `ao` open, tone on a |
| E-TB-020 | telex | `banj` | `bạn` | telex-basic | nặng on a with coda n |
| E-TB-021 | telex | `carm own` | `cảm ơn` | telex-basic | two words, hỏi on a / ơ |
| E-TB-022 | telex | `khoer` | `khỏe` | telex-basic | modern style: `oe` open → tone on o |
| E-TB-023 | telex | `yeeu` | `yêu` | telex-basic | ê wins over u in `yêu` |
| E-TB-024 | telex | `hieeru` | `hiểu` | telex-basic | hỏi stays on ê when a vowel follows |
| E-TB-025 | telex | `chuyeenj` | `chuyện` | telex-basic | nặng on ê, `uyê` closed |
| E-TB-026 | telex | `nguwowif` | `người` | telex-basic | ư + ơ + i, huyền on ơ |
| E-TB-027 | telex | `dduwowcj` | `được` | telex-basic | nặng on ơ, đ onset |
| E-TB-028 | telex | `truwowngf` | `trường` | telex-basic | huyền on ơ, coda ng |
| E-TB-029 | telex | `phuowng` | `phương` | telex-basic | ươ with no tone |
| E-TB-030 | telex | `huwowngs` | `hướng` | telex-basic | sắc on ơ |
| E-TB-031 | telex | `tuwowngr` | `tưởng` | telex-basic | hỏi on ơ |
| E-TB-032 | telex | `muwowif` | `mười` | telex-basic | huyền on ơ in `ươi` |
| E-TB-033 | telex | `ruwowuj` | `rượu` | telex-basic | `ươu` open, tone on ơ (not on u) |
| E-TB-034 | telex | `cuwowif` | `cười` | telex-basic | tone on ơ of `ươi` |
| E-TB-035 | telex | `tuwowis` | `tưới` | telex-basic | sắc on ơ of `ươi` |
| E-TB-036 | telex | `luwowis` | `lưới` | telex-basic | same cluster, different onset |
| E-TB-037 | telex | `ddaay` | `đây` | telex-basic | â + y, no coda |
| E-TB-038 | telex | `ddeesn` | `đến` | telex-basic | ê + coda n + sắc |
| E-TB-039 | telex | `ddepj` | `đẹp` | telex-basic | nặng on e, coda p |
| E-TB-040 | telex | `lafm` | `làm` | telex-basic | huyền on a, coda m (`f`, not `s`: `lasm` would give `lám`) |
| E-TB-041 | telex | `vieejc` | `việc` | telex-basic | nặng on ê, coda c |
| E-TB-042 | telex | `hieenj` | `hiện` | telex-basic | nặng on ê, coda n |
| E-TB-043 | telex | `bieets` | `biết` | telex-basic | sắc on ê, coda t |
| E-TB-044 | telex | `khoong` | `không` | telex-basic | `oo`→ô, no tone |
| E-TB-045 | telex | `chungs` | `chúng` | telex-basic | u + coda ng + sắc |
| E-TB-046 | telex | `ngheef` | `nghề` | telex-basic | ngh onset + ê + huyền |
| E-TB-047 | telex | `nghieemj` | `nghiệm` | telex-basic | nặng on ê, coda m |
| E-TB-048 | telex | `baif` | `bài` | telex-basic | tone key between vowel and y-less open syllable |
| E-TB-049 | telex | `thuaan` | `thuân` | telex-basic | u + â + n; `aa` must not swallow the u |
| E-TB-050 | telex | `gios` | `gió` | telex-basic | `gi` onset + o + sắc |
| E-TB-051 | telex | `giaays` | `giấy` | telex-basic | `gi` + â + y + sắc |
| E-TB-052 | telex | `gioir` | `giỏi` | telex-basic | hỏi on o in `oi` (no coda → first vowel) |
| E-TB-053 | telex | `quyeets` | `quyết` | telex-basic | qu + y + ê + t + sắc |
| E-TB-054 | telex | `quoocs` | `quốc` | telex-basic | qu + ô + c + sắc (ô needs the doubled `oo`) |
| E-TB-055 | telex | `thuwowngf` | `thường` | telex-basic | alternative `uwow` spelling of ươ must equal `uow` |
| E-TB-056 | telex | `muwa` | `mưa` | telex-basic | ư + a open, no tone, tone target is ư |
| E-TB-057 | telex | `cuwar` | `cửa` | telex-basic | hỏi on ư in `ưa` |
| E-TB-058 | telex | `nuwax` | `nữa` | telex-basic | ngã on ư |
| E-TB-059 | telex | `chuwax` | `chữa` | telex-basic | ngã on ư, coda-less |
| E-TB-060 | telex | `baor` | `bảo` | telex-basic | hỏi on a in `ao` |

### 1.2 telex-tone — 62 rows (every trigger, every vowel base)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-TT-001 | telex | `as` | `á` | telex-tone | s = sắc on a |
| E-TT-002 | telex | `af` | `à` | telex-tone | f = huyền on a (jfwz off) |
| E-TT-003 | telex | `ar` | `ả` | telex-tone | r = hỏi on a |
| E-TT-004 | telex | `ax` | `ã` | telex-tone | x = ngã on a |
| E-TT-005 | telex | `aj` | `ạ` | telex-tone | j = nặng on a |
| E-TT-006 | telex | `es` | `é` | telex-tone | sắc on e |
| E-TT-007 | telex | `ef` | `è` | telex-tone | huyền on e |
| E-TT-008 | telex | `er` | `ẻ` | telex-tone | hỏi on e |
| E-TT-009 | telex | `ex` | `ẽ` | telex-tone | ngã on e |
| E-TT-010 | telex | `ej` | `ẹ` | telex-tone | nặng on e |
| E-TT-011 | telex | `is` | `í` | telex-tone | sắc on i |
| E-TT-012 | telex | `if` | `ì` | telex-tone | huyền on i |
| E-TT-013 | telex | `ir` | `ỉ` | telex-tone | hỏi on i |
| E-TT-014 | telex | `ix` | `ĩ` | telex-tone | ngã on i |
| E-TT-015 | telex | `ij` | `ị` | telex-tone | nặng on i |
| E-TT-016 | telex | `os` | `ó` | telex-tone | sắc on o |
| E-TT-017 | telex | `of` | `ò` | telex-tone | huyền on o with jfwz off (contrast E-TT-061) |
| E-TT-018 | telex | `or` | `ỏ` | telex-tone | hỏi on o |
| E-TT-019 | telex | `ox` | `õ` | telex-tone | ngã on o |
| E-TT-020 | telex | `oj` | `ọ` | telex-tone | nặng on o |
| E-TT-021 | telex | `us` | `ú` | telex-tone | sắc on u |
| E-TT-022 | telex | `uf` | `ù` | telex-tone | huyền on u |
| E-TT-023 | telex | `ur` | `ủ` | telex-tone | hỏi on u |
| E-TT-024 | telex | `ux` | `ũ` | telex-tone | ngã on u |
| E-TT-025 | telex | `uj` | `ụ` | telex-tone | nặng on u |
| E-TT-026 | telex | `ys` | `ý` | telex-tone | sắc on y |
| E-TT-027 | telex | `yf` | `ỳ` | telex-tone | huyền on y |
| E-TT-028 | telex | `yr` | `ỷ` | telex-tone | hỏi on y |
| E-TT-029 | telex | `yx` | `ỹ` | telex-tone | ngã on y |
| E-TT-030 | telex | `yj` | `ỵ` | telex-tone | nặng on y |
| E-TT-031 | telex | `aas` | `ấ` | telex-tone | sắc on â |
| E-TT-032 | telex | `aaf` | `ầ` | telex-tone | huyền on â |
| E-TT-033 | telex | `aar` | `ẩ` | telex-tone | hỏi on â |
| E-TT-034 | telex | `aax` | `ẫ` | telex-tone | ngã on â |
| E-TT-035 | telex | `aaj` | `ậ` | telex-tone | nặng on â |
| E-TT-036 | telex | `aws` | `ắ` | telex-tone | sắc on ă |
| E-TT-037 | telex | `awf` | `ằ` | telex-tone | huyền on ă |
| E-TT-038 | telex | `awr` | `ẳ` | telex-tone | hỏi on ă |
| E-TT-039 | telex | `awx` | `ẵ` | telex-tone | ngã on ă |
| E-TT-040 | telex | `awj` | `ặ` | telex-tone | nặng on ă |
| E-TT-041 | telex | `ees` | `ế` | telex-tone | sắc on ê |
| E-TT-042 | telex | `eef` | `ề` | telex-tone | huyền on ê |
| E-TT-043 | telex | `eer` | `ể` | telex-tone | hỏi on ê |
| E-TT-044 | telex | `eex` | `ễ` | telex-tone | ngã on ê |
| E-TT-045 | telex | `eej` | `ệ` | telex-tone | nặng on ê |
| E-TT-046 | telex | `oos` | `ố` | telex-tone | sắc on ô |
| E-TT-047 | telex | `oof` | `ồ` | telex-tone | huyền on ô |
| E-TT-048 | telex | `oor` | `ổ` | telex-tone | hỏi on ô |
| E-TT-049 | telex | `oox` | `ỗ` | telex-tone | ngã on ô |
| E-TT-050 | telex | `ooj` | `ộ` | telex-tone | nặng on ô |
| E-TT-051 | telex | `ows` | `ớ` | telex-tone | sắc on ơ |
| E-TT-052 | telex | `owf` | `ờ` | telex-tone | huyền on ơ |
| E-TT-053 | telex | `owr` | `ở` | telex-tone | hỏi on ơ |
| E-TT-054 | telex | `owx` | `ỡ` | telex-tone | ngã on ơ |
| E-TT-055 | telex | `owj` | `ợ` | telex-tone | nặng on ơ |
| E-TT-056 | telex | `uws` | `ứ` | telex-tone | sắc on ư |
| E-TT-057 | telex | `uwf` | `ừ` | telex-tone | huyền on ư |
| E-TT-058 | telex | `uwr` | `ử` | telex-tone | hỏi on ư |
| E-TT-059 | telex | `uwx` | `ữ` | telex-tone | ngã on ư |
| E-TT-060 | telex | `uwj` | `ự` | telex-tone | nặng on ư |
| E-TT-061 | telex | `of` | `of` | telex-tone | with `jfwz_as_consonant=true` the f is a consonant, not huyền — must match EVKey |
| E-TT-062 | telex | `asi` | `ái` | telex-tone | tone applied to a bare vowel must survive a later vowel arriving |

### 1.3 telex-escape — 18 rows (double trigger reverts to raw)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-TE-001 | telex | `as` | `á` | telex-escape | baseline: single trigger applies |
| E-TE-002 | telex | `ass` | `as` | telex-escape | repeated sắc reverts and emits the trigger letter literally |
| E-TE-003 | telex | `aff` | `af` | telex-escape | repeated huyền |
| E-TE-004 | telex | `arr` | `ar` | telex-escape | repeated hỏi |
| E-TE-005 | telex | `axx` | `ax` | telex-escape | repeated ngã |
| E-TE-006 | telex | `ajj` | `aj` | telex-escape | repeated nặng |
| E-TE-007 | telex | `aaa` | `aa` | telex-escape | third `a` after â escapes; result is two literal a |
| E-TE-008 | telex | `eee` | `ee` | telex-escape | same for ê |
| E-TE-009 | telex | `ooo` | `oo` | telex-escape | same for ô |
| E-TE-010 | telex | `ddd` | `dd` | telex-escape | same for đ |
| E-TE-011 | telex | `ss` | `ss` | telex-escape | no vowel present: both literal, no phantom tone |
| E-TE-012 | telex | `aass` | `aas` | telex-escape | escape on a marked vowel must not lose the circumflex twice |
| E-TE-013 | telex | `bass` | `bas` | telex-escape | escape keeps the onset |
| E-TE-014 | telex | `thuowngff` | `thuowngf` | telex-escape | escape inside a long syllable with ươ |
| E-TE-015 | telex | `aas` | `ấ` | telex-escape | control: mark + tone, not an escape |
| E-TE-016 | telex | `asss` | `ass` | telex-escape | escape is idempotent per extra trigger |
| E-TE-017 | vni | `a11` | `a1` | telex-escape | VNI repeated tone digit escapes the same way |
| E-TE-018 | vni | `a66` | `a6` | telex-escape | VNI repeated mark digit escapes |

### 1.4 telex-w — 20 rows (`w` is both the ư letter and the ă/ơ/ư mark key)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-TW-001 | telex | `w` | `ư` | telex-w | `w` alone is ư in Telex |
| E-TW-002 | telex | `ww` | `w` | telex-w | escape to literal w (see OQ-3 — must be confirmed against EVKey) |
| E-TW-003 | telex | `aw` | `ă` | telex-w | w after a = breve |
| E-TW-004 | telex | `ow` | `ơ` | telex-w | w after o = horn |
| E-TW-005 | telex | `uw` | `ư` | telex-w | w after u = horn |
| E-TW-006 | telex | `uow` | `ươ` | telex-w | w after uo marks both vowels |
| E-TW-007 | telex | `aww` | `aw` | telex-w | repeated mark key reverts to literal |
| E-TW-008 | telex | `oww` | `ow` | telex-w | same for ơ |
| E-TW-009 | telex | `uww` | `uw` | telex-w | same for ư |
| E-TW-010 | telex | `awng` | `ăng` | telex-w | ă inside a closed syllable |
| E-TW-011 | telex | `ngoawn` | `ngoăn` | telex-w | w after `oa` must pick a, not o |
| E-TW-012 | telex | `thuowng` | `thương` | telex-w | w after uo with coda |
| E-TW-013 | telex | `huwowu` | `hươu` | telex-w | ư + ơ + u, no tone |
| E-TW-014 | telex | `cuowis` | `cưới` | telex-w | ươ + i + sắc, tone on ơ |
| E-TW-015 | telex | `uowng` | `ương` | telex-w | bare nucleus ương |
| E-TW-016 | telex | `w<sp>` | `ư<sp>` | telex-w | boundary after a `w`-produced letter |
| E-TW-017 | telex | `aw<sp>` | `ă<sp>` | telex-w | boundary after ă |
| E-TW-018 | telex | `tw` | `tư` | telex-w | w directly after a consonant (see OQ-4) |
| E-TW-019 | telex | `uwow` | `ươ` | telex-w | both spellings of ươ must converge |
| E-TW-020 | telex | `owng` | `ơng` | telex-w | ơ + coda without tone |

### 1.5 vni — 60 rows (digits are marks and tones)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-VN-001 | vni | `a1` | `á` | vni | 1 = sắc |
| E-VN-002 | vni | `a2` | `à` | vni | 2 = huyền |
| E-VN-003 | vni | `a3` | `ả` | vni | 3 = hỏi |
| E-VN-004 | vni | `a4` | `ã` | vni | 4 = ngã |
| E-VN-005 | vni | `a5` | `ạ` | vni | 5 = nặng |
| E-VN-006 | vni | `e1` | `é` | vni | tone digit on e |
| E-VN-007 | vni | `i1` | `í` | vni | tone digit on i |
| E-VN-008 | vni | `o1` | `ó` | vni | tone digit on o |
| E-VN-009 | vni | `u1` | `ú` | vni | tone digit on u |
| E-VN-010 | vni | `y1` | `ý` | vni | tone digit on y |
| E-VN-011 | vni | `a61` | `ấ` | vni | 6 then tone |
| E-VN-012 | vni | `a62` | `ầ` | vni | huyền on â |
| E-VN-013 | vni | `a63` | `ẩ` | vni | hỏi on â |
| E-VN-014 | vni | `a64` | `ẫ` | vni | ngã on â |
| E-VN-015 | vni | `a65` | `ậ` | vni | nặng on â |
| E-VN-016 | vni | `a81` | `ắ` | vni | 8 = ă then sắc |
| E-VN-017 | vni | `a82` | `ằ` | vni | huyền on ă |
| E-VN-018 | vni | `a83` | `ẳ` | vni | hỏi on ă |
| E-VN-019 | vni | `a84` | `ẵ` | vni | ngã on ă |
| E-VN-020 | vni | `a85` | `ặ` | vni | nặng on ă |
| E-VN-021 | vni | `e61` | `ế` | vni | 6 = ê then sắc |
| E-VN-022 | vni | `e62` | `ề` | vni | huyền on ê |
| E-VN-023 | vni | `o61` | `ố` | vni | 6 = ô then sắc |
| E-VN-024 | vni | `o62` | `ồ` | vni | huyền on ô |
| E-VN-025 | vni | `o71` | `ớ` | vni | 7 = ơ then sắc |
| E-VN-026 | vni | `o72` | `ờ` | vni | huyền on ơ |
| E-VN-027 | vni | `u71` | `ứ` | vni | 7 = ư then sắc |
| E-VN-028 | vni | `u72` | `ừ` | vni | huyền on ư |
| E-VN-029 | vni | `Tie6ng1 Vie6t5` | `Tiếng Việt` | vni | full sentence, VNI digits, caps preserved |
| E-VN-030 | vni | `d9u7o7ng2` | `đường` | vni | 9 = đ, 7 twice for ươ |
| E-VN-031 | vni | `thu7o7ng2` | `thường` | vni | tone digit at the end of a long syllable |
| E-VN-032 | vni | `ngu7o7i2` | `người` | vni | ươi, huyền on ơ |
| E-VN-033 | vni | `d9u7o7c5` | `được` | vni | nặng on ơ |
| E-VN-034 | vni | `muo6n1` | `muốn` | vni | 6 on the o of `uo` = uô |
| E-VN-035 | vni | `cuo6i1` | `cuối` | vni | uô + i + sắc, tone on ô |
| E-VN-036 | vni | `tuye6t1` | `tuyết` | vni | ê wins the tone inside `uyê` |
| E-VN-037 | vni | `quye6t1` | `quyết` | vni | qu onset excluded from nucleus |
| E-VN-038 | vni | `nguye6n4` | `nguyễn` | vni | ngã on ê |
| E-VN-039 | vni | `hoa2` | `hòa` | vni | modern tone style driven by option, not by method |
| E-VN-040 | vni | `hoan2` | `hoàn` | vni | coda forces tone onto a in both styles |
| E-VN-041 | vni | `thuy3` | `thủy` | vni | modern `uy` open → tone on u |
| E-VN-042 | vni | `khuyu3` | `khuỷu` | vni | `uyu` nucleus, tone on y |
| E-VN-043 | vni | `uynh2` | `uỳnh` | vni | `uy` + coda → tone on y |
| E-VN-044 | vni | `nghie6ng0` | `nghieng` | vni | 0 removes the tone of the current word |
| E-VN-045 | vni | `d9i` | `đi` | vni | 9 alone = đ, no vowel mark |
| E-VN-046 | vni | `d9a4` | `đã` | vni | đ + ngã |
| E-VN-047 | vni | `ca6n2` | `cần` | vni | â + coda + huyền |
| E-VN-048 | vni | `ma8t5` | `mặt` | vni | ă + coda t + nặng |
| E-VN-049 | vni | `ba8ng2` | `bằng` | vni | ă + ng + huyền |
| E-VN-050 | vni | `chung1` | `chúng` | vni | tone digit after the coda must still target u |
| E-VN-051 | vni | `quo6c1` | `quốc` | vni | qu + ô + c + sắc |
| E-VN-052 | vni | `quan3` | `quản` | vni | qu + a + n + hỏi |
| E-VN-053 | vni | `gia2` | `già` | vni | gi onset excluded from nucleus |
| E-VN-054 | vni | `gio1` | `gió` | vni | gi + o + sắc |
| E-VN-055 | vni | `giu7a4` | `giữa` | vni | ư then ngã then a |
| E-VN-056 | vni | `d9e6m` | `đêm` | vni | ê with no tone |
| E-VN-057 | vni | `d9ung1` | `đúng` | vni | tone digit after ng |
| E-VN-058 | vni | `hoc5` | `học` | vni | nặng on a short closed syllable |
| E-VN-059 | vni | `ta6t1` | `tất` | vni | â + t + sắc |
| E-VN-060 | vni | `vie6t5` | `việt` | vni | ê + t + nặng |

### 1.7 tone-placement — 59 rows (which vowel takes the mark)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-TP-001 | telex | `hoar` | `hỏa` | tone-placement | `oa` open, modern → tone on o |
| E-TP-002 | telex | `hoanf` | `hoàn` | tone-placement | `oa` closed → tone moves to a |
| E-TP-003 | telex | `hoas` | `hóa` | tone-placement | sắc on o of open `oa` |
| E-TP-004 | telex | `khoer` | `khỏe` | tone-placement | `oe` open → tone on o |
| E-TP-005 | telex | `khoes` | `khóe` | tone-placement | `oe` open + sắc → `khóe mắt` spelling |
| E-TP-006 | telex | `thuyr` | `thủy` | tone-placement | `uy` open → tone on u |
| E-TP-007 | telex | `thuyf` | `thùy` | tone-placement | huyền on u of `uy` |
| E-TP-008 | telex | `thuys` | `thúy` | tone-placement | sắc on u of `uy` |
| E-TP-009 | telex | `thuyj` | `thụy` | tone-placement | nặng on u of `uy` |
| E-TP-010 | telex | `khuya` | `khuya` | tone-placement | `uya` nucleus survives untransformed |
| E-TP-011 | telex | `khuyur` | `khuỷu` | tone-placement | `uyu` → tone on y |
| E-TP-012 | telex | `chuyeenj` | `chuyện` | tone-placement | ê overrides the cluster |
| E-TP-013 | telex | `nguyeenx` | `nguyễn` | tone-placement | ngã on ê of `uyê` |
| E-TP-014 | telex | `tuyeets` | `tuyết` | tone-placement | `uyê` + t + sắc on ê (ê needs the doubled `ee`) |
| E-TP-015 | telex | `duyeetj` | `duyệt` | tone-placement | nặng on ê |
| E-TP-016 | telex | `huyeenj` | `huyện` | tone-placement | nặng on ê, coda n |
| E-TP-017 | telex | `guwir` | `gửi` | tone-placement | hỏi on ư of `ưi` |
| E-TP-018 | telex | `luwowif` | `lười` | tone-placement | ơ beats ư in `ươi` |
| E-TP-019 | telex | `thuwowngr` | `thưởng` | tone-placement | hỏi on ơ |
| E-TP-020 | telex | `truwowngr` | `trưởng` | tone-placement | hỏi on ơ, long onset `tr` |
| E-TP-021 | telex | `buwowcs` | `bước` | tone-placement | sắc on ơ, coda c |
| E-TP-022 | telex | `thuwowcs` | `thước` | tone-placement | sắc on ơ, coda c |
| E-TP-023 | telex | `huwowngf` | `hường` | tone-placement | huyền on ơ, coda ng |
| E-TP-024 | telex | `phuowngf` | `phường` | tone-placement | huyền on ơ, `ph` onset |
| E-TP-025 | telex | `tuowngr` | `tưởng` | tone-placement | alt `uow` spelling of ươ |
| E-TP-026 | telex | `huwowngr` | `hưởng` | tone-placement | hỏi on ơ of ương |
| E-TP-027 | telex | `thuoocs` | `thuốc` | tone-placement | uô + c + sắc, tone on ô |
| E-TP-028 | telex | `tuooir` | `tuổi` | tone-placement | uô + i + hỏi, tone on ô |
| E-TP-029 | telex | `cuoois` | `cuối` | tone-placement | uô + i + sắc, tone on ô |
| E-TP-030 | telex | `muoois` | `muối` | tone-placement | tone on ô, not on i |
| E-TP-031 | telex | `ddoongf` | `đồng` | tone-placement | ô + ng + huyền |
| E-TP-032 | telex | `hoongf` | `hồng` | tone-placement | ô + ng + huyền |
| E-TP-033 | telex | `thoong` | `thông` | tone-placement | ô with no tone |
| E-TP-034 | telex | `khoor` | `khổ` | tone-placement | hỏi on ô, no coda |
| E-TP-035 | telex | `mowr` | `mở` | tone-placement | hỏi on ơ |
| E-TP-036 | telex | `chowf` | `chờ` | tone-placement | huyền on ơ |
| E-TP-037 | telex | `thowr` | `thở` | tone-placement | hỏi on ơ |
| E-TP-038 | telex | `chieeuf` | `chiều` | tone-placement | iê + u, tone on ê |
| E-TP-039 | telex | `mieengj` | `miệng` | tone-placement | iê + ng + nặng |
| E-TP-040 | telex | `kieens` | `kiến` | tone-placement | iê + n + sắc |
| E-TP-041 | telex | `giups` | `giúp` | tone-placement | gi onset + u + p + sắc |
| E-TP-042 | telex | `hoaij` | `hoại` | tone-placement | `oai` → tone on middle a |
| E-TP-043 | telex | `khoais` | `khoái` | tone-placement | `oai` + sắc on a |
| E-TP-044 | telex | `thoair` | `thoải` | tone-placement | `oai` + hỏi on a |
| E-TP-045 | telex | `ngoaij` | `ngoại` | tone-placement | `oai` + nặng on a |
| E-TP-046 | telex | `xoays` | `xoáy` | tone-placement | `oay` → tone on middle a |
| E-TP-047 | telex | `ngoeos` | `ngoéo` | tone-placement | `oeo` → tone on middle e |
| E-TP-048 | telex | `toanf` | `toàn` | tone-placement | `oa` + coda n + huyền on a |
| E-TP-049 | telex | `ddoanf` | `đoàn` | tone-placement | đ onset + `oa` + coda |
| E-TP-050 | telex | `huynhf` | `huỳnh` | tone-placement | `uy` + coda → tone on y |
| E-TP-051 | telex | `uynhf` | `uỳnh` | tone-placement | same with no onset |
| E-TP-052 | telex | `huychs` | `huých` | tone-placement | `uy` + ch + sắc |
| E-TP-053 | telex | `khuychj` | `khuỵch` | tone-placement | `uy` + ch + nặng |
| E-TP-054 | telex | `huyts` | `huýt` | tone-placement | `uy` + t + sắc |
| E-TP-055 | telex | `traanj` | `trận` | tone-placement | â + n + nặng |
| E-TP-056 | telex | `khuaays` | `khuấy` | tone-placement | `uây` → tone on â (â needs the doubled `aa`) |
| E-TP-057 | telex | `quaays` | `quấy` | tone-placement | qu onset excluded, `ây` + sắc |
| E-TP-058 | telex | `ngoawnf` | `ngoằn` | tone-placement | `oă` + coda, tone on ă |
| E-TP-059 | telex | `loas` | `lóa` | tone-placement | modern partner of E-TO-009 (`loá`) |

### 1.8 tone-old-style — 20 rows (`modern_tone=false`)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-TO-001 | telex | `hoaf` | `hoà` | tone-old-style | same input as E-TB-060, different option → different output |
| E-TO-002 | telex | `HOAF` | `HOÀ` | tone-old-style | caps + old style |
| E-TO-003 | telex | `hoef` | `hoè` | tone-old-style | `oe` old → tone on e |
| E-TO-004 | telex | `thuyf` | `thuỳ` | tone-old-style | `uy` old → tone on y |
| E-TO-005 | telex | `thuyr` | `thuỷ` | tone-old-style | hỏi on y |
| E-TO-006 | telex | `thuys` | `thuý` | tone-old-style | sắc on y |
| E-TO-007 | telex | `thuyj` | `thuỵ` | tone-old-style | nặng on y |
| E-TO-008 | telex | `khoer` | `khoẻ` | tone-old-style | old `oe` spelling of khỏe |
| E-TO-009 | telex | `loas` | `loá` | tone-old-style | prompt pair: old `loá` vs modern `lóa` |
| E-TO-010 | telex | `loaf` | `loà` | tone-old-style | huyền on a |
| E-TO-011 | telex | `khoas` | `khoá` | tone-old-style | old spelling of khóa |
| E-TO-012 | telex | `ngoas` | `ngoá` | tone-old-style | oa + sắc → a in old style |
| E-TO-013 | telex | `toanf` | `toàn` | tone-old-style | control: coda forces a in both styles |
| E-TO-014 | telex | `quas` | `quá` | tone-old-style | control: qu is immune to the option |
| E-TO-015 | telex | `hoanf` | `hoàn` | tone-old-style | control: coda n |
| E-TO-016 | telex | `THUYF` | `THUỲ` | tone-old-style | all caps, old style |
| E-TO-017 | vni | `hoa2` | `hoà` | tone-old-style | option is method-independent |
| E-TO-018 | vni | `thuy2` | `thuỳ` | tone-old-style | 2 after uy |
| E-TO-019 | vni | `thuy3` | `thuỷ` | tone-old-style | 3 after uy |
| E-TO-020 | vni | `khoa1` | `khoá` | tone-old-style | 1 after oa |

### 1.9 modified-letter — 21 rows (the 7 letters, standalone and in words)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-ML-001 | telex | `aa` | `â` | modified-letter | circumflex trigger |
| E-ML-002 | telex | `aw` | `ă` | modified-letter | breve trigger |
| E-ML-003 | telex | `ee` | `ê` | modified-letter | ê |
| E-ML-004 | telex | `oo` | `ô` | modified-letter | ô |
| E-ML-005 | telex | `ow` | `ơ` | modified-letter | ơ |
| E-ML-006 | telex | `uw` | `ư` | modified-letter | ư |
| E-ML-007 | telex | `dd` | `đ` | modified-letter | đ |
| E-ML-008 | telex | `caan` | `cân` | modified-letter | â in a closed syllable |
| E-ML-009 | telex | `cawng` | `căng` | modified-letter | ă + ng |
| E-ML-010 | telex | `veef` | `về` | modified-letter | ê + huyền |
| E-ML-011 | telex | `coo` | `cô` | modified-letter | ô with no coda |
| E-ML-012 | telex | `mow` | `mơ` | modified-letter | ơ with no coda |
| E-ML-013 | telex | `tuw` | `tư` | modified-letter | ư with no coda |
| E-ML-014 | telex | `ddos` | `đó` | modified-letter | đ + o + sắc |
| E-ML-015 | vni | `a6` | `â` | modified-letter | 6 = circumflex |
| E-ML-016 | vni | `a8` | `ă` | modified-letter | 8 = breve |
| E-ML-017 | vni | `e6` | `ê` | modified-letter | 6 on e |
| E-ML-018 | vni | `o6` | `ô` | modified-letter | 6 on o |
| E-ML-019 | vni | `o7` | `ơ` | modified-letter | 7 on o |
| E-ML-020 | vni | `u7` | `ư` | modified-letter | 7 on u |
| E-ML-021 | vni | `d9` | `đ` | modified-letter | 9 on d |

### 1.10 qu-gi — 21 rows (`qu`/`gi` onsets are not tone targets)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-QG-001 | telex | `quar` | `quả` | qu-gi | tone must skip the u of qu |
| E-QG-002 | telex | `que` | `que` | qu-gi | qu + e, no transform |
| E-QG-003 | telex | `quees` | `quế` | qu-gi | ê takes the tone after qu |
| E-QG-004 | telex | `quyf` | `quỳ` | qu-gi | tone on y, u is onset |
| E-QG-005 | telex | `quyj` | `quỵ` | qu-gi | nặng on y |
| E-QG-006 | telex | `quynhf` | `quỳnh` | qu-gi | qu + y + nh + huyền |
| E-QG-007 | telex | `quyeen` | `quyên` | qu-gi | qu + yê + n, no tone |
| E-QG-008 | telex | `quyeetj` | `quyệt` | qu-gi | nặng on ê |
| E-QG-009 | telex | `quanr` | `quản` | qu-gi | qu + a + n + hỏi |
| E-QG-010 | telex | `quangs` | `quáng` | qu-gi | sắc on a, not on u |
| E-QG-011 | telex | `quawn` | `quăn` | qu-gi | qu + ă + n |
| E-QG-012 | telex | `quyeens` | `quyến` | qu-gi | sắc on ê (ê needs the doubled `ee`) |
| E-QG-013 | telex | `gi` | `gi` | qu-gi | bare gi is a word and must not be rewritten |
| E-QG-014 | telex | `giar` | `giả` | qu-gi | tone skips the i of gi |
| E-QG-015 | telex | `giowf` | `giờ` | qu-gi | ơ takes the tone after gi |
| E-QG-016 | telex | `giuwx` | `giữ` | qu-gi | ngã on ư |
| E-QG-017 | telex | `giuwax` | `giữa` | qu-gi | ư takes the tone, a follows |
| E-QG-018 | telex | `gieengs` | `giếng` | qu-gi | gi + ê + ng + sắc |
| E-QG-019 | telex | `giuwowngf` | `giường` | qu-gi | gi + ươ + ng + huyền |
| E-QG-020 | telex | `gias` | `giá` | qu-gi | gi + a + sắc |
| E-QG-021 | telex | `giair` | `giải` | qu-gi | `ai` open → tone on a |

### 1.11 uppercase — 26 rows

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-UC-001 | telex | `AA` | `Â` | uppercase | mark keys are case-insensitive |
| E-UC-002 | telex | `AW` | `Ă` | uppercase | breve from shift+w |
| E-UC-003 | telex | `EE` | `Ê` | uppercase | ê |
| E-UC-004 | telex | `OO` | `Ô` | uppercase | ô |
| E-UC-005 | telex | `OW` | `Ơ` | uppercase | ơ |
| E-UC-006 | telex | `UW` | `Ư` | uppercase | ư |
| E-UC-007 | telex | `DD` | `Đ` | uppercase | đ |
| E-UC-008 | telex | `BAS` | `BÁ` | uppercase | tone on an uppercase vowel |
| E-UC-009 | telex | `BAWS` | `BẮ` | uppercase | tone on Ă |
| E-UC-010 | telex | `BAAS` | `BẤ` | uppercase | tone on Â |
| E-UC-011 | telex | `BEES` | `BẾ` | uppercase | tone on Ê |
| E-UC-012 | telex | `BOOS` | `BỐ` | uppercase | tone on Ô |
| E-UC-013 | telex | `BOWS` | `BỚ` | uppercase | tone on Ơ |
| E-UC-014 | telex | `BUWS` | `BỨ` | uppercase | tone on Ư |
| E-UC-015 | telex | `DDUWOWNG` | `ĐƯƠNG` | uppercase | full all-caps đ + ươ, no tone key in the input |
| E-UC-016 | telex | `TUONGWF` | `TƯỜNG` | uppercase | all-caps with the w-after-uo form |
| E-UC-017 | telex | `HOAF` | `HÒA` | uppercase | modern tone style in caps |
| E-UC-018 | telex | `TIEENGS` | `TIẾNG` | uppercase | ô/ê/â stay uppercase after tone |
| E-UC-019 | telex | `VIEETJ` | `VIỆT` | uppercase | nặng on uppercase Ê |
| E-UC-020 | telex | `NGUYEENX` | `NGUYỄN` | uppercase | ngã on uppercase Ê |
| E-UC-021 | telex | `Tieengs` | `Tiếng` | uppercase | title case only affects the first letter |
| E-UC-022 | telex | `HOA` | `HOA` | uppercase | no-tone caps word passes through |
| E-UC-023 | telex | `DDI` | `ĐI` | uppercase | Đ + I |
| E-UC-024 | telex | `QUYEETS` | `QUYẾT` | uppercase | qu onset in caps |
| E-UC-025 | telex | `GIUWAX` | `GIỮA` | uppercase | gi onset in caps |
| E-UC-026 | telex | `THUWOWNG` | `THƯƠNG` | uppercase | ư + ơ in caps |

### 1.12 backspace — 21 rows (recompute from the raw keystrokes)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-BS-001 | telex | `tieengs<bs>` | `tiêng` | backspace | removing the tone key must drop only the tone |
| E-BS-002 | telex | `tieengs<bs><bs>` | `tiên` | backspace | second backspace drops the n, ê survives |
| E-BS-003 | telex | `tieengs<bs><bs><bs>` | `tiê` | backspace | third backspace drops the g |
| E-BS-004 | telex | `tieengs<bs><bs><bs><bs>` | `tie` | backspace | backspace past the mark must also drop ê |
| E-BS-005 | telex | `Hoas<bs>` | `Hoa` | backspace | initial capital is preserved through recompute |
| E-BS-006 | telex | `dduwowngf<bs>` | `đương` | backspace | đ and ươ survive a tone removal |
| E-BS-007 | telex | `dduwowngf<bs><bs>` | `đươn` | backspace | two removals drop f and g (`đươ` would need a third) |
| E-BS-008 | telex | `vieejt<bs>` | `việ` | backspace | nặng kept, coda removed |
| E-BS-009 | telex | `aas<bs>` | `â` | backspace | tone removed, circumflex kept |
| E-BS-010 | telex | `ddi<bs>` | `đ` | backspace | word reduced to the đ onset |
| E-BS-011 | telex | `khoer<bs>` | `khoe` | backspace | modern `oe` reverts to plain letters |
| E-BS-012 | telex | `thuyr<bs>` | `thuy` | backspace | `uy` cluster survives |
| E-BS-013 | telex | `tieengs<bs>s` | `tiếng` | backspace | retyping the tone after a removal must re-apply it |
| E-BS-014 | telex | `hoaf<bs>f` | `hòa` | backspace | same for `oa` |
| E-BS-015 | telex | `tieengs<bs><bs>s` | `tiến` | backspace | tone re-applied to a shorter raw word |
| E-BS-016 | telex | `quyeets<bs>` | `quyêt` | backspace | tone removal inside qu + yê |
| E-BS-017 | telex | `mowr<bs>` | `mơ` | backspace | ơ survives, hỏi removed |
| E-BS-018 | telex | `<bs>` | `<empty>` | backspace | backspace on an empty word is a no-op, not a crash |
| E-BS-019 | vni | `nghie6ng2<bs>` | `nghiêng` | backspace | VNI: tone digit removal keeps letters (ê included) |
| E-BS-020 | vni | `To6i1<bs>` | `Tôi` | backspace | VNI capital preserved |
| E-BS-021 | vni | `d9i<bs>` | `đ` | backspace | VNI: 9 survives, i removed |

### 1.13 boundary-commit — 20 rows (space, punctuation, tab, enter, digit)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-BC-001 | telex | `tieengs<sp>` | `tiếng<sp>` | boundary-commit | space commits and resets word state |
| E-BC-002 | telex | `tieengs,` | `tiếng,` | boundary-commit | comma commits |
| E-BC-003 | telex | `tieengs.` | `tiếng.` | boundary-commit | period commits |
| E-BC-004 | telex | `tieengs!` | `tiếng!` | boundary-commit | exclamation commits |
| E-BC-005 | telex | `tieengs?` | `tiếng?` | boundary-commit | question mark commits |
| E-BC-006 | telex | `tieengs;` | `tiếng;` | boundary-commit | semicolon commits |
| E-BC-007 | telex | `tieengs:` | `tiếng:` | boundary-commit | colon commits |
| E-BC-008 | telex | `tieengs-` | `tiếng-` | boundary-commit | hyphen commits (used in compound words) |
| E-BC-009 | telex | `tieengs/` | `tiếng/` | boundary-commit | slash commits (URLs) |
| E-BC-010 | telex | `tieengs(` | `tiếng(` | boundary-commit | opening bracket commits |
| E-BC-011 | telex | `(tieengs)` | `(tiếng)` | boundary-commit | both brackets, no stray backspace |
| E-BC-012 | telex | `tieengs<ent>` | `tiếng<ent>` | boundary-commit | Enter commits |
| E-BC-013 | telex | `tieengs<tab>` | `tiếng<tab>` | boundary-commit | Tab commits |
| E-BC-014 | telex | `tieengs1` | `tiếng1` | boundary-commit | digit commits the word and is inserted literally |
| E-BC-015 | telex | `hoaf<sp>hoaf` | `hòa hòa` | boundary-commit | state resets between identical words |
| E-BC-016 | telex | `dduwowngf<sp>phoos` | `đường phố` | boundary-commit | two transformed words in a row |
| E-BC-017 | telex | `vieejt<sp>nam` | `việt nam` | boundary-commit | reset must not re-apply the previous tone |
| E-BC-018 | telex | `1<sp>tieengs` | `1 tiếng` | boundary-commit | leading digit then a word |
| E-BC-019 | telex | `tieengs<sp><sp>` | `tiếng<sp><sp>` | boundary-commit | two consecutive spaces produce two spaces |
| E-BC-020 | telex | `tieengs"` | `tiếng"` | boundary-commit | quote commits |

### 1.14 macro — 20 rows

Macro table used by every row in this block (exactly these 7 entries):

| trigger | expansion |
|---|---|
| `btw` | `by the way` |
| `vn` | `Việt Nam` |
| `ko` | `không` |
| `dc` | `được` |
| `ts` | `tiến sĩ` |
| `aa` | `@@` |
| `vietnam` | `Việt Nam` |

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-MC-001 | telex | `btw<sp>` | `by the way<sp>` | macro | ASCII expansion, trigger consumed |
| E-MC-002 | telex | `btw,` | `by the way,` | macro | punctuation triggers the expansion too |
| E-MC-003 | telex | `btw<ent>` | `by the way<ent>` | macro | Enter triggers the expansion |
| E-MC-004 | telex | `vn<sp>` | `Việt Nam<sp>` | macro | expansion containing Vietnamese diacritics |
| E-MC-005 | telex | `vn,` | `Việt Nam,` | macro | expansion then punctuation |
| E-MC-006 | telex | `vn.` | `Việt Nam.` | macro | expansion then period |
| E-MC-007 | telex | `vn!` | `Việt Nam!` | macro | expansion then exclamation |
| E-MC-008 | telex | `vn<tab>` | `Việt Nam<tab>` | macro | Tab triggers the expansion |
| E-MC-009 | telex | `ko<sp>` | `không<sp>` | macro | expansion with ô |
| E-MC-010 | telex | `dc<sp>` | `được<sp>` | macro | expansion with đ and ươ |
| E-MC-011 | telex | `ts<sp>` | `tiến sĩ<sp>` | macro | expansion with a space and two tones |
| E-MC-012 | telex | `aa<sp>` | `@@<sp>` | macro | macro wins over the Telex `aa`→â transform |
| E-MC-013 | telex | `aa` | `â` | macro | without a boundary the transform still applies |
| E-MC-014 | telex | `vietnam<sp>` | `Việt Nam<sp>` | macro | longest trigger must win over the `vn` prefix |
| E-MC-015 | telex | `vn<bs><sp>` | `v<sp>` | macro | backspace before the boundary deletes the n and cancels the trigger |
| E-MC-016 | telex | `vn<sp>` | `vn<sp>` | macro | `macro_enabled=false` disables substitution entirely |
| E-MC-017 | telex | `btw` | `btw` | macro | no boundary, no expansion |
| E-MC-018 | telex | `vn<sp>vn<sp>` | `Việt Nam<sp>Việt Nam<sp>` | macro | trigger is reusable immediately after a boundary |
| E-MC-019 | telex | `dc<sp>rooif` | `được rồi` | macro | normal typing continues after an expansion |
| E-MC-020 | telex | `ko<sp>bieets` | `không biết` | macro | expansion then a transformed word |

### 1.15 spell-check — 15 rows (`spell_check=true`)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-SC-001 | telex | `tieengs` | `tiếng` | spell-check | valid syllable is still transformed |
| E-SC-002 | telex | `windows` | `windows` | spell-check | invalid syllable must not be transformed |
| E-SC-003 | telex | `qe` | `qe` | spell-check | `q` without `u` is invalid |
| E-SC-004 | telex | `zebra` | `zebra` | spell-check | z onset is invalid |
| E-SC-005 | telex | `f` | `f` | spell-check | lone invalid consonant stays literal |
| E-SC-006 | telex | `wq` | `wq` | spell-check | invalid cluster stays literal |
| E-SC-007 | telex | `nghieemj` | `nghiệm` | spell-check | valid ngh onset must still transform |
| E-SC-008 | telex | `traanj` | `trận` | spell-check | valid closed syllable |
| E-SC-009 | telex | `khoong` | `không` | spell-check | valid, tone-free |
| E-SC-010 | telex | `dduwowcj` | `được` | spell-check | valid with đ and ươ |
| E-SC-011 | telex | `test` | `test` | spell-check | English word must not become `tét` |
| E-SC-012 | telex | `hello` | `hello` | spell-check | no vowels are wrongly toned |
| E-SC-013 | telex | `giuwowngf` | `giường` | spell-check | valid gi + ươ |
| E-SC-014 | telex | `chuwax` | `chữa` | spell-check | valid ưa + ngã |
| E-SC-015 | telex | `ngieengs` | `ngieengs` | spell-check | `ngi` before a vowel is invalid (see OQ-9) |

### 1.16 restore-word — 16 rows (`restore_wrong_word=true`; E-RW-013…016 also `spell_check=true`)

`restore_wrong_word` is **not** an independent option: as in EVKey, it only restores when the
spell checker is on (the UI greys it out otherwise, `Cần bật Kiểm tra chính tả`). `E-RW-002/004/006/012`
therefore pin the inert case with `spell_check=false` (the transform stays), and `E-RW-013/014/015/016`
pin the same keystrokes with `spell_check=true` (the raw word comes back). See `28-engine-rulings.md` D4.

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-RW-001 | telex | `qwerty<sp>` | `qwerty<sp>` | restore-word | invalid word is restored to its raw keystrokes at the boundary |
| E-RW-002 | telex | `asdf<sp>` | `àd<sp>` | restore-word | `spell_check=false`: restore is inert, so the transform stays (`asdf` → `àd`); pair of E-RW-013 |
| E-RW-003 | telex | `tieengs<sp>` | `tiếng<sp>` | restore-word | a valid word must NOT be restored |
| E-RW-004 | telex | `nghieengxq<sp>` | `nghiễngq<sp>` | restore-word | `spell_check=false`: restore is inert, only the transform shows (`iee`→`iê`, `ng`→`ng`); pair of E-RW-014 |
| E-RW-005 | telex | `hwllo<sp>` | `hwllo<sp>` | restore-word | restore must undo a produced ư |
| E-RW-006 | telex | `vieetjq<sp>` | `việtq<sp>` | restore-word | `spell_check=false`: the invalid `q` suffix cannot trigger restore (`ee`→`ê`, `j`→`ệ`); pair of E-RW-015 |
| E-RW-007 | telex | `of<sp>` | `of<sp>` | restore-word | jfwz off: `of` was toned to `ò`, restore returns `of` |
| E-RW-008 | telex | `12ab<sp>` | `12ab<sp>` | restore-word | digits break the word before restore can fire |
| E-RW-009 | telex | `dduwowngf<sp>` | `đường<sp>` | restore-word | valid đ word must not be restored |
| E-RW-010 | telex | `quyeets<sp>` | `quyết<sp>` | restore-word | valid qu word must not be restored |
| E-RW-011 | telex | `hello<sp>` | `hello<sp>` | restore-word | English word unaffected |
| E-RW-012 | telex | `tieengsq<sp>` | `tiếngq<sp>` | restore-word | `spell_check=false`: restore is inert, so the valid prefix stays transformed and `q` is literal; pair of E-RW-016 |
| E-RW-013 | telex | `asdf<sp>` | `asdf<sp>` | restore-word | `spell_check=true` + restore: invalid word returns as raw keystrokes, tone keys stay letters |
| E-RW-014 | telex | `nghieengxq<sp>` | `nghieengxq<sp>` | restore-word | `spell_check=true` + restore: trailing junk makes the word invalid, whole raw word returns |
| E-RW-015 | telex | `vieetjq<sp>` | `vieetjq<sp>` | restore-word | `spell_check=true` + restore: invalid suffix restores the whole raw word |
| E-RW-016 | telex | `tieengsq<sp>` | `tieengsq<sp>` | restore-word | `spell_check=true` + restore: extra invalid letter reverts even the valid prefix |

### 1.17 english-passthrough — 26 rows (`spell_check=true`)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-EN-001 | telex | `hello` | `hello` | english-passthrough | the most common passthrough word |
| E-EN-002 | telex | `chrome` | `chrome` | english-passthrough | `r` must stay a letter, not become hỏi |
| E-EN-003 | telex | `test` | `test` | english-passthrough | `s` in the middle must not tone the word |
| E-EN-004 | telex | `google` | `google` | english-passthrough | `oo` must not become ô |
| E-EN-005 | telex | `windows` | `windows` | english-passthrough | `w` inside a word must not become ư |
| E-EN-006 | telex | `Microsoft` | `Microsoft` | english-passthrough | mixed case preserved |
| E-EN-007 | telex | `GitHub` | `GitHub` | english-passthrough | internal capitals preserved |
| E-EN-008 | telex | `docker` | `docker` | english-passthrough | `c` + `k` cluster |
| E-EN-009 | telex | `python` | `python` | english-passthrough | `y` must not be toned |
| E-EN-010 | telex | `javascript` | `javascript` | english-passthrough | `j` must stay a letter, not become nặng |
| E-EN-011 | telex | `async` | `async` | english-passthrough | `a` + `s` must not produce `á` |
| E-EN-012 | telex | `await` | `await` | english-passthrough | `w` at word start must not become ư |
| E-EN-013 | telex | `install` | `install` | english-passthrough | double l |
| E-EN-014 | telex | `keyboard` | `keyboard` | english-passthrough | `y` + `b` |
| E-EN-015 | telex | `performance` | `performance` | english-passthrough | 11 letters, no false transform |
| E-EN-016 | telex | `email@gmail.com` | `email@gmail.com` | english-passthrough | `@` and `.` boundaries |
| E-EN-017 | telex | `https://vietkb.app` | `https://vietkb.app` | english-passthrough | URL with colons and slashes |
| E-EN-018 | telex | `src/main.rs` | `src/main.rs` | english-passthrough | path with slash and dot |
| E-EN-019 | telex | `Ctrl+Shift` | `Ctrl+Shift` | english-passthrough | hotkey spelling typed as text |
| E-EN-020 | telex | `null` | `null` | english-passthrough | double l, no vowels |
| E-EN-021 | telex | `printf` | `printf` | english-passthrough | trailing f must not become huyền |
| E-EN-022 | telex | `console.log` | `console.log` | english-passthrough | dot boundary inside a token |
| E-EN-023 | telex | `npm run dev` | `npm run dev` | english-passthrough | three space-separated English words |
| E-EN-024 | telex | `build.rs` | `build.rs` | english-passthrough | file name with dot |
| E-EN-025 | telex | `3.14` | `3.14` | english-passthrough | numeric literal |
| E-EN-026 | telex | `floor` | `floor` | english-passthrough | `oo` inside a word must not become ô |

### 1.18 isolated — 18 rows (single keys and lone punctuation)

| id | method | input (raw keystrokes) | expected output | category | why it matters |
|---|---|---|---|---|---|
| E-IS-001 | telex | `a` | `a` | isolated | single vowel, no trigger |
| E-IS-002 | telex | `e` | `e` | isolated | single vowel |
| E-IS-003 | telex | `i` | `i` | isolated | single vowel |
| E-IS-004 | telex | `o` | `o` | isolated | single vowel |
| E-IS-005 | telex | `u` | `u` | isolated | single vowel |
| E-IS-006 | telex | `y` | `y` | isolated | single vowel |
| E-IS-007 | telex | `d` | `d` | isolated | d without a second d stays d |
| E-IS-008 | telex | `q` | `q` | isolated | lone consonant |
| E-IS-009 | telex | `s` | `s` | isolated | tone key with no vowel is a literal letter |
| E-IS-010 | telex | `f` | `f` | isolated | same for f |
| E-IS-011 | telex | `j` | `j` | isolated | same for j |
| E-IS-012 | telex | `r` | `r` | isolated | same for r |
| E-IS-013 | telex | `x` | `x` | isolated | same for x |
| E-IS-014 | telex | `z` | `z` | isolated | same for z |
| E-IS-015 | telex | `123` | `123` | isolated | digits pass through untouched |
| E-IS-016 | telex | `!@#` | `!@#` | isolated | symbols pass through untouched |
| E-IS-017 | telex | `a<sp>a` | `a a` | isolated | two isolated letters with a boundary |
| E-IS-018 | telex | `a<sp><sp>b` | `a<sp><sp>b` | isolated | consecutive spaces are preserved exactly |

---

## 2. Charset round-trip cases

Sample sentence (S1): **`Đường về quê hương`**
Six representative letters plus đ, both cases: `ă â ê ô ơ ư đ` / `Ă Â Ê Ô Ơ Ư Đ`.

Two shipped forms: `unicode` (NFC, the only value the UI exposes) and `unicode_nfd`
(config-file only, for the rare program that asks for it).

Round-trip expectation:
- **R1 encode→decode** — `normalize(encode(text)) == NFC(text)` for every Vietnamese letter.
- **R2 byte-exactness** — encoding is deterministic; no silent re-normalisation drift.
- **R3 no-loss** — encoding must never drop a diacritic silently; a letter with no code
  point in the target form is a **defect**, not a fallback to the base letter.

### 2.1 `unicode` (NFC) — verified by codepoint

| id | letter | codepoint | notes |
|---|---|---|---|
| CS-UN-01 | ă | U+0103 | LATIN SMALL LETTER A WITH BREVE |
| CS-UN-02 | â | U+00E2 | |
| CS-UN-03 | ê | U+00EA | |
| CS-UN-04 | ô | U+00F4 | |
| CS-UN-05 | ơ | U+01A1 | LATIN SMALL LETTER O WITH HORN |
| CS-UN-06 | ư | U+01B0 | LATIN SMALL LETTER U WITH HORN |
| CS-UN-07 | đ | U+0111 | LATIN SMALL LETTER D WITH STROKE |
| CS-UN-08 | Ă | U+0102 | |
| CS-UN-09 | Â | U+00C2 | |
| CS-UN-10 | Ê | U+00CA | |
| CS-UN-11 | Ô | U+00D4 | |
| CS-UN-12 | Ơ | U+01A0 | |
| CS-UN-13 | Ư | U+01AF | |
| CS-UN-14 | Đ | U+0110 | |

S1 code points: `Đ U+0110, ư U+01B0, ờ U+1EDD, n, g, space, v U+0076, ề U+1EC1, space, q, u, ê U+00EA, space, h, ư U+01B0, ơ U+01A1, n, g`. R1+R2 required. **Status: VERIFIED** (table is Unicode-standard).

### 2.2 `unicode_nfd` — verified by codepoint

Every marked letter is base + combining mark; `đ` has **no** decomposition and stays U+0111.

| id | letter | codepoints | notes |
|---|---|---|---|
| CS-NF-01 | ă | U+0061 U+0306 | a + COMBINING BREVE |
| CS-NF-02 | â | U+0061 U+0302 | a + COMBINING CIRCUMFLEX |
| CS-NF-03 | ê | U+0065 U+0302 | |
| CS-NF-04 | ô | U+006F U+0302 | |
| CS-NF-05 | ơ | U+006F U+031B | o + COMBINING HORN |
| CS-NF-06 | ư | U+0075 U+031B | u + COMBINING HORN |
| CS-NF-07 | đ | U+0111 | no decomposition |
| CS-NF-08 | Ă | U+0041 U+0306 | |
| CS-NF-09 | Â | U+0041 U+0302 | |
| CS-NF-10 | Ê | U+0045 U+0302 | |
| CS-NF-11 | Ô | U+004F U+0302 | |
| CS-NF-12 | Ơ | U+004F U+031B | |
| CS-NF-13 | Ư | U+0055 U+031B | |
| CS-NF-14 | Đ | U+0110 | no decomposition |

Tone marks in NFD are also combining (e.g. `ế` = U+0065 U+0302 U+0301), with the **mark before the
tone**. R1+R2 required. **Status: VERIFIED**; ordering check (`U+0302` before `U+0301`) is part of R2.

## 3. UI/UX acceptance tests

Screens under test (sidebar order): **1 Tổng quan** (`overview`), **2 Gõ** (`typing`),
**3 Bảng mã** (`charset`), **4 Gõ tắt** (`macro`), **5 Loại trừ** (`apps`),
**6 Phím tắt** (`hotkeys`), **7 Hệ thống** (`system`), **8 Giới thiệu** (`about`).
Window baseline 1040×720 (contract `ui.window_*`).

| id | area | steps | expected | prio |
|---|---|---|---|---|
| UI-1 | navigation | Click each of the 8 sidebar entries in order, then reverse order | each opens its screen, exactly one entry is marked active, content changes, no full reload, no console error | P0 |
| UI-2 | navigation | Reach every screen with the keyboard only (Tab to the sidebar, arrow keys, Enter) | all 8 reachable, focus ring visible on the sidebar, no trap | P0 |
| UI-3 | navigation | Resize 1024×700 → 1366×768 → 1440×900 | no horizontal scrollbar, no clipped label, no overlap, sidebar stays fixed width | P0 |
| UI-4 | Tổng quan | Open the screen with the app enabled | shows enabled state, current kiểu gõ, current bảng mã, version, config path; changing method here works | P0 |
| UI-5 | Tổng quan | Click the enable switch | switch flips immediately, tray icon reflects the new state within 1 s, no Save button involved | P0 |
| UI-6 | Gõ | Open the screen | every checkbox from the contract `[typing]` block is present with its EVKey Vietnamese label verbatim (`Đặt dấu kiểu mới`, `Đặt dấu tự do`, `Cho phép 'f j w z' làm phụ âm`, `Tự động viết hoa sau Enter . ! ?`, `Bật kiểm tra chính tả`, `Tự động khôi phục phím với từ sai`, …) | P0 |
| UI-7 | Gõ | Toggle `Đặt dấu kiểu mới` off, type `hoaf` in Notepad, toggle on, retype | first yields `hoà`, second yields `hòa`, with no restart and no Save | P0 |
| UI-8 | Bảng mã | Open the screen | exactly 9 bảng mã options listed with their Vietnamese names: Unicode dựng sẵn, Unicode tổ hợp, TCVN3 (ABC), VNI Windows, VIQR, VISCII, BK HCM1, VPS, UTF-8 | P0 |
| UI-9 | Bảng mã | Select each charset and read the sample preview below the list | preview re-renders the same sample sentence for each charset and is legible; unknown charsets show a placeholder, never mojibake silently | P0 |
| UI-10 | Gõ tắt | Open the screen with 7 macros configured | table shows trigger + expansion, one row per macro, sorted or searchable, count shown | P0 |
| UI-11 | Gõ tắt | Search `vn` in the filter field | list filters to matching triggers and expansions, case-insensitive, empty state shown for no match | P1 |
| UI-12 | Gõ tắt | Add `btw` → `by the way`, press the row's confirm | row appears, typing `btw` + space in Notepad expands immediately (live apply) | P0 |
| UI-13 | Gõ tắt | Edit an existing expansion, then delete a row (single) and delete 3 selected rows at once | edit persists, each delete removes exactly the selected rows, no confirmation-less bulk loss of more than selected | P0 |
| UI-14 | Gõ tắt | Import a file with 5000 entries, then export | import reports the inserted count; export writes a file that re-imports to the same 5000 entries; UI stays responsive (< 1 s per action, no freeze) | P0 |
| UI-15 | Gõ tắt | Import a malformed file (bad line, duplicate trigger, empty trigger) | error message names the offending line, valid rows either all rejected or all imported consistently — never a half-applied state | P1 |
| UI-16 | Loại trừ | Click "add app", use the running-app picker | picker lists running processes with a readable name and `process.exe`, search filters it, choosing one adds the rule | P0 |
| UI-17 | Loại trừ | Add `game.exe` with enabled=false, start a dummy process with that name | typing in that process produces raw ASCII; other apps still transform | P0 |
| UI-18 | Loại trừ | Remove a rule | rule disappears, typing in that process transforms again, no restart | P0 |
| UI-19 | Loại trừ | Toggle `Loại trừ ứng dụng tự động` (`apps.remember_per_app`) on, then use the tray on two apps | remembered rules appear in the list and survive a restart | P1 |
| UI-20 | Phím tắt | Focus the "bật" hotkey field and press `Ctrl+Alt+V`, then Enter | field shows the canonical string `Ctrl+Alt+V`, Enter commits, the hotkey works system-wide | P0 |
| UI-21 | Phím tắt | In the capture field press Esc after typing a combo | capture cancels, previous value restored, nothing persisted | P0 |
| UI-22 | Phím tắt | In the capture field press Backspace | field clears to empty, and empty means "no hotkey" (contract §5), persisted as `""` | P0 |
| UI-23 | Phím tắt | Set `enable` and `disable` to the same combo | inline conflict message naming the other field, the value is not persisted, other fields stay editable | P0 |
| UI-24 | Phím tắt | Type a bare letter with no modifier into the field | inline error, not persisted; UI must not silently accept `V` | P0 |
| UI-25 | Phím tắt | Press "reset" on one field and on the whole section | resets to contract defaults (`F1`/`F2`, `F11`, `F6`, `F8`, `F9`, empty others) | P1 |
| UI-26 | Phím tắt | Check every field in the section | exactly the 9 contract hotkeys are present, no extras, no missing (`enable`, `disable`, `toggle`, `restore_word`, `app_exclude`, `convert_tool`, `macro_panel`, `macro_toggle`, `open_settings`) | P0 |
| UI-27 | Hệ thống | Toggle theme light → dark → system | whole window repaints with no flash of unstyled content, both tokens sets readable, no hardcoded colour left behind | P0 |
| UI-28 | Hệ thống | Switch language vi → en → vi | every visible string changes on the same screen without reload; product terms stay Vietnamese where they are EVKey terminology; no missing-key placeholder text | P0 |
| UI-29 | Hệ thống | Toggle `Khởi động cùng Windows` | state persists and `HKCU\...\Run` (or Task Scheduler) entry appears/disappears accordingly | P1 |
| UI-30 | Hệ thống | Click "mở thư mục cấu hình" | Explorer opens `%APPDATA%\VietKB` (created if absent), no console window flashes | P2 |
| UI-31 | Giới thiệu | Open the screen | version, arch, hook state, config path from `app_info()` are shown and match reality | P1 |
| UI-32 | live-apply | Change 10 settings across 4 screens quickly | each change is debounced (~150 ms), no lost write, final config file matches the last UI state | P0 |
| UI-33 | live-apply | Make a change with the config file read-only | a toast reports the persistence failure, the UI does not pretend the change was saved | P0 |
| UI-34 | live-apply | Search every screen for a "Save"/"Lưu" button | none exists; no screen requires an explicit save | P0 |
| UI-35 | focus | Tab through a full screen | focus order follows visual order, every focused control has a visible ring with ≥ 3:1 contrast against its background | P0 |
| UI-36 | focus | Open a dialog/panel (macro editor, app picker) | focus moves into it, Esc closes, focus returns to the invoking control | P0 |
| UI-37 | contrast | Measure text and control colours in both themes | body text ≥ 4.5:1, large text ≥ 3:1, borders of required controls ≥ 3:1; report measured pairs | P1 |
| UI-38 | tray | Right-click the tray icon | menu contains exactly: bật/tắt, kiểu gõ submenu (Telex/VNI/VIQR), bảng mã submenu (9), mở cài đặt, thoát | P0 |
| UI-39 | tray | Toggle enable from the tray, then from the UI | icon changes between the enabled and disabled artwork, the UI switch reflects it within 1 s | P0 |
| UI-40 | tray | Change kiểu gõ and bảng mã from the tray only | the UI shows the new values next time the window opens; typing honours them immediately | P0 |
| UI-41 | window | Close the settings window with the X, with `start_minimized`/close-to-tray behaviour | process keeps running, tray icon stays, hook still transforms; "thoát" from the tray exits the process | P0 |
| UI-42 | window | Launch the exe a second time while running | no second tray icon, no second hook; the existing window is focused (single instance) | P0 |
| UI-43 | first-run | Delete `%APPDATA%\VietKB`, first launch | first-run experience appears (language choice and/or short intro), defaults match the contract `Default`, config file created at exit or on first change | P0 |
| UI-44 | persistence | Change method, charset, 3 toggles, 1 hotkey, 1 macro, 1 app rule; quit; relaunch | every value is identical after restart; `version = 1` present; unknown keys written by hand are still present after a UI write | P0 |
| UI-45 | persistence | Truncate `config.toml` mid-line, relaunch | app starts with defaults, shows a recovery notice, and **backs up** the corrupt file instead of deleting it | P0 |
| UI-46 | persistence | Set `config.toml` to a newer `version = 99` | app refuses to downgrade silently: either preserves the file and runs read-only defaults, or migrates; chosen behaviour must be documented and tested once | P1 |
| UI-47 | scale | Open Gõ tắt with 5000 macros, scroll to the end, type in the search box | scroll stays smooth (no multi-second freeze), search responds < 200 ms | P1 |
| UI-48 | empty states | Open Loại trừ with no rules, Gõ tắt with no macros | explicit empty state text, not a blank panel; the add action is visible | P2 |

---

## 4. Performance protocol (P1–P7)

**Nothing in this section has been measured.** The Dev builds are in progress; measuring now would
be invalid. QA runs this only after Lead announces the gate build.

### 4.0 Isolation rules (apply to every row below)

1. One measurement at a time. `log.md` line `BENCH: qa start <UTC ts>` before, `BENCH: qa end` after.
2. No other agent session, no `cargo build`/`tauri build`, no `npm run dev` on the machine.
3. Power plan **High performance**, on AC, screen on, no sleep, no Windows Update, Defender real-time
   scan paused for `E:\vietkb\target` and `%APPDATA%\VietKB`.
4. Only the measurement target is in the foreground; nothing else animating (no browser tabs playing
   media, no OBS, no screen share).
5. Build once in `--release` **before** the run; nothing writes `target/` during measurement.
6. Three repetitions; report each, plus median. A budget fails if the **median** misses it, or if any
   repetition misses it by more than 20 % (instability is itself a defect).
7. Machine facts to record with every run: OS build, CPU, RAM, build hash, timestamp.

### 4.1 Budgets

| id | budget | exact command | samples | warm-up | failure rule |
|---|---|---|---|---|---|
| P1 | `process_key` p99 ≤ 50 µs, zero heap alloc on the steady path | `cargo bench -p vkb-core --bench engine -- --warm-up-time 1 --measurement-time 5 --save-baseline qa-s1` then `cargo test -p vkb-core --test no_alloc --release` | 100 000 iterations per case (charter) | 1 s criterion warm-up + first 10 000 iterations discarded | p99 > 50 µs, or the counting allocator observes **any** alloc on the `Noop`/single-letter path → defect |
| P2 | e2e p95 ≤ 8 ms, p99 ≤ 15 ms | instrumented build, script in §4.2; `vkb-cli e2e --target notepad --words docs/evkey-clone-work/perf/500w.txt --runs 5 --csv out-notepad.csv` and the same with `--target chrome` | 500 words × 5 runs = 2500 keystrokes per app, per repetition; 3 repetitions | first 50 words of run 1 discarded (app start, cold font cache) | p95 > 8 ms or p99 > 15 ms on either app, median of 3 |
| P3 | idle RSS ≤ 20 MB, GUI closed | `Get-Counter '\Process(vkb)\Working Set - Private' -SampleInterval 60 -MaxSamples 3` (process name without `.exe`); cross-check `(Get-Process vkb).PrivateMemorySize64` | 3 samples after 10 min idle, taken at 60 s spacing | 10 min idle, window closed, tray only | median of the 3 > 20 MB (20 971 520 bytes) |
| P4 | idle RSS ≤ 90 MB with settings window open | same counter with the window open and on the Tổng quan screen, 10 min idle | 3 samples | 10 min idle | median > 90 MB (94 371 840 bytes) |
| P5 | CPU ≤ 0.1 % average over 1 h idle | `(Get-Counter '\Process(vkb)\% Processor Time' -SampleInterval 5 -MaxSamples 720).CounterSamples \| Measure-Object CookedValue -Average` | 720 samples × 5 s = 1 h | 5 min idle before sampling starts | average > 0.1 (counter is % of one core; report the raw number, do not sum cores) |
| P6 | cold start → hook-ready ≤ 400 ms | `.\target\release\vkb.exe --bench-startup --runs 10 --csv startup.csv` (instrumented; prints process start → first successful `SetWindowsHookEx` + first `GetMessage`) | 10 cold runs (after reboot); 5 warm-up runs discarded | 5 warm runs | median of 10 cold runs > 400 ms, or any run > 480 ms |
| P7 | installer ≤ 10 MB | `Get-ChildItem target\release\bundle\nsis\*.exe, target\release\bundle\msi\*.msi \| Select-Object Name, Length` | 1 artifact each | n/a | either artifact > 10 485 760 bytes |

### 4.2 P2 script plan (500 words → Notepad and Chrome)

Missing piece the Dev must provide: an `--e2e-trace` build that (a) stamps `QueryPerformanceCounter`
on hook entry, (b) stamps it again after the injector's `SendInput` returns, (c) appends
`app,word_index,char_index,us` to a CSV. Without those two stamps the budget cannot be measured.

1. Prepare `perf/500w.txt`: exactly 500 Vietnamese words, no duplicates in the first 200, all
   syllables valid, mixed tone positions, ~2 % English words, one word per line.
2. Prepare the driver: `vkb-cli e2e --target notepad --words perf/500w.txt --interval-ms 60 --runs 5
   --csv perf/out-notepad.csv`. The CLI must focus the target window (`SetForegroundWindow`),
   synthesise keystrokes so the real hook fires, and wait `--interval-ms` between words so the app
   can paint (the interval is not measured; only hook→SendInput is).
3. Notepad run: open a blank Notepad, click into the edit control, confirm the caret is at 0, start the
   driver, verify the resulting text equals `500w.txt` joined with spaces (any mismatch invalidates the
   run — correctness before latency).
4. Chrome run: open a new tab with `data:text/html,<textarea id=t style="width:100%;height:100%">`,
   focus the textarea, same command with `--target chrome`, same text equality check.
5. Repeat 3× per app; keep every CSV. Compute p50/p95/p99 with
   `Get-Content perf\out-notepad.csv | ConvertFrom-Csv | Measure-Object -Property us -Percentile 95,99`
   (or a 20-line Rust helper in `vkb-cli` since PowerShell has no native percentile — preferred).
6. Record: per-app table (p50, p95, p99, max, n), the text-equality result, `interval-ms`, build hash.
7. Invalid run (must be redone, not averaged in): text mismatch, another window stealing focus, a
   keystroke dropped, interval jitter > 20 %, any background build.

**Caveat to settle first (OQ-27):** synthesised `SendInput` keys are also seen by our own low-level
hook, so the harness must either run with the hook in pass-through-for-synthetic-keys mode or prove the
loopback does not double-count. If neither is acceptable, P2 falls back to a human-typed 500-word run
with the CSV stamping enabled, which is valid but not repeatable to the millisecond.

---

## 5. Defect log template + open questions

### 5.1 Defect row format (`docs/evkey-clone-work/60-defects.md`)

| field | rule |
|---|---|
| `id` | `D-<nnn>`, monotonically increasing, never reused |
| `severity` | `S1` crash/data loss/typing impossible · `S2` wrong Vietnamese output, budget breach · `S3` wrong-but-recoverable UI/behaviour · `S4` cosmetic |
| `area` | `engine` / `charset` / `hook` / `config` / `ui` / `tray` / `installer` / `perf` |
| `repro` | numbered, minimal, starting from a clean state; include the exact keystrokes |
| `expected` | quote the test id and its expected string verbatim |
| `actual` | exact observed string, codepoints if invisible (`U+1EDD` vs `U+00F2` + `U+0300`) |
| `evidence` | log excerpt, screenshot path, CSV row range, hex dump — one artefact per claim |
| `test_id` | `E-*`, `CS-*`, `UI-*` or `P<n>`; a defect with no test id means the corpus is incomplete |
| `status` | `open` → `fixed` (Dev, with commit) → `verified` (QA, re-ran the same test id) → `closed` |

Rules: one defect per row; a defect is only `verified` after the **same** test id passes on a build
that contains the fix; corpus gaps found by a defect are added to §1 in the same change.

### 5.2 Open questions (a test cannot settle these)

| id | question | blocks |
|---|---|---|
| OQ-1 | `free_tone` ("Đặt dấu tự do") is undefined in the contract. Does it mean "tone key may be typed anywhere in the syllable" (corpus assumes yes), or the legacy `smart_mark` mark-position switch that the old code used? If it is the mark-position switch it collides with `modern_tone`. | E-TB-008, E-TT-062, §1.2 generally |
| OQ-2 | Exact `modern_tone` rule table. QA assumes: `oa`/`oe`/`uy` open → first vowel; any vowel cluster with a coda → last vowel; `â/ê/ô/ơ/ư` wins; 3-vowel nuclei → middle. Is `uê`/`uơ`/`uya` covered, and does the option ever affect `qu`/`gi`? | §1.7, §1.8 |
| OQ-3 | `ww` escape output. Corpus pins `w`; `uw` is equally defensible from the `ass`→`as` rule. Needs one EVKey screenshot. | E-TW-002 |
| OQ-4 | Does `w` after **any** consonant produce ư (`tw`→`tư`), or only after `u`/`o`/`a`/word start? Corpus pins `tư` pending confirmation. | E-TW-018 |
| OQ-5 | VNI: is `8` accepted on the `uo` nucleus (`thuong8` → `thương`) as legacy tolerated, or must users type `7` twice? | E-VN-030..033 |
| OQ-6 | VNI `0`: removes only the tone, or also `đ`/`ă`/`â`? Legacy `di90` → `di` implies it strips đ, which is surprising. | E-VN-044 |
| OQ-7 | `MAX_WORD` (32) overflow: commit-then-reset, or `Reject`? And does the boundary fire with the 33rd key emitted literally? | not yet in corpus (needs an answer) |
| OQ-8 | English passthrough algorithm: validity table, and which table? What exactly makes `test` pass through while a genuine misspelling like `tét` must not be "corrected"? | §1.17 |
| OQ-9 | Spell-check syllable inventory: which list is authoritative and what is its licence? Does `ngi` + vowel count as invalid (corpus pins raw)? | E-SC-015 |
| OQ-10 | Macro matching: case-sensitive? Only at word start? What does `xvn` + space do? | E-MC-014, §1.14 |
| OQ-11 | Is a macro expansion converted to the active bảng mã (e.g. `vn` → TCVN3 bytes)? | E-MC-004 with charset ≠ unicode |
| OQ-12 | May a macro trigger contain a Telex tone key (`as`, `af`)? Who wins — macro or transform? Corpus only pins `aa`. | E-MC-012 |
| OQ-13 | `auto_capitalize` trigger set: Enter, `.`, `!`, `?` only? Does it fire after a digit or inside a URL? Does it uppercase the first letter of an acronym? | no test yet |
| OQ-14 | Which reference implementation is authoritative for TCVN3 / VNI Windows / VISCII / BK HCM1 / VPS? No charset of these five can be signed off without one. | §2.5–2.9 |
| OQ-15 | What is `utf8` as a typing output charset, distinct from `unicode`? If identical, why two slugs? | §2.3 |
| OQ-16 | `disable_on_non_us_layout`: does the app disable the hook, pass keys through raw, or show a warning? Where is the state observable? | no test yet |
| OQ-17 | `use_clipboard` fallback: trigger condition, ordering vs `SendInput`, and the exact clipboard-restore guarantee for non-text formats (charter C4). | no test yet |
| OQ-18 | `mechanism = "ime"`: in scope for Sprint 1, or an accepted-but-stubbed value? | config schema |
| OQ-19 | Tray "icon reflects state": how many artworks (enabled/disabled? V/E?), who produces them, what DPI sizes? | UI-39 |
| OQ-20 | `30-plan.md` names 6 screens, this corpus tests 8. Confirm the 8 screen ids, since `ui.last_tab` must accept them. | UI-1..UI-3 |
| OQ-21 | Hotkey canonical form for a modifier-only combo: `Ctrl+Shift` with no key segment? Contract says order `Ctrl+Alt+Shift+Win+Key` but the key is optional. | UI-20 |
| OQ-22 | Conflict detection scope: only our own 9 fields, or also OS-reserved combos (`Win+L`, `Ctrl+Alt+Del`)? Exact message text? | UI-23 |
| OQ-23 | Per-app rules: precedence when the exclude list and a per-app override both match, and is process-name matching case-insensitive? | UI-17, UI-19 |
| OQ-24 | `restore_word` hotkey after the caret has moved: the engine only knows its own word buffer, so what is the defined behaviour when the user clicked elsewhere? | §1.16, hotkey `restore_word` |
| OQ-25 | Legacy `settings.ini` migration: which keys map to which v1 fields, and what happens when the INI is partially invalid (skip + report, or abort)? | UI-43, UI-44 |
| OQ-26 | Bảng mã preview: does it convert the live sample typed by the user, or always a fixed sentence? Does it convert the clipboard on demand (Sprint 2 "chuyển đổi bảng mã")? | UI-9 |
| OQ-27 | Is a scripted `SendInput` harness valid evidence for P2, given our own hook sees synthetic keys? If not, P2 needs the human-typed protocol in §4.2. | P2 |
| OQ-28 | `suggestion_fix_1`, `suggestion_fix_2`, `metro_fix`, `send_standard` exist in the schema but have no defined semantics anywhere in the contract. Ship them as no-ops with a tooltip, or implement? | Gõ screen, UI-6 |
| OQ-29 | What is the exact escape rule for repeating a *mark* key on an already-marked syllable (`aass` → `aas` assumed), and does it apply uniformly to `aw`/`ow`/`uw`? | §1.3 |
| OQ-30 | Does `boundary()` clear the word buffer on caret movement only, or also on mouse click inside the same word? Affects every backspace/restore test. | §1.12, §1.16 |