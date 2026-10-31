# VietKB 0.2.7

## Mới

- **VietKB cho macOS.** Bản đầu tiên cho Mac, một tệp `.dmg` dùng cho cả máy Apple silicon lẫn Intel, cần macOS 11 trở lên. Cách cài và cấp quyền xem mục *Cài trên macOS* bên dưới.
- **Giao diện tiếng Anh.** Mọi cửa sổ, thông báo và menu có cả tiếng Việt lẫn tiếng Anh. Nút `VI | EN` nằm cạnh nút Sáng/Tối, bấm là đổi ngay, không cần mở lại.
- **Giao diện theo hệ thống.** Mặc định VietKB sáng hay tối theo Windows/macOS và tự đổi khi hệ thống đổi. Bấm nút Sáng/Tối để chọn hẳn một kiểu; lựa chọn được lưu lại. `Đặt lại tất cả` đưa về theo hệ thống. Ai đang dùng nền tối ở bản cũ thì vẫn giữ nền tối.

## Sửa lỗi

- Đặt phím tắt mới không còn kích hoạt phím tắt cũ trong lúc bấm (ví dụ đặt lại `Ctrl+Shift+Z` không còn khôi phục từ).
- Bật/tắt tiếng Việt bằng phím tắt đúng lúc đang đổi cài đặt không còn làm mất một trong hai thay đổi.
- Mở cửa sổ con từ một cửa sổ con khác (ví dụ `Giới thiệu` từ `Nâng cao`) giờ nằm đúng trên cửa sổ đã mở nó.

## Thay đổi nhỏ

- Cửa sổ cài đặt mở giữa vùng làm việc của màn hình đang dùng, không đè lên thanh tác vụ.
- Dòng chú thích ở biểu tượng khay ngắn lại: `VietKB — đang gõ tiếng Việt` / `đã tắt tiếng Việt`.
- `Nâng cao` có thêm liên kết mở thư mục cấu hình.

## Cài trên macOS

1. Tải `VietKB_0.2.7_universal.dmg`, mở ra và kéo `VietKB` vào `Applications`.
2. Lần mở đầu, bản này chưa được Apple công chứng nên macOS sẽ chặn:
   - macOS 14 trở về trước: bấm chuột phải vào `VietKB` trong `Applications`, chọn `Mở`, rồi `Mở` lần nữa.
   - macOS 15 trở lên: mở `VietKB` một lần, rồi vào `Cài đặt hệ thống` > `Quyền riêng tư & Bảo mật`, bấm `Vẫn mở`.
   - Hoặc chạy trong Terminal: `xattr -dr com.apple.quarantine /Applications/VietKB.app`
3. Cấp quyền `Trợ năng` và `Giám sát đầu vào` khi VietKB hỏi (thanh vàng trong cửa sổ cài đặt có nút mở đúng chỗ). VietKB tự bắt đầu gõ khi đủ quyền, không cần mở lại.

Tắt các bộ gõ tiếng Việt khác (EVKey, OpenKey, bộ gõ Telex của macOS) trước khi dùng.

Trên Mac:

- Phím tắt mặc định: `⌃⇧` bật/tắt tiếng Việt, `⌃⇧Z` khôi phục từ gõ sai, `⌃⌥S` mở cài đặt. F1, F2, F8, F9, F11 không còn là mặc định vì là phím độ sáng, âm lượng và Hiện màn hình nền.
- Biểu tượng `V`/`E` trên thanh menu; khi cửa sổ cài đặt đang mở thì có thêm biểu tượng trên Dock.
- Ô mật khẩu và ô nhập an toàn giữ nguyên phím. Nếu một ứng dụng khác bật nhập an toàn, VietKB báo trong cửa sổ cài đặt.
- Cập nhật: VietKB báo khi có bản mới và mở trang tải về; chưa tự cập nhật.
- Chỉ có bảng mã Unicode.
