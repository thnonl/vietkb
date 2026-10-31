# VietKB

Bộ gõ tiếng Việt cho Windows và macOS, viết bằng Rust + Tauri 2. Nhẹ, nằm trong khay hệ thống (Windows) hoặc trên thanh menu (macOS), gõ Telex hoặc VNI.

- Trang giới thiệu: **https://thnonl.github.io/vietkb/**
- Tải bản mới nhất: **https://github.com/thnonl/vietkb/releases/latest**
- Ủng hộ: **https://ko-fi.com/N4X728S6TT** (buy me a coffee)

## Có gì

- Telex và VNI; chữ ra Unicode. Bản Windows chọn được thêm Unicode NFD.
- Kiểu gõ cũ/mới, đặt dấu tự do, kiểm tra chính tả, tự khôi phục từ gõ sai, viết hoa đầu câu. Địa chỉ web, email và đường dẫn tệp tự giữ nguyên.
- Gõ tắt (kể cả khi đã tắt tiếng Việt), bảng gõ tắt riêng, gõ tắt theo hoa/thường.
- Phím tắt mặc định: Windows `Ctrl+Shift`, `Ctrl+Shift+Z`, `Ctrl+Alt+S`, F1/F2/F8/F9/F11; macOS `⌃⇧`, `⌃⇧Z`, `⌃⌥S` (các phím F để trống). Cửa sổ `Phím tắt…` đổi được bốn việc: bật/tắt tiếng Việt, đổi kiểu gõ, khôi phục từ gõ sai, mở cài đặt.
- Quy tắc theo từng ứng dụng.
- Biểu tượng V/E trong khay (Windows) hoặc trên thanh menu (macOS): nhấp trái bật/tắt tiếng Việt, nhấp phải (Mac: cả Control-click) mở menu. Trên Mac, khi đang mở cửa sổ thì có thêm biểu tượng trên Dock.
- Giao diện tiếng Việt và tiếng Anh: nút `VI | EN` và nút Sáng/Tối nằm trên thanh tiêu đề. Mặc định sáng/tối theo hệ thống; bấm Sáng/Tối thì lựa chọn được lưu.
- Chạy ẩn, `Khởi động cùng Windows` / `Mở cùng macOS`, kiểm tra cập nhật ngay trong app. Có bản mới thì biểu tượng và thanh tiêu đề hiện dấu báo (macOS: báo bản mới và mở trang tải về, chưa tự cập nhật).

Không có: bảng mã cũ (TCVN3, VNI-Windows, VISCII…), VIQR, công cụ chuyển mã văn bản.

## Cài

**Windows** (Windows 10 1809+ và Windows 11, x64): tải `VietKB_<version>_x64-setup.exe` ở mục Releases rồi chạy. Bản phát hành **chưa ký số**, Windows sẽ hỏi xác nhận.

**macOS** (macOS 11 trở lên, Apple silicon và Intel):

1. Tải `VietKB_<version>_universal.dmg` ở mục Releases, mở ra, kéo `VietKB` vào `Applications`.
2. Lần mở đầu: bản Mac **chưa được Apple công chứng**. Bấm chuột phải vào `VietKB` → `Mở`. Nếu vẫn bị chặn: `Cài đặt hệ thống` → `Quyền riêng tư & Bảo mật` → `Vẫn mở`, hoặc chạy `xattr -dr com.apple.quarantine /Applications/VietKB.app`.
3. Cấp quyền `Trợ năng` và `Giám sát đầu vào`: bấm `Cấp quyền…` trên thanh vàng trong cửa sổ cài đặt, rồi trong `Quyền riêng tư & Bảo mật` bật công tắc cạnh `VietKB` ở cả `Trợ năng` lẫn `Giám sát đầu vào`. Đủ quyền là gõ được ngay, không cần mở lại.

Tắt bộ gõ khác (EVKey, OpenKey, bộ gõ tiếng Việt có sẵn của macOS) trước khi dùng.

## Build từ mã nguồn

**Windows**: cần Rust (MSVC), Node 20+, WebView2.

```powershell
npm --prefix ui install
cargo test --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-prod.ps1
```

Installer ra ở `target\release\bundle\nsis\`.

Bản dev chạy song song, cấu hình riêng: `scripts\build-dev.ps1` → `target-dev\` (`VietKB Dev`).

**macOS**: cần Xcode Command Line Tools, Rust với hai target `aarch64-apple-darwin` và `x86_64-apple-darwin`, Node 20+.

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm --prefix ui ci
cargo test --workspace
cd crates/vkb-app && ../../ui/node_modules/.bin/tauri build --target universal-apple-darwin --features custom-protocol
```

Tệp `.dmg` ra ở `target/universal-apple-darwin/release/bundle/dmg/`. Bản build ký ad-hoc (`tauri.macos.conf.json`), chưa được Apple công chứng.

## Cấu trúc

| Thư mục | Nội dung |
|---|---|
| `crates/vkb-core` | engine gõ (thuần, không I/O) |
| `crates/vkb-windows` | Windows: hook bàn phím, gửi phím, phím tắt, khởi động cùng Windows |
| `crates/vkb-macos` | macOS: `CGEventTap`, gửi phím, phím tắt, quyền Trợ năng, LaunchAgent cho Mở cùng macOS |
| `crates/vkb-config` | schema `config.toml` |
| `crates/vkb-platform` | matcher phím tắt, hotkey thuần |
| `crates/vkb-app` | Tauri app, cửa sổ, khay / thanh menu, cập nhật |
| `crates/vkb-cli` | công cụ dòng lệnh: `simulate`, `bench` |
| `ui/` | giao diện Svelte 5 |
| `site/` | trang GitHub Pages |
| `docs/evkey-clone-work/` | charter, spec, hợp đồng, test, báo cáo |

## Ủng hộ

Nếu thấy hữu ích: **https://ko-fi.com/N4X728S6TT**

