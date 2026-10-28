use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

/// Magic value written to `dwExtraInfo` on every INPUT we inject so we can
/// recognise our own events when they re-enter the hook.
const INJECTED_MAGIC: usize = 0x5649_454B; // "VIEK"

use vietkb_config::{FunctionKeyConfig, PerAppConfig, SystemConfig, TypingConfig};
use vietkb_core::{
    Charset, EngineAction, EngineOptions, InputMethod, KeyCode, KeyInput, SimpleVietnameseEngine,
    VietnameseEngine,
};
use windows::Win32::Foundation::{CloseHandle, HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardState, SendInput, ToUnicode, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK,
    VK_CAPITAL, VK_CONTROL, VK_F1, VK_F12, VK_F2, VK_F3, VK_F4, VK_LCONTROL, VK_LMENU, VK_LSHIFT,
    VK_LWIN, VK_MENU, VK_PACKET, VK_RCONTROL, VK_RETURN, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
    VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_SYSKEYDOWN,
};

static STATE: OnceLock<Mutex<HookState>> = OnceLock::new();

type EnabledCallback = Arc<dyn Fn(bool) + Send + Sync + 'static>;
type CommandCallback = Arc<dyn Fn(KeyboardCommand) + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardCommand {
    SetEnabled(bool),
    SetCharset(Charset),
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SendStrategy {
    UnicodeSendInput,
    VirtualKeyStandard,
    ClipboardPaste,
}

struct HookState {
    engine: SimpleVietnameseEngine,
    enabled: bool,
    send_strategy: SendStrategy,
    hotkey: Hotkey,
    restore_hotkey: Hotkey,
    function_keys: FunctionKeyConfig,
    excluded_apps_enabled: bool,
    excluded_apps: BTreeSet<String>,
    hotkey_down: bool,
    restore_hotkey_down: bool,
    command_down: bool,
    on_enabled_changed: Option<EnabledCallback>,
    on_command: Option<CommandCallback>,
}

#[derive(Clone, Copy)]
struct Hotkey {
    ctrl: bool,
    shift: bool,
    alt: bool,
    win: bool,
    key: Option<u32>,
}

impl Default for Hotkey {
    fn default() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
            win: false,
            key: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn set_keyboard_config(
    enabled: bool,
    method: InputMethod,
    charset: Charset,
    hotkey: &str,
    restore_hotkey: &str,
    function_keys: &FunctionKeyConfig,
    typing: &TypingConfig,
    system: &SystemConfig,
    apps: &std::collections::BTreeMap<String, PerAppConfig>,
) {
    let state = STATE.get_or_init(default_state);
    if let Ok(mut state) = state.lock() {
        state.enabled = enabled;
        state.hotkey = Hotkey::parse(hotkey);
        state.restore_hotkey = Hotkey::parse(restore_hotkey);
        state.function_keys = function_keys.clone();
        state.excluded_apps_enabled = system.excluded_apps;
        state.excluded_apps = apps
            .iter()
            .filter(|(_, app)| app.enabled == Some(false))
            .map(|(name, _)| normalize_exe_name(name))
            .collect();
        state.send_strategy = send_strategy_from_config(typing, system);
        state.engine.set_input_method(method);
        state.engine.set_charset(charset);
        state.engine.set_options(EngineOptions {
            old_oa_uy: typing.old_oa_uy,
            smart_mark: typing.smart_mark,
            use_j_f_w_as_consonants: typing.use_j_f_w_as_consonants,
            spell_check: typing.spell_check,
            restore_wrong_word: typing.restore_wrong_word,
            skip_english_words: typing.skip_english_words,
        });
    }
}

pub fn set_enabled_changed_callback(callback: impl Fn(bool) + Send + Sync + 'static) {
    let state = STATE.get_or_init(default_state);
    if let Ok(mut state) = state.lock() {
        state.on_enabled_changed = Some(Arc::new(callback));
    }
}

pub fn set_command_callback(callback: impl Fn(KeyboardCommand) + Send + Sync + 'static) {
    let state = STATE.get_or_init(default_state);
    if let Ok(mut state) = state.lock() {
        state.on_command = Some(Arc::new(callback));
    }
}

pub fn is_keyboard_enabled() -> bool {
    STATE
        .get()
        .and_then(|state| state.lock().ok().map(|state| state.enabled))
        .unwrap_or(true)
}

pub fn run_keyboard_loop() -> anyhow::Result<()> {
    let hook =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE::default(), 0)? };
    STATE.get_or_init(default_state);

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        let _ = UnhookWindowsHookEx(hook);
    }
    Ok(())
}

fn default_state() -> Mutex<HookState> {
    Mutex::new(HookState {
        engine: SimpleVietnameseEngine::new(),
        enabled: true,
        send_strategy: SendStrategy::UnicodeSendInput,
        hotkey: Hotkey::default(),
        restore_hotkey: Hotkey::parse("Shift+Alt"),
        function_keys: FunctionKeyConfig::default(),
        excluded_apps_enabled: false,
        excluded_apps: BTreeSet::new(),
        hotkey_down: false,
        restore_hotkey_down: false,
        command_down: false,
        on_enabled_changed: None,
        on_command: None,
    })
}

fn send_strategy_from_config(typing: &TypingConfig, system: &SystemConfig) -> SendStrategy {
    if typing.use_clipboard_to_send_keys || system.metro_apps_support {
        SendStrategy::ClipboardPaste
    } else if system.standard_key_sending {
        SendStrategy::VirtualKeyStandard
    } else {
        SendStrategy::UnicodeSendInput
    }
}

impl HookState {
    fn active_app_excluded(&self) -> bool {
        self.excluded_apps_enabled
            && active_process_name()
                .as_deref()
                .is_some_and(|name| self.excluded_apps.contains(name))
    }
}

fn normalize_exe_name(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(value)
        .trim()
        .to_ascii_lowercase()
}

fn active_process_name() -> Option<String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }

    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };
    if process_id == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()? };
    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
    };
    let _ = unsafe { CloseHandle(handle) };
    if !ok || size == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buffer[..size as usize]);
    Some(normalize_exe_name(&path))
}

impl Hotkey {
    fn parse(value: &str) -> Self {
        let mut hotkey = Self {
            ctrl: false,
            shift: false,
            alt: false,
            win: false,
            key: None,
        };

        for part in value
            .split('+')
            .map(|part| part.trim().to_ascii_lowercase())
        {
            match part.as_str() {
                "ctrl" | "control" => hotkey.ctrl = true,
                "shift" => hotkey.shift = true,
                "alt" => hotkey.alt = true,
                "win" | "windows" | "meta" => hotkey.win = true,
                "space" => hotkey.key = Some(VK_SPACE.0 as u32),
                "enter" | "return" => hotkey.key = Some(VK_RETURN.0 as u32),
                "backspace" => hotkey.key = Some(VK_BACK.0 as u32),
                key if key.len() == 1 => {
                    if let Some(ch) = key.chars().next() {
                        hotkey.key = Some(ch.to_ascii_uppercase() as u32);
                    }
                }
                _ => {}
            }
        }

        hotkey
    }

    fn pressed(self, vk: u32) -> bool {
        self.key
            .map_or_else(|| is_modifier_vk(vk), |hotkey_vk| vk == hotkey_vk)
            && modifier_down_exact(VK_CONTROL.0, self.ctrl, vk)
            && modifier_down_exact(VK_SHIFT.0, self.shift, vk)
            && modifier_down_exact(VK_MENU.0, self.alt, vk)
            && (modifier_down(VK_LWIN.0, vk) || modifier_down(VK_RWIN.0, vk)) == self.win
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let is_key_down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
    if !is_key_down {
        if let Some(state) = STATE.get() {
            if let Ok(mut state) = state.lock() {
                state.hotkey_down = false;
                state.restore_hotkey_down = false;
                state.command_down = false;
            }
        }
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let info = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

    // Always pass through VK_PACKET (Unicode injection via WM_CHAR path).
    if info.vkCode == VK_PACKET.0 as u32 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    // Skip events we injected ourselves (identified by dwExtraInfo magic).
    if info.dwExtraInfo == INJECTED_MAGIC {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    if let Some(state) = STATE.get() {
        if let Ok(mut state) = state.lock() {
            if state.hotkey.pressed(info.vkCode) {
                if !state.hotkey_down {
                    state.hotkey_down = true;
                    state.enabled = !state.enabled;
                    let enabled = state.enabled;
                    let callback = state.on_enabled_changed.clone();
                    drop(state);
                    if let Some(callback) = callback {
                        callback(enabled);
                    }
                }
                return LRESULT(1);
            }
            state.hotkey_down = false;

            if state.restore_hotkey.pressed(info.vkCode) {
                if !state.restore_hotkey_down {
                    state.restore_hotkey_down = true;
                    let action = state.engine.restore_word();
                    let inputs = action_to_inputs(&mut state, action);
                    drop(state);
                    send_inputs(&inputs);
                }
                return LRESULT(1);
            }
            state.restore_hotkey_down = false;

            if let Some(command) = function_key_command(&state.function_keys, info.vkCode) {
                if !state.command_down {
                    state.command_down = true;
                    handle_keyboard_command(&mut state, command);
                    let callback = state.on_command.clone();
                    drop(state);
                    if let Some(callback) = callback {
                        callback(command);
                    }
                }
                return LRESULT(1);
            }
            state.command_down = false;

            if !state.enabled || state.active_app_excluded() {
                state.engine.reset();
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            if let Some(boundary) = boundary_key(info.vkCode) {
                let action = state.engine.process_key(KeyInput::plain(boundary));
                if should_intercept_boundary(&action) {
                    let inputs = action_to_inputs(&mut state, action);
                    drop(state);
                    send_inputs(&inputs);
                    return LRESULT(1);
                }
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            if let Some(ch) = vk_to_char(info.vkCode, info.scanCode) {
                let action = state.engine.process_key(KeyInput::from_char(ch));
                if should_intercept(ch, &action) {
                    let inputs = action_to_inputs(&mut state, action);
                    drop(state);
                    send_inputs(&inputs);
                    return LRESULT(1);
                }
            } else {
                state.engine.reset();
            }
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn handle_keyboard_command(state: &mut HookState, command: KeyboardCommand) {
    match command {
        KeyboardCommand::SetEnabled(enabled) => state.enabled = enabled,
        KeyboardCommand::SetCharset(charset) => state.engine.set_charset(charset),
        KeyboardCommand::Reset => state.engine.reset(),
    }
}

fn function_key_command(config: &FunctionKeyConfig, vk: u32) -> Option<KeyboardCommand> {
    if !function_key_modifiers_pressed(config, vk) {
        return None;
    }

    match VIRTUAL_KEY(vk as u16) {
        VK_F1 if config.f1_toggle_vietnamese_on => Some(KeyboardCommand::SetEnabled(true)),
        VK_F2 if config.f2_toggle_vietnamese_off => Some(KeyboardCommand::SetEnabled(false)),
        VK_F3 if config.f3_unicode => Some(KeyboardCommand::SetCharset(Charset::Unicode)),
        VK_F4 if config.f4_charset => Some(KeyboardCommand::SetCharset(
            charset_from_function_key_value(&config.f4_charset_value),
        )),
        VK_F12 if config.f12_reset => Some(KeyboardCommand::Reset),
        _ => None,
    }
}

fn function_key_modifiers_pressed(config: &FunctionKeyConfig, vk: u32) -> bool {
    modifier_down_exact(VK_CONTROL.0, config.ctrl, vk)
        && modifier_down_exact(VK_SHIFT.0, config.shift, vk)
        && modifier_down_exact(VK_MENU.0, config.alt, vk)
        && (modifier_down(VK_LWIN.0, vk) || modifier_down(VK_RWIN.0, vk)) == config.win
}

fn charset_from_function_key_value(value: &str) -> Charset {
    match value.to_ascii_lowercase().as_str() {
        "vni" | "vni-windows" => Charset::VniWindows,
        "tcvn3" | "abc" => Charset::Tcvn3,
        _ => Charset::Unicode,
    }
}

fn should_intercept(ch: char, action: &EngineAction) -> bool {
    match action {
        EngineAction::Replace { .. } => true,
        EngineAction::Commit(text) => text != &ch.to_string(),
        EngineAction::ToggleVietnamese | EngineAction::BeepOrReject | EngineAction::Preedit(_) => {
            true
        }
        EngineAction::Noop => false,
    }
}

fn should_intercept_boundary(action: &EngineAction) -> bool {
    matches!(action, EngineAction::Replace { .. })
}

fn action_to_inputs(state: &mut HookState, action: EngineAction) -> Vec<INPUT> {
    let mut inputs = Vec::new();
    match action {
        EngineAction::Commit(text) | EngineAction::Preedit(text) => {
            push_text(state, &mut inputs, &text)
        }
        EngineAction::Replace {
            delete_graphemes,
            insert,
        } => {
            for _ in 0..delete_graphemes {
                push_backspace(state, &mut inputs);
            }
            push_text(state, &mut inputs, &insert);
        }
        EngineAction::Noop | EngineAction::ToggleVietnamese | EngineAction::BeepOrReject => {}
    }
    inputs
}

fn push_backspace(_state: &mut HookState, inputs: &mut Vec<INPUT>) {
    inputs.push(keyboard_input_ex(VK_BACK.0, KEYBD_EVENT_FLAGS(0)));
    inputs.push(keyboard_input_ex(VK_BACK.0, KEYEVENTF_KEYUP));
}

fn push_text(state: &mut HookState, inputs: &mut Vec<INPUT>, text: &str) {
    match state.send_strategy {
        SendStrategy::VirtualKeyStandard if text.is_ascii() => push_ascii_text(inputs, text),
        SendStrategy::ClipboardPaste
        | SendStrategy::UnicodeSendInput
        | SendStrategy::VirtualKeyStandard => {
            for unit in text.encode_utf16() {
                inputs.push(unicode_input(unit, KEYBD_EVENT_FLAGS(0)));
                inputs.push(unicode_input(unit, KEYEVENTF_KEYUP));
            }
        }
    }
}

fn push_ascii_text(inputs: &mut Vec<INPUT>, text: &str) {
    for ch in text.chars() {
        if let Some(vk) = ascii_vk(ch) {
            inputs.push(keyboard_input_ex(vk, KEYBD_EVENT_FLAGS(0)));
            inputs.push(keyboard_input_ex(vk, KEYEVENTF_KEYUP));
        } else {
            let mut buffer = [0; 2];
            for unit in ch.encode_utf16(&mut buffer) {
                inputs.push(unicode_input(*unit, KEYBD_EVENT_FLAGS(0)));
                inputs.push(unicode_input(*unit, KEYEVENTF_KEYUP));
            }
        }
    }
}

fn ascii_vk(ch: char) -> Option<u16> {
    match ch {
        'a'..='z' => Some(ch.to_ascii_uppercase() as u16),
        'A'..='Z' | '0'..='9' => Some(ch as u16),
        ' ' => Some(VK_SPACE.0),
        _ => None,
    }
}

fn send_inputs(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe {
        let _ = SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

fn keyboard_input_ex(
    vk: u16,
    flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS,
) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INJECTED_MAGIC,
            },
        },
    }
}

fn unicode_input(
    unit: u16,
    flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS,
) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: KEYEVENTF_UNICODE | flags,
                time: 0,
                dwExtraInfo: INJECTED_MAGIC,
            },
        },
    }
}

fn key_down(vk: u16) -> bool {
    let state = unsafe { GetKeyState(vk as i32) };
    state < 0
}

fn modifier_down_exact(vk: u16, expected: bool, current_vk: u32) -> bool {
    modifier_down(vk, current_vk) == expected
}

fn modifier_down(vk: u16, current_vk: u32) -> bool {
    key_down(vk) || modifier_matches(vk, current_vk)
}

fn modifier_matches(vk: u16, current_vk: u32) -> bool {
    match VIRTUAL_KEY(vk) {
        VK_CONTROL => {
            current_vk == VK_CONTROL.0 as u32
                || current_vk == VK_LCONTROL.0 as u32
                || current_vk == VK_RCONTROL.0 as u32
        }
        VK_SHIFT => {
            current_vk == VK_SHIFT.0 as u32
                || current_vk == VK_LSHIFT.0 as u32
                || current_vk == VK_RSHIFT.0 as u32
        }
        VK_MENU => {
            current_vk == VK_MENU.0 as u32
                || current_vk == VK_LMENU.0 as u32
                || current_vk == VK_RMENU.0 as u32
        }
        VK_LWIN | VK_RWIN => current_vk == VK_LWIN.0 as u32 || current_vk == VK_RWIN.0 as u32,
        _ => false,
    }
}

fn is_modifier_vk(vk: u32) -> bool {
    vk == VK_CONTROL.0 as u32
        || vk == VK_LCONTROL.0 as u32
        || vk == VK_RCONTROL.0 as u32
        || vk == VK_SHIFT.0 as u32
        || vk == VK_LSHIFT.0 as u32
        || vk == VK_RSHIFT.0 as u32
        || vk == VK_MENU.0 as u32
        || vk == VK_LMENU.0 as u32
        || vk == VK_RMENU.0 as u32
        || vk == VK_LWIN.0 as u32
        || vk == VK_RWIN.0 as u32
}

fn boundary_key(vk: u32) -> Option<KeyCode> {
    match VIRTUAL_KEY(vk as u16) {
        VK_SPACE => Some(KeyCode::Space),
        VK_RETURN => Some(KeyCode::Enter),
        VK_BACK => Some(KeyCode::Backspace),
        _ => None,
    }
}

fn vk_to_char(vk: u32, scan_code: u32) -> Option<char> {
    if (b'A' as u32..=b'Z' as u32).contains(&vk) {
        let shift_down = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
        let caps_on = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
        let mut ch = char::from_u32(vk)?.to_ascii_lowercase();
        if shift_down ^ caps_on {
            ch = ch.to_ascii_uppercase();
        }
        return Some(ch);
    }

    let mut keyboard_state = [0u8; 256];
    unsafe {
        if GetKeyboardState(&mut keyboard_state).is_err() {
            return None;
        }
    }

    let mut buffer = [0u16; 8];
    let len = unsafe { ToUnicode(vk, scan_code, Some(&keyboard_state), &mut buffer, 0) };

    if len != 1 {
        return None;
    }

    char::from_u32(buffer[0] as u32).filter(|ch| !ch.is_control())
}

#[cfg(test)]
mod tests {
    use super::{
        charset_from_function_key_value, function_key_command, send_strategy_from_config,
        should_intercept, KeyboardCommand, SendStrategy,
    };
    use vietkb_config::{FunctionKeyConfig, SystemConfig, TypingConfig};
    use vietkb_core::{Charset, EngineAction};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_F1, VK_F12, VK_F9};

    fn all_modifiers_off() -> FunctionKeyConfig {
        FunctionKeyConfig {
            ctrl: false,
            shift: false,
            alt: false,
            win: false,
            ..FunctionKeyConfig::default()
        }
    }

    #[test]
    fn send_strategy_prefers_clipboard_then_standard_then_unicode() {
        let typing_clip = TypingConfig {
            use_clipboard_to_send_keys: true,
            ..TypingConfig::default()
        };
        assert_eq!(
            send_strategy_from_config(&typing_clip, &SystemConfig::default()),
            SendStrategy::ClipboardPaste
        );

        let system_standard = SystemConfig {
            standard_key_sending: true,
            metro_apps_support: false,
            ..SystemConfig::default()
        };
        assert_eq!(
            send_strategy_from_config(&TypingConfig::default(), &system_standard),
            SendStrategy::VirtualKeyStandard
        );

        let system_plain = SystemConfig {
            standard_key_sending: false,
            metro_apps_support: false,
            ..SystemConfig::default()
        };
        assert_eq!(
            send_strategy_from_config(&TypingConfig::default(), &system_plain),
            SendStrategy::UnicodeSendInput
        );
    }

    #[test]
    fn function_key_command_maps_configured_actions() {
        let config = all_modifiers_off();
        assert_eq!(
            function_key_command(&config, VK_F1.0 as u32),
            Some(KeyboardCommand::SetEnabled(true))
        );
        assert_eq!(
            function_key_command(&config, VK_F12.0 as u32),
            Some(KeyboardCommand::Reset)
        );
        assert_eq!(function_key_command(&config, VK_F9.0 as u32), None);
    }

    #[test]
    fn charset_from_function_key_value_parses_known_labels() {
        assert_eq!(charset_from_function_key_value("VNI"), Charset::VniWindows);
        assert_eq!(charset_from_function_key_value("TCVN3"), Charset::Tcvn3);
        assert_eq!(charset_from_function_key_value("Unicode"), Charset::Unicode);
    }

    #[test]
    fn does_not_intercept_unchanged_plain_commits() {
        assert!(!should_intercept(
            't',
            &EngineAction::Commit("t".to_string())
        ));
    }

    #[test]
    fn intercepts_transformed_actions() {
        assert!(should_intercept(
            '1',
            &EngineAction::Replace {
                delete_graphemes: 4,
                insert: "thái".to_string(),
            },
        ));
    }
}
