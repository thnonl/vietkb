# p1 — Mực tím

Mốc so sánh: mực tím đang dùng trong giao diện.

Bảng màu: mực `#1C1C1A` · giấy (ô sáng) `#F7F7F5` · nhấn sáng `#5844C8` · nhấn tối `#8B7BFF` · ô tối nền mực `#1C1C1A` với nét giấy `#F2F2F5` · tắt (E) xám `#6B6B70` / `#A8A8A8`.

| Cặp | Tỉ lệ |
|---|---|
| ink / tile sáng | 15.91 |
| accent / tile sáng | 6.37 |
| accent / ink | 2.50 |
| giấy / tile tối | 15.28 |
| accent tối / tile tối | 5.18 |
| accent tối / nét giấy | 2.95 |
| accent / taskbar sáng #F3F3F3 | 6.15 |
| accent tối / taskbar tối #202020 | 4.95 |
| UI: accent chữ trên #F7F7F5 (≥4.5) | 6.37 |
| UI: chữ trắng trên nút accent (≥4.5) | 6.83 |
| UI: accent tối / #202020 (≥3) | 4.95 |
| UI: accent tối / #272727 (≥3) | 4.53 |
| UI: chữ ink trên nút accent tối (≥4.5) | 5.18 |

Không đạt: accent / ink, accent tối / nét giấy.

Giao diện: Không cần đổi token giao diện (đây là màu hiện tại).

Ghi chú: mực tím hiện tại quá tối để tách khỏi nét mực (2.50:1) và #8B7BFF quá sáng cạnh nét giấy trên nền tối (2.95:1). Bản chỉnh cùng sắc độ để lọt dải 3:1 ở cả hai phía: #6D57D8 / #8A79E0.
