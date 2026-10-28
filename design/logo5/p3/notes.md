# p3 — Mực đỏ son

Son mài, câu đối, dấu đỏ trên tranh in — Tết và chữ viết tay.

Bảng màu: mực `#1C1C1A` · giấy (ô sáng) `#F7F7F5` · nhấn sáng `#CD2D21` · nhấn tối `#E35C52` · ô tối nền mực `#1C1C1A` với nét giấy `#F2F2F5` · tắt (E) xám `#6B6B70` / `#A8A8A8`.

| Cặp | Tỉ lệ |
|---|---|
| ink / tile sáng | 15.91 |
| accent / tile sáng | 4.90 |
| accent / ink | 3.25 |
| giấy / tile tối | 15.28 |
| accent tối / tile tối | 4.81 |
| accent tối / nét giấy | 3.18 |
| accent / taskbar sáng #F3F3F3 | 4.74 |
| accent tối / taskbar tối #202020 | 4.59 |
| UI: accent chữ trên #F7F7F5 (≥4.5) | 4.90 |
| UI: chữ trắng trên nút accent (≥4.5) | 5.26 |
| UI: accent tối / #202020 (≥3) | 4.59 |
| UI: accent tối / #272727 (≥3) | 4.21 |
| UI: chữ ink trên nút accent tối (≥4.5) | 4.81 |

Không đạt: không có.

Giao diện: Cần đổi token: `--acc`/`--focus`/`--rail` sáng = `#CD2D21`, `--acc-text` sáng = `#CD2D21`; tối: `--acc`/`--rail` = `#E35C52`, chữ trên nền accent tối = `#1C1C1A`; các tông nhạt (`--acc-tint`, `--acc-tint-2`, `--acc-line`) suy lại từ cùng sắc độ.
