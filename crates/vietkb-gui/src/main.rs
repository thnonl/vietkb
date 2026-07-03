#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
fn main() {
    if let Err(err) = app::run() {
        eprintln!("vietkb-gui failed: {err}");
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("vietkb-gui is only available on Windows");
}

#[cfg(target_os = "windows")]
mod app {
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::{env, fs, thread};

    use serde::{Deserialize, Serialize};
    use tauri::image::Image;
    use tauri::menu::{CheckMenuItem, Menu, MenuItem, Submenu};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::{AppHandle, Manager, State, Window, Wry};
    use vietkb_config::{
        AppConfig, CharsetConfig, HotkeyConfig, InputMethodConfig, SystemConfig, TypingConfig,
    };
    use vietkb_core::InputMethod;
    use vietkb_ui::SettingsViewModel;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct UiSettings {
        enabled: bool,
        method: String,
        charset: String,
        hotkeys: HotkeyConfig,
        typing: TypingConfig,
        system: SystemConfig,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SaveSettings {
        enabled: bool,
        method: String,
        charset: String,
        hotkeys: HotkeyConfig,
        typing: TypingConfig,
        system: SystemConfig,
    }

    struct AppState {
        config: Mutex<AppConfig>,
        tray_menu: Mutex<Option<TrayMenu>>,
    }

    #[derive(Clone)]
    struct TrayMenu {
        method_telex: CheckMenuItem<Wry>,
        method_vni: CheckMenuItem<Wry>,
        method_viqr: CheckMenuItem<Wry>,
        charset_unicode: CheckMenuItem<Wry>,
        charset_vni_windows: CheckMenuItem<Wry>,
        charset_tcvn3: CheckMenuItem<Wry>,
        toggle: CheckMenuItem<Wry>,
    }

    pub fn run() -> anyhow::Result<()> {
        thread::spawn(|| {
            if let Err(err) = vietkb_windows::run_keyboard_loop() {
                eprintln!("keyboard hook failed: {err}");
            }
        });

        tauri::Builder::default()
            .setup(|app| {
                let config = load_config_from_disk().unwrap_or_default();
                let enabled = config.input.enabled;
                let method = input_method_from_slug(method_config_to_slug(config.input.method));
                vietkb_windows::set_keyboard_config(
                    enabled,
                    method,
                    &config.hotkeys.toggle,
                    config.typing.skip_english_words,
                );

                app.manage(AppState {
                    config: Mutex::new(config.clone()),
                    tray_menu: Mutex::new(None),
                });

                let app_handle = app.handle().clone();
                vietkb_windows::set_enabled_changed_callback(move |enabled| {
                    apply_config(&app_handle, enabled, |_config| {});
                });

                let method_telex = CheckMenuItem::with_id(
                    app,
                    "method_telex",
                    "Telex",
                    true,
                    config.input.method == InputMethodConfig::Telex,
                    None::<&str>,
                )?;
                let method_vni = CheckMenuItem::with_id(
                    app,
                    "method_vni",
                    "VNI",
                    true,
                    config.input.method == InputMethodConfig::Vni,
                    None::<&str>,
                )?;
                let method_viqr = CheckMenuItem::with_id(
                    app,
                    "method_viqr",
                    "VIQR",
                    true,
                    config.input.method == InputMethodConfig::Viqr,
                    None::<&str>,
                )?;
                let charset_unicode = CheckMenuItem::with_id(
                    app,
                    "charset_unicode",
                    "Unicode",
                    true,
                    config.input.charset == CharsetConfig::Unicode,
                    None::<&str>,
                )?;
                let charset_vni_windows = CheckMenuItem::with_id(
                    app,
                    "charset_vni_windows",
                    "VNI Windows",
                    true,
                    config.input.charset == CharsetConfig::VniWindows,
                    None::<&str>,
                )?;
                let charset_tcvn3 = CheckMenuItem::with_id(
                    app,
                    "charset_tcvn3",
                    "TCVN3 (ABC)",
                    true,
                    config.input.charset == CharsetConfig::Tcvn3,
                    None::<&str>,
                )?;
                let charset_menu = Submenu::with_items(
                    app,
                    "Chuyển mã",
                    true,
                    &[&charset_unicode, &charset_vni_windows, &charset_tcvn3],
                )?;
                let toggle =
                    CheckMenuItem::with_id(app, "toggle", "Bật/tắt", true, enabled, None::<&str>)?;
                let settings =
                    MenuItem::with_id(app, "settings", "Mở cài đặt", true, None::<&str>)?;
                let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?;
                let menu = Menu::with_items(
                    app,
                    &[
                        &method_telex,
                        &method_vni,
                        &method_viqr,
                        &charset_menu,
                        &toggle,
                        &settings,
                        &about,
                        &quit,
                    ],
                )?;
                if let Ok(mut tray_menu) = app.state::<AppState>().tray_menu.lock() {
                    *tray_menu = Some(TrayMenu {
                        method_telex: method_telex.clone(),
                        method_vni: method_vni.clone(),
                        method_viqr: method_viqr.clone(),
                        charset_unicode: charset_unicode.clone(),
                        charset_vni_windows: charset_vni_windows.clone(),
                        charset_tcvn3: charset_tcvn3.clone(),
                        toggle: toggle.clone(),
                    });
                }

                TrayIconBuilder::with_id("main")
                    .tooltip("VietKB")
                    .icon(tray_icon(enabled)?)
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "method_telex" => set_input_method(app, InputMethod::Telex),
                        "method_vni" => set_input_method(app, InputMethod::Vni),
                        "method_viqr" => set_input_method(app, InputMethod::Viqr),
                        "charset_unicode" => set_charset(app, CharsetConfig::Unicode),
                        "charset_vni_windows" => set_charset(app, CharsetConfig::VniWindows),
                        "charset_tcvn3" => set_charset(app, CharsetConfig::Tcvn3),
                        "toggle" => toggle_keyboard(app),
                        "settings" | "about" => show_main_window(app),
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            show_main_window(tray.app_handle());
                        }
                    })
                    .build(app)?;

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                load_settings,
                save_settings,
                set_enabled,
                keyboard_enabled,
                close_window,
                exit_app
            ])
            .run(tauri::generate_context!())?;

        Ok(())
    }

    fn show_main_window(app: &AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }

    fn tray_icon(enabled: bool) -> tauri::Result<Image<'static>> {
        let bytes: &[u8] = if enabled {
            include_bytes!("../icons/V.ico")
        } else {
            include_bytes!("../icons/E.ico")
        };
        Image::from_bytes(bytes)
    }

    fn update_tray_icon(app: &AppHandle, enabled: bool) {
        if let Some(tray) = app.tray_by_id("main") {
            if let Ok(icon) = tray_icon(enabled) {
                let _ = tray.set_icon(Some(icon));
            }
        }
    }

    fn apply_config(app: &AppHandle, enabled: bool, mutate: impl FnOnce(&mut AppConfig)) {
        let state = app.state::<AppState>();
        let Ok(mut config) = state.config.lock() else {
            return;
        };
        config.input.enabled = enabled;
        mutate(&mut config);
        let method = input_method_from_slug(method_config_to_slug(config.input.method));
        vietkb_windows::set_keyboard_config(
            enabled,
            method,
            &config.hotkeys.toggle,
            config.typing.skip_english_words,
        );
        let _ = save_config_to_disk(&config);
        update_tray_menu(app, config.input.method, config.input.charset, enabled);
        update_tray_icon(app, enabled);
    }

    fn settings_path() -> PathBuf {
        env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.join("settings.ini")))
            .unwrap_or_else(|| PathBuf::from("settings.ini"))
    }

    fn load_config_from_disk() -> Option<AppConfig> {
        let input = fs::read_to_string(settings_path()).ok()?;
        AppConfig::from_ini(&input).ok()
    }

    fn save_config_to_disk(config: &AppConfig) -> std::io::Result<()> {
        fs::write(settings_path(), config.to_ini())
    }

    fn update_tray_menu(
        app: &AppHandle,
        method: InputMethodConfig,
        charset: CharsetConfig,
        enabled: bool,
    ) {
        let state = app.state::<AppState>();
        let Ok(tray_menu) = state.tray_menu.lock() else {
            return;
        };
        let Some(tray_menu) = tray_menu.as_ref() else {
            return;
        };

        let _ = tray_menu
            .method_telex
            .set_checked(method == InputMethodConfig::Telex);
        let _ = tray_menu
            .method_vni
            .set_checked(method == InputMethodConfig::Vni);
        let _ = tray_menu
            .method_viqr
            .set_checked(method == InputMethodConfig::Viqr);
        let _ = tray_menu
            .charset_unicode
            .set_checked(charset == CharsetConfig::Unicode);
        let _ = tray_menu
            .charset_vni_windows
            .set_checked(charset == CharsetConfig::VniWindows);
        let _ = tray_menu
            .charset_tcvn3
            .set_checked(charset == CharsetConfig::Tcvn3);
        let _ = tray_menu.toggle.set_checked(enabled);
    }

    fn set_input_method(app: &AppHandle, method: InputMethod) {
        let enabled = vietkb_windows::is_keyboard_enabled();
        apply_config(app, enabled, |config| {
            config.input.method = method_to_config(method)
        });
    }

    fn set_charset(app: &AppHandle, charset: CharsetConfig) {
        let enabled = vietkb_windows::is_keyboard_enabled();
        apply_config(app, enabled, |config| config.input.charset = charset);
    }

    fn toggle_keyboard(app: &AppHandle) {
        let enabled = !vietkb_windows::is_keyboard_enabled();
        apply_config(app, enabled, |_config| {});
    }

    #[tauri::command]
    fn load_settings(state: State<'_, AppState>) -> UiSettings {
        let config = state
            .config
            .lock()
            .map(|config| config.clone())
            .unwrap_or_default();
        let view = SettingsViewModel::from(&config);

        UiSettings {
            enabled: vietkb_windows::is_keyboard_enabled(),
            method: method_config_to_slug(view.method).to_owned(),
            charset: charset_config_to_slug(config.input.charset).to_owned(),
            hotkeys: config.hotkeys,
            typing: config.typing,
            system: config.system,
        }
    }

    #[tauri::command]
    fn save_settings(
        app: AppHandle,
        settings: SaveSettings,
        state: State<'_, AppState>,
    ) -> Result<(), String> {
        let method = input_method_from_slug(&settings.method);
        vietkb_windows::set_keyboard_config(
            settings.enabled,
            method,
            &settings.hotkeys.toggle,
            settings.typing.skip_english_words,
        );

        let (method_config, charset) = {
            let mut config = state
                .config
                .lock()
                .map_err(|_| "Không thể khóa cấu hình".to_owned())?;
            config.hotkeys = settings.hotkeys;
            config.typing = settings.typing;
            config.system = settings.system;
            config.input.charset = charset_from_slug(&settings.charset);
            let mut view = SettingsViewModel::from(&*config);
            view.enabled = settings.enabled;
            view.method = method_to_config(method);
            view.apply_to(&mut config);
            save_config_to_disk(&config).map_err(|err| err.to_string())?;
            (config.input.method, config.input.charset)
        };

        update_tray_menu(&app, method_config, charset, settings.enabled);
        update_tray_icon(&app, settings.enabled);

        Ok(())
    }

    #[tauri::command]
    fn set_enabled(
        app: AppHandle,
        enabled: bool,
        _state: State<'_, AppState>,
    ) -> Result<(), String> {
        apply_config(&app, enabled, |_config| {});
        Ok(())
    }

    #[tauri::command]
    fn keyboard_enabled() -> bool {
        vietkb_windows::is_keyboard_enabled()
    }

    #[tauri::command]
    fn close_window(window: Window) -> Result<(), String> {
        window.hide().map_err(|err| err.to_string())
    }

    #[tauri::command]
    fn exit_app(app: AppHandle) {
        app.exit(0);
    }

    fn input_method_from_slug(value: &str) -> InputMethod {
        match value {
            "vni" => InputMethod::Vni,
            "viqr" => InputMethod::Viqr,
            _ => InputMethod::Telex,
        }
    }

    fn method_to_config(method: InputMethod) -> InputMethodConfig {
        match method {
            InputMethod::Vni => InputMethodConfig::Vni,
            InputMethod::Viqr => InputMethodConfig::Viqr,
            InputMethod::Telex => InputMethodConfig::Telex,
        }
    }

    fn method_config_to_slug(method: InputMethodConfig) -> &'static str {
        match method {
            InputMethodConfig::Vni => "vni",
            InputMethodConfig::Viqr => "viqr",
            InputMethodConfig::Telex => "telex",
        }
    }

    fn charset_from_slug(value: &str) -> CharsetConfig {
        match value {
            "vni-windows" => CharsetConfig::VniWindows,
            "tcvn3" => CharsetConfig::Tcvn3,
            _ => CharsetConfig::Unicode,
        }
    }

    fn charset_config_to_slug(charset: CharsetConfig) -> &'static str {
        match charset {
            CharsetConfig::Unicode => "unicode",
            CharsetConfig::VniWindows => "vni-windows",
            CharsetConfig::Tcvn3 => "tcvn3",
        }
    }
}
