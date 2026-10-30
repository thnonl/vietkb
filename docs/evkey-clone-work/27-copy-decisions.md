# 27 — Copy & behaviour decisions (Lead, 2026-10-28)

Answers to every `CẦN CHỐT` in `design/COPY.md`, plus the one copy change I reject.
These are binding for design, dev and QA.

## A. Rejected copy change

**`Tự động khôi phục từ gõ sai` keeps its dependency on `Kiểm tra chính tả`.**
The copy deck claims the spec says otherwise. It does not: EVKey's own help string is
*"Khi từ đặt dấu bị sai chính tả, chương trình sẽ tự động khôi phục thành nguyên gốc phím nhập,
chỉ thực hiện khi sử dụng chung với chức năng kiểm tra chính tả"* (extracted from `EVKey64.exe`).
Restoring needs a notion of "wrong", and that notion is the spell checker. Keep the greyed row,
keep the inline "Bật" link, and write the reason in plain words:

- label: `Tự động khôi phục từ gõ sai`
- description when disabled: `Cần bật Kiểm tra chính tả`
- the bad rewrite `gôgle tự trở về google…` is not usable — it mixes cases and reads as a bug.

## B. Off-state wording — **approved**
`Đã tắt tiếng Việt`, not `English`. The language option already owns the word "English", and the
state is about Vietnamese input, not about the user's language. Tray icon stays `V` / `E`.

## C. The 14 answers

| # | Question | Decision |
|---|---|---|
| 1 | `Tắt khi bàn phím không phải US` is in design but not in spec | **Keep.** It is EVKey parity (`Tắt TV khi Layout bàn phím khác US`) and already in config as `typing.disable_on_non_us_layout`, default `true`. Spec C gets a row. |
| 2 | Typing delay unit and default | Field `typing.send_delay_us: u32`, unit **µs**, default **0**, range 0–5000, step 500. Visible only in `Nâng cao`. The mock's `8` becomes `0`. |
| 3 | Which charsets the `Đổi bảng mã` hotkey cycles through | Exactly these four, in this order: `Unicode → TCVN3 (ABC) → VNI Windows → Unicode tổ hợp`, then wrap. Unimplemented charsets never enter the cycle. |
| 4 | ClipBoard conversion source charset | Source = `Unicode` (accepts both NFC and NFD input, normalises through NFD internally); target = the currently selected `bảng mã`. One direction only; no auto-detect UI. |
| 5 | `UTF-8 Literal` | **Not a product concept here.** Do not show it. `utf8` stays in the enum for future config compatibility but is hidden from every picker. |
| 6 | `BK HCM1` / `VPS` | Hidden until their byte tables are verified (Sprint 2+). Pickers show the 6 verified charsets only: Unicode, Unicode tổ hợp, TCVN3, VNI Windows, VISCII, VIQR. |
| 7 | Does a VietKB hotkey stop the app from getting that key | Yes when it matches — that is what a toggle is. Two guards: reserved combos (`Win+L`, `Alt+Tab`, `Ctrl+Alt+Del`, `Ctrl+Shift+Esc`) are **rejected** at assignment with a calm message, and very common app shortcuts (`Ctrl+C/V/X/Z/A/S/F/Y`) show an amber warning but are allowed. |
| 8 | Macro import format | **TSV, UTF-8, no BOM**: `trigger<TAB>expansion`, one per line, optional first line `#vkb1` header, `\n` escapes a newline inside an expansion, `\t` escapes a tab. Max 5000 rows, 2 000 000 bytes. Wrong format = whole import rejected, nothing partially applied, error names the first bad line. |
| 9 | Empty macro expansion | **Rejected.** Expansion must be 1–500 characters. Message: `Phần thay thế không được để trống — muốn bỏ thì xoá cả mục.` |
| 10 | What `Đặt lại tất cả` wipes | Scope is per screen and stated in the confirm: on `Phím tắt` it resets **hotkeys only**; on `Hệ thống` it resets **typing + system + hotkeys**. **Macros and excluded apps are never touched** — they are user data. The confirm says so. |
| 11 | Does the engine restart after a crash | Yes. The hook watchdog re-installs and the engine resets its buffer; the user sees one calm toast `Đã khôi phục bộ gõ` and loses nothing already typed. No dialog. |
| 12 | Where the "runs as Admin" note lives, now that Tổng quan is gone | Two places: a persistent chip in the status strip only while elevated (`Quyền Admin`), and the row itself in `Hệ thống`. No popup. |
| 13 | The `…` menu on a macro row | `Nhân bản` and `Xoá`. No per-row export. |
| 14 | Off-state wording | Approved — see §B. |

## D. Contract additions from this pass

- `TypingConfig::send_delay_us: u32` (default 0) — new field, appended so old configs still load.
- Charset cycle order is fixed in code as `[Unicode, Tcvn3, VniWindows, UnicodeNfd]`.
- Macro import/export is TSV per C8; `import_macros` returns the row count, never a partial import.
- Tray tooltip: `VietKB — đang gõ tiếng Việt` / `VietKB — đã tắt tiếng Việt`.

### D.1 Copy that COPY.md does not cover (written by the Lead)

The toggle mode moved out of the `Phím tắt` row into `Nâng cao` on `Gõ` (round 5, option b):

| Vị trí | Chuỗi | Dài |
|---|---|---|
| label | `Kiểu chuyển bật/tắt` | 19 |
| option (default) | `Khi nhả phím` | 12 |
| option | `Khi nhấn phím` | 12 |
| description | `Nhả phím mới chuyển, hoặc đổi ngay khi nhấn.` | 45 |
| description, `Tự động khôi phục từ gõ sai` (enabled) | `Trả lại đúng các phím bạn đã gõ khi từ bị sai.` | 46 |

It maps to `hotkeys.toggle_mode = "release" | "press"` — a new field, default `release`.
The reason it is in `Nâng cao`: a bare `Nhả để chuyển` dropdown next to keycaps was unreadable,
and removing it from the shortcut row keeps that row at 46 px.


## E. Copy rules the deck got right, keep enforcing

Label ≤ 30 chars · description ≤ 64 · button ≤ 16 · tab ≤ 12 · toast ≤ 72 · error ≤ 110.
One term per concept. No ALL CAPS. Explain by example (`hoangf → hoàng`) instead of by rule.

## F. Withdrawn — sound picker copy (2026-10-30)

The **15 strings** written for the `Kiểu âm thanh` picker (7 pack labels, 7 one-line hints, 1
`schemeSilent` advisory) are **withdrawn, not shipped, not deleted**: owner ruling 2026-10-30 cut the
picker, `sound_pack`, `preview_sound` and the `Nâng cao` 388 → 424 growth, leaving one embedded
`go_moc` ON/OFF pair with `system.notification_sound` as the only switch. The verbatim text and the
three phrasing risks are kept for the record in `22-sound-spec.md` §9 (History), marked "removed with
the scope cut". Copy: **nothing to chase** — no new string exists for this feature, and the toggle
keeps its existing `Âm thanh khi bật/tắt` label untouched. Do not re-open without an owner ask.
