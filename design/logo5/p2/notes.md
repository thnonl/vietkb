# p2 — Mực xanh ngọc

Màu nước Việt — ngọc bích, vịnh, ruộng bậc thang mùa nước.

Bảng màu: mực `#1C1C1A` · giấy (ô sáng) `#F7F7F5` · nhấn sáng `#0E7A6B` · nhấn tối `#119986` · ô tối nền mực `#1C1C1A` với nét giấy `#F2F2F5` · tắt (E) xám `#6B6B70` / `#A8A8A8`.

| Cặp | Tỉ lệ |
|---|---|
| ink / tile sáng | 15.91 |
| accent / tile sáng | 4.88 |
| accent / ink | 3.26 |
| giấy / tile tối | 15.28 |
| accent tối / tile tối | 4.82 |
| accent tối / nét giấy | 3.17 |
| accent / taskbar sáng #F3F3F3 | 4.71 |
| accent tối / taskbar tối #202020 | 4.60 |
| UI: accent chữ trên #F7F7F5 (≥4.5) | 4.88 |
| UI: chữ trắng trên nút accent (≥4.5) | 5.23 |
| UI: accent tối / #202020 (≥3) | 4.60 |
| UI: accent tối / #272727 (≥3) | 4.21 |
| UI: chữ ink trên nút accent tối (≥4.5) | 4.82 |

Không đạt: không có.

Giao diện: Cần đổi token: `--acc`/`--focus`/`--rail` sáng = `#0E7A6B`, `--acc-text` sáng = `#0E7A6B`; tối: `--acc`/`--rail` = `#119986`, chữ trên nền accent tối = `#1C1C1A`; các tông nhạt (`--acc-tint`, `--acc-tint-2`, `--acc-line`) suy lại từ cùng sắc độ.
