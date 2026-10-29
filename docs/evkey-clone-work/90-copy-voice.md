# 90 — Vietnamese copy: voice guide

How VietKB speaks. Applies to site, app strings, toasts, errors, release notes.

## Who we write for

A Vietnamese office worker, 30–60. Types email, Word, Zalo, a browser form all day.
Not technical: does not know what a charset is, has never opened `config.toml`, and
does not want to. Knows Telex or VNI by muscle memory, not by name. Reads one line,
decides, moves on. If a sentence needs a second read, it is wrong.

## The voice in three rules

**1. Talk about what the person does, not what the program does internally.**
- Bad: `Hệ thống sẽ tự động khôi phục trạng thái trước đó.`
- Good: `VietKB mở lại đúng như lần trước.`

**2. A plain verb beats an abstract noun.**
- Bad: `Thực hiện việc nhập dữ liệu từ tệp.`
- Good: `Nhập từ tệp…`

**3. Say the consequence, not the mechanism.**
- Bad: `Bật cờ raw zone để bỏ qua biến đổi.`
- Good: `Email và địa chỉ web giữ nguyên, không bị bỏ dấu.`

## Pronouns

- **No `mình`, no `tôi`, no `chúng tôi`.** A product does not speak about itself in the first person.
- **`bạn` sparingly** — at most once per screen, and only where "you" is the point
  (`những gì bạn gõ`). Never in labels, never twice in one paragraph.
- **Default: no subject**, or **`VietKB` as the subject**.
  Good: `Không ghi lại nội dung gõ.` · `VietKB nằm trong khay hệ thống.`
- Never `quý khách`, `người dùng` as a form of address (fine as a noun in docs, not in UI).

## Banned words

Marketing register. If a sentence still works with the word removed, remove it.

`trải nghiệm` · `đột phá` · `tuyệt vời` · `giải pháp toàn diện` · `sức mạnh` ·
`nâng tầm` · `tối ưu hoá` · `mượt mà` · `thông minh` (as a claim) · `hệ sinh thái` ·
`đồng hành` · `cam kết` · `hoàn hảo` · `số 1`

Also banned: the em-dash as rhythm. One dash per page at most; a full stop is shorter.

## Terminology — one term per concept

| Concept | Use | Never |
|---|---|---|
| The product category | bộ gõ | IME, input method, phần mềm gõ |
| Input methods | Telex, VNI | TELEX, kiểu gõ số, kiểu gõ chữ |
| The setting | kiểu gõ | phương thức gõ, chế độ gõ |
| The feature | gõ tắt | macro, thay thế tự động |
| One entry | từ tắt | viết tắt, khoá tắt, mã tắt |
| The window | bảng gõ tắt | danh sách gõ tắt, quản lý gõ tắt |
| Tray | khay hệ thống | system tray, khay (alone), taskbar |
| Window | cửa sổ, cửa sổ chính | bảng điều khiển, panel, giao diện, màn hình |
| Shortcut | phím tắt | hotkey, tổ hợp phím (only for the combination itself) |
| Output encoding | bảng mã | charset, encoding, code page |
| Tone placement | đặt dấu kiểu mới / kiểu cũ | modern tone, kiểu dấu |

Rule: if a term is already in `ui/src/lib/catalog.ts`, it wins. Do not rename a feature to write a nicer sentence.

## Sentence length

- Aim **≤ 20 words**; one idea per sentence. Split rather than join.
- Labels ≤ 30 characters, descriptions ≤ 64, buttons ≤ 16, toasts ≤ 72, errors ≤ 110.
- Explain by example, not by rule: `gõ dc rồi bấm dấu cách sẽ ra “được”`.

---

## Appendix — approved copy (paste-ready)

### About

Những bộ gõ đang có vẫn gõ đúng, nhưng mang theo cả một thời phần mềm cũ: bảng mã 8-bit, hàng chục tuỳ chọn, nhiều tab.

VietKB có mặt vì phần dùng hằng ngày thì ít, phần phải mang theo thì nhiều. Ở đây chỉ có Telex, VNI, gõ tắt và loại trừ ứng dụng, không có mục nào để bạn phải đoán.

Miễn phí, mã nguồn mở, không quảng cáo, không ghi lại nội dung gõ. Cả bộ gõ nằm trong khay, tốn 14 MB; bộ cài 2,3 MB; mở cửa sổ mất khoảng 0,3 giây.

Một bộ gõ tốt là bộ gõ bạn không phải nghĩ tới.

Bảng mã cũ (TCVN3, VNI Windows, VISCII) và VIQR không có trong VietKB.

### Hero

- h1: `VietKB — bộ gõ tiếng Việt cho Windows`
- Subheadline: `Telex hoặc VNI, gọn trong khay hệ thống. 14 MB, không quảng cáo, không ghi lại những gì bạn gõ.`
- Card 1 — `Telex và VNI`: `Đổi qua lại bằng một phím tắt. Chữ ra Unicode, dán vào đâu cũng đúng.`
- Card 2 — `Gõ tắt`: `Một bảng riêng, nhập xuất được để mang sang máy khác, có công tắc bật tắt riêng.`
- Card 3 — `Loại trừ ứng dụng`: `Chọn ứng dụng không cần bộ gõ, như game hay phần mềm có phím tắt riêng.`

Links to `https://github.com/thnonl/vietkb` and the Ko-fi button stay as they are.
