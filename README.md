# VietKB

Bộ gõ tiếng Việt cho Windows, viết bằng Rust + Tauri 2. Nhẹ, chạy trong khay hệ thống, gõ Telex hoặc VNI.

- Trang giới thiệu: **https://thnonl.github.io/vietkb/**
- Tải bản mới nhất: **https://github.com/thnonl/vietkb/releases/latest**
- Ủng hộ: **https://ko-fi.com/N4X728S6TT** (buy me a coffee)

## Có gì

- Telex và VNI; bảng mã Unicode (NFC) và Unicode NFD.
- Kiểu gõ cũ/mới, đặt dấu tự do, kiểm tra chính tả, tự khôi phục từ gõ sai, viết hoa đầu câu.
- Gõ tắt (kể cả khi đã tắt tiếng Việt), bảng gõ tắt riêng, gõ tắt theo hoa/thường.
- Phím tắt trong hook (F1/F2, Ctrl+Shift, …), quy tắc theo từng ứng dụng, khay hệ thống V/E.
- Chạy ẩn, khởi động cùng Windows, kiểm tra cập nhật ngay trong app.

Không có: bảng mã cũ (TCVN3, VNI-Windows, VISCII…), VIQR, công cụ chuyển mã văn bản.

## Cài

Tải `VietKB_<version>_x64-setup.exe` ở mục Releases rồi chạy. Bản phát hành **chưa ký số**, Windows sẽ hỏi xác nhận.

Windows 10 1809+ và Windows 11, x64.

## Build từ mã nguồn

Cần Rust (MSVC), Node 20+, WebView2.

```powershell
npm --prefix ui install
cargo test --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-prod.ps1
```

Installer ra ở `target\release\bundle\nsis\`.

Bản dev chạy song song, cấu hình riêng: `scripts\build-dev.ps1` → `target-dev\` (`VietKB Dev`).

## Cấu trúc

| Thư mục | Nội dung |
|---|---|
| `crates/vkb-core` | engine gõ (thuần, không I/O) |
| `crates/vkb-windows` | hook bàn phím, gửi phím, tray backend |
| crates/vkb-config | schema `config.toml` |
| `crates/vkb-platform` | matcher phím tắt, hotkey thuần |
| `crates/vkb-app` | Tauri app, cửa sổ, cập nhật |
| `ui/` | giao diện Svelte 5 |
| `site/` | trang GitHub Pages |
| `docs/evkey-clone-work/` | charter, spec, hợp đồng, test, báo cáo |

## Ủng hộ

Nếu thấy hữu ích: **https://ko-fi.com/N4X728S6TT**

