# VietKB

Bộ gõ tiếng Việt cho Windows và macOS, viết bằng Rust + Tauri 2. Nhẹ, nằm trong khay hệ thống (Windows) hoặc trên thanh menu (macOS), gõ Telex hoặc VNI.

- Trang giới thiệu: **https://thnonl.github.io/vietkb/**
- Tải bản mới nhất: **https://github.com/thnonl/vietkb/releases/latest**
- Ủng hộ: **https://ko-fi.com/N4X728S6TT** (buy me a coffee)

## Có gì

- Telex và VNI; bảng mã Unicode (NFC) và Unicode NFD.
- Kiểu gõ cũ/mới, đặt dấu tự do, kiểm tra chính tả, tự khôi phục từ gõ sai, viết hoa đầu câu.
- Gõ tắt (kể cả khi đã tắt tiếng Việt), bảng gõ tắt riêng, gõ tắt theo hoa/thường.
- Phím tắt tự đặt (Windows: F1/F2, Ctrl+Shift, …; macOS: ⌃⇧, ⌃⇧Z, ⌃⌥S), quy tắc theo từng ứng dụng, biểu tượng V/E trong khay hoặc trên thanh menu.
- Chạy ẩn, Khởi động cùng Windows / Mở cùng macOS, kiểm tra cập nhật ngay trong app (macOS: báo bản mới, chưa tự cập nhật).

Không có: bảng mã cũ (TCVN3, VNI-Windows, VISCII…), VIQR, công cụ chuyển mã văn bản.

## Cài

**Windows** (Windows 10 1809+ và Windows 11, x64): tải `VietKB_<version>_x64-setup.exe` ở mục Releases rồi chạy. Bản phát hành **chưa ký số**, Windows sẽ hỏi xác nhận.

**macOS** (macOS 11 trở lên, Apple silicon và Intel):

1. Tải `VietKB_<version>_universal.dmg` ở mục Releases, mở ra, kéo `VietKB` vào `Applications`.
2. Lần mở đầu: bản Mac **chưa được Apple công chứng**. Bấm chuột phải vào `VietKB` → `Mở`. Nếu vẫn bị chặn: `Cài đặt hệ thống` → `Quyền riêng tư & Bảo mật` → `Vẫn mở`, hoặc chạy `xattr -dr com.apple.quarantine /Applications/VietKB.app`.
3. Cấp quyền `Trợ năng`: bấm `Cấp quyền…` trên thanh vàng trong cửa sổ cài đặt, rồi bật công tắc cạnh `VietKB` trong `Quyền riêng tư & Bảo mật` → `Trợ năng`.

Tắt bộ gõ khác (EVKey, OpenKey, bộ gõ tiếng Việt có sẵn của macOS) trước khi dùng.

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

