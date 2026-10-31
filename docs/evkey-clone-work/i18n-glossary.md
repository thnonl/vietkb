# i18n glossary — Vietnamese → English

One English term per concept, same rule as `90-copy-voice.md`. Used by the site (`site/en/`)
and the app's English UI (27-mac-decisions M6). If a term changes, change it here first.
English voice: plain, short, no "we"; "you" only where it is the point; sentence case for labels.
US spelling ("recognize", "capitalization"). Contractions in messages and errors ("Couldn't save…", "can't"), not in labels. No em-dash inside sentences; use a full stop, comma or parentheses (window titles like "Shortcuts — VietKB" keep theirs). Hints under a label are third person ("Starts when…", "Switches…"). Ellipsis only on buttons and menu items that open a window or dialog ("Shortcuts…", "Open settings…").

## Product and platform

- bộ gõ → **input method** — product category in prose ("a Vietnamese input method"). Never "IME" in user copy.
- tiếng Việt (state) → **Vietnamese** — "Vietnamese on", "Vietnamese off", "Vietnamese on/off".
- Đã tắt tiếng Việt → **Vietnamese off** — state text and tray tooltip; never "English" (27-copy-decisions §B).
- khay hệ thống → **system tray** — Windows only; "tray" alone is fine on second mention.
- thanh menu → **menu bar** — macOS only.
- cửa sổ chính / cài đặt → **Settings** (window title "VietKB Settings"); "Open settings" as an action.
- Trợ năng → **Accessibility** — the macOS permission; "Allow Accessibility", "Accessibility access".
- Quyền riêng tư & Bảo mật → **Privacy & Security** — use Apple's exact English names for System Settings paths.
- nguồn nhập (macOS) → **input source** — Apple's term; "ABC" stays as is.

## Typing

- kiểu gõ → **Typing method** — the Telex / VNI setting. "Change typing method" for the hotkey (`cycleMethod`).
- Telex, VNI, Unicode → unchanged — never translated or upper-cased.
- bảng mã → **Encoding** — the output setting. Values: "Unicode", "Unicode (decomposed, NFD)" for Unicode tổ hợp; legacy names unchanged (TCVN3, VNI Windows, VISCII).
- đặt dấu kiểu mới / kiểu cũ → **New-style / Old-style tone marks** — example in the hint: hòa vs hoà.
- đặt dấu tự do → **Free tone placement** — tone key can come anywhere after the vowel.
- kiểm tra chính tả → **Spell check** — checks Vietnamese syllables, not a dictionary.
- khôi phục từ gõ sai → **Restore misspelled words** (setting); **Restore misspelled word** (hotkey, one word). Means: put back exactly the keys typed when a word isn't valid Vietnamese. Hint: "Puts back the keys you typed when a word isn't valid Vietnamese."
- viết hoa đầu câu → **Auto-capitalize sentences**.
- vùng gõ thô → **Raw text zones** — URLs/code left exactly as typed.
- âm báo / âm thanh khi bật/tắt → **On/off sound**. Not "Toggle sound" (reads as a command).
- kiểu chuyển bật/tắt → **Switch when** — options **Key released** (default) / **Key pressed**. Not "Toggle on" (reads as "turn on").

## Text expansion (gõ tắt)

- gõ tắt → **Text expansion** — the feature. Never "macro", "shortcut" or "auto-replace" (shortcut is reserved for keyboard shortcuts).
- từ tắt → **abbreviation** — one entry's trigger ("dc").
- phần thay thế → **expansion** — what the abbreviation becomes ("được").
- bảng gõ tắt → **Abbreviations** — the window/list; button "Abbreviations…".
- Cho phép gõ tắt → **Text expansion** (switch label) / "Turn on text expansion" (action).
- Vẫn dùng gõ tắt khi đã tắt tiếng Việt → **Expand even when Vietnamese is off**.
- Viết hoa theo từ tắt → **Match capitalization**.

## Apps, shortcuts, system

- loại trừ ứng dụng → **Excluded apps** — feature and list; button "Excluded apps…"; hotkey "Excluded apps on/off".
- phím tắt → **shortcut** / window **Shortcuts** — keyboard shortcuts only. "tổ hợp phím" → "key combination" only when the combination itself is meant.
- Chuyển bật / tắt (hotkey) → **Vietnamese on/off**.
- Khởi động cùng Windows → **Start with Windows**.
- Mở cùng macOS → **Start with macOS** — mirrors Windows' "Start with Windows" (owner rule M10: consistent with Windows).
- Chạy ẩn → **Start hidden**.
- Thoát hẳn / Thoát VietKB → **Quit** / **Quit VietKB**.
- Nâng cao → **Advanced**. Giới thiệu → **About**.
- Kiểm tra cập nhật → **Check for updates**; Cập nhật ngay → **Update now**; Mở trang tải về → **Open download page** (macOS, notify-only updates).
- Đặt lại tất cả → **Reset all**.
- Chưa đặt (no shortcut) → **Not set**.
- ký số → **code signing** (Windows); công chứng (Apple) → **notarized**. Keep the two distinct.
