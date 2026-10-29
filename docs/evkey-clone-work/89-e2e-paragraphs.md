# E2E — gõ đoạn văn dài, Telex và VNI

Ngày 2026-10-29. Bổ sung cho `88-e2e-report.md` (12 case ngắn + 20 cấu hình): lần này
gõ **đoạn văn hoàn chỉnh** cho nhiều mục đích, có tiếng Anh trộn, cả hai kiểu gõ.

## Kết quả

| Tầng | Phạm vi | Kết quả |
|---|---|---|
| Engine (headless, `vkb simulate`) | 10 đoạn × Telex + VNI | **20/20 PASS** |
| Engine (headless) | 95 edge case | **95/95 PASS** |
| Desktop E2E (hook thật, `SendKeys`) | 10 đoạn × 2 kiểu + 7 case phím × 2 | **34/34 PASS** |

Máy: desktop thật (không phải Sandbox), build dev `target-dev\release\vietkb.exe`.

## Corpus (`qa/e2e/corpus/`)

| Tệp | Mục đích | Điểm đáng chú ý |
|---|---|---|
| 01-chat | chat thân mật | câu ngắn, từ đệm, dấu phẩy |
| 02-email | email công việc | kính ngữ, ngày tháng bằng chữ |
| 03-ky-thuat | tài liệu kỹ thuật | `hook`, `engine`, `buffer` |
| 04-tin-tuc | bản tin | số liệu, phần trăm |
| 05-hanh-chinh | văn bản hành chính | câu dài, "căn cứ", "trách nhiệm" |
| 06-marketing | quảng cáo | khẩu hiệu, câu mệnh lệnh |
| 07-hoc-tap | ghi chú bài học | từ Hán-Việt, liệt kê |
| 08-dia-danh | địa danh, tên riêng | viết hoa `Đ`, `Hà Nội`, `Nguyễn Du` |
| 09-trung-anh | đặc tả trộn tiếng Anh | `engine`, `UI`, `desktop`, `team` |
| 10-dau-cau | câu hỏi, ngoặc kép, `%` | dấu nháy, chấm phẩy, phần trăm |

`encode-keys.ps1` biến đoạn văn thành phím: tách dấu bằng Unicode NFD nên không cần
bảng chữ viết tay; `ê` → `ee`, `ơ` → `ow`, `đ` → `dd` (Telex) và `6/7/8/9` + `1..5`
(VNI). Cùng một đoạn được gõ hai lần bằng hai kiểu.

## Edge case (`qa/e2e/edge-cases.tsv`)

95 dòng, mỗi dòng một khẳng định: 5 dấu thanh, circumflex/breve/horn, `đ`, cụm
`ươ/ươi/ươu/uyê`, `qu`/`gi`/`ngh`, viết hoa toàn từ, kiểu dấu cũ (`--old-tone-style`),
gõ lại phím dấu (`ass → as`), số và phần trăm, từ tiếng Anh (`deadline`, `class`,
`test`, `review`), VNI `uo7 → ươ`, `d9`, `a11 → a1`, và `w` theo hai trạng thái
`jfwz_as_consonant`.

## Phát hiện

- **`w` đơn lẻ** — engine đã đúng hành vi EVKey, không cần sửa. EVKey mặc định coi
  `j f w z` là phụ âm nên `w` giữ nguyên (`w`, `tw`); khi tắt tuỳ chọn đó thì
  `w → ư`, `tw → tư`, `nhw → như`, `W → Ư` (`20-spec.md` luật 6). Thêm cờ
  `--no-jfwz-consonant` cho `vkb simulate` để khoá cả hai trạng thái bằng test.
- **Sandbox không dùng được làm oracle cho phần dấu.** Chạy đúng bộ test trong
  Windows Sandbox: 7 case ASCII PASS, 18 case có dấu FAIL; text nhận được là chữ gốc
  không dấu (`Bộ → Bo`). Nguyên nhân đã biết: Sandbox không nhận `KEYEVENTF_UNICODE`.
  Vì vậy test gõ phải chạy trên desktop thật.
- **Focus là phần khó nhất của harness**, không phải engine: app giữ foreground trong
  lúc WebView2 vẽ lần đầu, nên các case đầu nhận chuỗi rỗng. Cách xử lý:
  1. thu nhỏ cửa sổ app sau khi khởi động,
  2. click thật vào form thử nghiệm khi `SetForegroundWindow` bị chặn,
  3. chờ foreground ổn định 3 mẫu liên tiếp trước mỗi case,
  4. warm-up tối đa 60 s sau khi app khởi động.
  Sau bốn bước đó: 34/34 PASS, không còn case rỗng.

## Chạy lại

```powershell
# engine, nhanh
powershell -NoProfile -File docs\evkey-clone-work\qa\e2e\check-corpus.ps1
powershell -NoProfile -File docs\evkey-clone-work\qa\e2e\check-edge-cases.ps1

# desktop thật, hook + SendKeys (~10 phút)
powershell -NoProfile -ExecutionPolicy Bypass -File docs\evkey-clone-work\qa\e2e\host-e2e-paragraphs.ps1

# trong Sandbox (chỉ ASCII mới kiểm được)
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-sandbox-e2e.ps1
```
