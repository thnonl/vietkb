# VietKB: ghi chú spec cho 3 hướng giao diện kiểu macOS

Mockup: `design/macos-style/index.html` (mở bằng trình duyệt; nút trên đầu đổi macOS/Windows và màu nhấn).
Chưa chọn hướng nào; không có thay đổi nào trong `ui/` hay `crates/`.

## Giả định chung

- Chữ tiếng Việt lấy từ `ui/src/lib/locales/vi.ts`. Không thêm khoá chuỗi mới ở A và B. C chỉ dùng lại chuỗi sẵn có (Đóng, Phím tắt…, Nâng cao…).
- Kích thước hiện nay: cửa sổ chính 480 × 460 (`main_content_height` = `content_height(460)`), Phím tắt 520 × 460, Nâng cao 560 × 496 theo DESIGN-SPEC v4.2. Caption Windows 36 px, caption macOS 28 px (`MAC_CAPTION_PX`).
- Màn chính hiện có 8 hàng: Bật tiếng Việt, Kiểu gõ (chỉ Telex, VNI), Đặt dấu kiểu mới, Kiểm tra chính tả (mặc định tắt), Cho phép gõ tắt, Loại trừ ứng dụng, Khởi động cùng / Mở cùng, Chạy ẩn. Dòng Bảng mã không còn trong `Quick.svelte`, nên mockup bỏ.
- Phím tắt trong mockup là ví dụ minh hoạ, không phải mặc định thật. Trên macOS hiện ký hiệu ⌃ ⌥ ⇧ ⌘; trên Windows hiện Ctrl, Alt, Shift, Win.
- Nền tảng mặc định: dùng chung tokens, khác nhau qua `data-platform` (đã có trong `mac.css`).
- Accent: macOS có thể dùng `AccentColor` của hệ thống (WKWebView hỗ trợ); Windows và khi người dùng thích thì dùng tím Móc `#5844c8`. Đề xuất: `--acc: AccentColor` trên mac với `@supports`, còn lại `#5844c8`. Chủ sở hữu cần chọn.

## Tokens dùng chung (thay `tokens.css`, áp cho cả 3 hướng)

| Token | Sáng | Tối |
|---|---|---|
| nền cửa sổ `--win` | `#f5f5f7` | `#1e1e1e` |
| nhóm `--grp` | `#ffffff` | `#2c2c2e` |
| thanh bên / thanh công cụ | `#e8e8ec` / `#e4e4e8` | `#2a2a2d` / `#2d2d30` |
| chữ 1 / 2 | `#1d1d1f` / `rgba(60,60,67,.65)` | `#f5f5f7` / `rgba(235,235,245,.62)` |
| đường ngăn `--sep` | `rgba(0,0,0,.09)` | `rgba(255,255,255,.1)` |
| accent hệ thống | `#007aff` | `#0a84ff` |
| accent Móc | `#5844c8` | `#8b79ee` |
| nút thường | nền `#fff`, viền `rgba(0,0,0,.15)` | nền `#4a4a4d` |

- Font: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif`, 13 px, nhãn hàng weight 400 (nay đang 600). Hint 11 px màu chữ 2. Giữ Sitka 15 px cho `hòa, thúy`.
- Bo góc: nhóm 10 (A, C) hoặc 8 (B), nút và pop-up 6, ô chia đoạn 8 (nút chọn bên trong 6), cửa sổ mac 10 (hệ thống vẽ), cửa sổ Windows 8.
- Bóng: chỉ `box-shadow` tĩnh, không `filter`, không `backdrop-filter` (trừ C, xem dưới).

## Linh kiện dùng chung (làm một lần, dùng cho cả 3)

- Công tắc: 38 × 22, núm 18, bật = accent, tắt = `rgba(120,120,128,.3)` sáng / `.5` tối. Thay `Toggle` hiện nay (đang 44 × 24?). Giữ `role="switch"` và bàn phím.
- Pop-up (`Select`): cao 22, bo 6, chevron lên xuống 12 px. Biến thể B có “nắp” accent 16 px bên phải.
- Chia đoạn: nền `rgba(0,0,0,.06)`, nút chọn trắng có bóng nhẹ, cao 20 trong khung 24.
- Nút: cao 22 (24 trong chân trang B), bo 6; nút mặc định (Đóng) tô accent chữ trắng.
- Hàng: cao tối thiểu 34 (A), 30 (B), 32 (C); đường ngăn 1 px thụt vào 12 px; hint dưới nhãn 11 px, hint ngắn (`hòa, thúy`, `6 ứng dụng`, phím tắt) cùng dòng, `white-space:nowrap`.
- Ô ghi phím tắt: cao 24, bo 6; trạng thái đang ghi viền accent 2 px; “Chưa đặt” màu chữ 3; chip “Dễ trùng” nền vàng nhạt.

## A. System Settings

**Bố cục** (520 × 460, cố định): thanh bên 148 px (đặc, `--side`, viền phải 1 px) + vùng nội dung.
- Thanh bên: padding trên 40 (36 trên Windows), mục cao 28, bo 6, biểu tượng ô vuông 20 × 20 bo 5 (glyph 12 px nét 2, màu: Chung `#8e8e93`, Phím tắt `#5e5ce6`, Gõ tắt `#ff9f0a`, Nâng cao `#30b0c7`, Giới thiệu `#34c759`). Mục đang chọn nền accent chữ trắng. “Thoát hẳn” ghim đáy thanh bên (biểu tượng nguồn `#ff453a`). Không có nút “Đóng” riêng: dùng nút đóng của cửa sổ.
- Nội dung: tiêu đề trang 15/700 cao 34, rồi các nhóm, lề ngang 16, khoảng cách nhóm 14. Màn chính cao khoảng 324 px nội dung, còn dư.
- Phím tắt: dòng hướng dẫn 11 px, 6 hàng + một nhóm “Kiểu chuyển bật/tắt”. Nhãn bị cắt bằng dấu ba chấm khi hẹp.
- Gõ tắt (bảng dữ liệu) và Giới thiệu: trang riêng trong cùng cửa sổ; Nâng cao dùng 3 nhóm, nội dung cuộn dọc nếu cần. Nếu Nâng cao cần 560 px rộng thì cho cả cửa sổ là 560 × 496.

**Token cần đổi**: bộ tokens chung ở trên; thêm `--side`, `--grp`, `--sep`.
**Dùng lại**: `Toggle`, `Select`, `Hotkey capture`, các hàng của Quick/Advanced (đổi CSS, giữ cấu trúc).
**Thay mới**: `Sidebar.svelte` (mới), shell cửa sổ với điều hướng trang trong cùng cửa sổ (thay vì mở cửa sổ Tauri phụ cho Phím tắt, Nâng cao, Giới thiệu, Bảng gõ tắt). `.v4win`, `.cap-bar` thu gọn.
**Mac**: giữ overlay title bar, đèn giao thông thật nằm trên thanh bên; vùng kéo = dải 28 px trên cùng.
**Windows**: không đèn giao thông; dải 28 px trên cùng có logo Móc 14 px + “VietKB” 12 px bên trái và nút thu nhỏ/đóng 44 × 28 bên phải (đóng hover đỏ như nay). Cuộn, kéo cửa sổ qua dải đó.
**Chi phí**: không blur, không ảnh; 5 SVG nhỏ nội tuyến. RAM tương đương hiện nay.

## B. Thanh công cụ (Preferences)

**Bố cục** (480 × 460, cố định, như nay): thanh tiêu đề 28 + thanh công cụ 56 (nền `--tool`, viền dưới 1 px) + nội dung + chân trang 44 (viền trên 1 px).
- Tab: rộng 68, cao 50, glyph 22 nét 1.5 + nhãn 11 px; đang chọn nền `--hl` và chữ/biểu tượng accent. 5 tab: Chung, Phím tắt, Gõ tắt, Nâng cao, Giới thiệu.
- Tiêu đề (mac): tên tab hiện giữa thanh tiêu đề, weight 600.
- Nội dung: padding 12 16 0, nhóm bo 8 viền 1 px `--sep`, hàng cao 30, khoảng cách nhóm 12. Màn chính cần khoảng 322 px trong 332 px còn lại (đã vừa). Nâng cao cuộn dọc, mờ dần 22 px ở đáy (`mask-image`, rẻ).
- Chân trang: “Thoát hẳn” (chữ phụ) bên trái, “Đóng” (nút mặc định) bên phải.

**Token đổi**: bộ chung + `--tool`, `--hl`.
**Dùng lại**: gần như toàn bộ Quick, Hotkeys, Advanced (đổi thành nội dung tab); chân trang hiện có.
**Thay mới**: thanh tab (một component), bỏ cửa sổ phụ.
**Mac**: đèn giao thông thật trên thanh tiêu đề 28 px.
**Windows**: thanh tiêu đề thành logo Móc + “VietKB”, nút thu nhỏ/đóng 44 × 28 phải; thanh công cụ giữ nguyên.
**Chi phí**: thấp nhất; không hiệu ứng.

## C. Popover thanh menu / khay

**Bố cục**: popover 340 × 424 (cao +80 nếu có banner Trợ năng) bo 12 (mac) / 8 (Windows), padding 10, khoảng cách khối 8.
- Đầu: ô Móc 28 × 28 bo 8 + “Bật tiếng Việt” 14/600 + phím tắt + công tắc, cao 44.
- Kiểu gõ: chia đoạn Telex | VNI (đổi bằng một lần bấm; dành cho chỉ 2 kiểu hiện có).
- Các nhóm còn lại như A nhưng hàng cao 32.
- Chân trang 30: `Phím tắt…`, `Nâng cao…`, “Thoát hẳn” bên phải.
- Sheet: trượt xuống từ cạnh trên, chừa 30 px phần màn chính phía sau (tối 18%), tiêu đề giữa, “Đóng” accent bên phải. Phím tắt nằm trong 340 px (ô ghi 92 px tối thiểu).
- Cửa sổ vị trí: cạnh biểu tượng thanh menu (mac) hoặc phía trên khay (Windows); đóng khi mất focus hoặc nhấn Esc.

**Chi phí / hiệu ứng**: mờ nền là phần tốn nhất. Mac: dùng NSVisualEffectView qua Tauri (hệ thống vẽ, rẻ) và nền CSS trong suốt. Windows: Mica/Acrylic qua crate `window-vibrancy` hoặc nền đặc alpha .94. Không bật `backdrop-filter` CSS trong bản phát hành. Mock dùng `backdrop-filter` chỉ để minh hoạ, có `@supports` fallback nền đặc.
**Windows**: không đèn giao thông, không thanh tiêu đề, không nút thu nhỏ; bóng nhẹ + viền 1 px; neo vào khay.
**Thay mới nhiều nhất**: định vị cửa sổ theo biểu tượng khay, ẩn khi mất focus, sheet, loại bỏ cửa sổ thường (có thể giữ lối thoát “Mở cửa sổ đầy đủ” nếu cần).

## Kiểm tra nghiệm thu (cho coder / QA sau khi chọn)

1. Sáng và Tối: không chữ nào dưới độ tương phản 4.5:1 (chữ phụ 11 px phải đạt 4.5:1; kiểm tra `--t2` nền `--grp`). Đổi giao diện hệ thống thì đổi theo (`data-theme="system"`).
2. Kích thước cửa sổ đúng như spec trên mỗi hệ điều hành; không có thanh cuộn ngang; mọi nhãn tiếng Việt hiện đủ hoặc cắt có dấu “…” (không tràn ra ngoài nhóm); tiếng Anh (`en.ts`) cũng vừa.
3. Từng hàng: Tab đi hết các điều khiển theo thứ tự trực quan; viền focus 2 px accent thấy rõ; công tắc có `role="switch"` và `aria-checked`; Space/Enter bật tắt.
4. Trạng thái: tắt, bật, vô hiệu, pop-up đang mở (Esc đóng, mũi tên chọn), ô ghi phím tắt (đang ghi, chưa đặt, dễ trùng, trùng thật).
5. Màn chính: bật/tắt tiếng Việt, đổi Telex/VNI, hai hàng hệ thống đổi đúng văn bản theo OS (“Mở cùng macOS” / “Khởi động cùng Windows”; hint “thanh menu” / “khay”).
6. macOS: đèn giao thông đúng vị trí, vùng kéo 28 px hoạt động, banner Trợ năng (+80) và gợi ý nhập bảo mật (+36) không làm hỏng bố cục; vẫn đúng khi Reduce Transparency bật.
7. Windows: nút thu nhỏ/đóng hover; không còn dấu vết đèn giao thông; DPI 100/125/150% không cắt chữ.
8. Hiệu năng: RAM của tiến trình giao diện không tăng quá 5% so với bản hiện tại (đo với `Get-Process` sau 30 s ở trạng thái rảnh); mở cửa sổ ≤ 0,3 giây; không `backdrop-filter` trong CSS phát hành (grep); không ảnh bitmap mới.
9. Ảnh chụp so sánh: sáng/tối × mac/Windows × từng màn (chính, Phím tắt, Nâng cao, Giới thiệu) lưu vào `design/_shots`.
10. Không phá logic gõ: chạy bộ test giao diện hiện có và một vòng gõ thật (Telex, VNI) sau khi đổi giao diện.

## Màu nhấn lấy từ logo (cập nhật 2026-11-09, chữ trắng trên nền màu nhấn)

**Phán quyết 1 của chủ sở hữu (2026-11-09):** “các màu nhấn mới bị dark quá, không hợp, tăng tone cho sáng lên được không?”. Nền màu nhấn (công tắc bật, ô chia đoạn đang chọn, nút mặc định, mục thanh bên A, nắp pop-up B, ô Móc C, huy hiệu) không bao giờ tối hơn màu logo; chế độ tối sáng bằng hoặc hơn chế độ sáng. Tương phản công tắc bật so với nền trắng/rãnh có thể dưới 3:1 vì vị trí núm và `aria-checked` mang trạng thái.

**Phán quyết 2 của chủ sở hữu (2026-11-09):** “Màu chữ trên nền màu nhấn nên để màu trắng như cách làm với màu xanh với tím móc trước đó”. Chữ và biểu tượng trên mọi nền màu nhấn đều là trắng `#FFFFFF` (`--on`), cho mọi lựa chọn, cả hai bậc độ sáng, sáng và tối. Quy tắc cũ “dùng mực `#1C1C1A` trên nền khi chữ trắng không đạt” **đã bị bỏ**. Không tối nền để bù. Để dễ đọc hơn mà không đổi màu: chữ trên nền màu nhấn đậm 600, từ 13 px trở lên (nút mặc định, mục thanh bên A, mục đang chọn trong menu pop-up), và dùng `text-shadow: 0 .5px 0 rgba(0,0,0,.18)` rất nhẹ (đã dùng trong mock, trông sạch, tắt được nếu muốn phẳng hơn). Đề xuất khi triển khai: giữ cả hai (đậm 600 và bóng nhẹ).

`index.html` có nút “Độ sáng màu logo”: **Logo gốc** và **Sáng hơn** (mặc định, vẫn đề xuất). Nút “Màu nhấn”: Xanh hệ thống, Tím Móc, Hồng logo, Cam logo, Xanh logo, Logo V (hồng + cam, mặc định), Theo chế độ gõ (demo). A, B, C, sáng và tối, đều đổi theo.

Token mỗi màu: `--acc` (nền), `--on` (luôn `#ffffff`), `--acct` (chữ liên kết), `--acc2` (viền focus/ô đang ghi phím), `--acc2t` (chữ của `--acc2`). Logo V: `--acc` hồng, `--acc2` cam.

**Tỉ lệ chữ trắng trên nền** (WCAG 2.x). “Phán quyết” = dưới 4,5:1 nhưng được chủ sở hữu chấp nhận, không phải FAIL:

| Lựa chọn | Bậc | Chế độ | Nền `--acc` | Trắng / nền | Ghi chú |
|---|---|---|---|---|---|
| Hồng | gốc | sáng | `#F06D95` | 2,87 | phán quyết |
| Hồng | gốc | tối | `#F27FA3` | 2,52 | phán quyết |
| Hồng | sáng hơn | sáng | `#F77FA3` | 2,46 | phán quyết |
| Hồng | sáng hơn | tối | `#F992B0` | 2,15 | phán quyết |
| Cam | gốc | sáng | `#D97A1C` | 3,11 | phán quyết |
| Cam | gốc | tối | `#E8923A` | 2,44 | phán quyết |
| Cam | sáng hơn | sáng | `#F28F2C` | 2,40 | phán quyết |
| Cam | sáng hơn | tối | `#F6A040` | 2,10 | phán quyết |
| Xanh logo | gốc | sáng | `#1C6CC6` | 5,24 | đạt |
| Xanh logo | gốc | tối | `#4A97E8` | 3,05 | phán quyết |
| Xanh logo | sáng hơn | sáng | `#3D8BE0` | 3,52 | phán quyết |
| Xanh logo | sáng hơn | tối | `#5AA2EE` | 2,68 | phán quyết |
| Logo V | cả hai | sáng, tối | hồng như trên; cam phụ chỉ làm viền, không có chữ trắng trên cam | như hồng | phán quyết |
| Xanh hệ thống | | sáng | `#007aff` | 4,02 | phán quyết |
| Xanh hệ thống | | tối | `#0a84ff` | 3,65 | phán quyết |
| Tím Móc | | sáng | `#5844c8` | 6,83 | đạt |
| Tím Móc | | tối | `#8b79ee` | 3,46 | phán quyết |

Nền so với bề mặt kề bên (công tắc bật, tham khảo, đều “được chấp nhận” theo phán quyết 1): hồng sáng hơn 2,46 so với trắng, cam sáng hơn 2,40, xanh sáng hơn 3,52; ở chế độ tối mọi nền >= 4,57 so với `--grp`.

Không còn mục FAIL nào ngoài các mục “phán quyết”. Điểm cần nhắc cho a11y: chữ trắng 13 px trên nền sáng hơn 2,1 đến 3,5:1 dưới ngưỡng WCAG AA 4,5:1; đã giảm nhẹ bằng chữ đậm 600 và bóng nhẹ. Nếu sau này có phàn nàn về độ đọc, phương án quay lại là tối bớt nền (không phải đổi màu chữ).

Chữ liên kết/nhãn màu nhấn không tô màu ở bản sáng của hồng và cam (dùng mực thường), xanh logo giữ `#1C6CC6`, bản tối dùng ngay màu nền. Nhãn tab B đang chọn dùng mực, chỉ biểu tượng tô màu nhấn.

Ý tưởng “Theo chế độ gõ” (cần chủ sở hữu duyệt): gõ tiếng Việt thì accent Logo V, tiếng Anh thì Xanh logo (cũng chữ trắng). Demo trong `index.html`. Cài thật: đặt `data-mode` trên `:root` khi bật/tắt tiếng Việt; chuyển màu tức thì, không animate.

Triển khai: thêm 5 token vào `tokens.css` cho sáng và tối, thay `#fff` cứng trên nền accent bằng `var(--on)` (đã là trắng), thêm chữ đậm 600 cho điều khiển trên nền accent.

---

# B final: spec triển khai (2026-11-09)

Bản dựng mẫu: `design/macos-style/B-final.html` (mọi màn, sáng và tối, macOS và Windows; nút đổi hệ điều hành ở đầu trang; số đo trong chú thích mỗi khung là đo trực tiếp từ mockup). `index.html` giữ A và C làm lịch sử, B nằm trên cùng.

## 1. Phán quyết của chủ sở hữu (còn hiệu lực, ghi lại nguyên văn)

| Ngày | Lời chủ sở hữu | Hệ quả |
|---|---|---|
| 2026-11-09 | “Tôi nghĩ hãy dùng màu nhấn xanh logo đi và phong cách B.” | Phong cách B (thanh công cụ), màu nhấn Xanh logo |
| 2026-11-09 | “các màu nhấn mới bị dark quá, không hợp, tăng tone cho sáng lên được không?” | Nền màu nhấn không tối hơn logo; tối sáng hơn hoặc bằng sáng |
| 2026-11-09 | “Màu chữ trên nền màu nhấn nên để màu trắng như cách làm với màu xanh với tím móc trước đó” | Chữ/biểu tượng trên nền màu nhấn luôn `#FFFFFF`, đậm 600 |
| 2026-11-09 | (công tắc dưới 3:1) | ~~Chấp nhận~~ đã bị thay thế bởi dòng bên dưới |
| 2026-11-09 | “Nhớ chú ý tưởng phản hợp lý” (tương phản hợp lý) | Thay thế mọi “chấp nhận tương phản thấp”: chữ trắng trên nền >= 4,5:1; thành phần không chữ >= 3:1; giữ chữ trắng và tông xanh logo sáng |

## 2. Màu nhấn: bản tinh chỉnh “tương phản hợp lý” (2026-11-09, thay thế bản “Sáng hơn”)

**Chỉ thị mới của chủ sở hữu (2026-11-09): “Nhớ chú ý tưởng phản hợp lý”** (tương phản hợp lý). Chỉ thị này **thay thế** quy tắc “chấp nhận tương phản thấp theo phán quyết”: mọi ghi chú “chấp nhận dưới 4,5:1” và “công tắc dưới 3:1” ở trên và ở bảng cũ phía dưới là **đã bị thay thế (superseded)**. Vẫn giữ: chữ trắng trên nền màu nhấn, tông Xanh logo sáng (không xỉn).

Nguyên tắc: (a) nền có chữ (nút chính, tab/đoạn đang chọn, mục menu chọn, chip, huy hiệu): chữ trắng >= 4,5:1 ở cả sáng và tối; (b) thành phần không chữ (công tắc bật, vòng focus, viền chọn): >= 3:1 so với màu kề bên; (c) liên kết/nhãn tô màu: >= 4,5:1 trên nền của nó.

| Token | Sáng | Tối | Chữ trắng / nền | Tương phản nền với màu kề |
|---|---|---|---|---|
| `--acc` (nền có chữ) | `#2876D2` | `#2876D2` | 4,55 / 4,55 | sáng: 4,55 so với trắng, 4,18 so với `#f5f5f7`; tối: 3,06 so với nhóm `#2c2c2e`, 3,66 so với cửa sổ `#1e1e1e` |
| `--acc-hover` | `#2572CC` | `#2572CC` | 4,82 / 4,82 | hover tối hơn một bậc để chữ trắng vẫn >= 4,5 |
| `--acc-press` | `#1F6FCA` | `#1F6FCA` | 5,02 / 5,02 | |
| `--acc-switch` (mới, công tắc bật, không chữ) | `#3D8BE0` | `#5AA2EE` | không có chữ | sáng: 3,52 so với trắng, 3,23 so với `#f5f5f7`; tối: 5,20 so với nhóm, 6,22 so với cửa sổ |
| `--focus` (vòng, viền chọn) | `#2876D2` đặc, 2 px ngoài, cách 1 px | `#5AA2EE` đặc, 2 px ngoài, cách 1 px | không có chữ | sáng: 4,55 so với trắng; tối: 5,20 so với nhóm |
| `--acc-text` (liên kết) | `#1C6CC6` | `#6AB0F2` | không dùng | 5,24 trên trắng, 4,81 trên `#f5f5f7`, 4,53 trên `#E0F1FD`; tối: 6,04 trên nhóm, 7,23 trên cửa sổ |
| `--acc-tint` | `#E0F1FD` | `rgba(90,162,238,.2)` | | nền hàng chọn |
| icon tab đang chọn | `#2876D2` | `#5AA2EE` | không có chữ | 3,8 so với thanh công cụ sáng `#e8e8ec`; tối >= 5 |
| `--on-acc` | `#FFFFFF` | `#FFFFFF` | | đậm 600, cỡ >= 13 px; bóng chữ `0 .5px 0 rgba(0,0,0,.18)` |
| `--bad` | `#d92d20` | `#ff6961` | | chữ lỗi 11 px |

Khác biệt so với bản “Sáng hơn” cũ: nền có chữ tối đi từ `#3D8BE0` / `#5AA2EE` xuống `#2876D2` (vẫn là xanh logo sáng nhất đạt chữ trắng 4,5:1; ở bản tối `#2876D2` là xanh trung bình giống macOS, không nhạt). Công tắc và vòng focus vẫn dùng xanh sáng hơn. Hai ô so sánh cạnh nhau “hiện tại” và “tinh chỉnh” nằm ở mục 0b của `B-final.html`.

Ghi chú rãnh công tắc: so với rãnh tắt, nền bật chỉ cao hơn 2,2–3,4:1 (sáng 2,43 với `#3D8BE0`; tối 2,90 với `#5AA2EE`). Đạt 3:1 so với rãnh cần xanh nhạt hơn nhiều (L >= 0,45), nên giữ tiêu chí so với bề mặt kề bên và để núm trái/phải + `aria-checked` mang trạng thái (không chỉ dựa vào màu).

Phương án thay thế “Logo gốc” (`#1C6CC6` / `#4A97E8`) không còn cần: `#2876D2` gần logo và đạt chữ trắng 4,55:1.

### Bảng cũ “Sáng hơn” (đã bị thay thế, giữ để tham chiếu)

Bản cũ dùng `--acc` `#3D8BE0` sáng / `#5AA2EE` tối với chữ trắng 3,52 / 2,68 và ghi chú “chấp nhận theo phán quyết”. Mục “chấp nhận dưới 4,5:1” đó **không còn hiệu lực**.

## 3. Kiến trúc cửa sổ (cần chủ sở hữu xác nhận, xem mục 10)

Một cửa sổ “VietKB” duy nhất, rộng **560 px cố định**, thanh công cụ 5 tab: **Chung, Phím tắt, Gõ tắt, Nâng cao, Giới thiệu**. Chiều cao đổi theo tab (như Cài đặt của macOS; không có dải trống), không bao giờ quá **560 px**. Thay cho cửa sổ phụ hiện nay (`show_aux` cho hotkeys 520, advanced 560, macros 720, about 560 và cửa sổ chính 480). Hộp thoại (Loại trừ ứng dụng, Thêm/Sửa từ tắt) là sheet gắn dưới thanh công cụ.

Chiều cao đo từ mockup (px, gồm cả thanh tiêu đề; macOS / Windows):

| Màn | macOS | Windows | Ghi chú |
|---|---|---|---|
| Chung | 412 | 420 | +80 khi banner Trợ năng (chỉ macOS) = 492; +36 khi gợi ý nhập bảo mật |
| Phím tắt | 322 | 330 | +27 khi có dòng lỗi xung đột |
| Gõ tắt | 560 | 560 | cố định; danh sách cuộn bên trong |
| Nâng cao | 446 | 560 | macOS không có mục Tương thích; Windows quá 560 nên nội dung cuộn |
| Giới thiệu (mới nhất) | 375 | 383 | |
| Giới thiệu (có cập nhật) | 489 | 497 | nhật ký thay đổi tối đa 104 px, cuộn bên trong |

Chiều cao thật phải do `fitWindowToContent` (đã có) tính, với trần 560; không đặt cứng các con số trên.

## 4. Ngân sách pixel (chống khoảng trống, chân trang lơ lửng, điều khiển quá to)

| Hạng mục | Số đo |
|---|---|
| Thanh tiêu đề | macOS 28, Windows 36 (như hiện nay: `MAC_CAPTION_PX`, `CAPTION_PX`) |
| Thanh công cụ | 52 (tab 64 × 44, padding 4, tab cách nhau 2) |
| **Chrome trên nội dung đầu tiên (tối đa)** | **80 macOS, 88 Windows**, rồi padding nội dung 12 |
| Lề ngang nội dung | 16; khoảng cách giữa nhóm 12; nhóm bo 8 |
| Hàng một dòng | **28** (công tắc 20 cao + 4 + 4) |
| Hàng có hint | **38**; hint 11 px, tối đa 1 dòng ở 560 px |
| Công tắc | 36 × 20 (núm 16) |
| Nút, pop-up, chia đoạn, ô nhập | cao **22** (chia đoạn: 18 + khung 2 + 2); nút trong chân trang 22 |
| Ô tick | 16 × 16, bo 4 |
| Keycap | cao 18 (mono 20); chữ 11 |
| Chân trang | **40**, ghim đáy (`flex: none`, phần tử cuối cùng); viền trên 1 px |
| Chiều rộng cửa sổ | 560 |
| Khoảng trống | 0: ở màn tự co (Chung, Phím tắt, Giới thiệu, Nâng cao macOS) tổng chiều cao nội dung = chiều cao cửa sổ, không có khoảng đệm đẩy chân trang; ở màn cố định 560 (Gõ tắt, trạng thái cuộn) vùng danh sách `flex: 1` lấp đầy, chân trang vẫn ngay dưới |

## 5. Ánh xạ token (`ui/src/styles/tokens.css`, `v4.css`, `kit.css`, `mac.css`)

Chỉ liệt kê token đổi giá trị hoặc mới (sáng / tối theo hai khối `:root` hiện có của `tokens.css`).

| Token hiện có | Sáng | Tối | Dùng cho |
|---|---|---|---|
| `--plane` | `#e8e8ec` | `#2d2d30` | thanh tiêu đề + thanh công cụ |
| `--surface` | `#f5f5f7` | `#1e1e1e` | nền cửa sổ/nội dung, chân trang |
| `--raised` | `#ffffff` | `#2c2c2e` | nhóm hàng |
| `--float` | `#ffffff` | `#3a3a3d` | toast, menu, sheet |
| `--line-1` | `rgba(0,0,0,.09)` | `rgba(255,255,255,.10)` | đường ngăn hàng, viền chân trang |
| `--line-2`, `--ctl-edge` | `rgba(0,0,0,.15)` | `rgba(255,255,255,.10)` | viền nút/ô nhập |
| `--ctl-bg` | `#ffffff` | `#4a4a4d` | nền nút, pop-up, ô nhập |
| `--t1` / `--t2` / `--t3` | `#1d1d1f` / `rgba(60,60,67,.66)` / `rgba(60,60,67,.42)` | `#f5f5f7` / `rgba(235,235,245,.62)` / `rgba(235,235,245,.34)` | chữ |
| `--tg-off` | `rgba(120,120,128,.30)` | `rgba(120,120,128,.50)` | rãnh công tắc tắt |
| `--hover` / `--hover-plane` | `rgba(0,0,0,.05)` / `rgba(0,0,0,.07)` | `rgba(255,255,255,.07)` / `rgba(255,255,255,.10)` | hover hàng/tab |
| `--acc`, `--acc-hover`, `--acc-text`, `--acc-tint`, `--on-acc`, `--focus` | xem mục 2 | xem mục 2 | |
| `--acc-tint-2` | `#CFE6FA` | `rgba(90,162,238,.28)` | hover trên hàng đang chọn |
| `--acc-line`, `--rail`, `--acc-a` | bỏ `--rail`; `--acc-line` = `--acc`; `--acc-a` = giá trị vòng focus ở mục 2 | | |
| `--warn-tint` / `--warn` | `#fdf0d5` / `#9a5b00` | `#4a3a14` / `#ffcc66` | banner Trợ năng, chip “Dễ trùng” |
| `--bad` / `--bad-tint` | `#d92d20` / `#fdecea` | `#ff6961` / `#4a2220` | lỗi, xoá |
| `--ui` | `-apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif` | | font 13 px, nhãn weight 400, nút mặc định 600 |
| `--app-w` | 560 | | |
| `--row-h` / `--short-row-h` | 28 / 38 | | hàng một dòng / hàng có hint |
| (mới) `--acc-press` | `#2F7DD0` | `#4C97E6` | |
| (mới) `--strip-h` | 52 | | thanh công cụ |

- `v4.css`: `.v4win` giữ; `.cap-bar` (Windows) giữ 36 px, thêm cụm phải: chấm cập nhật (nếu có), nút giao diện, `VI | EN` (`LangToggle`, cao 18), rồi thu nhỏ/đóng 44 × 36. `.v4sheet` bỏ viền trên; `.v4row` cao 28/38, đường ngăn thụt 12 px; `.v4foot` cao 40; `.v4lk` dùng `--acc-text`.
- `mac.css`: `.cap-bar.mac` giữ 28 px nhưng nay chứa đèn giao thông (trái) + tên tab đang chọn (giữa, 600) + cụm phải. Banner Trợ năng giữ 72 + 8 (`AX_BANNER_PX`/`BANNER_PX` = 80) và gợi ý nhập bảo mật 28 + 8 = 36; nền banner dùng `--warn-tint`.
- `kit.css`: `.tg` 36 × 20; `.btn` cao 22, bo 6, `.pri` dùng `--acc` / `--on-acc` đậm 600; `.sel` (pop-up) cao 22 với nắp 16 px `--acc`; `.seg` 22 (đoạn chọn nền `--raised`); `.cb` 16 × 16; `.k` (keycap) cao 18; `.input` cao 22; `.tab` hiện có thay bằng `Toolbar` mới.

## 6. Linh kiện

| Linh kiện | Việc cần làm |
|---|---|
| `Toggle` | giữ API (`role="switch"`, `aria-checked`, `disabled`); đổi kích thước/màu; vòng focus 3 px; hover `--acc-hover`, nhấn `--acc-press`; không bỏ núm |
| `Segmented` | giữ; đổi kích thước/màu; đoạn chọn nền `--raised` + bóng nhẹ |
| `Select` | giữ; thêm nắp `--acc` 16 px có biểu tượng lên-xuống trắng; menu mở: mục chọn nền `--acc` chữ trắng 600, dấu ✓ ở mục hiện tại; Esc đóng, mũi tên chọn |
| `Row` | giữ; chiều cao 28/38; hint dưới nhãn 11 px (hint ngắn như `hòa, thúy`, `6 ứng dụng`, phím tắt nằm cùng dòng, `nowrap`); hàng vô hiệu `opacity .42` + `aria-disabled` + không nhận chuột |
| `TitleBar` | restyle: macOS đèn giao thông + tên tab ở giữa; Windows logo Móc + “VietKB” + thu nhỏ/đóng 44 × 36; cụm phải dùng chung (chấm cập nhật, giao diện, `VI | EN`) |
| `Modal` (`<dialog class="sheetdlg">`) | restyle thành sheet: gắn ngay dưới thanh công cụ (`margin-top` = chrome 80/88), rộng 440, bo `0 0 12 12`, `::backdrop` `rgba(0,0,0,.28)`; Esc đóng; focus vào ô đầu; trả focus về nút mở |
| `Toast` | restyle: nằm trên chân trang (cách 10 px), thẻ `--float`, biểu tượng thông tin (`--acc`) hoặc cảnh báo (`--bad`), hành động liên kết `--acc-text` 600, nút đóng 22 |
| `Checkbox` | 16 × 16; chọn = nền `--acc`, dấu ✓ trắng; một phần cũng nền `--acc` |
| `Keycap` | cao 18; trigger gõ tắt dùng biến thể mono 20 (SF Mono / Cascadia) |
| `CaptureField` | cao 22; đang ghi viền `--acc` 2 px chữ `--acc-text`; lỗi viền `--bad` 2 px; “Dễ trùng” chip `--warn-tint` |
| `AccessibilityBanner`, `SecureInputHint` | giữ kích thước 72 + 8 và 28 + 8; chỉ đổi màu theo token |
| `AppRulesDialog` | thành sheet 440 trong `Modal`; hàng loại trừ 28/38 |
| **Mới: `Toolbar` / `ToolbarTab`** | xem mục 7 |

## 7. Thanh công cụ (linh kiện mới)

- `<nav role="tablist" aria-label="Cài đặt VietKB">`, cao 52, nền `--plane`, viền dưới 1 px `--line-1`, các tab ở giữa, cách nhau 2 px.
- Mỗi tab: `role="tab"`, rộng 64 × cao 44, bo 7, biểu tượng 20 px (nét 1.5) trên nhãn 11/13 px. Mặc định chữ `--t2`; đang chọn nền `--hover-plane`, chữ `--t1`, biểu tượng `--acc`; hover nền `--hover`.
- Tab: Chung (bánh răng), Phím tắt (bàn phím), Gõ tắt (tia chớp), Nâng cao (thanh trượt), Giới thiệu (chữ i). Nhãn dùng chuỗi hiện có (`win.hotkeys.title`, `win.macros.title`, `win.advanced.title`, `win.about.title`); cần thêm khoá `tab.general` = “Chung”.
- Bàn phím: một điểm dừng Tab cho thanh công cụ (roving `tabindex`); Trái/Phải chuyển và kích hoạt tab (xoay vòng); Home/End về tab đầu/cuối; `Ctrl/Cmd + 1..5` chuyển tab. Panel nội dung `role="tabpanel"` có `aria-labelledby`.

## 8. Bố cục từng màn

Chung: nội dung `padding: 12px 16px`, khoảng cách nhóm 12, chân trang 40 ghim đáy: trái tuỳ màn (xem dưới), phải là nút mặc định **Đóng**.

1. **Chung** (chân trang trái “Thoát hẳn”): [banner Trợ năng, macOS khi thiếu quyền] [gợi ý nhập bảo mật khi có]. Nhóm 1: Bật tiếng Việt (phím tắt cùng dòng) + công tắc; Kiểu gõ + pop-up (Telex, VNI). Nhóm 2: Đặt dấu kiểu mới (`hòa, thúy` Sitka 15) + công tắc; Kiểm tra chính tả + công tắc (mặc định tắt); Cho phép gõ tắt + nút “Bảng gõ tắt…” (chuyển sang tab Gõ tắt; vô hiệu khi gõ tắt tắt) + công tắc; Loại trừ ứng dụng (`6 ứng dụng`) + nút “Danh sách ứng dụng…” (mở sheet). Nhóm 3: “Mở cùng macOS” / “Khởi động cùng Windows” + hint; Chạy ẩn + hint.
2. **Phím tắt** (chân trái “Đặt lại tất cả”): dòng hướng dẫn 11 px; nhóm 4 hàng: Bật/tắt tiếng Việt, Đổi kiểu gõ, Khôi phục từ gõ sai, Mở cài đặt (ô ghi cao 22, rộng tối thiểu 104); nhóm “Kiểu chuyển bật/tắt” chia đoạn (Khi nhả phím | Khi nhấn phím). Trạng thái: đang ghi, chưa đặt (“Chưa đặt”), dễ trùng (chip), xung đột (viền `--bad` + dòng lỗi dưới nhóm, dùng lại `hotkeys.conflict.*`). Nhãn quá dài cắt “…”.
3. **Gõ tắt** (chân trái “Có N mục · cuộn để xem hết” hoặc “Chưa có mục nào”): thanh trên (12 / 16 / 8): ô tìm (flex 1) + “Nhập từ tệp…” + “Xuất ra tệp…” + “Thêm” (`pri`); bảng: tiêu đề 24 (Từ tắt, Nội dung), hàng 28 (ô tick 16, keycap mono, nội dung cắt “…” kèm thẻ “+1 dòng”, mũi tên). Chọn hàng: nền `--acc-tint`; thanh thao tác hàng loạt 30 (Đã chọn n trong N; Sửa, Xuất…, Xoá, Bỏ chọn `Esc`). Gõ tắt tắt: dải `--warn-tint` “Gõ tắt đang tắt, các mục chưa có tác dụng · Bật”. Trống: minh hoạ `dc → được`, tiêu đề, mô tả, 3 mục mẫu, “Thêm cả 3 mục”, “Nhập từ tệp…”. Sheet “Thêm từ tắt” / “Sửa từ tắt”: Từ tắt (ô nhập + gợi ý), Nội dung (vùng văn bản 60 px + bộ đếm ký tự), lỗi `macros.err.*` đỏ 11 px, nút Huỷ / Lưu (`pri`).
4. **Nâng cao** (chân trái “Đặt lại tất cả…”): nhóm Gõ (6 công tắc), Tương thích (chỉ Windows: Vẫn dùng gõ tắt khi đã tắt tiếng Việt [vô hiệu khi gõ tắt tắt], Viết hoa theo từ tắt, Gửi chữ qua Clipboard), Hệ thống (Âm thanh khi bật/tắt, Tự kiểm tra cập nhật, hàng “Đặt lại tất cả…” + nút). Tiêu đề nhóm 11 px/600 màu `--t2`. Nội dung cuộn khi vượt 560.
5. **Giới thiệu** (chân trái GitHub · Báo lỗi · Ko-fi): logo 40 + “VietKB” + phiên bản; lời giới thiệu; “Vì sao có VietKB” (gập được); đường kẻ; trạng thái cập nhật + “Kiểm tra cập nhật”; khi có bản mới: “Có bản cập nhật vX” (`--acc-text` 600) + “Cập nhật ngay” (`pri`) + khối nhật ký 104 px cuộn; đường kẻ; dòng riêng tư. Chấm `--acc` 7 px ở nút giao diện trên thanh tiêu đề báo có bản cập nhật.
6. **Menu khay**: hệ điều hành vẽ, không restyle; chỉ biểu tượng khay dùng V (tiếng Việt) / E (tiếng Anh) đúng bộ logo.

## 9. Bàn phím, focus, vùng bấm

- Thứ tự Tab: thanh công cụ (1 điểm dừng) → nội dung theo thứ tự nhìn thấy → chân trang trái → “Đóng”. Trong sheet: bẫy focus, Esc đóng, Enter = nút mặc định.
- `Cmd + W` (macOS) / `Ctrl + W` (Windows) đóng cửa sổ (ẩn, VietKB vẫn chạy); `Esc` chỉ đóng sheet/menu đang mở, không đóng cửa sổ.
- Focus: vòng 3 px `--focus` ngoài điều khiển, luôn thấy, không bị cắt (padding nhóm đủ chỗ).
- Vùng bấm tối thiểu 24 × 24 cho biểu tượng 22 (thêm `::before` mở rộng); nút, pop-up cao 22 (điều khiển desktop); nhãn hàng công tắc bấm được để đổi công tắc.
- Không chỉ dựa vào màu: công tắc có núm trái/phải, lỗi có chữ, “Dễ trùng” có chữ.

## 10. Câu hỏi còn lại cho chủ sở hữu

**Đã trả lời (chủ sở hữu, 2026-11-09): "Gộp, giữ 36 px".** Một cửa sổ 560 px với 5 tab thay các cửa sổ riêng; Gõ tắt hẹp lại 560 px (nội dung dài cắt "…"); thanh tiêu đề Windows giữ 36 px (chrome Windows = 88).

1. **Một cửa sổ 560 px với 5 tab** (chiều cao đổi theo tab) có được chấp nhận không? Nó thay các cửa sổ phụ hiện nay (480/520/560/720) và là phần việc lớn nhất của coder.
2. **Gõ tắt hẹp lại 560 px** (hiện 720): bảng vẫn đủ chỗ (nội dung dài cắt “…”), có chấp nhận không?
3. Thanh tiêu đề Windows giữ **36 px** như hiện nay (yêu cầu ghi 44 × 28, nhưng số thật trong code là 36), hay thu về 28 để chrome Windows = 80?

## 11. Kiểm tra nghiệm thu

QA chạy từng dòng; mỗi dòng áp cho cả macOS và Windows, sáng và tối, trừ khi ghi khác.

Cửa sổ và chrome
1. Cửa sổ rộng đúng 560 ở mọi tab; chiều cao đúng bảng mục 3 (± 2 px) và không vượt 560.
2. Chrome trên nội dung đầu tiên: macOS 28 + 52 = 80, Windows 36 + 52 = 88 (đo thước).
3. Chân trang cao 40, dính đáy cửa sổ ở mọi tab và mọi trạng thái; không dải trống giữa nội dung và chân trang ở màn tự co.
4. Không thanh cuộn ngang ở màn nào; thanh cuộn dọc chỉ ở Gõ tắt (bảng), Nâng cao khi quá 560, Giới thiệu khi nhật ký dài.
5. macOS: đèn giao thông thật, tên tab ở giữa, kéo cửa sổ bằng dải 28 px; Windows: logo Móc + “VietKB”, thu nhỏ/đóng 44 × 36, hover đóng đỏ.
6. Cụm phải thanh tiêu đề: nút giao diện đổi Sáng/Tối, `VI | EN` đổi ngôn ngữ ngay, chấm xanh chỉ hiện khi có bản cập nhật.
7. Đổi giao diện hệ thống Sáng/Tối thì cửa sổ đổi theo (`data-theme="system"`), không nháy trắng.

Màu nhấn
8. Nền công tắc bật, ô chia đoạn chính, nút “Đóng”, nắp pop-up, ô tick, mục menu chọn đúng `--acc` `#2876D2` (nền có chữ) và công tắc bật `#3D8BE0` sáng / `#5AA2EE` tối.
9. Chữ và biểu tượng trên mọi nền màu nhấn là `#FFFFFF`, đậm 600 (nút “Đóng”, “Cập nhật ngay”, “Thêm”, “Lưu”).
10. Hover nút chính `#2572CC`; nhấn giữ `#1F6FCA`; chữ trắng trên mọi trạng thái >= 4,5:1 (đo bằng công cụ, cả sáng và tối).
11. Liên kết/nhãn nhấn mạnh dùng `--acc-text` (`#1C6CC6` sáng, `#6AB0F2` tối), tương phản >= 4,5:1; vòng focus và viền chọn >= 3:1 so với màu kề.
12. Không còn tím Móc hay xanh cũ (`#5844c8`, `#8b79ee`) trên giao diện.

Điều khiển (mỗi điều khiển: mặc định, hover, nhấn, focus bàn phím, vô hiệu)
13. Công tắc 36 × 20; bật/tắt bằng chuột và Space; có `role="switch"` và `aria-checked` đúng; núm đổi vị trí trái/phải.
14. Pop-up cao 22 với nắp màu nhấn; mở bằng chuột/Space/Enter/mũi tên xuống; Esc đóng; ✓ ở mục hiện tại.
15. Chia đoạn: mũi tên trái/phải đổi đoạn; đoạn đang chọn nhìn rõ ở cả hai giao diện.
16. Ô tick 16 × 16: bỏ, chọn, một phần.
17. Vòng focus 3 px thấy rõ trên mọi điều khiển, không bị cắt ở mép nhóm hay cửa sổ.
18. Hàng vô hiệu mờ (`.42`), không nhận chuột, `aria-disabled="true"`, bàn phím bỏ qua.
19. Hàng một dòng cao 28, có hint cao 38, hint ngắn cùng dòng; không hàng nào cao hơn do chữ hoặc điều khiển.

Thanh công cụ
20. 5 tab đúng thứ tự: Chung, Phím tắt, Gõ tắt, Nâng cao, Giới thiệu; tab chọn nền xám nhạt, biểu tượng xanh.
21. Trái/Phải đổi tab và nội dung; Home/End; `Ctrl/Cmd + 1..5`; Tab vào thanh chỉ một lần rồi sang nội dung.
22. Đổi tab đổi chiều cao cửa sổ gọn (không nháy), không dải trống, nội dung tab trước không còn sót.
23. `Cmd/Ctrl + W` ẩn cửa sổ, VietKB vẫn chạy; mở lại từ khay mở đúng tab gần nhất (hoặc Chung).

Chung
24. Hàng: Bật tiếng Việt (phím tắt `⌃⇧` trên macOS, `Ctrl Shift` trên Windows), Kiểu gõ (Telex, VNI), Đặt dấu kiểu mới (`hòa, thúy` Sitka), Kiểm tra chính tả (mặc định tắt khi cài mới), Cho phép gõ tắt, Loại trừ ứng dụng (số ứng dụng đúng).
25. Bật/tắt từng công tắc lưu ngay (đóng mở lại vẫn giữ); lỗi lưu hiện toast đỏ trên chân trang.
26. “Bảng gõ tắt…” chuyển sang tab Gõ tắt; vô hiệu khi “Cho phép gõ tắt” tắt.
27. “Danh sách ứng dụng…” mở sheet Loại trừ ứng dụng; thêm, xoá, đổi kiểu gõ theo ứng dụng hoạt động; “Đóng” và Esc đóng sheet.
28. Hàng hệ thống: macOS “Mở cùng macOS” + hint “…đăng nhập macOS”, “Chạy ẩn” hint “…thanh menu”; Windows “Khởi động cùng Windows” + “…khay”.
29. macOS thiếu quyền: banner Trợ năng 72 + 8, cửa sổ cao thêm đúng 80; cấp quyền xong banner mất và cửa sổ co lại.
30. Gợi ý nhập bảo mật hiện (28 + 8) khi ứng dụng khác bật nhập bảo mật, mất khi hết.
31. “Thoát hẳn” (chân trang trái) thoát VietKB và tắt biểu tượng khay/thanh menu.

Phím tắt
32. 4 hàng đúng nhãn; ô ghi cao 22; bấm vào ô thì hiện “Nhấn tổ hợp phím…” viền xanh; Esc bỏ; Backspace xoá thành “Chưa đặt”.
33. Tổ hợp trùng phím khác: viền đỏ + dòng lỗi `hotkeys.conflict.*` đọc được; không lưu tổ hợp trùng.
34. Tổ hợp dễ trùng với ứng dụng khác: chip “Dễ trùng”, vẫn lưu được.
35. Phím bị hệ điều hành chiếm (`hk.reserved.*`): từ chối kèm lời giải thích.
36. “Kiểu chuyển bật/tắt” đổi và lưu; “Đặt lại tất cả” đưa mọi ô về mặc định và hiện toast.
37. Nhãn quá dài bị cắt “…” (kiểm bằng English và chữ lớn), không đẩy ô ghi ra ngoài.

Gõ tắt
38. Thanh trên: ô tìm co giãn, 3 nút đủ chỗ ở 560, không xuống dòng.
39. Danh sách: hàng 28, keycap mono, nội dung dài cắt “…”, thẻ “+1 dòng” cho nội dung nhiều dòng; cuộn mượt với 1000 mục.
40. Chọn một/nhiều hàng: nền `--acc-tint`, thanh thao tác hàng loạt (Sửa, Xuất…, Xoá, Bỏ chọn); Esc bỏ chọn; ô tick một phần khi chọn một số.
41. Xoá hiện toast “Đã xoá n mục gõ tắt” có “Hoàn tác”; hoàn tác khôi phục đúng.
42. Gõ tắt tắt: dải vàng “…chưa có tác dụng · Bật”; bấm “Bật” bật lại và dải mất.
43. Trống: minh hoạ, 3 mục mẫu bấm thêm từng mục, “Thêm cả 3 mục”, “Nhập từ tệp…”.
44. Sheet Thêm/Sửa: lỗi (`macros.err.*`: trống, có dấu cách, quá 32/500 ký tự, từ tắt có chữ số với Telex) đúng chữ và viền đỏ; bộ đếm ký tự đúng; Lưu thêm/cập nhật; Huỷ/Esc đóng không lưu; focus vào ô đầu và trả về nút “Thêm” khi đóng.
45. Nhập/xuất tệp: toast đúng (`macros.imported`, `macros.exported`, bỏ qua mục có chữ số).

Nâng cao
46. macOS có 2 nhóm (Gõ, Hệ thống) cao 446; Windows có 3 nhóm, vượt 560 thì cuộn trong vùng nội dung, chân trang vẫn ghim.
47. Mỗi công tắc (11 hàng trên Windows, 8 trên macOS) lưu được; hàng phụ thuộc gõ tắt vô hiệu khi gõ tắt tắt.
48. “Đặt lại tất cả…” hỏi xác nhận trước khi đặt lại và hiện toast `adv.resetDone`.

Giới thiệu
49. Phiên bản đúng, lời giới thiệu theo hệ điều hành, “Vì sao có VietKB” gập/mở được bằng chuột và bàn phím.
50. “Kiểm tra cập nhật”: các trạng thái đang kiểm tra, mới nhất, có bản mới, lỗi đều đúng chữ.
51. Có bản mới: “Có bản cập nhật vX” màu nhấn, “Cập nhật ngay” chính, nhật ký cuộn được bằng bàn phím (focus được), chấm xanh ở thanh tiêu đề.
52. Liên kết GitHub, Báo lỗi, Ko-fi mở trình duyệt mặc định.

Sheet, toast, trạng thái chung
53. Sheet gắn ngay dưới thanh công cụ (y = 80/88), rộng 440, nền sau mờ 28%; thanh tiêu đề vẫn kéo được.
54. Toast nằm ngay trên chân trang (cách 10 px), tự đóng sau thời gian đặt, có nút đóng; “Hoàn tác” bấm được; lỗi dùng `role="alert"`.
55. Chữ tiếng Việt có dấu hiển thị đủ (Ă Â Ê Ô Ơ Ư Đ, dấu hỏi/ngã) ở 100%, 125%, 150% DPI (Windows) và Retina (macOS); không cắt đỉnh dấu.
56. Bản English (`en.ts`) cũng vừa chiều rộng 560 ở mọi hàng (cắt “…” nếu cần), không đẩy điều khiển ra ngoài.

Hiệu năng và hồi quy
57. RAM tiến trình giao diện sau 30 s rảnh không tăng quá 5% so với bản hiện tại (`Get-Process` / Activity Monitor).
58. Mở cửa sổ <= 0,3 giây; đổi tab <= 100 ms.
59. Không `backdrop-filter`, `filter`, ảnh bitmap mới trong CSS phát hành (grep); chỉ `box-shadow`/màu đặc và SVG nội tuyến.
60. Gõ Telex và VNI thật (một vòng) sau khi đổi giao diện; trạng thái bật/tắt và kiểu gõ trên khay khớp với cửa sổ.
61. Chụp ảnh so sánh (sáng/tối × macOS/Windows × các khung trong `B-final.html`) lưu vào `design/_shots`, đối chiếu từng khung.

Tổng: **61 kiểm tra** (#1 đến #61).

## 12. Chỉnh sau phản hồi chủ sở hữu (2026-11-09)

- "state focus của input hơi bị đậm quá, nhìn hơi khó chịu": ô nhập, textarea và pop-up dùng viền 1 px `--focus` (đặc, >= 3:1) cộng quầng mờ 3 px `--focus-soft` (sáng `rgba(40,118,210,.28)`, tối `rgba(90,162,238,.36)`). Nút, tab, công tắc giữ vòng focus cũ (chỉ hiện khi dùng bàn phím).
- "padding dưới hơi ít, làm cho những dấu nặng hay chữ g chữ y bị khó đọc rõ": nút, pop-up, ô nhập, ô ghi phím tắt cao 24 px (thay 22); chữ trong ô nhập cao dòng 20 px, đệm dưới 1 px; textarea 64 px, đệm dưới 7 px. Hàng 28 px vẫn chứa được điều khiển 24 px. QA kiểm dấu nặng và chữ g/y/q không chạm mép dưới ở 100/125/150%.
- "hòa, thúy nên viết kiểu khác hoàn toàn ... vì giống về font weight": chữ ví dụ ở hàng "Đặt dấu kiểu mới" là nhãn nhỏ (UI font 12 px, `--t2`, nền `--well`, bo 4 px), không dùng Sitka nữa.
- Icon Chung: bánh răng 8 răng đồng tâm (12, 12); path cũ lệch tâm.
- Gõ tắt: nút "Nhập…" / "Xuất…" rút gọn để ô tìm kiếm đủ chỗ; "cuộn để xem hết" chỉ hiện khi danh sách tràn.
