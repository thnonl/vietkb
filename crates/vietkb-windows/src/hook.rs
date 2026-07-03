use std::sync::{Arc, Mutex, OnceLock};

use vietkb_core::{
    EngineAction, InputMethod, KeyCode, KeyInput, SimpleVietnameseEngine, VietnameseEngine,
};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardState, SendInput, ToUnicode, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK,
    VK_CAPITAL, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_PACKET,
    VK_RCONTROL, VK_RETURN, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

static STATE: OnceLock<Mutex<HookState>> = OnceLock::new();

type EnabledCallback = Arc<dyn Fn(bool) + Send + Sync + 'static>;

struct HookState {
    engine: SimpleVietnameseEngine,
    enabled: bool,
    injecting: usize,
    hotkey: Hotkey,
    hotkey_down: bool,
    on_enabled_changed: Option<EnabledCallback>,
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

pub fn set_keyboard_config(
    enabled: bool,
    method: InputMethod,
    hotkey: &str,
    skip_english_words: bool,
) {
    let state = STATE.get_or_init(default_state);
    if let Ok(mut state) = state.lock() {
        state.enabled = enabled;
        state.hotkey = Hotkey::parse(hotkey);
        state.engine.set_input_method(method);
        state.engine.set_skip_english_words(skip_english_words);
    }
}

pub fn set_enabled_changed_callback(callback: impl Fn(bool) + Send + Sync + 'static) {
    let state = STATE.get_or_init(default_state);
    if let Ok(mut state) = state.lock() {
        state.on_enabled_changed = Some(Arc::new(callback));
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
        injecting: 0,
        hotkey: Hotkey::default(),
        hotkey_down: false,
        on_enabled_changed: None,
    })
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
            }
        }
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let info = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    if info.vkCode == VK_PACKET.0 as u32 || is_injected_key(&info) {
        if let Some(state) = STATE.get() {
            if let Ok(mut state) = state.lock() {
                state.injecting = state.injecting.saturating_sub(1);
            }
        }
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    if let Some(state) = STATE.get() {
        if let Ok(mut state) = state.lock() {
            if state.injecting > 0 {
                state.injecting -= 1;
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

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

            if !state.enabled {
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            if let Some(boundary) = boundary_key(info.vkCode) {
                let _ = state.engine.process_key(KeyInput::plain(boundary));
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

fn push_backspace(state: &mut HookState, inputs: &mut Vec<INPUT>) {
    state.injecting += 1;
    inputs.push(keyboard_input(VK_BACK.0, KEYBD_EVENT_FLAGS(0)));
    inputs.push(keyboard_input(VK_BACK.0, KEYEVENTF_KEYUP));
}

fn push_text(state: &mut HookState, inputs: &mut Vec<INPUT>, text: &str) {
    for unit in text.encode_utf16() {
        inputs.push(unicode_input(unit, KEYBD_EVENT_FLAGS(0)));
        inputs.push(unicode_input(unit, KEYEVENTF_KEYUP));
        state.injecting += 1;
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

fn keyboard_input(
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
                dwExtraInfo: 0,
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
                dwExtraInfo: 0,
            },
        },
    }
}

fn is_injected_key(info: &KBDLLHOOKSTRUCT) -> bool {
    const LLKHF_INJECTED: u32 = 0x10;
    info.flags.0 & LLKHF_INJECTED != 0
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
