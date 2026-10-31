# English UI strings for the copywriter

Generated from `ui/scripts/strings.py` (edit the script and rerun it, not this file). Keys are stable; `{name}` placeholders must be kept. Vietnamese is the source and does not change. English is a draft.

## Shared controls

Buttons and words reused across windows.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `common.close` | Đóng | Close |
| `common.cancel` | Huỷ | Cancel |
| `common.save` | Lưu | Save |
| `common.delete` | Xoá | Delete |
| `common.edit` | Sửa | Edit |
| `common.add` | Thêm | Add |
| `common.loading` | Đang tải… | Loading… |
| `common.undo` | Hoàn tác | Undo |
| `common.unknownError` | Lỗi không xác định | Unknown error |

## Window titles

In-page window titles (caption bar on Windows). The product name VietKB is never translated.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `win.macros.title` | Bảng gõ tắt | Abbreviations |
| `win.hotkeys.title` | Phím tắt | Shortcuts |
| `win.advanced.title` | Nâng cao | Advanced |
| `win.about.title` | Giới thiệu | About |

## Title bar and language toggle

Windows caption bar buttons; the VI | EN toggle also appears at the top right of every window on macOS.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `titlebar.update.aria` | Có bản cập nhật — mở Giới thiệu | Update available — open About |
| `titlebar.update.title` | Có bản cập nhật v{version} — mở Giới thiệu | Update v{version} available — open About |
| `titlebar.theme.light` | Sáng | Light |
| `titlebar.theme.dark` | Tối | Dark |
| `titlebar.theme.aria` | Giao diện: {theme}. Bấm để đổi | Appearance: {theme}. Click to change. |
| `titlebar.theme.title` | Giao diện: {theme} — bấm để đổi | Appearance: {theme}. Click to change. |
| `titlebar.minimize` | Thu nhỏ | Minimize |
| `titlebar.close` | Đóng, VietKB vẫn chạy | Close (VietKB keeps running) |
| `lang.group` | Ngôn ngữ | Language |
| `lang.vi.short` | VI | VI |
| `lang.en.short` | EN | EN |
| `lang.vi.name` | Tiếng Việt | Tiếng Việt |
| `lang.en.name` | English | English |

## Toasts and banners

Transient messages at the bottom of a window.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `toast.close` | Đóng thông báo | Dismiss |
| `toast.loadConfigFailed` | Không đọc được cấu hình: {error} | Couldn't read settings: {error} |
| `toast.saveFailed` | Không lưu được thay đổi: {error} | Couldn't save the change: {error} |
| `toast.previewReset` | Chế độ xem trước: không có gì để đặt lại. | Preview mode: nothing to reset. |
| `toast.resetDone` | Đã đặt lại mục này về mặc định. | Section reset to defaults. |
| `toast.resetFailed` | Không đặt lại được: {error} | Couldn't reset: {error} |

## Accessibility banner (macOS)

Warning band at the top of the main window until the user grants the Accessibility permission in System Settings.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `ax.title` | Cần quyền Trợ năng | Allow Accessibility |
| `ax.body` | Chưa cấp quyền thì VietKB chưa gõ được tiếng Việt. | VietKB can't type Vietnamese until you allow it. |
| `ax.grant` | Cấp quyền… | Allow access… |

## Main window

The small settings panel opened from the tray / menu bar. Short labels, one line each; keep English about as long as the Vietnamese.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `quick.enable` | Bật tiếng Việt | Vietnamese on/off |
| `quick.method` | Kiểu gõ | Typing method |
| `quick.modernTone` | Đặt dấu kiểu mới | New-style tone marks |
| `quick.modernTone.example` | hòa, thúy | hòa, thúy |
| `quick.spellCheck` | Kiểm tra chính tả | Spell check |
| `quick.macroAllow` | Cho phép gõ tắt | Text expansion |
| `quick.macroPanel` | Bảng gõ tắt… | Abbreviations… |
| `quick.appExclude` | Loại trừ ứng dụng | Excluded apps |
| `quick.appCount.one` | ứng dụng | app |
| `quick.appCount.other` | ứng dụng | apps |
| `quick.appList` | Danh sách ứng dụng… | Excluded apps… |
| `quick.autostart.win` | Khởi động cùng Windows | Start with Windows |
| `quick.autostart.mac` | Mở cùng macOS | Open at login |
| `quick.autostart.hint.win` | Chạy ngay khi đăng nhập Windows | Starts when you sign in to Windows |
| `quick.autostart.hint.mac` | Chạy ngay khi đăng nhập macOS | Starts when you log in to macOS |
| `quick.autostart.fail.win` | Không đổi được khởi động cùng Windows | Couldn't change Start with Windows |
| `quick.autostart.fail.mac` | Không đổi được mở cùng macOS | Couldn't change Open at login |
| `quick.startHidden` | Chạy ẩn | Start hidden |
| `quick.startHidden.hint.win` | Mở lên không hiện cửa sổ, chỉ nằm trong khay | Starts with no window, just the tray icon |
| `quick.startHidden.hint.mac` | Mở lên không hiện cửa sổ, chỉ nằm trên thanh menu | Starts with no window, just the menu bar icon |
| `quick.hotkeys` | Phím tắt… | Shortcuts… |
| `quick.advanced` | Nâng cao… | Advanced… |
| `quick.quit` | Thoát hẳn | Quit |
| `quick.quit.title.win` | Thoát hẳn, tắt luôn biểu tượng trong khay | Quit and remove the tray icon |
| `quick.quit.title.mac` | Thoát hẳn, tắt luôn biểu tượng trên thanh menu | Quit and remove the menu bar icon |
| `quick.rulesLoadFailed` | Không đọc được quy tắc | Couldn't load excluded apps |

## App exclusion dialog

Dialog listing apps where Vietnamese input is switched off (or overridden) per app.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `rules.title` | Loại trừ ứng dụng | Excluded apps |
| `rules.pickTitle` | Chọn ứng dụng đang chạy | Choose a running app |
| `rules.auto` | Loại trừ ứng dụng tự động | Remember per app |
| `rules.auto.note` | Nhớ trạng thái bật/tắt riêng cho từng ứng dụng | Remembers Vietnamese on/off for each app |
| `rules.empty` | Chưa có quy tắc nào. Thêm ứng dụng bạn chỉ dùng tiếng Anh. | No excluded apps yet. Add apps where you only type English. |
| `rules.enableIn` | Bật tiếng Việt trong {app} | Vietnamese in {app} |
| `rules.delete` | Xoá quy tắc {app} | Remove {app} |
| `rules.delete.title` | Xoá quy tắc | Remove |
| `rules.method` | Kiểu gõ | Typing method |
| `rules.methodFor` | Kiểu gõ cho {app} | Typing method for {app} |
| `rules.methodDefault` | Mặc định | Default |
| `rules.search` | Tìm ứng dụng… | Search apps… |
| `rules.searchAria` | Tìm ứng dụng | Search apps |
| `rules.searching` | Đang tìm ứng dụng đang mở… | Looking for running apps… |
| `rules.noneFound` | Không thấy ứng dụng nào. Mở ứng dụng rồi quay lại. | No apps found. Open the app, then come back. |
| `rules.added` | Đã thêm | Added |
| `rules.addedTitle` | {app} đã có trong danh sách loại trừ, đang được tô sáng. | {app} is already in the list (highlighted). |
| `rules.addTitle` | Thêm {app} | Add {app} |
| `rules.pickButton` | Chọn ứng dụng | Choose app |
| `rules.back` | ‹ Danh sách quy tắc | ‹ Excluded apps |
| `rules.saveFailed` | Không lưu được quy tắc | Couldn't save the setting |
| `rules.overrideFailed` | Không lưu được ghi đè | Couldn't save the app setting |
| `rules.deleteFailed` | Không xoá được quy tắc | Couldn't remove the app |
| `rules.addFailed` | Không thêm được ứng dụng | Couldn't add the app |

## Hotkeys window

Lets the user record keyboard shortcuts. Key names (Esc, Backspace) appear on keycaps.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `hotkeys.intro.a` | Bấm vào ô, nhấn tổ hợp mới. | Click a field, then press the keys. |
| `hotkeys.intro.esc` | để bỏ, | to cancel, |
| `hotkeys.intro.del` | để xoá phím tắt. | to clear. |
| `hotkeys.row.toggle` | Bật/tắt tiếng Việt | Vietnamese on/off |
| `hotkeys.row.cycleMethod` | Đổi kiểu gõ | Change typing method |
| `hotkeys.row.restoreWord` | Khôi phục từ gõ sai | Restore misspelled word |
| `hotkeys.row.openSettings` | Mở cài đặt | Open settings |
| `hotkeys.mode` | Kiểu chuyển bật/tắt | Switch when |
| `hotkeys.mode.release` | Khi nhả phím | Key released |
| `hotkeys.mode.press` | Khi nhấn phím | Key pressed |
| `hotkeys.resetAll` | Đặt lại tất cả | Reset all |
| `hotkeys.resetDone` | Đã đặt lại tất cả phím tắt | All shortcuts reset |
| `hotkeys.conflict.stored` | “{key}” đã dùng cho {others}. Chọn tổ hợp khác. | “{key}” is already used for {others}. Choose another. |
| `hotkeys.conflict.pending` | Tổ hợp này đã dùng cho {action}. Chọn tổ hợp khác. | Already used for {action}. Choose another. |

## Hotkey capture field

The recording well used in the Hotkeys window.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `capture.aria` | {label} — phím tắt hiện tại {value} | {label}, current shortcut: {value} |
| `capture.unset` | chưa đặt | not set |
| `capture.prompt` | Nhấn tổ hợp phím… | Press a key combination… |
| `capture.cancelHint` | huỷ | cancel |
| `capture.clash` | Trùng | Conflict |
| `capture.mayClash` | Dễ trùng | May conflict |
| `capture.empty` | Chưa đặt | Not set |
| `capture.setKey` | Đặt phím | Set key |
| `capture.cancelAria` | Huỷ ghi {label} | Cancel recording {label} |
| `capture.resetAria` | Đặt lại mặc định cho {label} | Reset {label} to default |
| `capture.reset` | Đặt lại mặc định | Reset to default |

## Hotkey validation messages

Shown under a hotkey field when the combination is invalid, reserved by the OS, or common in other apps.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `hk.err.unknownKey` | Không nhận ra phím: {key} | Unrecognized key: {key} |
| `hk.err.needModifier` | Phím tắt cần ít nhất một phím Ctrl/Alt/Shift/Win: {key} | A shortcut needs Ctrl, Alt, Shift or Win: {key} |
| `hk.err.empty` | Phím tắt rỗng | Empty shortcut |
| `hk.reserved.winL` | Windows đã dùng Win + L để khoá máy. Hãy nhấn tổ hợp khác. | Windows uses Win + L to lock the PC. Choose another. |
| `hk.reserved.altTab` | Windows đã dùng Alt + Tab để chuyển cửa sổ. Hãy nhấn tổ hợp khác. | Windows uses Alt + Tab to switch windows. Choose another. |
| `hk.reserved.cad` | Windows đã dùng Ctrl + Alt + Del để mở màn hình bảo mật. Hãy nhấn tổ hợp khác. | Windows uses Ctrl + Alt + Del for its security screen. Choose another. |
| `hk.reserved.taskmgr` | Windows đã dùng Ctrl + Shift + Esc để mở Task Manager. Hãy nhấn tổ hợp khác. | Windows uses Ctrl + Shift + Esc for Task Manager. Choose another. |
| `hk.common.undo` | Hoàn tác | Undo |
| `hk.common.redo` | Làm lại | Redo |
| `hk.common.copy` | Sao chép | Copy |
| `hk.common.paste` | Dán | Paste |
| `hk.common.cut` | Cắt | Cut |
| `hk.common.selectAll` | Chọn tất cả | Select all |
| `hk.common.save` | Lưu | Save |
| `hk.common.find` | Tìm | Find |
| `hk.common.reopenTab` | Mở lại tab | Reopen closed tab |
| `hk.common.winKeyboard` | Đổi bàn phím của Windows | Switch keyboard layout |
| `hk.common.cjk` | Bộ gõ tiếng Trung, Nhật | Switch Chinese/Japanese input |
| `hk.common.quit` | Thoát ứng dụng | Quit app |
| `hk.common.closeWindow` | Đóng cửa sổ | Close window |
| `hk.common.spotlight` | Spotlight | Spotlight |
| `hk.common.macKeyboard` | Đổi bàn phím của macOS | Switch keyboard layout |
| `hk.warning` | {combo} là phím {action} ở hầu hết ứng dụng. Vẫn đặt được nếu bạn muốn. | {combo} is “{action}” in most apps. You can still use it. |

## Hotkey and input-method catalogue

Names and one-line hints of the actions that can have a shortcut (config.toml supports all of them; the Hotkeys window shows four).

| Key | Vietnamese | English (draft) |
|---|---|---|
| `method.telex.hint` | Gõ chữ cái để thêm dấu: aa → â, ow → ơ, s → dấu sắc. | Type letters to add marks: aa → â, ow → ơ, s → acute. |
| `method.vni.hint` | Gõ số để thêm dấu: a1 → á, a6 → â, o7 → ơ. | Type digits to add marks: a1 → á, a6 → â, o7 → ơ. |
| `tone.modern` | Kiểu mới | New-style |
| `tone.classic` | Kiểu cũ | Old-style |
| `hotkey.enable.label` | Bật tiếng Việt | Vietnamese on |
| `hotkey.enable.hint` | Chuyển sang chế độ gõ tiếng Việt, giữ nguyên kiểu gõ đang chọn. | Switches to Vietnamese, keeping the current typing method. |
| `hotkey.disable.label` | Tắt tiếng Việt | Vietnamese off |
| `hotkey.disable.hint` | Trả bàn phím về gõ tiếng Anh bình thường. | Switches back to plain English typing. |
| `hotkey.toggle.label` | Chuyển bật / tắt | Vietnamese on/off |
| `hotkey.toggle.hint` | Một phím duy nhất để đổi trạng thái, tiện khi chỉ muốn một phím tắt. | One shortcut that flips between on and off. |
| `hotkey.restoreWord.label` | Khôi phục từ gõ sai | Restore misspelled word |
| `hotkey.restoreWord.hint` | Khôi phục đúng các phím bạn vừa gõ khi từ bị nhận sai. | Puts back the keys you typed when a word isn't valid Vietnamese. |
| `hotkey.appExclude.label` | Bật / tắt loại trừ ứng dụng | Excluded apps on/off |
| `hotkey.appExclude.hint` | Tạm bỏ qua danh sách loại trừ cho cửa sổ đang mở. | Ignores the excluded apps list for the current window. |
| `hotkey.macroPanel.label` | Bảng gõ tắt | Abbreviations |
| `hotkey.macroPanel.hint` | Mở nhanh danh sách gõ tắt để tra hoặc sửa. | Opens your abbreviations to look up or edit. |
| `hotkey.macroToggle.label` | Bật / tắt gõ tắt | Text expansion on/off |
| `hotkey.macroToggle.hint` | Tắt gõ tắt mà không phải vào cài đặt. | Turns text expansion on or off without opening settings. |
| `hotkey.openSettings.label` | Mở cài đặt | Open settings |
| `hotkey.openSettings.hint` | Mở cửa sổ này từ bất cứ đâu. | Opens this window from anywhere. |
| `hotkey.cycleMethod.label` | Đổi kiểu gõ | Change typing method |
| `hotkey.cycleMethod.hint` | Chuyển qua lại Telex ↔ VNI. | Switches between Telex and VNI. |

## Advanced window

Checkbox grid of less common options, in three sections.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `adv.sec.typing` | Gõ | Typing |
| `adv.sec.compat` | Tương thích | Compatibility |
| `adv.sec.system` | Hệ thống | System |
| `adv.freeTone` | Gõ dấu ở cuối từ | Free tone placement |
| `adv.autoHornUo` | Tự sửa ưo thành ươ | Auto-correct ưo to ươ |
| `adv.autoCapitalize` | Viết hoa đầu câu | Auto-capitalize sentences |
| `adv.jfwz` | Cho phép f j w z làm phụ âm | Allow f j w z as consonants |
| `adv.restoreWrong` | Tự động khôi phục từ gõ sai | Restore misspelled words |
| `adv.macroWhenOff` | Vẫn dùng gõ tắt khi đã tắt tiếng Việt | Expand even when Vietnamese is off |
| `adv.macroCase` | Viết hoa theo từ tắt | Match capitalization |
| `adv.clipboard` | Gửi chữ qua Clipboard | Send text via the clipboard |
| `adv.sound` | Âm thanh khi bật/tắt | On/off sound |
| `adv.checkUpdates` | Tự kiểm tra cập nhật | Check for updates automatically |
| `adv.resetAll` | Đặt lại tất cả… | Reset all… |
| `adv.about` | Giới thiệu… | About… |
| `adv.resetDone` | Đã đặt lại tất cả cài đặt | All settings reset |

## About window

Product story, update status and changelog. Tone: calm, plain, no hype (see 90-copy-voice.md).

| Key | Vietnamese | English (draft) |
|---|---|---|
| `about.tagline` | Bộ gõ tiếng Việt cho {os}. Nhẹ, nhanh, và không làm phiền bạn. | A Vietnamese input method for {os}. Light, fast, and out of your way. |
| `about.versionTitle` | Bảng mã cũ (TCVN3, VNI Windows, VISCII) và VIQR không có trong VietKB. | Legacy encodings (TCVN3, VNI Windows, VISCII) and VIQR are not included in VietKB. |
| `about.why` | Vì sao có VietKB | Why VietKB exists |
| `about.story1` | Những bộ gõ đang có vẫn gõ đúng, nhưng mang theo cả một thời phần mềm cũ: bảng mã 8-bit, hàng chục tuỳ chọn, nhiều tab. | Existing input methods still type correctly, but they carry a whole era of old software: 8-bit encodings, dozens of options, many tabs. |
| `about.story2` | VietKB có mặt vì phần dùng hằng ngày thì ít, phần phải mang theo thì nhiều. Ở đây chỉ có Telex, VNI, gõ tắt và loại trừ ứng dụng, không có mục nào để bạn phải đoán. | VietKB exists because the part you use every day is small and the baggage is large. It has only Telex, VNI, text expansion and excluded apps, and nothing you have to guess. |
| `about.story3` | Miễn phí, mã nguồn mở, không quảng cáo, không ghi lại nội dung gõ. | Free, open source, no ads, and it never records what you type. |
| `about.story3.win` | Cả bộ gõ nằm trong khay, tốn 14 MB; bộ cài khoảng 3 MB; mở cửa sổ mất khoảng 0,3 giây. | It all lives in the system tray: 14 MB in use, a 3 MB installer, and the window opens in about 0.3 seconds. |
| `about.story3.mac` | Cả bộ gõ nằm trên thanh menu, nhẹ và mở cửa sổ gần như tức thì. | It all lives in the menu bar, stays light, and opens its window almost instantly. |
| `about.close` | Một bộ gõ tốt là bộ gõ bạn không phải nghĩ tới. | A good input method is one you never have to think about. |
| `about.status.checking` | Đang kiểm tra… | Checking… |
| `about.status.none` | Bạn đang dùng bản mới nhất | You're up to date |
| `about.status.available` | Có bản cập nhật v{version} | Update v{version} available |
| `about.status.error` | Không kiểm tra được bản cập nhật. | Couldn't check for updates. |
| `about.status.idle` | Chưa kiểm tra | Not checked yet |
| `about.check` | Kiểm tra cập nhật | Check for updates |
| `about.downloading` | Đang tải… | Downloading… |
| `about.installing` | Đang cập nhật… | Updating… |
| `about.openDownload` | Mở trang tải về | Open download page |
| `about.updateNow` | Cập nhật ngay | Update now |
| `about.changelog` | Nhật ký thay đổi — v{version} | Changelog — v{version} |
| `about.changelogAria` | Nhật ký thay đổi | Changelog |
| `about.noNotes` | Bản phát hành không kèm mô tả. | No release notes. |
| `about.github` | GitHub | GitHub |
| `about.reportBug` | Báo lỗi | Report a bug |
| `about.kofi` | Ko-fi | Ko-fi |
| `about.privacy` | Không quảng cáo. Không thu thập nội dung bạn gõ. | No ads. Nothing you type is collected. |

## Update messages (UI side)

Notes shown under the update button in About, and demo text for the browser preview.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `update.note.preview` | Chế độ xem trước: đang hiện bản cập nhật mẫu. | Preview mode: showing a sample update. |
| `update.note.previewNoDownload` | Chế độ xem trước: không tải được tệp cài đặt. | Preview mode: can't download the installer. |
| `update.note.macOpened` | Đã mở trang tải về. Tải tệp .dmg và kéo VietKB vào Applications. | Download page opened. Get the .dmg and drag VietKB to Applications. |
| `update.note.downloading` | Đang tải bản cập nhật… | Downloading the update… |
| `update.note.verified` | Đã tải và xác minh SHA256 — bản {version}. Đang cập nhật, ứng dụng sẽ tự mở lại. | Version {version} downloaded and SHA256-verified. Updating; VietKB will reopen on its own. |
| `update.note.downloaded` | Đã tải bản {version}. Đang cập nhật, ứng dụng sẽ tự mở lại. | Version {version} downloaded. Updating; VietKB will reopen on its own. |

## Macros window

The macro (gõ tắt) table: a short trigger expands into longer text when followed by a space. Sample macros such as dc → được are Vietnamese content and stay as they are.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `macros.search` | Tìm từ tắt hoặc nội dung | Search abbreviations or expansions |
| `macros.import` | Nhập từ tệp… | Import from file… |
| `macros.export` | Xuất ra tệp… | Export to file… |
| `macros.exportShort` | Xuất… | Export… |
| `macros.add` | Thêm | Add |
| `macros.empty.title` | Chưa có gõ tắt nào | No abbreviations yet |
| `macros.empty.body` | Gõ dc rồi bấm dấu cách sẽ ra “được”. Thêm vài mục mẫu để thử: | Type dc, then Space, and you get “được”. Add a few samples to try: |
| `macros.empty.addAll` | Thêm cả 3 mục | Add all 3 |
| `macros.noMatch` | Không có mục nào khớp “{query}” | Nothing matches “{query}” |
| `macros.addThis` | Thêm mục này | Add this entry |
| `macros.addQuery` | Thêm “{query}” | Add “{query}” |
| `macros.deselectAll` | Bỏ chọn tất cả | Deselect all |
| `macros.selected` | Đã chọn | Selected |
| `macros.selectedOf` | trong | of |
| `macros.deselect` | Bỏ chọn | Deselect |
| `macros.off` | Gõ tắt đang tắt, các mục chưa có tác dụng · | Text expansion is off; these entries do nothing · |
| `macros.turnOn` | Bật | Turn on |
| `macros.col.trigger` | Từ tắt | Abbreviation |
| `macros.col.text` | Nội dung | Expansion |
| `macros.selectRow` | Chọn {trigger} | Select {trigger} |
| `macros.extraLine` | +1 dòng | +1 line |
| `macros.editRow` | Sửa {trigger} | Edit {trigger} |
| `macros.moreRow` | Tuỳ chọn khác cho {trigger} | More options for {trigger} |
| `macros.more` | Tuỳ chọn khác | More options |
| `macros.duplicate` | Nhân bản | Duplicate |
| `macros.footer.one` | Có 1 mục | 1 entry |
| `macros.footer.results` | kết quả trong | of |
| `macros.footer.entries` | mục · cuộn để xem hết | entries · scroll to see all |
| `macros.editor.new` | Thêm từ tắt | Add abbreviation |
| `macros.editor.edit` | Sửa từ tắt | Edit abbreviation |
| `macros.field.trigger` | Từ tắt | Abbreviation |
| `macros.field.triggerPh` | ví dụ: dc | e.g. dc |
| `macros.field.triggerHint` | Từ tắt viết liền, không có dấu cách. | One word, no spaces. |
| `macros.field.text` | Nội dung | Expansion |
| `macros.field.textPh` | ví dụ: được · Enter để xuống dòng | e.g. được · Enter for a new line |
| `macros.field.chars` | {n} ký tự | {n} characters |
| `macros.saving` | Đang lưu… | Saving… |
| `macros.err.noTrigger` | Nhập từ tắt, ví dụ dc | Enter an abbreviation, for example dc |
| `macros.err.spaces` | Từ tắt viết liền, không có dấu cách | An abbreviation is one word, no spaces |
| `macros.err.triggerLong` | Từ tắt dài tối đa 32 ký tự, đang có {n} | Abbreviations can be up to 32 characters (this one has {n}) |
| `macros.err.noText` | Phần thay thế không được để trống — muốn bỏ thì xoá cả mục. | The expansion can't be empty. To remove it, delete the entry. |
| `macros.err.textLong` | Nội dung dài tối đa 500 ký tự, đang có {n} | Expansions can be up to 500 characters (this one has {n}) |
| `macros.err.previewRename` | Chế độ xem trước: không đổi tên được. | Preview mode: can't rename. |
| `macros.deleted` | Đã xoá {n} mục gõ tắt | Deleted {n} abbreviations |
| `macros.deleteFailed` | Không xoá được | Couldn't delete |
| `macros.duplicateFailed` | Không nhân bản được | Couldn't duplicate |
| `macros.needBackendImport` | Cần backend để nhập tệp. | Import works only in the app. |
| `macros.needBackendExport` | Cần backend để xuất tệp. | Export works only in the app. |
| `macros.importDialog` | Chọn tệp gõ tắt | Choose an abbreviations file |
| `macros.fileFilter` | Tệp gõ tắt | Abbreviation files |
| `macros.imported` | Đã nhập {n} mục | Imported {n} entries |
| `macros.importFailed` | Không nhập được tệp | Couldn't import the file |
| `macros.exportDialog` | Lưu danh sách gõ tắt | Save abbreviations |
| `macros.exportFile` | vietkb-go-tat.tsv | vietkb-abbreviations.tsv |
| `macros.exported` | Đã xuất {n} mục ra {name} | Exported {n} entries to {name} |
| `macros.exportFailed` | Không xuất được tệp | Couldn't export the file |
| `macros.addedOne` | Đã thêm 1 mục | Added 1 entry |
| `macros.addOneFailed` | Không thêm được mục | Couldn't add the entry |
| `macros.seedFailed` | Không thêm được ví dụ | Couldn't add the samples |

## Tray / menu bar (Rust)

Menu items and tooltip of the tray icon (Windows) or menu-bar extra (macOS). Generated by Rust; shown here for review.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `tray.toggle` | Bật/tắt tiếng Việt | Vietnamese on/off |
| `tray.method` | Kiểu gõ | Typing method |
| `tray.settings` | Mở cài đặt… | Open settings… |
| `tray.hotkeys` | Phím tắt… | Shortcuts… |
| `tray.about` | Giới thiệu | About |
| `tray.update` | Có bản cập nhật v{version} | Update v{version} available |
| `tray.quit` | Thoát VietKB | Quit VietKB |
| `tray.tip.on` | VietKB · Tiếng Việt · {method} | VietKB · Vietnamese · {method} |
| `tray.tip.off` | VietKB · Đã tắt tiếng Việt | VietKB · Vietnamese off |
| `tray.tip.update` | {base} · Có bản cập nhật v{version} | {base} · Update v{version} available |

## Window titles (Rust)

Native window titles.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `rwin.main` | Cài đặt VietKB | VietKB Settings |
| `rwin.hotkeys` | Phím tắt — VietKB | Shortcuts — VietKB |
| `rwin.advanced` | Nâng cao — VietKB | Advanced — VietKB |
| `rwin.macros` | Bảng gõ tắt — VietKB | Abbreviations — VietKB |
| `rwin.about` | Giới thiệu — VietKB | About — VietKB |
| `rwin.invalid` | cửa sổ không hợp lệ: {kind} | invalid window: {kind} |

## Messages returned by Rust commands

Error strings that reach the UI as toasts or status lines.

| Key | Vietnamese | English (draft) |
|---|---|---|
| `err.notReady` | ứng dụng chưa sẵn sàng | the app is not ready yet |
| `err.macro.noTrigger` | Nhập từ tắt, ví dụ dc | Enter an abbreviation, for example dc |
| `err.macro.spaces` | Từ tắt viết liền, không có dấu cách | An abbreviation is one word, no spaces |
| `err.macro.triggerLong` | Từ tắt dài tối đa 32 ký tự, đang có {n} | Abbreviations can be up to 32 characters (this one has {n}) |
| `err.macro.noText` | Phần thay thế không được để trống — muốn bỏ thì xoá cả mục. | The expansion can't be empty. To remove it, delete the entry. |
| `err.macro.textLong` | Nội dung dài tối đa 500 ký tự, đang có {n} | Expansions can be up to 500 characters (this one has {n}) |
| `err.import.tooBig` | Không đọc được tệp {file}: tệp lớn hơn 2 MB. | Couldn't read {file}: it's larger than 2 MB. |
| `err.import.notUtf8` | Không đọc được tệp {file}: tệp không phải UTF-8. | Couldn't read {file}: it isn't UTF-8. |
| `err.import.badLine` | Không đọc được tệp {file}: dòng {line} không đúng dạng. Chưa nhập mục nào. | Couldn't read {file}: line {line} isn't in the expected format. Nothing was imported. |
| `err.import.tooMany` | Không đọc được tệp {file}: quá 5.000 mục. Chưa nhập mục nào. | Couldn't read {file}: more than 5,000 entries. Nothing was imported. |
| `err.import.noLines` | Không đọc được tệp {file}: không có dòng nào. Chưa nhập mục nào. | Couldn't read {file}: it's empty. Nothing was imported. |
| `err.rule.noProcess` | tên tiến trình không được để trống | the app name can't be empty |
| `err.method.invalid` | kiểu gõ không hợp lệ | invalid typing method |
| `err.openFolder` | không mở được thư mục: {error} | couldn't open the folder: {error} |
| `err.update.noInstaller` | Bản phát hành không có bộ cài cho Windows x64 | This release has no installer for Windows x64 |
| `err.update.noDmg` | Bản phát hành không có tệp .dmg cho macOS | This release has no .dmg file for macOS |
| `err.update.connect` | Không kết nối được máy chủ cập nhật: {error} | Couldn't reach the update server: {error} |
| `err.update.rateLimited` | GitHub đang giới hạn lượt kiểm tra (HTTP {status}). Thử lại sau vài phút. | GitHub is rate-limiting update checks (HTTP {status}). Try again in a few minutes. |
| `err.update.http` | Máy chủ cập nhật trả lời HTTP {status}. | The update server returned HTTP {status}. |
| `err.update.parse` | Phản hồi cập nhật không đọc được: {error} | Couldn't read the update response: {error} |
| `err.update.checkFailed` | Không kiểm tra được: {error} | Couldn't check: {error} |
| `err.update.nothingToDownload` | Chưa có bản cập nhật để tải. | No update to download. |
| `err.update.downloadFailed` | Không tải được bản cập nhật: {error} | Couldn't download the update: {error} |
| `err.update.openPage` | Không mở được trang tải về: {error} | Couldn't open the download page: {error} |
| `err.update.notDownloaded` | Chưa tải bản cập nhật. | The update hasn't been downloaded yet. |
| `err.update.installerGone` | Tệp cài đặt không còn trong thư mục tạm. | The installer is no longer in the temp folder. |
| `err.update.exePath` | Không xác định được tệp chạy: {error} | Couldn't find the VietKB program file: {error} |
| `err.update.writeScript` | Không ghi được kịch bản cập nhật: {error} | Couldn't write the update script: {error} |
| `err.update.runInstaller` | Không chạy được trình cài đặt: {error} | Couldn't start the installer: {error} |
| `err.update.winOnly` | Cài đặt tại chỗ chỉ hỗ trợ Windows. | In-place updates work only on Windows. |
| `err.update.linkBlocked` | Liên kết không được phép mở. | This link can't be opened. |
| `err.update.openLink` | Không mở được liên kết: {error} | Couldn't open the link: {error} |
| `err.update.mkdir` | Không tạo được thư mục tải: {error} | Couldn't create the download folder: {error} |
| `err.update.serverDownload` | Máy chủ tải bản cập nhật trả lời HTTP {status}. | The download server returned HTTP {status}. |
| `err.update.writeFile` | Không ghi được tệp tải: {error} | Couldn't save the download: {error} |
| `err.update.interrupted` | Tải bản cập nhật bị gián đoạn: {error} | The download was interrupted: {error} |
| `err.update.finalize` | Không hoàn tất được tệp tải: {error} | Couldn't finish saving the download: {error} |
| `err.update.reread` | Không đọc lại được tệp tải: {error} | Couldn't re-read the download: {error} |
| `err.update.shaMismatch` | SHA256 của tệp tải về không khớp với bản phát hành — đã huỷ tệp. | The download's SHA256 doesn't match the release. The file was deleted. |

Total: 317 keys.
