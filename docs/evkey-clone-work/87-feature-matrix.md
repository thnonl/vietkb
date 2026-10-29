# 87 — Every UI control, proven for Telex and VNI

Rule: a control that only works for one method is a defect, not a feature. This table is the
contract; `crates/vkb-core/tests/feature_matrix.rs` and `feature_matrix_macros.rs` are its
executable form. Status is filled from the test run, then from the sandbox E2E for the cases a
user can actually reach.

## Quick panel

| Control | Key | Telex expectation | VNI expectation | Test | Status |
|---|---|---|---|---|---|
| Bật/tắt tiếng Việt | `input.enabled` | off → keys stay literal | same | engine + E2E F2/F1 | pending |
| Kiểu gõ | `input.method` | Telex triggers `s f r x j z w` | VNI digits `1`–`9`, `0` | corpus | covered (555 cases) |
| Kiểu gõ mới | `typing.modern_tone` | on `hòa`, off `hoà` | on `hoà` via `2`, off differs | `modern_tone_*` | pending |
| Kiểm tra chính tả | `typing.spell_check` | invalid syllable → literal trigger | same | `spell_check_*` | pending |
| Gõ tắt | `typing.macro_enabled` | expansion at boundary | same | `macro_*` | pending |
| Bảng gõ tắt | macros table | editor + import/export | same | UI | pending |

## Nâng cao

| Control | Key | Telex expectation | VNI expectation | Test | Status |
|---|---|---|---|---|---|
| Đặt dấu tự do | `typing.free_tone` | `hoafn` → `hoàn` | `hoan2` → `hoàn` | `free_tone_*` | pending |
| Tự động thêm móc | `typing.auto_horn_uo` | `thuowng` → `thương` | `thu7ong` → `thương` | `auto_horn_uo_*` | pending |
| Viết hoa đầu câu | `typing.auto_capitalize` | `tieengs. vieetj` → `tiếng. Việt` | VNI equivalent | `auto_capitalize_*` | pending |
| j/f/w/z là phụ âm | `typing.jfwz_as_consonant` | z/j/f/w stay consonants | no effect (VNI has no letter triggers) | `jfwz_*` | pending |
| Tự khôi phục từ gõ sai | `typing.restore_wrong_word` | `google` restored at commit | same | `restore_*` | pending |
| Gõ tắt khi đã tắt tiếng Việt | `typing.macro_when_disabled` | expands while off | same | `macro_when_disabled_*` | not wired |
| Gõ tắt theo hoa/thường | `typing.macro_case_follow` | `Btw` → `By the way` | same | `macro_case_follow_*` | not wired |
| Bảng mã | `input.charset` | Unicode / NFD | same | `charset_*` | pending |

Removed by owner and therefore not a contract: `run_as_admin`, `suggestion_fix_1/2`,
`metro_fix`, `disable_on_non_us_layout`, `send_delay_us`. `raw_zone` is reserved, no UI.

## Phím tắt

Nine actions, method-independent: enable, disable, toggle, restore word, exclude app, macro
panel, macro toggle, open settings, cycle method. Each needs one test that the action fires
(`feature_matrix` covers the engine-visible ones; the backend ones need the E2E run).

## Giới thiệu (About)

Version, update check with changelog, install, GitHub, Ko-fi link. UI-only; the one behaviour
with a real failure mode is the modal itself: capability must include `about`, tray must open
it, X must close it and hand the main panel its input back
(`every_window_has_a_capability` plus the E2E case).

## Method-specific rules that the matrix must not blur

- Telex letter triggers are `s f r x j z w`; VNI tone digits are `1`–`5`, modifiers `6`–`9`.
- `jfwz_as_consonant` is meaningful only for Telex.
- VNI `7` on the `uo` cluster must produce `ươ` (EVKey's own rule: "Dấu móc cho uo thành ươ"),
  not `uơ` — this was a real defect found by the matrix.
