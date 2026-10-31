# 60 — UI defects, round 3 (rebuilt shell)

**Date:** 2026-10-29 · **Owner:** QA · **Build:** `target\release\vietkb.exe`
(2026-10-29 03:41:02, SHA-256 `15BCDA2A3DB7576FEB459B7F54F60E5AAF73F1D67BA22F2301430B33C55F29F3`;
re-verified 2026-10-29 05:25:51 on SHA-256 `B12F9FCF2916BCCB10E464C613E0196F16D61A450AEAD409AE0A47DC39655757`).
**Scope:** failures from `50-test-cases.md` §3 (`UI-1…UI-46`), 2560 × 1440 @ 125 %.
No product code was changed by this pass. Size shown as P1 (feature unreachable) →
P3 (cosmetic); the Lead sets the final priority.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-UI-01 | P1 | Bảng mã (charset) removed from every UI surface | 1) open the main window and look for `Bảng mã`; 2) open the tray menu and look for `Bảng mã ▸` | the main window offers the charset picker (as the brief lists) **and** the tray offers `Bảng mã ▸` with the 6 verified charsets, both live-applied | neither exists: the main window has no `Bảng mã` label/select (`UI-11`), and the tray menu is `Bật/tắt tiếng Việt · Kiểu gõ · Mở cài đặt… · Phím tắt… · Thoát VietKB` with no charset submenu (`UI-23`, `UI-26`). `input.charset` can only be changed by editing `config.toml` | `qa/evidence/UI-11.txt`, `UI-23.txt`, `UI-26.txt`; `ui/src/screens/Quick.svelte` has no `PICKER_CHARSETS` import; `crates/vkb-app/src/tray.rs` builds only the method submenu | OPEN |
| D-UI-02 | P2 | language switch is a no-op (**FIXED + VERIFIED 2026-10-29** — the control was removed; see §Re-verification) | `Nâng cao` → `Ngôn ngữ hiển thị` → `English`; wait 1.5 s; read the main window | the visible strings switch to English (product terms such as `Kiểu gõ`, `Bảng mã`, `Gõ tắt` stay Vietnamese) with no missing-key placeholders | `system.language` becomes `"en"` and persists, but every label on every window stays Vietnamese (`Bật tiếng Việt`, `Kiểm tra chính tả`, `Đóng`, headings) — the switch has no visible effect | `qa/evidence/UI-17.txt`; `ui/src/lib/i18n.ts` dictionaries are never read by `Quick/Advanced/Hotkeys/Macros` | OPEN → **FIXED (control removed) / VERIFIED** |
| D-UI-03 | P2 | modifier-only hotkeys cannot be captured and bypass the conflict check (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | `Phím tắt` → focus `Đổi kiểu gõ` → press `Ctrl+Shift` (the toggle's chord) → Enter | the field records `Ctrl+Shift`, shows a `Trùng` warning naming `Bật/tắt tiếng Việt`, and does not commit | the field stays unchanged: `held` is filled on key-down but `onkeyup` resets `draft` to the stored value when the last modifier is released (`CaptureField.onkeyup`), so nothing is captured; no conflict check runs and nothing is persisted | `qa/evidence/UI-32.txt` (`fieldCaughtModifierOnly=False`, `config=""`, no warning) | OPEN → **FIXED / VERIFIED** |
| D-UI-04 | P2 | a hotkey conflict already in `config.toml` is invisible (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | stop the app; set `cycle_method = "Ctrl+Shift+Z"` (same as `restore_word`); start; open `Phím tắt` | both fields show a conflict state on load (the file is a supported way to edit settings) | both fields render `Ctrl+Shift+Z` with no `Trùng` marker; `check_hotkey_conflict` is only called while a field is being edited | `qa/evidence/UI-33.txt` (`cycleField=True`, `restoreField=True`, `warnings=`) | OPEN → **FIXED / VERIFIED** |
| D-UI-05 | P2 | `Kiểm tra chính tả` dependency is not live across windows (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | open `Nâng cao`, then turn `Kiểm tra chính tả` off in the main window and wait 1.6 s | `Tự động khôi phục từ gõ sai` disables within 1 s in the already-open window (and stays disabled when reopened) | the open window keeps the row enabled; only a reopened `Nâng cao` shows it disabled (each window holds its own settings snapshot; only `vkb:status` input fields are broadcast) | `qa/evidence/UI-34.txt` (`disabledLive=False`, `disabledAfterReopen=True`) | OPEN → **FIXED / VERIFIED** |
| D-UI-06 | P2 | macro table hides everything after the 8th row (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | write 12 macros to `config.toml`, start, open `Bảng gõ tắt` | all 12 rows are reachable (render them, or give the table a scroll area) | only 8 row buttons render (`rows.slice(0, 8)` in `Macros.svelte`), there is no scrollbar, and the footer admits `Đang hiện 8 trong 12`; entries 9–12 are only reachable through search | `qa/evidence/UI-40.txt` | OPEN → **FIXED / VERIFIED** |
| D-UI-07 | P2 | VIQR is no longer selectable (contract deviation, observation) | read the `Kiểu gõ` options and the tray submenu | the contract lists three methods (Telex, VNI, VIQR); both surfaces should offer them | both offer Telex and VNI only (`UI-10` passes against the 2-item catalogue; `UI-25` shows 2 tray items); `vkb-core` still implements VIQR | `qa/evidence/UI-10.txt`, `UI-25.txt`; `ui/src/lib/catalog.ts` `METHODS` has 2 entries | OPEN |

## Notes on the three non-defect observations

* `UI-24` tooltip evidence shows two strings concatenated
  (`VietKB · Đã tắt tiếng Việt VietKB · Tiếng Việt · Telex`). This is a **shell ghost icon** left by
  force-killing the app between harness groups (Windows keeps the dead icon until the flyout is
  refreshed); it also makes the automation see two buttons. A normal `Thoát VietKB` leaves one icon
  (verified by `UI-28`). Not a product defect.
* `UI-39` failed intermittently only when the native “Open” dialog lost foreground to the macro
  window; after focusing the dialog itself the round-trip is stable
  (export 4 lines → delete `ko` → import → `Đã nhập 3 mục`). Harness issue, not a UI defect.
* `UI-8`/`UI-44` are measured at 125 % only: the display scaling was not changed, and the focus-ring
  contrast was read from `tokens.css`, not measured on screen.

## Re-verification of D-UI-02…06 (2026-10-29)

**Independent re-run by QA-subagent — the fix report was not trusted.** Build:
`target\release\vietkb.exe`, 2026-10-29 05:25:51 (+07:00), SHA-256
`B12F9FCF2916BCCB10E464C613E0196F16D61A450AEAD409AE0A47DC39655757`
(`npm --prefix ui run build` + `cargo build -p vkb-app --release --features custom-protocol`).
Method: same UIA/MSAA harness (`qa/lib.ps1`), 2560 × 1440 @ 125 %, no screenshots, no window moves;
the app was launched hidden and a second launch was used to show the window. Baseline
`config.toml` was restored byte-identical at the end (`Compare-Object` empty;
`system.language="vi"`, `restore_word="Ctrl+Shift+Z"`, `spell_check=true`, macros `m01`…`m12`).

| id | measurement (this run) | verdict |
|---|---|---|
| D-UI-02 | all windows enumerated for a text/combobox matching `Ngôn ngữ`: **0 controls**; `system.language` still `"vi"` | **PASS** — the dead i18n switch is gone, no language surface remains |
| D-UI-03 | focus `Đổi kiểu gõ`, hold then release `Ctrl+Shift` **with no Enter**: field label became `… phím tắt hiện tại Ctrl+Shift`, warnings `Trùng với Bật/tắt tiếng Việt — chọn tổ hợp khác.` (+ the mirror warning on the toggle row), `hotkeys.cycle_method` unchanged (still unset). Same chord on `Khôi phục từ gõ sai`: label `Ctrl+Shift`, same `Trùng … Bật/tắt tiếng Việt` warning, `restore_word` unchanged `Ctrl+Shift+Z` | **PASS** — modifier-only chords are captured and blocked by the conflict check |
| D-UI-04 | `cycle_method = "Ctrl+Shift+Z"` (= `restore_word`) written then app restarted: both fields present, **2 warnings on load** (`Trùng "Ctrl + Shift + Z" với Khôi phục từ gõ sai…` and `… với Đổi kiểu gõ…`); clearing the cycle field (Backspace+Enter) left **0 warnings** and `cycle_method` empty | **PASS** |
| D-UI-05 | `Nâng cao` left open the whole time: restore row `IsEnabled=True` with spell check on; toggled `Kiểm tra chính tả` off in the main window → row `IsEnabled=False` in the **same** window after 1.6 s (`spell_check=false` persisted); toggled back on → `True` again | **PASS** — dependency is live across windows |
| D-UI-06 | 12 macros in `config.toml`: **12 row buttons** render (`Sửa m*`), **9 visible** at 900 × 700 physical / 720 × 560 logical, 3 offscreen (`m7`, `m8`, `m9` at string order). `ScrollItemPattern.ScrollIntoView()` on `m7` → `IsOffscreen=False`, rows scroll (`m1/m10/m11` go offscreen); `Sửa m12` present and visible, and invoking `Sửa m7` opened its editor | **PASS** — every row reachable, footer no longer truncates at 8 |

No defect was reopened. Open items stay as before: D-UI-01 (charset surface) and D-UI-07 (VIQR),
neither in this scope.


## Closure — 2026-10-29
- **D-UI-01 closed as intentional**: charsets were removed by owner decision; UI-11/23/26 are N/A, not defects.
- **D-UI-02 … D-UI-06 fixed and independently re-verified** by QA on build B12F9FCF…: 43 pass / 3 N/A / 0 fail.
- **D-UI-07 closed as documentation**: the contract now lists Telex and VNI only.

---

# 60 — Engine special cases, round 2026-10-30 (stream B1)

**Owner:** QA · **Scope:** the `/` report from `91-round-plan-sound-cases.md` plus its siblings,
measured on the pure engine through `vkb_core::simulate` / `simulate_keys` / `Engine` and the
`vkb-cli simulate` command. Full inventory and the root-cause analysis:
`51-special-cases.md`. Raw output: `qa/special/matrix.txt` (78-row matrix) and `qa/special/repro.txt`
(CLI transcript + action trace).
**Layer:** `vkb-core` only. The `vkb-windows` hook is provably neutral — the char path and the real
virtual-key path produce identical output (`special_cases.rs::vk_path_matches_char_path`, PASS).
**Shared cause for all four rows:** `Engine::boundary_key` (`crates/vkb-core/src/engine.rs:429`)
treats `'.' | '/' | ':' | '@' | '_' | '\\' | '#' | '=' | '?'` as a raw-zone opener; the next
`word_key` sets `literal_from = 0`, so the whole following word is typed without any transform.
`20-spec.md` §A9 specifies exactly this, so these are spec-level defects blocked by an A9 amendment,
not coding slips. Turning `raw_zone` off makes every row pass (SC-46/47/48).

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-SC-01 | P1 | `/` between two Vietnamese words leaves every trigger of the second word literal (the owner report) | VNI: `ba65t/ta8t1` · Telex: `baajt/tawts` (not `taats` — that spells `tất`) · also `/ta8t1`, `ba65t//ta8t1`, `ba65t /ta8t1`, `ba65t/ta8t1/ba65t`, `Ba65t/Ta8t1`, numpad digits | `bật/tắt` (and `/tắt`, `bật//tắt`, `bật /tắt`, `bật/tắt/bật`, `Bật/Tắt`) | `bật/ta8t1`, `bật/taats`, `/ta8t1`, `bật//ta8t1`, `bật /ta8t1`, `bật/ta8t1/ba65t`, `Bật/Ta8t1`. Every VNI trigger is dead in the second word: `ba65t/a8`→`bật/a8` (want `bật/ă`), `a61`→`bật/a61` (`bật/ấ`), `d9`→`bật/d9` (`bật/đ`), `o7`→`bật/o7` (`bật/ơ`), `u7`→`bật/u7` (`bật/ư`). SC-02/28/29/31/32/33/54/56/58/60/62/63/73/75 | `qa/special/matrix.txt`, `qa/special/repro.txt` (§1, §3, §4); `crates/vkb-core/tests/special_cases.rs::d_sc_01_slash_between_two_vni_words` (`--ignored`) | **FIXED + VERIFIED (2026-10-30)** — `92-verify-report.md` |
| D-SC-02 | P2 | `.` and `?` as a sentence end with no space kill the next word | VNI: `ba65t.ta8t1`, `ba65t?ta8t1` | `bật.tắt`, `bật?tắt` | `bật.ta8t1`, `bật?ta8t1` | `qa/special/matrix.txt` SC-03, SC-23; `::d_sc_02_dot_and_question_between_words` | **FIXED + VERIFIED (2026-10-30)** — `92-verify-report.md` |
| D-SC-03 | P3 | `_`, `@`, `:` kill the next word | VNI: `ba65t_ta8t1`, `ba65t@ta8t1`, `ba65t:ta8t1` | `bật_tắt`, `bật@tắt`, `bật:tắt` | `bật_ta8t1`, `bật@ta8t1`, `bật:ta8t1` | `qa/special/matrix.txt` SC-06, SC-07, SC-08; `::d_sc_03_underscore_at_colon_between_words` | **FIXED + VERIFIED (2026-10-30)** — `92-verify-report.md` |
| D-SC-04 | P3 | `=`, `#`, `\` kill the next word | VNI: `ba65t=ta8t1`, `ba65t#ta8t1`, `ba65t\ta8t1` | `bật=tắt`, `bật#tắt`, `bật\tắt` | `bật=ta8t1`, `bật#ta8t1`, `bật\ta8t1` | `qa/special/matrix.txt` SC-17, SC-21, SC-27; `::d_sc_04_code_punctuation_openers` | **FIXED + VERIFIED (2026-10-30)** — `92-verify-report.md` |
| D-SC-05 | P2 | a chained path opener after `:` (`\`, `/`) loses the raw zone, so a Windows path composes in Telex | Telex: `C:\Users\nam` (VNI already held) | `C:\Users\nam` | `C:\Úe\nam` | `special_cases.rs::amendment_2026_10_30_borderlines`; `20-spec.md` §A9 amendment §2b | **FIXED + VERIFIED (2026-10-30)** — `92-verify-report.md` |

**Dev note D-SC-05:** D-SC-05 resolved in 20-spec §2b (chained openers)

**Constraint on any fix (measured, must stay green):** URL/e-mail/code protection is the reason the
raw zone exists — `http://a.b/c`, `a@b.com`, `github.cof`, `file_name`, `--opt`, `2/3`, `1.5`,
`%20` all pass today (SC-35…SC-45) and are pinned by `crates/vkb-core/tests/raw_zone.rs` plus
`special_cases.rs::url_email_and_code_shapes_stay_literal`. Macro expansion after a punctuation
boundary also works and must keep working (`ko/` → `không/`, SC-76/77).

**Dev fix note (2026-10-30) — fix in `main`, pending QA verification.**
All four rows share one fix: `20-spec.md` §A9 amendment 2026-10-30 / `28-engine-rulings.md` D9.
`Engine::boundary_key` no longer arms on the opener alone; it arms only when the **final** left text
(after macro / A10 restore) is non-empty, plain ASCII, and — for `. / ?` only — not a valid A9
syllable (`crates/vkb-core/src/engine.rs:421-528`, helpers at `:640-666`). Opener set, ellipsis and
digit-dot exclusions, macro-at-boundary, `macro_only` and the Backspace clear are unchanged.
- D-SC-01: `ba65t/ta8t1` → `bật/tắt`; Telex `baajt/tawts` → `bật/tắt` (note: the audit's `taats`
  spells `tất`, so the Telex repro is `tawts`; both are now pinned). All five VNI triggers and the
  `/ . _ @ :` separators pass in Telex and VNI.
- D-SC-02/03/04: `bật.tắt`, `bật?tắt`, `bật_tắt`, `bật@tắt`, `bật:tắt`, `bật=tắt`, `bật#tắt`,
  `bật\tắt` all pass; the four `d_sc_*` repros are un-ignored regression tests.
- Matrix: 78/78 PASS (`matrix()` now asserts 0 failures; it used to only print). Two rows had their
  *expectation* corrected because it had pinned the defect: SC-72 `bật/ta8t` → `bật/tắ` (BS deletes
  one displayed character, B1) and SC-74 `bật/ta8t1`⏎ → `bật/tắt`⏎.
- Guards: `tests/raw_zone.rs` unchanged and green; `special_cases.rs::url_email_and_code_shapes_stay_literal`
  green; `no_alloc.rs` green (no new allocation on the steady path); P1 `process_key` p99 1.2 µs
  before and after (budget 50 µs). `crates/vkb-core/src/lib.rs:264` (O-7 stale doc) corrected in the
  same change.
- Side effect found and **not** covered by the amendment: `C:\Users\nam` survives in VNI but not in
  Telex — the `\` right after `:` is a second opener, so it sees an empty left token and arms nothing
  (§7 "opener runs lose protection"), after which Telex rewrites `Users` → `C:\Úe\nam`. The §6 row
  of the amendment declares `C:\Users\nam` without naming a method; pinned as measured in
  `special_cases.rs::amendment_2026_10_30_borderlines`. QA/Lead: decide whether that stays a declared
  loss or needs a spec term. → **Resolved 2026-10-30**: filed as D-SC-05, ruled into §2b (chained
  openers); the pinned `C:\Úe\nam` expectation is now `C:\Users\nam` and `chained_openers_extend_a_path`
  covers the four §6 chained rows plus the negative control.

**Not defects (measured, recorded in `51-special-cases.md` §6):** a macro trigger containing
punctuation can never match (O-1, by design — triggers match the word buffer); `?`/`!` asymmetry
(O-3, `!` is not in the opener list); keypad and main-row digits are equivalent (O-4); and the
`raw_zone` doc comment at `crates/vkb-core/src/lib.rs:264` still claims the feature is
"not implemented yet" while it is implemented and default-on (O-7, doc-only).

**User impact note:** `typing.raw_zone` has no UI control (`ui/src/lib/bindings.ts:25` "reserved,
not implemented"; `87-feature-matrix.md` lists it as reserved), so the defect cannot be worked
around from the settings window — only by editing `config.toml`.

## Re-verification of D-SC-01…05 (2026-10-30, stream B)

**Independent re-run by QA — the Dev fix note was not trusted.** Build: working tree at HEAD
`14bc655` + the uncommitted A9/§2b change; measured through `vkb_core::simulate` /
`simulate_keys` / `Engine` and `vkb-cli simulate`. Full detail, commands and asymmetry tables:
`92-verify-report.md`; evidence in `docs/evkey-clone-work/qa/verify/`.
Result: **78/78 matrix rows PASS**, `cargo test -p vkb-core` exit 0 (207 passed), `no_alloc`
green, P1 p99 0.8 µs (budget 50 µs). `cargo test --workspace` is blocked by an **unrelated**
uncommitted compile error in `vkb-windows/src/sound.rs:207` (E0015); workspace minus
`vkb-windows`/`vkb-app` is green. Two under-protection shapes are new relative to pre-fix and
are filed below as P3 — they do not reopen D-SC-01…05.

| id | measurement (this run) | verdict |
|---|---|---|
| D-SC-01 | `vni ba65t/ta8t1` → `bật/tắt`; `telex baajt/tawts` → `bật/tắt`; `/ta8t1` → `/tắt`; `//`, ` /`, three-word chain, `Ba65t/Ta8t1`, numpad digits and all five VNI triggers after `/` PASS | **PASS** |
| D-SC-02 | `ba65t.ta8t1` → `bật.tắt`; `ba65t?ta8t1` → `bật?tắt` | **PASS** |
| D-SC-03 | `_ @ :` → `bật_tắt`, `bật@tắt`, `bật:tắt` | **PASS** |
| D-SC-04 | `= # \` → `bật=tắt`, `bật#tắt`, `bật\tắt` | **PASS** |
| D-SC-05 | `C:\Users\nam` → `C:\Users\nam` in **both** Telex and VNI; `C:/Users/nam`, `C:\\server\share`, `https://x/y/z`, `http://a.b/c` literal in both; negative controls `xyz./tieesng` → `xyz./tiếng` and `baajt:_/tieesng` → `bật:_/tiếng` compose | **PASS** |

## Closed as expected (owner ruling)

Owner ruling 2026-10-30: both shapes filed after the stream-B re-run are **expected behaviour, not
defects**. Moved out of the open list so a future reader does not re-open them; repro text, measured
output and evidence are unchanged.

| id | severity | area | repro | expected | actual (measured 2026-10-30) | evidence | status |
|---|---|---|---|---|---|---|
| D-SC-06 | P3 | a single ASCII letter left of `. / ?` never arms the raw zone (a lone onset counts as a "valid syllable"), so a colon-less drive/path prefix composes; **new** (pre-fix every opener armed) | Telex `c/users`, `x/cof`, `x/tieesng` | `c/users`, `x/cof`, `x/tieesng` | `c/úe`, `x/cò`, `x/tiếng`; `C:/users`, `c:/users`, `C:\Users` stay literal | `92-verify-report.md` §6; `qa/verify/cli.txt`; `qa_92_verify.rs::arm_asymmetry_probe` | NOT A DEFECT — owner ruling 2026-10-30; declared behaviour, spec §A9 §6b + L6 |
| D-SC-07 | P3 | Telex digit-run parity after `/ . ?`: a second consecutive digit is a boundary, so the word is empty at the opener and the next word composes; VNI keeps the digits and stays literal — same keys, different text per method; **new** (visible only because of the §2 empty-left term) | Telex vs VNI `10/users`, `21/cof`, `2024/cof`, `v1/tieesng` | literal in both methods | Telex `10/úe`, `21/cò`, `2024/cò`, `v1/tiếng`; VNI `10/users`, `21/cof`, `2024/cof` literal | `92-verify-report.md` §6; `qa/verify/cli.txt` | NOT A DEFECT — owner ruling 2026-10-30; declared behaviour, spec §A9 §6b + L7 |

- **D-SC-06 — why expected:** a single ASCII letter before `. / ?` is already a valid A9 syllable
  (onset-only), so the opener correctly refuses to arm — the same rule that keeps `a/b` composing.
- **D-SC-07 — why expected:** in Telex a second digit ends the numeric run, so the left token is empty
  at the opener and an empty left token never arms (§2b chain requires a previous opener); VNI keeps
  the run, hence the literal result.

## Sound: toggle is silent (round 4 diagnosis, 2026-10-30)

**Reported:** "Tôi chọn âm thanh 7. Nhưng hiện tại đang không nghe tiếng đó khi thử bật/tắt bộ gõ."
Diagnosis + all measurements: `93-sound-silent.md`. Running build: `C:\Users\namth\AppData\Local\VietKB\vietkb.exe`
v`0.2.3`, SHA-256 `2F896F862B73…80612E`, started 2026-10-29 23:58:24. No product code changed.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-SND-01 | P1 | `system` pack (pack 7) is silent on a machine whose Windows scheme is "No Sounds" | 1) `system.notification_sound = true` (confirmed); 2) press `Alt+Shift` (or `F1`/`F2`) to toggle the IME; 3) listen | an audible on/off cue ("Theo Windows" follows the user's scheme) | nothing is heard. `HKCU:\AppEvents\Schemes` `(default) = .None` ("No Sounds"), every `Apps\.Default\<event>\.Current` value is empty, so `PlaySoundW("SystemAsterisk"/"SystemHand", SND_ALIAS\|SND_NODEFAULT)` returns `FALSE` (err 87) — 8/8 aliases fail; the same call with `Windows Background.wav` + `SND_FILENAME` returns `TRUE`, so the device stack is fine. `sound.rs:18`'s `SND_NODEFAULT` makes this silent by design, with no log | `93-sound-silent.md` §2–§3 (registry table + P/Invoke BOOL table); `crates/vkb-windows/src/sound.rs:10-22` | **FIXED + VERIFIED — QA 2026-10-30.** Independent run of `94-verify-sound.md`: `sound.rs:50-58 play_memory` is the only toggle call, `PlaySoundW(SND_MEMORY\|SND_ASYNC\|SND_NODEFAULT)` on the embedded `go_moc` pair (4 454 / 4 014 B), never the scheme alias. P/Invoke probe of the same flags on `on.wav` returned `TRUE` on this `.None` machine; a `RUST_LOG=debug` run (9 log lines, 1 DEBUG — filter live) logged **0** sound lines through an ON→OFF toggle, so `play_memory` never returned `FALSE`. Build: `target\release\vietkb.exe` 8 380 928 B, SHA-256 `BA22AE4F30470E28D00A99924EA263151399725E6ACB0ADF6AF04C58ED261A59`; `cargo test --workspace` 356/0/5. |
| D-SND-02 | P1 | packs 1–6 are defined in the spec and generated on disk, but not reachable in the shipped build (step A3 of `91-round-plan-sound-cases.md` has not landed) | open `Nâng cao` and look for a sound-pack picker; or set any pack in `config.toml` | 7 choices (`go_moc`, `click`, `gio_nuoc`, `eight_bit`, `co_khi`, `hai_not`, `system`) with the spec's default (`go_moc`) persisted as `system.sound_pack` | no picker exists: the only sound surface is the boolean `Âm thanh khi bật/tắt` (`ui/src/screens/Advanced.svelte:98-100`, `bindings.ts:47`). `vkb-config/src/schema.rs` has no `sound_pack` field (only `notification_sound`, line 277); no `soundPack`/`sound_pack` symbol exists in `ui/src`; neither exe embeds `go_moc/gio_nuoc/eight_bit/hai_not/co_khi` (Latin-1 string probe). `assets/sounds/**` holds all 12 WAVs, unused. So pack 7 is not a choice the owner could make — it is the only behaviour | `93-sound-silent.md` §5, §7.2; `assets/sounds/{6 dirs}/on\|off.wav` present (4 014–5 248 B) | **OUT OF SCOPE — owner ruling 2026-10-30** (picker/`sound_pack`/preview cut; only `go_moc` ships, see the scope-cut block below) |
| D-SND-03 | P3 | UI switch and tray toggle play the **on** alias for both directions | 1) open the main window; 2) flip the enable switch off, then on | off plays the off cue, on plays the on cue (as the hotkey route already does) | both directions call `play_toggle(true)`: `commands.rs:719 play_notification` → `lib.rs:97-99 play_notification_sound` → `sound::play_toggle(true)`. Masked today by D-SND-01; becomes audible the moment sound works | `crates/vkb-app/src/commands.rs:696-722`, `crates/vkb-windows/src/lib.rs:97`, `tray.rs:315` | **FIXED + VERIFIED — QA 2026-10-30.** All three toggle routes pass the state actually set: hotkey `state.rs:417-418 play_toggle(enabled)`, UI switch and tray (`tray.rs:281`, `tray.rs:315`) → `commands.rs:696 toggle_enabled` → `commands.rs:708 play_notification(enabled)` → `commands.rs:725 play_toggle(enabled)`. `Injection::Beep` stays asymmetric by ruling: `backend.rs:59` and `state.rs:507` call `play_error()` (`SystemHand`), never a toggle cue. The hard-coded `play_toggle(true)` that remains at `vkb-windows/src/lib.rs:97-99` (`play_notification_sound`) has **no callers** (grep 0) — dead surface, not a route. |
| D-SND-04 | P3 | the sound failure is unobservable | reproduce D-SND-01 and look for any log/trace record | when `PlaySoundW` returns `FALSE` the alias and reason are recorded once | `sound.rs:15` discards the result (`let _ = PlaySoundW(...)`) and no `tracing` call exists on the sound path; there is no line in `%APPDATA%\VietKB\logs` or `keys.log` that mentions sound, so neither the owner nor QA can tell "muted by scheme" from "never called" | `crates/vkb-windows/src/sound.rs:15`; `Get-ChildItem $env:APPDATA\VietKB` shows no sound-related log entry | **FIXED + VERIFIED — QA 2026-10-30.** `sound.rs:28 FAILURE_LOGGED` + `sound.rs:39-44 log_failure_once` emit exactly one `debug!` on the first failed playback and nothing on success (success path verified silent above: 0 sound lines in a debug run that toggled). `play_error` stays result-less by design. |

**Scope cut — owner ruling 2026-10-30** (owner: "Tôi chỉ muốn có âm thanh khi bật tắt bộ gõ, tôi không yêu
cầu thêm config hay UI mới nào cả"). The sound work is now one embedded `go_moc` pair, no config, no UI;
`22-sound-spec.md` is rewritten for the reduced feature and `91-round-plan-sound-cases.md` carries the cut.
Effect on the `D-SND` rows above (IDs as they stand in the table — an earlier agent brief had the last two
swapped, and the table is authoritative):
- **`D-SND-01` (silent toggle) — the only live item.** It stays OPEN; **QA verifies, Lead/Dev do not close it.**
- **`D-SND-03` (ON/OFF asymmetry) — fixed in flight, unchanged.** The symmetry fix is kept; the picker half of
  the old `D-SND-02` dev report is gone.
- **`D-SND-02` (packs 1–6 + picker + `preview_sound`) — OUT OF SCOPE, owner ruling 2026-10-30.** Not a defect
  to fix and not a feature to ship: the picker, `sound_pack`, `schemeSilent`, `preview_sound` and packs 2–6
  were built and then removed. **Do not re-open, do not re-add** (archive + 15 copy strings:
  `22-sound-spec.md` §9).
- **`D-SND-04` (failure unobservable) — folded into the spec, not a surface.** The reduced spec requires
  silence on failure, never a panic, ≤1 `debug` line per session; the existing one-line `tracing::debug!` stays.

**Dev fix note (2026-10-30 01:26:18, `main`, build `target\release\vietkb.exe` 8 465 408 B,
SHA-256 `BAE1DA2CAC6616C69AE50134335D5C645E20BE113E2F6C1088A4284ED174DD76`) —
`cargo test -p vkb-config -p vkb-windows` exit 0 (17 + 72/1 ignored + 1 integration), `cargo test --workspace`
exit 0 (357 passed / 0 failed / 5 ignored, all pre-existing), `cargo build -p vkb-app --release
--features custom-protocol` exit 0. QA closes.**

- **D-SND-01** — `vkb-windows/src/sound.rs` rewritten. `system` = `PlaySoundW(alias, SND_ALIAS|SND_ASYNC|SND_NODEFAULT)`
  with the `BOOL` **checked**: `TRUE` ⇒ unchanged alias playback; `FALSE` ⇒ the embedded `go_moc` ON/OFF pair
  via `SND_MEMORY|SND_ASYNC|SND_NODEFAULT`; both failing ⇒ silence, never a panic, never a blocking call.
  The alias thread is the existing `std::thread::spawn`; the buffers are `include_bytes!` `'static` data, so
  no `Vec` can drop while `SND_ASYNC` plays. Decision is one pure function, `sound::plan(pack, enabled)` →
  `Plan::Alias{alias, fallback}` / `Plan::Memory(bytes)`, executed by `run_plan` through a private
  `Player` seam (`Winmm` in production, a stub in tests) — no test opens the audio device.
- **D-SND-02** — all 12 WAVs are embedded in `vkb-windows` (`sound.rs` `GO_MOC/CLICK/HAI_NOT/GIO_NUOC/CO_KHI/EIGHT_BIT`,
  `include_bytes!("../../../assets/sounds/<id>/on|off.wav")`, 52 834 B); a missing asset is a compile error.
  New config key `system.sound_pack` (`SoundPack`, 7 slugs `system|go_moc|click|hai_not|gio_nuoc|co_khi|eight_bit`,
  default **`go_moc`** — owner decision, `22-sound-spec.md` §1/§7; the `#[default]` on the enum is the only place
  it lives, and a stored `sound_pack = "system"` keeps loading as `System`, never migrated); unknown/non-string
  value loads as `system` + one `debug` line, never a load error (the documented **fallback**, per Lead ruling —
  note `22-sound-spec.md` §7 line "unknown ⇒ `GoMoc`" / §9.17 still say the default, so that one line is stale
  against the ruling and the code);
  `SettingsView.system` gains `soundPack` + `schemeSilent`; `patch_settings` key `"soundPack"` (unknown slug →
  `"kiểu âm thanh không hợp lệ"`, `schemeSilent` accepted and ignored as read-only). New additive command
  `preview_sound(pack, enabled = None)` → `Ok` before the first sample ends; `None` plays ON then OFF with the
  320 ms gap, `Some(state)` plays one state; it never touches `input.enabled` or `config.toml`.
- **D-SND-03** — `commands.rs::toggle_enabled` now calls `play_notification(enabled)` → `vkb_windows::play_toggle_sound(enabled)`
  (OFF plays the off cue). `Injection::Beep` is **not** a toggle: it plays `sound::play_error()` (the
  `SystemHand` off alias) in both `state.rs` and `backend.rs`, so an invalid keystroke signals *error*, never
  "Vietnamese on" (Lead ruling 2026-10-30). Deliberately asymmetric — §9.14's five-route symmetry applies to the
  three toggle routes (hotkey / UI switch / tray); `Beep` is an error cue.
- **D-SND-03 (sound picker surface, `ui/` only, 2026-10-30)** — `ui/src/screens/Advanced.svelte`, section
  `Hệ thống`: the missing pack picker (the UI half of D-SND-02) is a full-width 36 px row directly under the
  master switch — `Select` with the 7 packs in §1 order (`1 · Cổ điển`, `2 · Click gọn`, `3 · Hai nốt`,
  `4 · Giọt nước`, `5 · Cơ học`, `6 · 8-bit`, `7 · Theo Windows`), a one-line character hint that follows the
  selection, the `system` + `schemeSilent` advisory in place of that hint (placeholder string, Copy owns
  final wording), and `Bật` / `Tắt` preview buttons → `api.previewSound(pack, enabled)` (`preview_sound`,
  fire-and-forget, both buttons disabled for the call with a 400 ms release guard, no toast, works with the
  master switch off and never toggles input). Selection persists through `patchSettings({ system: { soundPack } })`
  on the shared 150 ms debounce, so other windows follow through `vkb:settings`; nothing is written on mount.
  `bindings.ts` gains `soundPack` / `schemeSilent` + `SystemPatch` (`schemeSilent` never sent on a write,
  matching `apply_patch`'s no-op arm); `state.svelte.ts` demo settings carry both fields.
  `npm.cmd --prefix ui run check` → 0 errors / 0 warnings; `npm.cmd --prefix ui run build` → exit 0.
  **Open (not mine to change):** the `Nâng cao` window is 560 × 388 (`crates/vkb-app/src/main.rs:109`) and
  this row needs the 424 height of `22-sound-spec.md` §8 — a Rust design value; without it the last grid row
  and footer are clipped. No dirty/pending indicator exists anywhere in the UI to reuse, so the row shows none.
  `lib.rs::play_notification_sound()` kept unchanged for other callers but is no longer the toggle's path.
- **D-SND-03 (window height, 2026-10-30 01:54) — the 424 growth was withdrawn, no Rust change landed.**
  The `Nâng cao` window needed 560 × 424 only because of the 36 px sound row above; that row was cut by the
  sound revert (owner scope cut, `D-SND-02` out of scope), so `crates/vkb-app/src/main.rs:109` stays
  `560.0, 388.0` with its "content 295 … 388 → 388" comment — a 424 window would now be 36 px of dead space.
  The shipped 388 was measured at both scalings (per-monitor-v2 reader; a DPI-unaware reader reports 574 × 396
  and 96 dpi at 125 %, which is the reader's artefact): **125 %** primary, dpi 120 — outer (250,145) 718 × 495,
  client 700 × 485 physical = 560 × 388 CSS, work area 2560 × 1380, fully visible, last grid row bottom 553 and
  footer band 576–617 under a client bottom of 630, nothing clipped or overlapping; **100 %** secondary, dpi 96 —
  outer (2648,107) 574 × 396, client 558 × 387, work area 1920 × 1032, fully visible, footer 452–485 vs client
  bottom 494. Placement needed no change: `centre_on_parent` clamps to the monitor work area and
  `centre_on_monitor` pins to the origin when a window is larger than the monitor, so a taller window could not
  have gone off-screen. `cargo test -p vkb-app` exit 0 (20 passed / 2 ignored); release `vkb-app
  --features custom-protocol` exit 0 — `vietkb.exe` 8 380 928 B, SHA-256
  `45B571155238DBDAB1BCE5F26B683504520F8106C013CE16848A3A187BE0F957` (this unit adds no bytes to the revert
  tree). If the row ever returns, the single line is `main.rs:109` `388.0 → 424.0`. **QA closes.**
- **D-SND-04** — one `tracing::debug!` per session (alias + which step failed), guarded by an `AtomicBool`;
  the success path logs nothing.
- **Tests:** `vkb-config` 17 passed (`schema::tests::sound_pack_default_and_roundtrip`,
  `unknown_sound_pack_falls_back_to_system` are new), `vkb-windows` 72 passed + 1 ignored with 6 new sound
  tests (`packs_are_pcm16_22k_mono`, `durations_and_peaks_match_spec`, `levels_dc_offset_and_fades`,
  `chat_size_caps_and_slug_table`, `system_falls_back_to_go_moc_when_alias_fails`,
  `live_pack_round_trips_through_the_atomic`), `vkb-app` 22 passed + 2 ignored with 2 new tests
  (`patch_sets_the_sound_pack_and_rejects_unknown_slugs`, `preview_rejects_an_unknown_pack_without_playing`);
  `cargo test --workspace` exit 0 — **357 passed / 0 failed / 5 ignored** (all pre-existing).
  `vietkb.exe` **8 465 408 B**, delta **+94 720 B (+92.5 KiB)** vs 8 370 688 B (cap 120 KiB); embedded WAVs are
  52 834 B of it. The default flip + `play_error()` routing added +2 048 B over the previous 8 463 360 B.
- **Deviations and rulings (2026-10-30):**
  1. `Injection::Beep` — **ruled: error cue.** §6/§9.10/§11 said `play_error()` all along; the first landing made
     `Beep` play the pack pair for the current state per §9.14 and left `play_error()` unrouted. Reverted:
     `backend.rs:56` and `state.rs:506` now call `sound::play_error()` (SystemHand alias, no fallback, no pack).
     Deliberately asymmetric — `Beep` is an invalid keystroke, not a toggle, so §9.14's "all five routes"
     symmetry means the three toggle routes only (Lead ruling 2026-10-30).
  2. `preview_sound`'s second parameter is `Option<bool>`, not `bool` — **accepted as built.** Tauri rejects a
     missing key for a non-`Option` argument (`tauri/src/ipc/command.rs` "missing required key"), so the spec's
     one-argument `invoke("preview_sound", { pack })` and a two-argument call both work; `None` = ON then OFF.
     This is the contract-relevant shape: the signature is `preview_sound(pack: String, enabled: Option<bool>)`.
  3. `schemeSilent` is computed by reading `HKCU\AppEvents\Schemes\Apps\.Default\SystemAsterisk\.Current`
     (empty/missing ⇒ `true`) rather than by a `PlaySoundW` probe, because the probe would itself play a sound
     on a machine whose scheme has one — **accepted as built**; registry read, never a playing probe. Advisory
     only: it drives the §8 hint and never rewrites `sound_pack`.
- **Status column:** the four rows read `FIXED + VERIFIED (2026-10-30) — 92-verify-report.md`, which was a false
  mark: that report contains no `SND`/`sound` measurement (it is the D-SC verification), and it was written at
  01:12:52 while this unit's first compiling `sound.rs` landed at 01:11 and the release exe at 01:13:25. Reset
  to **OPEN — awaiting QA re-verification**; re-verify against the exe size/SHA in the Dev fix note above
  (8 465 408 B, `BAE1DA2C…`). **QA closes, not Dev.**

---

# 60 — E2E gate defects, 0.2.4 candidate (2026-10-30)

**Owner:** QA · **Build:** `target\release\vietkb.exe` 8 380 928 B, SHA-256
`0B566EE499A2F1E9DDCE23498C7EEA03DF0C7FF255B0032FE42209D503A743EF`.
**Scope:** independent E2E gate (`95-e2e-report.md`): hook against Notepad / Chrome / VS Code,
toggle routes, UI, budgets, 30-min soak. No product code changed by this pass.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-E2E-01 | P1 (blocking) | default injection silently corrupts text in Windows 11 Notepad | Notepad (Win11 26H2), Vietnamese ON, default `typing.send_standard = false`; type `ba6t5/ta8t1` (VNI for `bật/tắt`) at 55–700 ms | `bật/tắt` | `bật/ttt` (also `bật.ttt`, `bậttt`, `bật/`, `bật\tttt`): separator and second-syllable letters/digits lost, stray `t`s remain. `keys.log` shows the engine emitted the correct `Replace{delete:3,insert:"bật"}` / pass / `Replace{delete:2,insert:"tâ"}` / `Replace{delete:3,insert:"tật"}` | `95-e2e-report.md` §A (Notepad 11/19), `qa/e2e95/typing-notepad.tsv`, `qa/e2e95/app.log`; identical keystrokes PASS in Chrome (17/19) and VS Code (15/19; a 16th is a Tab-expectation artefact); `send_standard = true` makes 5/5 sequences correct at 120/250/400/700 ms. **Dev fix 2026-10-30 (`crates/vkb-windows/src/input.rs`, `build_segment`): a replacement types its ASCII characters as VK + scancode keystrokes and keeps `KEYEVENTF_UNICODE` for characters no US key produces; plain typing (`Injection::Type`) still uses the pure packet path.** Root cause measured with raw `SendInput` and no engine/hook (`qa/e2e95/lab-matrix*.ps1`): the Win11 Notepad text stack collapses a `VK_PACKET` run that follows backspaces onto the run's last character (`Replace{2,"tă"}` over `bật/ta` → `bật/ăă`, `Replace{3,"tắt"}` → `ttt`). Pure packet path 0/8 correct for `ba6t5/ta8t1`, new encoding 8/8. Re-measured 2026-10-30: Notepad 16/19 (A1/A2/A3/A4/A6/C2/D1/D2 flipped to PASS; A8 raw-zone by spec, C1 = D-E2E-02), Chrome 18/19 (A8 only), VS Code typing rows PASS. Residual, still OPEN as an app-defect note: a run of three consecutive non-ASCII characters (e.g. `được`) after a replacement can still collapse in Notepad under every encoding tried — it also reproduces with raw `SendInput`, so it is target-side, not this layer | FIXED — awaiting QA re-verification |
| D-E2E-02 | P1 (blocking) | `restore_word` hotkey `Ctrl+Shift+Z` never fires | type `ba6t5` (shows `bật`), press `Ctrl+Shift+Z` | current word replaced by its raw keystrokes `ba6t5` (`20-spec.md` "Manual hotkey `restore_word`", AC-46) | unchanged `bật` in Chrome, VS Code and Notepad; `keys.log`: `vk=0x5A mods=SC--- stage=shortcut action=none decision={swallow:false}` — the chord is treated as an app shortcut before the hotkey table | `qa/e2e95/typing-*.tsv` row `C1-restore-word`; `%APPDATA%\vietkb\keys.log` | **FIXED + VERIFIED on host (2026-10-30, build `0D39D42F…`)**: Ctrl+Shift+Z restores `vieejt` in Notepad, Chrome and VS Code (`97-host-e2e-0.2.4.md`) |
| D-E2E-03 | P2 | `F1` does not re-enable Vietnamese in VS Code | VS Code editor: `F2` (typing literal — correct), then `F1`, then type `ba6t5` | `bật` | `ba6t5` (stays disabled); Notepad and Chrome re-enable correctly with the same keys | `qa/e2e95/typing-vscode.tsv` row `E2-on-f1` | **FIXED + VERIFIED on host (2026-10-30, build `0D39D42F…`)**: in real VS Code, F2, F1, then `baatj` gives `bật` and the command palette stays closed |
| D-E2E-04 | P3 | main settings panel is 478 × 459 with no tabs (charter C9 deviation) | open the settings window, read `GetClientRect` + UIA tree | charter C9: 720 × 560, at most 4 tabs | client 478 × 459 at dpi 96, one scrolling page, no `Tab`/`TabItem` in the UIA tree; `Nâng cao` / `Phím tắt` / `Bảng gõ tắt` are separate windows | `qa/e2e95/ui-1-main.png`, `qa/e2e95/ui-results.txt` | **CLOSED — charter amended (owner, 2026-10-30)**: C9 now describes the compact panel; no UI work |
| D-E2E-05 | P2 | a bare Shift key-down ended the word, so per-letter Shift capitals (VIEETJ, AAS, DD) stayed literal | type `VIEETJ`, `AAS`, `DD` with Shift held on each capital (the bare-Shift key-down reset the engine, `state.rs` modifier branch) | `VIỆT`, `Ấ`, `Đ` | `VIEETJ`, `AAS`, `DD` stayed literal | `crates/vkb-windows/src/state.rs` (bare Shift now passes through, ~L443); test `state::tests::bare_shift_between_letters_does_not_end_the_word` (QA 2026-10-30); in-process harness edge rows T20/T44 with shift-wrapped capitals; `98-verify-round-close.md` | FIXED (0f70497) + VERIFIED on host via D-E2E-06 rows (2026-10-30) |

**Observations, not defects**

- `https://vietkb.dev/tải` stays literal (`…/ta3i`) in all three apps — matches the frozen `raw_zone` ruling (`20-spec.md` A9, `special_cases.rs::url_email_and_code_shapes_stay_literal`); the gate brief expected a transform. Contract decision for the Lead.
- `RUST_LOG=vkb=debug` yields **no** tracing output in the GUI process (launcher markers only): the `sound.rs:43` "toggle sound could not be played" line is unreachable in the field, so sound success/failure cannot be diagnosed from logs.
- Tray **menu item** `Bật/tắt tiếng Việt` is not exposed to UIA (only the icon button is); the icon's left-click toggle and its state tooltip are verified, the menu item is not.
- P6 first cold-cache run 433 ms (over the 400 ms budget once); 391/387 ms after.
- **Settings chord: the gate brief and the shipped default disagree.** The host-E2E brief (2026-10-30, QA prep unit) asks for `Ctrl+Shift+S` to open settings; the live `%APPDATA%\vietkb\config.toml` and the shipped default bind `hotkeys.open_settings = "Ctrl+Alt+S"`. `run-host-e2e.ps1` therefore gates on the config-declared chord and reports `Ctrl+Shift+S` as an informational NOTE. No code change made (this unit is QA-only); the Lead decides which one is the contract before the run is graded on it.



---

# 60 — Tone change and partial deletion, 2026-10-30 (QA, engine only)

**Owner:** QA. **Scope:** the owner's report “đổi dấu của chữ đã viết rồi … hoặc xóa 1 unit chữ có dấu
để viết lại mỗi chữ đó với dấu mới đôi khi không thành công”. **Build:** `target\debug\vkb.exe`
1 042 944 B, SHA-256 `3E6B33D3303AD423AEB32A19F0EDEA3C7FC9FE167ECD634D5E765F2FDE66D17D`, HEAD `14bc655`
+ uncommitted working tree. **Layer:** `vkb-core` char path (`vkb-cli simulate`); no hook, no app, no
product code changed. **Matrix and per-row justification:** `57-tone-change-cases.md` (123 rows:
88 PASS / 35 FAIL). **Evidence:** `qa/tone/matrix.txt`, `qa/tone/matrix.md`, `qa/tone/repro.txt`
(`--actions` trace per row below), harness `qa/tone/run.ps1`, traces `qa/tone/repro.ps1`.
IDs are grouped per root shape, not per keystroke; every row here is reproducible with the exact
keystrokes in the `repro` column.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-TN-01 | P2 | a tone key whose tone is **already in force** escapes the whole word to raw keystrokes when any letter sits between the two presses — the circumflex/breve/horn and the tone display are lost; VNI contradicts its own “immediately after its own transform” wording | Telex `baajtj` · VNI `ba65t5` · `Baajtj` · reopen route `baajt`␠`BS` `j` | the word is unchanged: `bật` (the tone is re-applied; §A1's escape table pins the **adjacent** double press only, §A2 says the digit escape is for a digit repeated *immediately*) | Telex `baatj` (mark and tone gone, trailing `j` kept); VNI `ba6t5`; reopen route `baatj`; NFD identical | `qa/tone/repro.txt` §D-TN-01; `qa/tone/matrix.txt` A-02, A-39, A-49, A-54, B-54; `method.rs:402-407` (`out.tone == tone` → `escape_to_raw`, `drop_for` removes the first trigger so `aa`/`65` literalise). The adjacent form is pinned as intended by `tests/editing.rs::a_tone_trigger_on_a_reopened_toned_word_keeps_the_escape_rule` and by `50-test-cases.md` `E-TE-002`/`E-TE-016`; **ruling needed**: keep the escape for an adjacent double press only, or for every repeat. The owner's “đổi dấu … không thành công” is this shape (a word can lose its mark by pressing the key that is already in force). | **FIXED + VERIFIED (engine, 2026-10-30)** — tone matrix 123/0 (`98-verify-round-close.md`) |
| D-TN-02 | P2 | Telex `z` (remove the tone) is not implemented at all — it is appended as a letter | Telex `basz` · `toansz` · `baajz` · `bật`+BS+`z` | `ba`, `toan`, `bâ`, `bâ` (§A1: “`z` remove tone … `asz` → `a`”; §A5 keeps the mark when the tone goes) | `basz`, `toansz`, `baajz`, `baajz` — the tone and the mark stay, `z` is literal text | `qa/tone/repro.txt` §D-TN-02; matrix A-11, A-13, A-14, A-55, B-07; `method.rs:250-259` (`step_telex` has arms for `s f r x j w a e o d` only — no `z`). VNI `0` does remove a tone (`a`/`ba` rows pass), so the two methods disagree. Only the `asz`/`basz` control (`az` → `az`) passes, and it passes by accident (a literal `z`). | **FIXED + VERIFIED (engine, 2026-10-30)** — QA re-run: tone matrix 123/0, `tone_change` 29/0, `no_alloc` 4/4, `raw_zone` 9/0 (`72fee6b`+`4afe9aa`, merged `eebbdb8`; `98-verify-round-close.md`) |
| D-TN-03 | P2 | VNI `0` (remove the tone) also clears **every letter's mark**, so the circumflex/breve/horn disappears with the tone | VNI `a60` · `ba650` · `ba65t0` · `ba65t`+BS+`0` | `â`, `bâ`, `bât`, `bâ` (§A2 “`0` remove tone”; §A5: mark and tone are independent fields) | `a`, `ba`, `bat`, `ba` — `â` became `a` | `qa/tone/repro.txt` §D-TN-03; matrix A-43, A-44, A-45, B-61; `method.rs:147-155` (`'0'` loops `letters` setting `mark = Mark::None`). Same family as D-TN-02 but a different function; a Telex `z` fix must not copy this behaviour. | **FIXED + VERIFIED (engine, 2026-10-30)** — QA re-run: tone matrix 123/0, `tone_change` 29/0, `no_alloc` 4/4, `raw_zone` 9/0 (`72fee6b`+`4afe9aa`, merged `eebbdb8`; `98-verify-round-close.md`) |
| D-TN-04 | P3 | a tone **change** is applied without the A9 coda check, so a finished word is rewritten into a spelling that cannot exist (`c ch p t` take only sắc/nặng). §A1 rule 5 says the trigger must stay literal (`tacf` → `tacf`) | Telex `baajtf` / `baajtr` / `baajtx` / `Baajtf` / `BAATJf` / `dduowcjf` · VNI `ba65t2` / `3` / `4` | `bậtf`, `bậtr`, `bậtx` (and `Bậtf`, `BẬTf`, `đượcf`, `bật2`, `bật3`, `bật4`) — trigger literal, word untouched | `bầt`, `bẩt`, `bẫt`, `Bầt`, `BẦT`, `đườc`, `bầt`, `bẩt`, `bẫt` | `qa/tone/repro.txt` §D-TN-04; matrix A-03…A-05, A-24, A-26, A-31, A-36…A-38; `method.rs:392-412` (`apply_tone` never calls `spell::`). Pre-existing sibling on a fresh word: `tacf` → `tàc` (trace in the same file) — this is not caused by the tone-*change* code path, it is the same missing check seen from a word that was already valid. The owner sees it as “the accent came out wrong”. | **FIXED + VERIFIED (engine, 2026-10-30)** — QA re-run: tone matrix 123/0, `tone_change` 29/0, `no_alloc` 4/4, `raw_zone` 9/0 (`72fee6b`+`4afe9aa`, merged `eebbdb8`; `98-verify-round-close.md`) |
| D-TN-05 | P3 | a trigger with **no target** (VNI mark digit on a vowel that already carries a mark) reverts the whole word to raw keystrokes instead of appending the literal digit | VNI `ba65t6` | `bật6` (§A2 rule 1: a digit is literal when the word offers no target) | `ba5t6` — word destroyed, the `6` that applied is gone and the second one is literal | `qa/tone/repro.txt` §D-TN-05; matrix A-47; `method.rs:452-455` (`mark_last` falls through to `escape_to_raw`). The same escape-on-no-target shape also produces the `a66 → a6` style result that the §A2 digit escape *does* want, so a fix must keep the adjacent-escape rows green. | **FIXED + VERIFIED (engine, 2026-10-30)** — QA re-run: tone matrix 123/0, `tone_change` 29/0, `no_alloc` 4/4, `raw_zone` 9/0 (`72fee6b`+`4afe9aa`, merged `eebbdb8`; `98-verify-round-close.md`) |
| D-TN-06 | P2 | after a Backspace the next key must re-run A4 (§A7 step 4) — instead the engine **freezes the display as literal characters**, so the word stops composing and a tone key is appended as text | Telex `toans`+BS+`f` / `s` / `r` · `hoais`+BS+`s` / `f` | `tòa`, `tóa`, `tỏa`, `hóa`, `hòa` — the surviving characters were `toá`/`hoá` (A7 step 4 keeps the old display) and the next key re-places the tone on the whole word | `toáf`, `toás`, `toár`, `hoás`, `hoáf` — the tone key is literal, the tone does not move | `qa/tone/repro.txt` §D-TN-06 (the `--actions` trace shows Backspace returns `Noop` and the raw buffer is already the display text `toá`/`hoá`); matrix B-42…B-44, B-46, B-47; `engine.rs:317-333` (when `compose(kept)` ≠ the displayed text, `frozen` = displayed chars and `literal_from = 0`). Trigger: a tone whose position depends on the deleted coda (`oa`/`oai` shapes). This is the owner's “xóa 1 unit … để viết lại … với dấu mới đôi khi không thành công”: after the deletion the word silently becomes uncomposable. | **FIXED + VERIFIED (engine, 2026-10-30)** — tone matrix 123/0 (`98-verify-round-close.md`) |
| D-TN-07 | P3 | a **plain letter** (not a trigger) that makes the syllable invalid rewrites the whole displayed word to its raw keystrokes | Telex `baajt`+BS+`b` · `tieengs`+BS+`n` · `Nguyeexn`+BS+`nn`; same without any deletion: `baajtb` | `bậb`, `tiếnn`, `Nguyễnn` — §A9 makes the **trigger** literal, it does not say a non-trigger letter un-composes the word | `baajb`, `tieensn`, `Nguyeexnn`, `baajtb` | `qa/tone/repro.txt` §D-TN-07 (including the no-Backspace control); matrix B-06, B-13, B-38; `method.rs:102-110` (`compose` calls `literalize` whenever the replayed word is not a valid syllable). Spec-vs-implementation question: the same rewrite happens while typing a fresh word, so it may be intended “show what was typed” behaviour — but here it destroys text that was already correct on screen, and the §A9 policy text only covers triggers. | **FIXED + VERIFIED (engine, 2026-10-30)** — tone matrix 123/0 (`98-verify-round-close.md`) |
| D-TN-08 | P3 | a letter whose base equals the last **composed** letter escapes the mark (drops the circumflex/ê and the tone) even though the previous keystroke was the tone key — the doubled-letter escape should need two letter **presses** in a row | Telex `Nguyeexn`+BS+`e` / `ee` · `kieems`+BS+`e` | `Nguyễe` / `Nguyễee`, `kiếe` — a plain letter appends (§A1 pins `ee` as *two `e` key presses*, and here the keys are `e e x e`) | `Nguyexe` / `Nguyexee`, `kiese` — the `ê`/`ễ` is dissolved into raw letters | `qa/tone/repro.txt` §D-TN-08 + the spec control `Nguyeexne` (which escapes correctly); matrix B-14, B-15, B-34; `method.rs:263-283` (`double_letter` looks at `out.letters.last()` instead of the previous raw key). | **FIXED + VERIFIED (engine, 2026-10-30)** — tone matrix 123/0 (`98-verify-round-close.md`) |

**Shared notes**

* No defect above is reachable from the UI settings: they are engine semantics. Nothing in this pass
  changed product code; the harness is `qa/tone/run.ps1` (123 rows) and `qa/tone/repro.ps1`.
* `legacy` is not a selectable method in this build (`vkb simulate --method legacy` →
  `unknown method "legacy"; expected one of telex, vni`); the legacy-adjacent dimension is
  `--old-tone-style`, exercised by A-50, B-63 and the `oa` rows (all PASS except the roots above).
* Adjacent double-press escapes (`bass → bas`, `hoangff → hoangf`, `ddi`+BS+`d → dd`,
  `ba65t`+BS+`5 → ba65`) are **documented behaviour**, verified by the matrix (A-10, A-16, A-34,
  B-19, B-26, B-30, B-33, B-37, B-39, B-49, B-60) — not defects. D-TN-01 is about the same rule
  firing where the presses are **not** adjacent.
* The owner's “xóa 1 unit rồi viết lại” also has a large PASS surface: retyping the deleted letter
  (`Nguyễn`, `hoàng`, `đi`, `được`, `kiếm`, `tiếng`, `toán`), an unrelated letter while the word stays
  valid (`bận`), a different tone key (`bấ`, `bầ`, `hoán`, `hoáng`), VNI `1`/`2` after BS, and the
  NFD variants all compose correctly (matrix B-01…B-05, B-11, B-12, B-16…B-18, B-20…B-25, B-27…B-29,
  B-31, B-32, B-35, B-36, B-40, B-41, B-45, B-48, B-50…B-60, B-62…B-68).

## Owner rulings — 2026-10-30 (round close-out)

| id | severity | area | ruling |
|---|---|---|---|
| D-TN-09 | P3 (UX) | VNI `u7` → `ư` also fires inside text that is not Vietnamese (`abcu7`), because `auto_horn_uo` / the VNI `7` mark (`method.rs`) does not consult spell check | **WON'T FIX (owner, 2026-10-30)** — keep the current behaviour; do not gate `7` on syllable validity. |
| D-GUARD-01 | P1 (ops) | `HKCU\…\Run\VietKB` pointed at an agent debug build (`97-stray-input-source.md`) | **RESOLVED (observed 2026-10-30)** — the `VietKB` value is no longer present under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. The installed app re-registers its own path when `start_with_windows` is saved; agents did not touch the key. |
| D-GUARD-02 | P2 (ops) | `EVKey64.lnk` still in the user Startup folder (a second Vietnamese IME) | **OWNER ACTION** — unchanged; agents must not remove it. |

## Host E2E, 0.2.4 candidate — 2026-10-30 (QA, Notepad only)

Report: `97-host-e2e-0.2.4.md`. Build `vietkb.exe` SHA-256 `E2189678…C1D02`. The run aborted in the VS Code block (guard false positive suspected), so VS Code and Chrome rows were not run.

| id | severity | area | repro | expected | actual | status |
|---|---|---|---|---|---|---|
| D-E2E-06 | P1 (blocking) | Shift capitals lose their case on the real desktop | Notepad, Telex, SendInput VK events with Shift held per capital: `VIEETJ`, `Vieejt`, sentence starting `Tieesng` | `VIỆT`, `Việt`, `Tiếng …` | `việt`, `việt`, `tiếng …`. The in-process harness passes the same rows (D-E2E-05), so the difference lies in real modifier state or injection. `AAS DD` → `Ấ Đ` passed. Dev's diagnosis, inferred because the race was not reproduced off-desktop: every whole-word rewrite retyped the unchanged head, and each capital went through an injected `Shift↓ letter Shift↑` pair. Notepad did not honour that pair, and the pair also released a Shift the user was still holding. | **FIXED in-process (`ce3c818`, `f32f57e`, `08230e0`, `966a7ef`, merged `32e6f06`)**. A rewrite now keeps the unchanged head on screen (`plan.rs` `keep_unchanged_head`, which never cuts at a combining mark). Shift and Caps state comes from the hook's own `ModifierState`. A held Shift is never released. A letter goes out as a packet only when the user holds Shift and the letter needs the other case (0 of the 286 harness rows). `repair_shift` releases only Shift keys that the batch pressed itself. Harness 286/286. **Host Notepad VERIFIED on build `0D39D42F…` (2026-10-30):** B1, B2, B4 (Shift held through the word) and B6 (Caps Lock) all PASS, no Shift left stuck down. Chrome all PASS (run v3). VS Code all PASS in run v7, with word suggestions off. An earlier `ViệtViệt` merge (run v3) came from VS Code's suggest widget: Enter accepted the suggestion. That is target behaviour, not a VietKB defect. **VERIFIED.** |
| D-E2E-01 (residual) | P1 | `được` after a replacement still collapses in Win11 Notepad | `dduwowcj` ×10 | 10/10 `được` | 3/10 correct on build `E2189678…`; sentence row also produced `eep` for `đẹp` | **Not reproduced on build `0D39D42F…`**: 10/10 `được` and the sentence row PASS in Notepad, probably because the head trim shortens each replacement. Kept OPEN as a watch item, since the target-side collapse still exists. |
| D-AUTO-01 | P1 (ops) | unit test `autostart::tests::registry_write_round_trip` wrote the developer's real `HKCU\…\Run\VietKB`, and its "restore" wrote the test binary path (`target\debug\deps\vkb_windows-<hash>.exe --autostart`). This is the source of D-GUARD-01. | `cargo test -p vkb-windows` on a machine with an existing `Run\VietKB` value | value unchanged | value replaced by the test binary path | FIXED (`a6a2aa9`, `1b5b790`, merged `72aac95`) — test writes only `HKCU\Software\VietKB\test-autostart-<pid>` with a Drop guard. The polluted value on the owner's machine is an **OWNER ACTION**. |
| D-GUARD-01 | P1 (ops) | reopened: `Run\VietKB` again points at `target\debug\deps\vkb_windows-703be73b8891cfe5.exe --autostart` (restored byte-identical by QA after the host run) | — | — | **CLOSED (owner, 2026-10-30)** — the owner deleted the value. |

Owner checks, 2026-10-30, after the 0.2.4 release:
- Toggle sound audibility: **VERIFIED by owner**.
- D-GUARD-02 (`EVKey64.lnk` in Startup): **WON'T FIX (owner)**. Leave it. Host runs must keep checking that EVKey is not running.


## Editing scenarios — 2026-10-30 (owner report + QA sweep)

Owner report: after a non-Vietnamese word, a space and Backspace back into it, the next Vietnamese word stayed raw even when the bad word was deleted completely; only a new word after a space recovered. QA then added `crates/vkb-core/tests/editing_scenarios.rs` (about 300 000 generated edit sequences, Telex and VNI × `spell_check` × `restore_wrong_word`), which found the classes below on `main` at `980ea44`.

Rule adopted for all of them: after a Backspace the word behaves as if its surviving keystrokes had been typed fresh (same composing, spell-check verdict and tone placement), provided that fresh typing reproduces a screen no longer than before; otherwise the surviving text stays frozen and the screen does not change.

| id | severity | area | repro | expected | actual (before fix) | status |
|---|---|---|---|---|---|---|
| D-BS-LIT-01 | P1 | a frozen word kept `literal_from = 0` through Backspace, across the reopen memory, and even after it was fully deleted, so every new key was raw | Telex `pass⌫⌫⌫dduwowcj`; VNI `d99`␠`⌫⌫⌫` then `d9e6m`; spell on: `test⌫⌫⌫⌫hif` | `được`, `đêm`, `hì` | `dduwowcj`, `d9e6m`, `hif` | FIXED (`6d654a5`, `94797b4`, `300a08a`) |
| D-BS-LIT-02 | P1 | a typo that made the word invalid froze it; Backspace over the typo did not recover it | spell on: `dduwowcjk⌫`, `gifk⌫`, `ddk⌫uwowcj` | `được`, `gì`, `được` | `dduwowcj`, `gif`, raw | FIXED (`94797b4`) — the frozen word is re-judged from the kept keystrokes |
| D-BS-LIT-03 | P2 | a digit ended the word without reopening it, so Backspace over the digit lost the word | Telex `d9⌫deei` | `đêi` (spell off) | `ddêi` | FIXED (`94797b4`) |
| D-BS-LIT-04 | P2 | the D-TN-06 tone pin also applied to a word that was never a valid syllable | spell off: `hoafk⌫` | `hòa` (modern tone) | `hoà` | FIXED (`94797b4`, `300a08a`) — the pin applies only when the old word was valid |
| D-BS-LIT-05 | P2 | spell check treated Telex `j` before a stop coda as English | spell on: `hojc`, `nhajt` | `học`, `nhặt` | `hojc`, `nhajt` | FIXED (`94797b4`, `300a08a`) — exemption limited to `j` + `c`/`ch`/`p`/`t` |
| D-BS-LIT-06 | P3 | the `edited` flag bypassed spell check for keys typed after an edit, so the same keys gave different words | spell on, VNI: fresh `d9e6i` vs `dog⌫⌫9e6i` | both raw `d9e6i` (`êi` is not a syllable) | fresh raw, edited `đêi` | FIXED (`94797b4`, `300a08a`) — `edit_len` protects only keystrokes that survived a Backspace |
| D-BS-LIT-07 | P3 | `Ctrl+Shift+Z` was swallowed with nothing to restore after an un-frozen remnant | `dxcv⌫⌫⌫` then `Ctrl+Shift+Z` | chord reaches the app (redo) | swallowed | FIXED (`94797b4`) — `can_restore()` is false when raw equals the screen |
| D-BS-LIT-08 | P3 | a frozen remnant longer than `MAX_WORD` in `unicode_nfd` spills its `SmallVec` to the heap on Backspace | `engine.rs` `backspace()` frozen fallback | allocation-free | heap allocation possible | OPEN (pre-existing, low impact) |

Owner rulings, 2026-10-30:
- A digit typed directly before letters does not block composing: `1dduwowcj` → `1được`, `3giowf` → `3giờ`, VNI `2d9e6m` → `2đêm` (spec A9 updated).
- VNI `d9e6i` with spell check on stays `d9e6i` (correct: `êi` is not a syllable), but deleting it fully or back to its first letter and retyping must compose — pinned by the `owner_*` tests in `editing.rs`.
- Accepted consequence of the re-judge rule: one Backspace may rewrite the word to what its surviving keys compose to (`test⌫` → `té`, `d9e6i⌫` → `đê`).

## macOS port — 2026-10-31

| id | severity | area | repro | expected | actual | status |
|---|---|---|---|---|---|---|
| D-TEST-01 | P3 (test harness) | `vkb-core/tests/no_alloc.rs` counts allocations with one global counter, so tests running in parallel threads count each other's allocations | `cargo test -p vkb-core --test no_alloc` ×5 on macOS (default parallel) | 5/5 pass | 3/5 pass; failures in `plain_append_and_boundary_do_not_allocate` or `ignored_keys_do_not_allocate`. `--test-threads=1`: 5/5 pass | FIXED in `21e40ba` — per-thread counter (const `thread_local` Cell); 10/10 pass in default parallel mode |

## macOS real-app E2E — 2026-10-31 (QA, build `qa-build/VietKB.app` 0.2.6, binary sha256 `9a056ae9…f759`)

Evidence: `qa/mac/run-full-2/`, `qa/mac/run-probe-vni-caps/`, report `95-e2e-report-mac.md`.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-MAC-01 | P2 | vkb-macos / VNI | VNI, spell check on (default), synthetic keys with Shift pressed per capital letter: `VIE6T5`, `VIE6T`, `VIE6`, `VIE5` (an all-caps word, then a mark or tone digit). Same result in TextEdit plain/rich, Notes, Stickies, Chrome textarea, VS Code, Terminal | `VIỆT`, `VIÊT`, `VIÊ`, `VIẸ` (`vkb_core::simulate` gives exactly these) | digits stay literal: `VIE6T5`, `VIE6T`, `VIE6`, `VIE5`. Mixed case `Vie6T` → `ViêT` and Telex all-caps `VIEETJ`/`DDUWOWCJ` work, so it is VNI + all-caps only, and only in the macOS backend path, not the engine | `qa/mac/run-probe-vni-caps/shots/textedit-plain-vni-VN-X*.png`, `…/run-full-2/shots/*-vni-VN-C1.png` | FIXED in `7be7a6f` (+ `f91169a`, letters only). Root cause: the tap trusted the event's Unicode string, which synthetic events (the E2E harness, UI automation, remapping tools) translate with a stale modifier state: the digit right after a Shift release arrived as `^`/`%` with clean flags, so the engine saw a plain character. `keys::reconcile_ascii` now follows the Shift/Caps flags when the string is one of the key's two US characters. Unit tests `vni_all_caps_word_takes_mark_and_tone_digits` (stale / Shift-ignoring strings, fails before) and `reconcile_follows_flags_for_the_keys_own_us_characters`. Final: `qa/mac/run-vni-fix2/` TextEdit plain VNI 29/29 after the review fix `7047677` (no plain→shifted promotion, AZERTY-safe). Earlier: `qa/mac/run-vni-fix/` (28/29; VN-C1, X1, X2 PASS), the one FAIL (VN-X3) was a wrong expectation (`VIẸ`; the engine gives `VỊE`, same as `simulate`), corrected in the harness and re-run PASS in `qa/mac/run-vni-fix-x3/`. Engine unchanged; Windows unaffected (real key state) |
| D-MAC-02 | P2 | settings window (macOS) | Launch VietKB with `start_minimized = false` (or open from the menu) | the whole settings window incl. the footer row (Phím tắt / Nâng cao / Thoát hẳn / Đóng) is visible | window is 480×424 pt with the native title bar; the footer row is cut off at the bottom edge (button labels half visible) | `qa/mac/run-full-1/shots/gui-settings-window.png` | FIXED in `3830882`. Root cause: on macOS Tauri's `inner_size` on a decorated window is the whole frame (title bar ~28 pt included) while the page only gets the area below it, so 424 pt minus the title bar clipped the footer. Windows get the old sizes; macOS windows now open at content + 28 pt, and the page measures itself (`ui/src/lib/fit.ts`) and grows the window by whatever is still hidden; the Accessibility banner resizes relatively. Evidence `qa/mac/run-d-mac-02-fix/shots/` (main 480x453 outer, footer fully visible; Hotkeys, Advanced, Giới thiệu, Bảng gõ tắt full). The banner variant (+80) could not be shown live: this shell holds the grant |
| D-MAC-03 | P3 | owner ops / EVKey | Quit EVKey by `osascript quit app "EVKey"` | exits within 5 s | ignored for >10 s on most runs; `pkill -x EVKey` needed (QA harness falls back). Not VietKB's defect; recorded because the guardrail text names the AppleScript quit | `qa/mac/run-full-1/guardrail.log` | INFO |
| D-MAC-04 | P1 | modal windows (macOS) | Open the settings window, press Phím tắt / Nâng cao / Bảng gõ tắt / Giới thiệu (or open one from the menu-bar menu) while another app is frontmost | the modal appears in front of its parent and of other apps | modal opened but was covered/hidden. Causes: (1) Accessory app is never activated, so new windows ordered behind the frontmost app; (2) `set_enabled(false)` on macOS begins a half-transparent sheet the size of the parent, ordered above the parent's child windows, which painted over the child-window modal | FIXED in `01864d9`. Every window open calls `enter_foreground` (Regular policy + `NSApp activateIgnoringOtherApps`); macOS no longer uses the sheet, the modal stays a child window of main and `Focused(true)` on main while a modal is open re-focuses the modal. Evidence `qa/mac/dock-modal/modal_hotkeys_from_main.png`, `modal_advanced_from_main.png` (modals on top of TextEdit and the other app, 100% visible). Menu-bar-menu path and Bảng gõ tắt / Giới thiệu not shot live (status item unreachable in a crowded menu bar, owner became active); same `show_aux` code path |
| D-MAC-05 | P2 | app shell (macOS) | Open the GUI | Dock icon while any VietKB window is visible, gone when the last closes (M3 revised) | no Dock icon at all (`LSUIElement` + permanent Accessory policy) | FIXED in `01864d9`. Regular while a window exists, back to Accessory on the last `Destroyed`; Dock click with no window (`RunEvent::Reopen`) shows the settings window; Cmd+Q / Dock Quit use the default menu `terminate:` and the `RunEvent::Exit` hook removes the tap; tray quit unchanged. Evidence `qa/mac/dock-modal/dock_with_window.png` (V icon) vs `dock_after_close.png`, `dock_after_close_2.png` (gone); run log `guardrail.log` |
| D-MAC-06 | P0 | macOS tap install (S-02) | Launch without Accessibility, window closed; grant it in System Settings; type | typing converts without restart | the install was retried only by the settings banner poll; no window, no retry until restart | unit tests | FIXED (`b689055`) - the backend worker wakes every 3 s while the tap is missing and Accessibility is granted, and stops once installed. Test `timer_installs_only_when_granted_missing_and_not_quitting` |
| D-MAC-07 | P0 | Input Monitoring (S-03, backend) | Input Monitoring denied | a way to probe and request it | no probe or request existed | unit tests | FIXED (`e1bbaca`, `b689055`) - `vkb_macos::has_input_monitoring_permission` / `request_input_monitoring_permission` (CGPreflight/CGRequestListenEventAccess); commands `input_monitoring_status`, `request_input_monitoring` registered. UI wiring is open (another Dev) |
| D-MAC-08 | P2 | exclusion picker (S-13) | Open the running-apps picker with Raycast/Alfred/Spotlight running | Accessory apps with a bundle id and name are listed | only Regular apps listed | unit tests | FIXED (`e1bbaca`) - policies Regular and Accessory, Prohibited daemons and VietKB itself skipped; test `picker_takes_regular_and_named_accessory_apps_only` |
| D-MAC-09 | P1 | per-app rule for panels (S-14) | Type in Spotlight/Alfred/Raycast over an app with a rule | rule of the receiving app | NSWorkspace.frontmostApplication keeps the old app | unit tests | PARTIAL (`e1bbaca`) - the event's `kCGEventTargetUnixProcessID` is now preferred when non-zero (test `target_pid_of_the_event_overrides_the_frontmost_app`). Hardware keystrokes carry 0 there (the window server routes after the tap), so panels typed by hand are NOT fixed. Not verified live (no app launch allowed). Follow-up: `AXUIElementCopyAttributeValue(systemWide, kAXFocusedApplicationAttribute)` with a short messaging timeout, cached, needs a live check |
| D-MAC-10 | P1 | LaunchAgent path (S-15) | Enable `Mở cùng macOS` from /Volumes/VietKB or an AppTranslocation path | no plist that breaks at login | `current_exe()` written verbatim | unit tests | FIXED (`e1bbaca`) - transient paths use /Applications/VietKB.app/Contents/MacOS/<exe> if present, else autostart is refused with a Vietnamese message; disable removes the plist without needing the exe. No `launchctl bootstrap` (launchd loads RunAtLoad plists at login; the app already runs). Tests with temp dirs |
| D-MAC-11 | P2 | config writes (S-17) | UI patch and hotkey write at the same moment | both fields kept | worker cloned the config and saved it whole, overwriting a concurrent UI field | unit tests | FIXED (`b689055`) - `commands::update_config` serializes every read-modify-write under one mutex; `patch_settings` and all worker writes use it (Windows `patch_settings` gets the same lock, no behaviour change) |
| D-MAC-12 | P2 | secure input (S-18) | Another app holds secure input; type in TextEdit | UI can tell why nothing converts | one log line a minute | unit tests | FIXED (backend, `e1bbaca`, `b689055`) - `vkb_macos::secure_input_blocking()` and `StatusView.secureInputBlocking` (true when the last key-down was passed through for secure input, within 15 s). UI display open |

S-11 / S-12 verdict, 2026-10-31: not Mac defects. `vkb-windows` `state.rs` has the same `Toggle = !effective_enabled()` and the same `remember_app` rule persistence (`86-feature-parity.md` decision 3: persist remembered state like EVKey's `blacklist`), so macOS matches Windows exactly; the macOS behaviour is pinned by `toggle_in_a_rule_off_app_matches_windows`. Changing either is a cross-platform product decision, not done here.
