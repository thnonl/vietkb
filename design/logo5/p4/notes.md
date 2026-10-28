# p4 — Mực chàm

Chàm nhuộm vải vùng cao phía Bắc; xanh tin cậy, gần mực tím nhưng lạnh hơn.

Bảng màu: mực `#1C1C1A` · giấy (ô sáng) `#F7F7F5` · nhấn sáng `#4863D1` · nhấn tối `#6D83DA` · ô tối nền mực `#1C1C1A` với nét giấy `#F2F2F5` · tắt (E) xám `#6B6B70` / `#A8A8A8`.

| Cặp | Tỉ lệ |
|---|---|
| ink / tile sáng | 15.91 |
| accent / tile sáng | 4.92 |
| accent / ink | 3.24 |
| giấy / tile tối | 15.28 |
| accent tối / tile tối | 4.80 |
| accent tối / nét giấy | 3.18 |
| accent / taskbar sáng #F3F3F3 | 4.75 |
| accent tối / taskbar tối #202020 | 4.58 |
| UI: accent chữ trên #F7F7F5 (≥4.5) | 4.92 |
| UI: chữ trắng trên nút accent (≥4.5) | 5.28 |
| UI: accent tối / #202020 (≥3) | 4.58 |
| UI: accent tối / #272727 (≥3) | 4.20 |
| UI: chữ ink trên nút accent tối (≥4.5) | 4.80 |

Không đạt: không có.

Giao diện: Cần đổi token: `--acc`/`--focus`/`--rail` sáng = `#4863D1`, `--acc-text` sáng = `#4863D1`; tối: `--acc`/`--rail` = `#6D83DA`, chữ trên nền accent tối = `#1C1C1A`; các tông nhạt (`--acc-tint`, `--acc-tint-2`, `--acc-line`) suy lại từ cùng sắc độ.
