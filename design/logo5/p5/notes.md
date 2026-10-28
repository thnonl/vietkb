# p5 — Mực nâu đất

Đất nung, ngói đỏ — ấm, trầm, không ồn.

Bảng màu: mực `#1C1C1A` · giấy (ô sáng) `#F7F7F5` · nhấn sáng `#AE502E` · nhấn tối `#CF6C49` · ô tối nền mực `#1C1C1A` với nét giấy `#F2F2F5` · tắt (E) xám `#6B6B70` / `#A8A8A8`.

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

Giao diện: Cần đổi token: `--acc`/`--focus`/`--rail` sáng = `#AE502E`, `--acc-text` sáng = `#AE502E`; tối: `--acc`/`--rail` = `#CF6C49`, chữ trên nền accent tối = `#1C1C1A`; các tông nhạt (`--acc-tint`, `--acc-tint-2`, `--acc-line`) suy lại từ cùng sắc độ.
