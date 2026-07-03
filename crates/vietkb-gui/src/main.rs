#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
fn main() {
    if let Err(err) = app::run() {
        native_windows_gui::modal_error_message(&native_windows_gui::Window::default(), "VietKB", &err.to_string());
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("vietkb-gui is only available on Windows");
}

#[cfg(target_os = "windows")]
mod app {
    use std::thread;

    use native_windows_gui as nwg;
    use vietkb_config::{AppConfig, InputMethodConfig};
    use vietkb_core::InputMethod;
    use vietkb_ui::SettingsViewModel;
    use winapi::um::winuser::{SendMessageW, SetForegroundWindow, TrackPopupMenu, BM_GETCHECK, BM_SETCHECK, BST_CHECKED, BST_UNCHECKED, CB_ERR, CB_GETCURSEL, CB_SETCURSEL, GetWindowTextLengthW, GetWindowTextW};

    const METHODS: [&str; 3] = ["Telex", "VNI", "VIQR"];
    const CHARSETS: [&str; 1] = ["Unicode"];

    #[derive(Default)]
    struct App {
        window: nwg::Window,
        icon: nwg::Icon,
        basic_header: nwg::Label,
        typing_header: nwg::Label,
        enabled: nwg::CheckBox,
        start_with_os: nwg::CheckBox,
        method_label: nwg::Label,
        method: nwg::ComboBox<&'static str>,
        charset_label: nwg::Label,
        charset: nwg::ComboBox<&'static str>,
        hotkey_label: nwg::Label,
        hotkey: nwg::TextInput,
        smart_mark: nwg::CheckBox,
        spell_check: nwg::CheckBox,
        restore_keys: nwg::CheckBox,
        browser_fix: nwg::CheckBox,
        game_mode: nwg::CheckBox,
        tray: nwg::TrayNotification,
        tray_menu: nwg::Menu,
        tray_toggle: nwg::MenuItem,
        tray_telex: nwg::MenuItem,
        tray_vni: nwg::MenuItem,
        tray_viqr: nwg::MenuItem,
        tray_exit: nwg::MenuItem,
        sync_timer: nwg::Timer,
        save: nwg::Button,
        exit: nwg::Button,
        handler: Option<nwg::EventHandler>,
    }

    pub fn run() -> anyhow::Result<()> {
        nwg::init()?;
        nwg::Font::set_global_family("Segoe UI")?;

        thread::spawn(|| {
            if let Err(err) = vietkb_windows::run_keyboard_loop() {
                eprintln!("keyboard hook failed: {err}");
            }
        });

        let config = AppConfig::default();
        let view = SettingsViewModel::from(&config);
        let mut app = App::default();

        app.icon = nwg::Icon::from_system(nwg::OemIcon::Information);

        nwg::Window::builder()
            .size((640, 420))
            .position((260, 220))
            .title("VietKB")
            .icon(Some(&app.icon))
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .build(&mut app.window)?;

        build_groups(&mut app)?;
        build_basic(&mut app, &config, &view)?;
        build_typing(&mut app)?;
        build_tray(&mut app, view.enabled)?;
        nwg::Timer::builder().parent(&app.window).interval(200).stopped(false).build(&mut app.sync_timer)?;

        nwg::Button::builder()
            .text("Lưu cài đặt")
            .parent(&app.window)
            .position((420, 338))
            .size((96, 32))
            .build(&mut app.save)?;

        nwg::Button::builder()
            .text("Thoát")
            .parent(&app.window)
            .position((528, 338))
            .size((72, 32))
            .build(&mut app.exit)?;

        let window = app.window.handle;
        let save = app.save.handle;
        let exit = app.exit.handle;
        let enabled = app.enabled.handle;
        let method = app.method.handle;
        let hotkey = app.hotkey.handle;
        let tray = app.tray.handle;
        let tray_menu = app.tray_menu.handle;
        let tray_toggle = app.tray_toggle.handle;
        let tray_telex = app.tray_telex.handle;
        let tray_vni = app.tray_vni.handle;
        let tray_viqr = app.tray_viqr.handle;
        let tray_exit = app.tray_exit.handle;
        let sync_timer = app.sync_timer.handle;

        app.handler = Some(nwg::full_bind_event_handler(&window, move |event, _event_data, handle| {
            if event == nwg::Event::OnWindowClose || (event == nwg::Event::OnButtonClick && handle == exit) {
                nwg::stop_thread_dispatch();
            }

            if event == nwg::Event::OnContextMenu && handle == tray {
                let (x, y) = nwg::GlobalCursor::position();
                popup_menu(tray_menu, x, y);
            }

            if event == nwg::Event::OnMenuItemSelected && handle == tray_exit {
                nwg::stop_thread_dispatch();
            }

            if event == nwg::Event::OnTimerTick && handle == sync_timer {
                set_checkbox_checked(enabled, vietkb_windows::is_keyboard_enabled());
            }

            if event == nwg::Event::OnMenuItemSelected && handle == tray_toggle {
                set_checkbox_checked(enabled, !checkbox_checked(enabled));
            }

            if event == nwg::Event::OnMenuItemSelected && handle == tray_telex {
                set_combo_selection(method, 0);
            } else if event == nwg::Event::OnMenuItemSelected && handle == tray_vni {
                set_combo_selection(method, 1);
            } else if event == nwg::Event::OnMenuItemSelected && handle == tray_viqr {
                set_combo_selection(method, 2);
            }

            if event == nwg::Event::OnButtonClick && handle == save
                || event == nwg::Event::OnMenuItemSelected && (handle == tray_toggle || handle == tray_telex || handle == tray_vni || handle == tray_viqr)
            {
                let enabled = checkbox_checked(enabled);
                let method = method_from_index(combo_selection(method));
                let hotkey = text_input(hotkey);
                vietkb_windows::set_keyboard_config(enabled, method, &hotkey);
            }
        }));

        nwg::dispatch_thread_events();
        Ok(())
    }

    fn build_groups(app: &mut App) -> anyhow::Result<()> {
        nwg::Label::builder()
            .text("Cơ bản")
            .parent(&app.window)
            .position((24, 18))
            .size((160, 24))
            .build(&mut app.basic_header)?;
        nwg::Label::builder()
            .text("Tùy chọn")
            .parent(&app.window)
            .position((24, 170))
            .size((160, 24))
            .build(&mut app.typing_header)?;
        Ok(())
    }

    fn build_basic(app: &mut App, config: &AppConfig, view: &SettingsViewModel) -> anyhow::Result<()> {
        nwg::CheckBox::builder()
            .text("Bật gõ tiếng Việt")
            .parent(&app.window)
            .position((24, 48))
            .size((260, 26))
            .check_state(checked(view.enabled))
            .build(&mut app.enabled)?;

        nwg::CheckBox::builder()
            .text("Khởi động cùng hệ điều hành")
            .parent(&app.window)
            .position((330, 48))
            .size((280, 26))
            .build(&mut app.start_with_os)?;

        label(&app.window, &mut app.method_label, "Kiểu gõ", 24, 88)?;
        combo(&app.window, &mut app.method, &METHODS, method_index(view.method), 150, 84, 170)?;
        label(&app.window, &mut app.charset_label, "Bảng mã", 24, 124)?;
        combo(&app.window, &mut app.charset, &CHARSETS, 0, 150, 120, 170)?;
        label(&app.window, &mut app.hotkey_label, "Phím bật/tắt", 330, 88)?;

        nwg::TextInput::builder()
            .text(&config.hotkeys.toggle)
            .parent(&app.window)
            .position((450, 84))
            .size((150, 26))
            .build(&mut app.hotkey)?;

        Ok(())
    }

    fn build_typing(app: &mut App) -> anyhow::Result<()> {
        checkbox(&app.window, &mut app.smart_mark, "Đặt dấu thanh thông minh", 24, 204, 280, true)?;
        checkbox(&app.window, &mut app.spell_check, "Kiểm tra chính tả tiếng Việt", 330, 204, 280, true)?;
        checkbox(&app.window, &mut app.restore_keys, "Khôi phục phím gốc khi hủy", 24, 242, 280, true)?;
        checkbox(&app.window, &mut app.browser_fix, "Sửa lỗi dấu trên trình duyệt", 330, 242, 280, true)?;
        checkbox(&app.window, &mut app.game_mode, "Chế độ tương thích game", 24, 280, 280, false)?;
        Ok(())
    }

    fn build_tray(app: &mut App, enabled: bool) -> anyhow::Result<()> {
        nwg::TrayNotification::builder()
            .parent(&app.window)
            .icon(Some(&app.icon))
            .tip(Some(tray_tip(enabled)))
            .build(&mut app.tray)?;

        nwg::Menu::builder()
            .popup(true)
            .parent(&app.window)
            .build(&mut app.tray_menu)?;

        menu_item(&app.tray_menu, &mut app.tray_toggle, toggle_label(enabled))?;
        menu_item(&app.tray_menu, &mut app.tray_telex, "Telex")?;
        menu_item(&app.tray_menu, &mut app.tray_vni, "VNI")?;
        menu_item(&app.tray_menu, &mut app.tray_viqr, "VIQR")?;
        menu_item(&app.tray_menu, &mut app.tray_exit, "Thoát")?;
        app.tray_telex.set_checked(true);
        Ok(())
    }

    fn menu_item(parent: &nwg::Menu, out: &mut nwg::MenuItem, text: &str) -> anyhow::Result<()> {
        nwg::MenuItem::builder().text(text).parent(parent).build(out)?;
        Ok(())
    }

    fn label(parent: &nwg::Window, out: &mut nwg::Label, text: &str, x: i32, y: i32) -> anyhow::Result<()> {
        nwg::Label::builder()
            .text(text)
            .parent(parent)
            .position((x, y))
            .size((120, 24))
            .build(out)?;
        Ok(())
    }

    fn combo(
        parent: &nwg::Window,
        out: &mut nwg::ComboBox<&'static str>,
        items: &[&'static str],
        index: usize,
        x: i32,
        y: i32,
        width: i32,
    ) -> anyhow::Result<()> {
        nwg::ComboBox::builder()
            .parent(parent)
            .position((x, y))
            .size((width, 90))
            .collection(items.to_vec())
            .selected_index(Some(index))
            .build(out)?;
        Ok(())
    }

    fn checkbox(
        parent: &nwg::Window,
        out: &mut nwg::CheckBox,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        is_checked: bool,
    ) -> anyhow::Result<()> {
        nwg::CheckBox::builder()
            .text(text)
            .parent(parent)
            .position((x, y))
            .size((width, 26))
            .check_state(checked(is_checked))
            .build(out)?;
        Ok(())
    }

    fn checked(value: bool) -> nwg::CheckBoxState {
        if value {
            nwg::CheckBoxState::Checked
        } else {
            nwg::CheckBoxState::Unchecked
        }
    }

    fn method_index(method: InputMethodConfig) -> usize {
        match method {
            InputMethodConfig::Telex => 0,
            InputMethodConfig::Vni => 1,
            InputMethodConfig::Viqr => 2,
        }
    }

    fn method_from_index(index: usize) -> InputMethod {
        match index {
            1 => InputMethod::Vni,
            2 => InputMethod::Viqr,
            _ => InputMethod::Telex,
        }
    }

    fn combo_selection(handle: nwg::ControlHandle) -> usize {
        let Some(hwnd) = handle.hwnd() else {
            return 0;
        };
        let index = unsafe { SendMessageW(hwnd, CB_GETCURSEL, 0, 0) };
        if index == CB_ERR {
            0
        } else {
            index as usize
        }
    }

    fn set_combo_selection(handle: nwg::ControlHandle, index: usize) {
        if let Some(hwnd) = handle.hwnd() {
            unsafe { SendMessageW(hwnd, CB_SETCURSEL, index, 0) };
        }
    }

    fn checkbox_checked(handle: nwg::ControlHandle) -> bool {
        let Some(hwnd) = handle.hwnd() else {
            return false;
        };
        unsafe { SendMessageW(hwnd, BM_GETCHECK, 0, 0) as usize == BST_CHECKED }
    }

    fn set_checkbox_checked(handle: nwg::ControlHandle, value: bool) {
        if let Some(hwnd) = handle.hwnd() {
            let state = if value { BST_CHECKED } else { BST_UNCHECKED };
            unsafe { SendMessageW(hwnd, BM_SETCHECK, state, 0) };
        }
    }

    fn text_input(handle: nwg::ControlHandle) -> String {
        let Some(hwnd) = handle.hwnd() else {
            return String::new();
        };
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        let mut buffer = vec![0u16; len as usize + 1];
        let copied = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) } as usize;
        String::from_utf16_lossy(&buffer[..copied])
    }

    fn popup_menu(handle: nwg::ControlHandle, x: i32, y: i32) {
        let Some((hwnd, menu)) = handle.pop_hmenu() else {
            return;
        };
        unsafe {
            SetForegroundWindow(hwnd);
            TrackPopupMenu(menu, 0, x, y, 0, hwnd, std::ptr::null());
        }
    }

    fn tray_tip(enabled: bool) -> &'static str {
        if enabled { "VietKB - V" } else { "VietKB - E" }
    }

    fn toggle_label(enabled: bool) -> &'static str {
        if enabled { "Tắt tiếng Việt (E)" } else { "Bật tiếng Việt (V)" }
    }
}
