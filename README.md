# VietKB

VietKB là bộ gõ tiếng Việt nhẹ, tập trung vào trải nghiệm dùng hằng ngày.

## Tính năng

- Kiểu gõ: Telex, VNI, VIQR.
- Bảng mã: Unicode.
- Bật/tắt nhanh bằng phím tắt cấu hình được.
- GUI Windows gọn, có tray icon + menu chuyển nhanh kiểu gõ.
- Engine Rust nhỏ, nhanh, dễ đóng gói cho nhiều nền tảng.

## Tải và cài đặt

### Windows

1. Mở PowerShell tại thư mục dự án.
2. Chạy:

```powershell
.\scripts\install-windows.ps1
```

Ứng dụng được cài vào `%LOCALAPPDATA%\VietKB` và có shortcut trong Start Menu.

### Linux

```sh
sh scripts/install-linux.sh
```

Script cài `vietkb-gui` vào `$HOME/.local/bin`.

### macOS

```sh
sh scripts/install-macos.sh
```

Script cài `vietkb-gui` vào `$HOME/.local/bin`.

## Build riêng từng nền tảng

Output tách riêng theo platform:

- Windows: `target/windows`, `dist/windows`
- Linux: `target/linux`, `dist/linux`
- macOS: `target/macos`, `dist/macos`

Lệnh build:

```powershell
.\scripts\build-windows.ps1
```

```sh
sh scripts/build-linux.sh
sh scripts/build-macos.sh
```

## Dùng trên Windows

- Mở `VietKB` từ Start Menu hoặc chạy `vietkb-gui.exe`.
- Chọn kiểu gõ: Telex, VNI, VIQR.
- Sửa phím bật/tắt tại ô `Phím bật/tắt`.
- Chuột phải tray icon để bật/tắt hoặc đổi kiểu gõ nhanh.
- Tray tooltip hiển thị `V` khi bật tiếng Việt, `E` khi tắt.

## Ghi chú

- GUI hiện hỗ trợ Windows.
- Các tuỳ chọn chưa có backend ổn định không hiển thị trong GUI.
- Linux/macOS build ra `vietkb-gui`, nhưng backend GUI/input hook chưa được triển khai.

## License

MIT
