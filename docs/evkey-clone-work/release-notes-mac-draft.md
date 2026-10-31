# VietKB cho macOS (bản nháp)

Bản đầu tiên cho Mac. Một tệp `.dmg` dùng được cho cả máy Apple silicon (M1 trở lên) lẫn máy Intel. Cần macOS 11 trở lên.

## Cài đặt

1. Tải `VietKB_<phiên bản>_universal.dmg` ở mục Assets của bản phát hành.
2. Mở tệp `.dmg`, kéo `VietKB` vào thư mục `Applications`.
3. Mở `VietKB` từ `Applications`. Biểu tượng `V` hiện trên thanh menu. Khi cửa sổ cài đặt đang mở, VietKB có thêm biểu tượng trên Dock; đóng cửa sổ thì chỉ còn biểu tượng trên thanh menu.

## Lần mở đầu: Gatekeeper

Bản này chưa được Apple công chứng, nên macOS có thể chặn lần mở đầu. Làm một trong hai cách:

- macOS 14 trở về trước: trong `Applications`, bấm chuột phải vào `VietKB`, chọn `Mở`, rồi chọn `Mở` lần nữa.
- macOS 15 trở lên: mở `VietKB` một lần (macOS sẽ chặn), rồi vào `Cài đặt hệ thống` > `Quyền riêng tư & Bảo mật`, kéo xuống và bấm `Vẫn mở`.
- Hoặc chạy trong Terminal:

```
xattr -dr com.apple.quarantine /Applications/VietKB.app
```

Chỉ cần làm một lần cho mỗi bản cài.

## Cấp quyền Trợ năng

VietKB cần quyền Trợ năng để đọc phím và gõ chữ có dấu.

1. Cửa sổ cài đặt hiện thanh vàng `Cần quyền Trợ năng`. Bấm `Cấp quyền…`.
2. macOS hỏi. Chọn `Mở Cài đặt hệ thống`.
3. Trong `Quyền riêng tư & Bảo mật` > `Trợ năng`, bật công tắc cạnh `VietKB`.

Thanh vàng tự mất khi quyền đã có. Nếu công tắc đã bật mà vẫn báo thiếu quyền (thường sau khi cài bản mới đè lên bản cũ), tắt rồi bật lại công tắc, hoặc xoá dòng `VietKB` bằng nút `−` rồi mở lại VietKB.

## Phím tắt mặc định

| Việc | Phím |
|---|---|
| Bật/tắt tiếng Việt | ⌃⇧ (Control + Shift) |
| Khôi phục từ gõ sai | ⌃⇧Z |
| Mở cài đặt | ⌃⌥S |

Các phím F1, F2, F8, F9, F11 không còn là mặc định trên Mac vì đã là phím sáng, âm lượng và Hiện màn hình nền. Muốn dùng vẫn đặt được trong `Phím tắt…`.

## Khác với bản Windows

- Biểu tượng trên thanh menu: bấm để mở menu. `V` là đang gõ tiếng Việt, `E` là đã tắt.
- Cửa sổ dùng thanh tiêu đề của macOS và theo giao diện sáng/tối của hệ thống.
- `Mở cùng macOS` thay cho `Khởi động cùng Windows`.
- Không có tuỳ chọn gửi chữ qua Clipboard.
- Cập nhật: VietKB báo khi có bản mới và mở trang tải về. Tải `.dmg` mới và kéo đè vào `Applications`. Chưa tự cập nhật.

## Giới hạn đã biết

- Chưa được Apple công chứng (xem Gatekeeper ở trên).
- Ô mật khẩu và các ô nhập an toàn: VietKB để nguyên phím, không gõ tiếng Việt.
- Nếu đang chạy bộ gõ khác (EVKey, OpenKey, bộ gõ Telex của macOS), tắt chúng trước. Hai bộ gõ cùng chạy sẽ gõ ra chữ lỗi.
- Chỉ hỗ trợ bảng mã Unicode.
