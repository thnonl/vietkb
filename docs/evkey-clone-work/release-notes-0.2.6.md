# VietKB 0.2.6

## Sửa lỗi

- Xóa một từ gõ sai (từ tiếng Anh, từ gõ nhầm phím) rồi gõ lại tiếng Việt không còn bị kẹt ở chữ thô. Trước đây, sau khi gõ một từ không phải tiếng Việt, cách ra rồi xóa ngược vào từ đó, từ gõ lại vẫn ra nguyên phím (`d9e6m` thay vì `đêm`), kể cả khi đã xóa hết. Chỉ cách ra gõ từ mới mới hết.
- Gõ thừa một phím làm từ sai chính tả rồi xóa phím đó, từ trở lại như cũ: `dduwowcjk` rồi xóa `k` ra `được`.
- Xóa chữ số gõ nhầm giữa từ rồi gõ tiếp không còn mất dấu.
- Sau khi xóa, dấu được đặt lại đúng chỗ (`hoafk` rồi xóa `k` ra `hòa`).
- Telex: gõ dấu nặng trước phụ âm cuối (`hojc`, `nhajt`) ra `học`, `nhặt` khi bật kiểm tra chính tả.
- Chữ số gõ liền trước chữ không còn chặn bỏ dấu: `1dduwowcj` ra `1được`, `3giowf` ra `3giờ`.

## Thay đổi nhỏ

- Phím xóa giờ đưa từ về đúng trạng thái như khi vừa gõ các phím còn lại. Ví dụ `test` xóa một lần ra `té`.

## Khác

- Thêm bộ kiểm thử tự động cho các thao tác sửa chữ hay gặp (gõ, xóa, gõ lại; gõ nhầm rồi sửa; số trước chữ) trên cả Telex và VNI.
