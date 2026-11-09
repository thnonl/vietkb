# Review của bản triển khai B (live) so với B-final.html và SPEC "B final"

Ngày 2026-11-09. Trang live: vite dev ở `http://127.0.0.1:5199/` (worktree `E:\vietkb-ui-b`), cửa sổ 560 px, `?platform=macos|windows`, `?theme=light|dark`, VI và EN. Ảnh: `design/macos-style/_review-shots/` (số thứ tự trong ngoặc). Ảnh chụp có độ phân giải thấp (khoảng 0,78 lần).

Chỉ thị mới của chủ sở hữu (2026-11-09): "Nhớ chú ý tưởng phản hợp lý" (tương phản hợp lý) thay thế mọi "chấp nhận tương phản thấp". Giữ chữ trắng trên nền màu nhấn và tông Xanh logo sáng.

Đo bằng `getBoundingClientRect` ở Windows và macOS. **Đạt** (khớp spec): token tối/sáng đúng giá trị mục 5; thanh tiêu đề 36 (Win) / 28 (mac); thanh công cụ 52; chrome 88 / 80; tab 64 × 44; công tắc 36 × 20; nút, pop-up, chia đoạn, ô nhập 22; ô tick 16; keycap 18/20; hàng 28 và 38; chân trang 40; Chung Win 420 (khớp spec), Hotkeys 332 (spec 330), Gõ tắt 560; Esc đóng sheet; mũi tên trái/phải đổi tab (roving tabindex, `aria-selected`), vòng focus 3 px; thứ tự Tab thanh công cụ rồi nội dung; EN không làm đổi chiều cao hàng; chữ dài trong bảng gõ tắt cắt dấu "…".

## Tóm tắt mức độ

must-fix: **3**; should-fix: **12**; nit: **9**.

## A. Must-fix

1. **Icon hiển thị thành khối đen đặc (mọi icon dùng `Icon`/`svg.i`).** Màn: tất cả (tìm kiếm, Nhập/Xuất/Thêm trong Gõ tắt, nút Đặt lại, nút đóng X của sheet, toast, banner, "Chọn ứng dụng", lỗi xung đột). Mong đợi: icon nét `currentColor`. Thực tế: `getComputedStyle(svg).fill = rgb(0,0,0)`, `stroke: none`; quy tắc `svg.i {fill:none; stroke:currentColor; stroke-width:1.5; ...}` và `svg.i.s16` có trong `HEAD:ui/src/styles/tokens.css` (dòng ~312) nhưng **không còn** trong `tokens.css` của worktree (grep `svg.i` ra rỗng). Ở nền tối icon đen gần như vô hình, ở nền sáng là chấm đen. Ảnh 04, 06, 10, 11, 14. Sửa: đưa lại hai khối `svg.i` vào `tokens.css` hoặc `kit.css`.
2. **Màu nhấn thiếu tương phản (chỉ thị mới).** Mong đợi: nền có chữ trắng >= 4,5:1. Thực tế `--acc` `#3d8be0` sáng (trắng 3,52:1) và `#5aa2ee` tối (2,68:1), hover `#4a97e8`/`#6ab0f2` (3,05 / 2,3), vòng focus alpha .5/.55 (dưới 3:1 so với nền). Ảnh 21. Đề xuất ở mục C, đã dựng so sánh cạnh nhau ở `B-final.html` mục 0b. Phải sửa vì trái chỉ thị mới.
3. **Chữ phụ 11 px dưới 4,5:1.** `--t2` `rgba(60,60,67,.66)`: hint trên trắng khoảng 4,0:1, nhãn tab thanh công cụ trên `#e8e8ec` khoảng 3,7:1 (tính từ giá trị token đo được). Đề xuất `--t2` alpha .78 (>= 5:1 trên trắng và trên `--plane`); bản tối `rgba(235,235,245,.62)` trên `#2c2c2e` khoảng 5,6:1, đạt.

## B. Should-fix

4. **Chân trang không ghim đáy khi cửa sổ cao hơn nội dung.** `.ct {flex: 0 1 auto}` (v4.css dòng 245), `.v4win` cao 100% nhưng `.ct` chỉ cao theo nội dung. Đo ở khung 720: chân trang nằm y 380–420, dưới đó 300 px trống; toast (`bottom:50px`) bay xuống y 636. Mong đợi (SPEC mục 4): chân trang phần tử cuối, dính đáy. Ở app thật cửa sổ được fit nên đúng, nhưng khi đổi tab, khi banner Trợ năng xuất hiện muộn, hoặc cửa sổ dư vài px thì chân trang nổi giữa. Sửa: `.ct {flex:1 1 auto}`, `.screen {flex:1}`. Ảnh 02, 06.
5. **Hộp Loại trừ ứng dụng chưa restyle (đã biết).** Hàng quy tắc cao **80 / 81 px** (spec 28/38), mỗi ứng dụng xếp 2 tầng, hàng "Loại trừ ứng dụng tự động" không nằm trong thẻ nhóm, ô Kiểu gõ trải full 340 px; icon đen (mục 1); sheet cao 292. Ảnh 11. Mong đợi: hàng 28/38, pop-up 22 trên cùng dòng.
6. **Select dùng chevron xuống, không phải lên-xuống (đã biết).** Thực tế `<select class="sel">` native 96 × 22 với `svg` chevron xuống phủ lên (path `M6.5 8.25 10 11.75l3.5-3.5`). Mong đợi (SPEC mục 6): nắp 16 px biểu tượng lên-xuống. Danh sách mở ra do hệ điều hành vẽ (nền trắng/đen, mục chọn không phải nền màu nhấn); chấp nhận là hạn chế của `<select>`, nên ghi rõ vào spec hoặc làm menu tự vẽ.
7. **Banner Trợ năng (macOS): chiếm 84 px thay vì 80.** Banner 72, nhưng khoảng cách trong `.body` là 12 (không phải lề 8): nhóm đầu bắt đầu y 176, banner kết thúc y 164. Chung mac có banner: ct 416, tổng **496** (spec 492). `AX_BANNER_PX`/`BANNER_PX` = 80 nên cửa sổ thiếu 4 px lúc mở rộng đầu tiên, rồi `fitWindowToContent` mới chỉnh lại: nhảy cửa sổ khi banner xuất hiện muộn. Sửa: `.axbanner {margin-bottom:-4px}` hoặc đổi hằng thành 84 ở cả hai nơi. (Không thể kiểm tra việc cửa sổ native lớn thêm ra sao: xem phần "không kiểm tra được".)
8. **Xung đột phím tắt: hai dòng lỗi chồng nhau, cửa sổ cao thêm 64 px.** Mong đợi (SPEC mục 8, +27): một dòng lỗi dưới nhóm. Thực tế hai dòng ("Tổ hợp này đã dùng cho Mở cài đặt…" và "…đã dùng cho Đổi kiểu gõ…"), mỗi dòng có icon (đen, mục 1), chân trang từ 332 lên 396. Ảnh 18. Nên gộp một dòng và giữ 28 px.
9. **Bảng Gõ tắt: dòng tiêu đề cột cao 36 px (spec 24)**, mũi tên cuối hàng không có (spec có). Đo: `tr.head` 36, hàng 28. Ảnh 04.
10. **Ô tìm gõ tắt cắt chữ tiếng Việt:** placeholder hiện "Tìm từ tắt hoặc n" (ô 202 px, có keycap Ctrl F bên trong). Ảnh 04. Mong đợi đủ chữ ("Tìm từ tắt hoặc nội dung"): hạ keycap hoặc rút placeholder, hoặc nới ô (nút Nhập/Xuất đang chiếm 118 + 112 px).
11. **Ít mục: vùng trắng chết lớn trong Gõ tắt.** 1 mục: bảng cao 381 px, hàng đầu 28 px, còn khoảng 250 px trống trong khung (ảnh 07); 8 mục còn khoảng 160 px trống (ảnh 04). Spec chọn cố định 560 nên đúng spec, nhưng chủ sở hữu từng từ chối khoảng trống: cần quyết định (thu nhỏ cửa sổ theo số mục, tối thiểu khoảng 400) hoặc xác nhận chấp nhận.
12. **Focus đầu tiên của sheet là nút đóng X**, không phải ô đầu tiên (SPEC mục 6, 11 #44). Đo `document.activeElement` ngay sau khi mở sheet "Thêm từ tắt": `BUTTON.btn.ghost.icon.sm`. Sửa: `autofocus` hoặc focus ô "Từ tắt" khi mở.
13. **Trang Giới thiệu chật:** hai nút (Kiểm tra cập nhật, Cập nhật ngay) nằm ngay dưới dòng trạng thái và sát tiêu đề "Nhật ký thay đổi" (khoảng cách chưa tới 4 px), không có đường kẻ ngăn các khối (spec có). Cao 440 (Win) với nhật ký 76 px. Ảnh 15.
14. **Banner "Gõ tắt đang tắt" dùng màu xanh nhạt (`--acc-tint`), không phải vàng (`--warn-tint`) như spec;** dải này đè lên hàng tiêu đề cột (tiêu đề "Từ tắt/Nội dung" biến mất). Ảnh 20.
15. **Thông báo thiếu icon/nút đóng nhìn thấy được** (hệ quả của mục 1) nên "toast" chỉ còn chấm đen và không thấy X. Ảnh 06. Sẽ tự hết khi sửa mục 1; ghi ở đây để QA kiểm lại.

## C. Nit

16. Dòng hướng dẫn phím tắt dùng Keycap cho `Esc` và `Backspace`, mock/spec là chữ thường. Ảnh 03.
17. Ô ghi phím tắt khi đang ghi hiện dấu "+" giữa các phím và nhãn "Esc huỷ" lẫn trong ô (ảnh 17, 18), rối hơn spec.
18. Ô chưa đặt ghi "Chưa đặt Đặt phím" (hai cụm liền nhau); mock chỉ "Chưa đặt". Ảnh 03.
19. Ô nhập trong sheet lỗi chỉ đổi chữ lỗi sang đỏ, **viền ô không đỏ** (spec: viền `--bad` 2 px). Ảnh 10.
20. Chân trang Gõ tắt ghi "0 mục · cuộn để xem hết" khi trống, và "9 mục · cuộn…" dù không cuộn. Mong đợi "Chưa có mục nào" khi trống.
21. Hàng bị vô hiệu ở nền tối (Advanced "Tự động khôi phục từ gõ sai"): công tắc tắt gần như biến mất do opacity .42 trên rãnh tắt; nên dùng viền hoặc opacity .55. Ảnh 14.
22. Nâng cao trên Windows cuộn 52 px không có mờ dần ở đáy (spec có mask 22 px): cuộn được nhưng không gợi ý. Ảnh 14.
23. Bộ đếm ký tự "0 ký tự" ở dưới trái ô nội dung, mock đặt ở phải nhãn. Ảnh 09.
24. `aria-controls="page"` giống nhau ở cả 5 tab (nên trỏ tới `tab-<id>` panel tương ứng). `role="tablist"` và `aria-selected` đúng.

## D. Đề xuất màu nhấn tinh chỉnh ("tương phản hợp lý"), thuộc must-fix #2

Tiêu chí: chữ trắng trên nền có chữ >= 4,5:1 ở sáng và tối; thành phần không chữ >= 3:1 so với màu kề; liên kết >= 4,5:1 trên nền của nó. WCAG 2.x, tính trực tiếp từ hex.

| Token | Hiện tại (sáng / tối) | Tinh chỉnh (sáng / tối) | Chữ trắng trên nền | Nền so với màu kề |
|---|---|---|---|---|
| `--acc` (nút chính, tab/đoạn chọn, mục menu, chip, huy hiệu) | `#3D8BE0` / `#5AA2EE` | **`#2876D2` / `#2876D2`** | 3,52 / 2,68 thành **4,55 / 4,55** | sáng 4,55 (trắng), 4,18 (`#f5f5f7`); tối 3,06 (nhóm `#2c2c2e`), 3,66 (cửa sổ `#1e1e1e`) |
| `--acc-hover` | `#4A97E8` / `#6AB0F2` | **`#2572CC`** | 3,05 / 2,3 thành **4,82** | hover tối hơn một bậc để chữ trắng không tụt |
| `--acc-press` | `#2F7DD0` / `#4C97E6` | **`#1F6FCA`** | 4,2 thành **5,02** | |
| `--acc-switch` (công tắc bật, mới) | `#3D8BE0` / `#5AA2EE` | giữ **`#3D8BE0` / `#5AA2EE`** | không chữ | sáng 3,52 (trắng), 3,23 (`#f5f5f7`); tối 5,20 (nhóm), 6,22 (cửa sổ). So với rãnh tắt: 2,43 sáng, 2,90 tối (cần L >= 0,45 mới tới 3:1, nên dùng tiêu chí so với bề mặt kề và núm trái/phải) |
| `--focus` (vòng focus, viền chọn, viền ô ghi) | alpha .5 / .55 (dưới 3:1) | **đặc `#2876D2` / `#5AA2EE`**, 2 px ngoài cách 1 px | không chữ | sáng 4,55 so với trắng; tối 5,20 so với nhóm |
| icon tab đang chọn | `#3D8BE0` / `#5AA2EE` | `#2876D2` / `#5AA2EE` | không chữ | 3,8 trên `#e8e8ec`; tối >= 5 |
| `--acc-text` (liên kết) | `#1C6CC6` / `#6AB0F2` | giữ | không dùng | 5,24 (trắng), 4,81 (`#f5f5f7`), 4,53 (`#E0F1FD`); tối 6,04 (nhóm), 7,23 (cửa sổ) |
| `--acc-tint` | `#E0F1FD` / `rgba(90,162,238,.2)` | giữ | | |

`#2876D2` là xanh sáng nhất trong họ xanh logo vẫn cho chữ trắng 4,55:1; thử `#2A78D2` chỉ 4,45:1 (không đạt). Ở bản tối `#2876D2` là xanh trung bình giống macOS (không nhạt), vẫn tách khỏi nền 3,06:1 trở lên. Phải đổi cả hai khối trong `tokens.css` (và khối `prefers-color-scheme: dark` thứ ba ở dòng ~194). So sánh trực quan: `design/macos-style/B-final.html` mục 0b, ảnh 21.

## E. Không kiểm tra được

- Gợi ý nhập bảo mật (`SecureInputHint`): chỉ hiện khi chạy native trên macOS; chỉ đọc code (28 + 8).
- Banner Trợ năng: chỉ thấy ở chế độ preview `?platform=macos`; không thấy hành vi native cửa sổ nở thêm 80 px khi cấp quyền muộn (`resizeBy` + `fitWindowToContent`).
- Cửa sổ Tauri thật: kích thước cửa sổ sau `fitWindowToContent`, kéo thả, đèn giao thông, nút thu nhỏ/đóng thật, `Ctrl/Cmd + W`, mở tab từ menu khay, menu khay.
- Giới thiệu trạng thái "mới nhất" (preview luôn giả lập có bản cập nhật v0.9.9).
- Popup của `<select>` (do hệ điều hành vẽ), DPI 125/150%, Retina, chế độ Reduce Transparency.
- macOS tối và các sheet ở chế độ macOS chưa chụp; thang đo macOS chỉ có Chung (banner) và Nâng cao (sáng), cùng số đo chiều cao các tab.
- Tương phản của `--t2` được tính từ giá trị token, không đo trên ảnh chụp.

---

# Round 2 (sau khi coder sửa), 2026-11-09

Cách kiểm: vite dev (PID 63840, cổng 5199), trình duyệt tích hợp 560 px, `?platform=windows|macos`, `?theme=light|dark`. Số đo bằng `getBoundingClientRect` và bản sao đúng logic `naturalHeight()` của `fit.ts` (chiều cao cửa sổ mà `fitWindowToContent` sẽ đặt).

Kết quả: 15 mục đạt (PASS), 3 mục chưa đạt (R9, R12, R14), 1 vấn đề mới must-fix (N1, cần xác nhận trong app thật), 3 nit mới; nit 16 đến 24 còn nguyên, trừ #20 đạt một nửa.

## Xác nhận từng mục coder khai báo

| # | Mục | Kết quả | Đo được |
|---|---|---|---|
| R1 | Icon (`svg.i` trong kit.css) | **PASS** | mọi `svg.i` hiển thị `fill: none`, nét theo màu chữ, cả sáng và tối; icon tìm kiếm, nhập/xuất, thêm, đặt lại, X, cảnh báo đều thấy |
| R2 | Nền có chữ `#2876D2` ở sáng và tối | **PASS** | `.btn.pri` `rgb(40,118,210)` ở cả hai chế độ; chữ trắng 4,55:1 |
| R2b | Hover `#2572CC`, pressed `#1F6FCA` | **PASS** | CSS `.btn.pri:hover {--acc-hover}`, `:active {--acc-press}`; chữ trắng 4,82 và 5,02 |
| R2c | Công tắc `--acc-switch` | **PASS** | `.tg.on` `rgb(61,139,224)` sáng, `rgb(90,162,238)` tối; so với nhóm 3,52 (trắng) và 5,20 (tối) |
| R2d | Vòng focus đặc | **PASS** | `--focus` sáng `#2876d2`, tối `#5aa2ee`; `box-shadow: 0 0 0 1px var(--raised), 0 0 0 3px var(--focus)` (khe 1 px, vòng thấy 2 px) |
| R2e | Liên kết giữ nguyên | **PASS** | `--acc-text` `#1c6cc6` sáng (5,24 trên trắng), `#6ab0f2` tối (6,04) |
| R3 | `--t2` alpha .78 sáng, .72 tối | **PASS** (hết must-fix) | sáng: trắng 5,62; `#f5f5f7` 5,32; `--plane #e8e8ec` 4,96; `#E0F1FD` 5,14. Tối (.72): nhóm `#2c2c2e` 6,91; cửa sổ `#1e1e1e` 7,91; thanh `#2d2d30` 6,81; toast `#3a3a3d` 5,82; nền nút `#4a4a4d` 4,78. Chữ 11 px đều >= 4,5:1; đổi tối từ .62 lên .72 là đúng (ở .62 trên `#4a4a4d` chỉ 4,00) |
| R4 | Chân trang ghim ở cả 5 tab | **PASS** | cửa sổ 720: chân trang y 680 đến 720 ở cả 5 tab; ở chiều cao tự nhiên không có dải trống (Chung 420, Phím tắt 332) |
| R5 | Hộp Loại trừ ứng dụng hàng 28 | **PASS** | hàng quy tắc 28 và 29, pop-up 112 × 22, công tắc 36 × 20, sheet cao 196 (trước 292); hàng "tự động" cao 36 (spec 38, nit) |
| R6 | Select lên-xuống | **PASS** | path `M7 8.25 10 5.25l3 3M7 11.75l3 3 3-3` trong nắp 16 × 16 nền `#2876d2`, nét trắng. Danh sách thả vẫn do hệ điều hành vẽ (hạn chế đã biết) |
| R7 | Banner Trợ năng 80 (macOS) | **PASS** | banner 72, `margin-bottom: -4px`: banner y 92 đến 164, nhóm kế y 172; chiều cao tự nhiên Chung mac với banner **492** (spec 492) |
| R7b | Gợi ý nhập bảo mật 36 | **PASS theo code** (không dựng được trên browser) | `.sechint {height: 28; margin-bottom: -4}` cộng khoảng 12 bằng 36 |
| R8 | Một dòng lỗi xung đột | **PASS** | một dòng "Tổ hợp này đã dùng cho Mở cài đặt…" cao 22, icon hiện; chiều cao tự nhiên 366 (nền 332, thêm 34; spec thêm 27: lệch 7 px, nit) |
| R9 | Ô tìm gõ tắt đủ chữ | **FAIL (should-fix, nhỏ)** | bỏ keycap Ctrl F là chấp nhận được (phím tắt vẫn dùng, chỉ mất gợi ý). Nhưng placeholder "Tìm từ tắt hoặc nội dung" rộng 142 px (đo canvas) trong ô 136 px: vẫn cắt khoảng 6 px ("…nội dun"). Cần thêm ít nhất 8 px (rút nút Thêm 72 px hoặc nới ô tìm) |
| R10 | Chiều cao Gõ tắt theo nội dung | **PASS có điều kiện, xem N1** | bảng số liệu bên dưới |
| R11 | Focus đầu tiên của sheet | **PASS** | `document.activeElement` = `INPUT` placeholder "ví dụ: dc" |
| R12 | Khoảng cách trang Giới thiệu | **PARTIAL (should-fix)** | đã có 2 đường kẻ (`.about-div` y 206 và 369). Nút (Kiểm tra cập nhật, Cập nhật ngay) cách dòng trạng thái **0 px**, cách tiêu đề "Nhật ký thay đổi" **6 px**; chiều cao tự nhiên 446. Nên thêm 6 đến 8 px giữa trạng thái và nút |
| R13 | Banner "đang tắt" lên trên bảng | **PASS** | dải vàng `#fdf0d5` (`--warn-tint`), cao 30, nằm trong `.tbl` ở y 154, trên dòng tiêu đề (y 184) |
| R14 | Dòng tiêu đề cột Gõ tắt (coder chưa sửa) | **FAIL (should-fix, giữ nguyên)** | `.tr.head` cao **36** ở mọi lần đo (spec 24) |

## Số liệu Gõ tắt (R10), Windows, chrome 88

Chiều cao tự nhiên theo số mục, đo ở cửa sổ đủ cao (720):

| Số mục | Tổng cao cửa sổ | Khoảng trống dưới hàng cuối trong khung danh sách |
|---|---|---|
| 0 (trống) | 503 | không: khối minh hoạ lấp đầy (3 mục mẫu và hai nút); chân trang "Chưa có gõ tắt nào" |
| 1 | 363 | khoảng 84 px trắng dưới hàng (danh sách tối thiểu 112) |
| 3 | 363 | khoảng 28 px |
| 8 | 475 | 0 |
| 30 | 587, bị trần 560 chặn | 0: danh sách cuộn, khung tối đa 336 (khoảng 309 khi cửa sổ 560) |

Gọn hơn rõ so với 560 cố định (hết dải trắng 250 px); ở 1 mục còn khoảng 84 px trắng, chấp nhận được (nit).

## Vấn đề mới

- **N1. MUST-FIX (cần xác nhận trong app thật): `naturalHeight()` của Gõ tắt phụ thuộc kích thước cửa sổ hiện tại, nên cửa sổ có thể kẹt nhỏ khi thêm mục.** Tái hiện (browser, cùng logic): đưa cửa sổ về 363 (đang 3 mục), rồi thêm mục: 8 mục vẫn đo **363** (đúng ra 475), 30 mục vẫn **363** (đúng ra 560). Nguyên nhân: danh sách có `flex-shrink`, khi cửa sổ nhỏ nó co về tối thiểu 112, nên `body.scrollHeight` không phản ánh số mục (khác với ghi chú "đáp án như nhau ở mọi kích thước" trong `fit.ts`). Hệ quả: người dùng đang ở cửa sổ 363 thêm nhiều mục thì cửa sổ không nở, danh sách chỉ cao 112 px và phải cuộn. Sửa: tính chiều cao danh sách từ số hàng (hàng × 28 + tiêu đề, kẹp 112 đến 336), hoặc đo với `flex-shrink: 0`. Chưa thử trong cửa sổ Tauri: nếu `fitWindowToContent` luôn chạy khi cửa sổ được phóng tạm thì có thể không gặp, nhưng không nên dựa vào đó.
- N2. Nit: khi chọn hàng, thanh thao tác hàng loạt thay thế dòng tiêu đề cột (tiêu đề biến mất); nên giữ tiêu đề và đặt thanh thao tác ở trên như banner.
- N3. Nit: ô ghi phím tắt khi đang ghi hiện thêm nút X, nhãn "Esc huỷ" và dấu "+" giữa các phím (như #17): hơi rối, chức năng đúng.
- N4. Nit: icon tab đang chọn ở nền tối là `#2876D2` trên thanh `#2d2d30`: 3,02:1 (đạt 3:1 sát ngưỡng); dùng `--acc-switch` `#5AA2EE` (5,12:1) chắc hơn.

## Nit 16 đến 24 (không thuộc phạm vi coder sửa lần này)

| # | Kết quả | Ghi chú |
|---|---|---|
| 16 | Còn nguyên | dòng hướng dẫn vẫn dùng 2 keycap `Esc`, `Backspace` |
| 17 | Còn nguyên | dấu "+" giữa phím khi ghi |
| 18 | Còn nguyên | ô chưa đặt vẫn "Chưa đặt Đặt phím" |
| 19 | Còn nguyên | viền ô nhập lỗi không đỏ (`aria-invalid` trống); chỉ chữ lỗi đỏ |
| 20 | Đạt một nửa | trống đã ghi "Chưa có gõ tắt nào"; vẫn ghi "8 mục · cuộn để xem hết" khi không cuộn |
| 21 | Còn nguyên | hàng vô hiệu opacity .42, công tắc tắt gần như mất |
| 22 | Còn nguyên | Nâng cao Windows cuộn 52 px, không mờ dần (`mask-image: none`) |
| 23 | Còn nguyên | bộ đếm "0 ký tự" ở dưới trái ô |
| 24 | Còn nguyên | `aria-controls="page"` ở cả 5 tab |

## Tóm tắt Round 2

- Còn must-fix: 1 (N1, cần xác nhận trong app thật). Mọi must-fix vòng 1 (icon, tương phản accent, `--t2`) đã đạt.
- Còn should-fix: 3 (R9 placeholder cắt 6 px, R12 khoảng cách About, R14 tiêu đề cột 36 px).
- Nit mới: 3 (N2 đến N4); nit cũ còn nguyên: 8.

## Không kiểm tra được (Round 2)

- Hành vi cửa sổ Tauri thật: `fitWindowToContent` sau khi thêm/xoá mục, kéo/đóng, Ctrl/Cmd+W, khay.
- `SecureInputHint` (chỉ native macOS): chỉ đọc code.
- Popup `<select>` do hệ điều hành vẽ, DPI 125/150%, Retina, macOS tối, các sheet ở chế độ macOS.
- Tương phản tính từ giá trị token, không đo trên ảnh chụp.
