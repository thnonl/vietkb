# VietKB 0.2.4

## Mới

- Có tiếng báo ngắn khi bật và khi tắt tiếng Việt. Tiếng bật và tiếng tắt khác nhau, nghe là biết đang ở chế độ nào.
- Không muốn nghe thì tắt mục `Âm thanh khi bật/tắt` trong `Nâng cao`.

## Gõ đúng hơn

- Gõ ngay sau `/ . ? : \ @ = # _` vẫn ra tiếng Việt: `baajt/tawts` ra `bật/tắt`.
- Địa chỉ web, email và đường dẫn tệp như `C:\Users\nam` vẫn giữ nguyên.
- Xoá bớt một chữ rồi gõ dấu mới thì dấu đổi đúng: `toán`, bấm Backspace, gõ `f` ra `tòa`.
- Telex: gõ `z` để bỏ dấu, `toans` thêm `z` ra `toan`.
- VNI: `0` chỉ bỏ dấu, giữ nguyên â, ă, ơ, ư, đ. `ba65t0` ra `bât`.
- Dấu không hợp với từ thì phím đó ra chữ thường: `bật` gõ thêm `f` ra `bậtf`, không thành `bầt`. Áp dụng khi bật Kiểm tra chính tả.
- Chữ hoa gõ bằng Shift cho từng chữ giờ ra đúng: `VIEETJ` ra `VIỆT`, `DDUWOWCJ` ra `ĐƯỢC`.
- Bấm chuột sang chỗ khác là kết thúc từ đang gõ. Chữ gõ tiếp không bị nối vào từ trước.

## Sửa lỗi

- Notepad trên Windows 11 không còn làm sai chữ, như `bật/tắt` thành `bật/ttt`.
- Phím tắt `Ctrl+Shift+Z` (Khôi phục từ gõ sai) đã dùng được: trả lại đúng các phím vừa gõ.
- F1 và F2 chỉ bật, tắt tiếng Việt, không còn lọt sang ứng dụng. Trong VS Code, F1 không còn mở bảng lệnh.

## Còn lại

- Trong Notepad trên Windows 11, từ có nhiều chữ có dấu liền nhau như `được` thỉnh thoảng vẫn bị sai.
