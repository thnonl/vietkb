# p6 — Mực vàng kim

Mực đen ánh vàng kim — sang nhất và rủi ro nhất: vàng tối dễ thành màu bùn.

Bảng màu: mực `#1C1C1A` · giấy (ô sáng) `#F7F7F5` · nhấn sáng `#8A660B` · nhấn tối `#AE800E` · ô tối nền mực `#1C1C1A` với nét giấy `#F2F2F5` · tắt (E) xám `#6B6B70` / `#A8A8A8`.

| Cặp | Tỉ lệ |
|---|---|
| ink / tile sáng | 15.91 |
| accent / tile sáng | 4.91 |
| accent / ink | 3.24 |
| giấy / tile tối | 15.28 |
| accent tối / tile tối | 4.79 |
| accent tối / nét giấy | 3.19 |
| accent / taskbar sáng #F3F3F3 | 4.75 |
| accent tối / taskbar tối #202020 | 4.57 |
| UI: accent chữ trên #F7F7F5 (≥4.5) | 4.91 |
| UI: chữ trắng trên nút accent (≥4.5) | 5.27 |
| UI: accent tối / #202020 (≥3) | 4.57 |
| UI: accent tối / #272727 (≥3) | 4.19 |
| UI: chữ ink trên nút accent tối (≥4.5) | 4.79 |

Không đạt: không có.

Giao diện: Cần đổi token: `--acc`/`--focus`/`--rail` sáng = `#8A660B`, `--acc-text` sáng = `#8A660B`; tối: `--acc`/`--rail` = `#AE800E`, chữ trên nền accent tối = `#1C1C1A`; các tông nhạt (`--acc-tint`, `--acc-tint-2`, `--acc-line`) suy lại từ cùng sắc độ.

Đánh giá bằng mắt ở 16 px: số đo đạt, nhưng nét thanh #8A660B trên taskbar sáng đọc thành nâu ô-liu (màu bùn), không còn là vàng kim. Làm sáng nét lên (ví dụ #B8890F) thì cứu được logo nhưng không dùng được làm màu chữ UI trên giấy (dưới 4.5:1), nên bảng màu này không hợp làm accent của sản phẩm.
