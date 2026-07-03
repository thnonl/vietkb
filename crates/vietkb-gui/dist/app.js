const invoke = window.__TAURI__.core.invoke;

let state = null;

const themeToggle = document.querySelector("#themeToggle");
const enabled = document.querySelector("#enabled");
const statusText = document.querySelector("#statusText");
const charset = document.querySelector("#charset");
const method = document.querySelector("#method");
const toggleHotkey = document.querySelector("#toggleHotkey");
const restoreHotkey = document.querySelector("#restoreHotkey");
const message = document.querySelector("#message");

function getByPath(obj, path) {
  return path.split(".").reduce((value, key) => value?.[key], obj);
}

function setByPath(obj, path, value) {
  const keys = path.split(".");
  const last = keys.pop();
  const target = keys.reduce((value, key) => value[key], obj);
  target[last] = value;
}

function syncHotkeyChecks(name, value) {
  const parts = value.split("+").map((part) => part.trim().toLowerCase());
  document.querySelectorAll(`[data-mod="${name}"]`).forEach((checkbox) => {
    checkbox.checked = parts.includes(checkbox.value.toLowerCase());
  });
}

function readHotkeyChecks(name, fallback) {
  const parts = [...document.querySelectorAll(`[data-mod="${name}"]:checked`)].map((checkbox) => checkbox.value);
  return parts.length ? parts.join("+") : fallback;
}

function applyTheme(isDark) {
  document.body.classList.toggle("light", !isDark);
  themeToggle.checked = isDark;
  localStorage.setItem("vietkb.darkMode", String(isDark));
}

function render() {
  enabled.checked = state.enabled;
  statusText.textContent = state.enabled ? "Đang bật" : "Đang tắt";
  charset.value = state.charset;
  method.value = state.method;
  toggleHotkey.value = state.hotkeys.toggle;
  restoreHotkey.value = state.hotkeys.restoreWord;
  syncHotkeyChecks("toggle", state.hotkeys.toggle);
  syncHotkeyChecks("restore", state.hotkeys.restoreWord);

  document.querySelectorAll("[data-path]").forEach((element) => {
    const value = getByPath(state, element.dataset.path);
    if (element.type === "checkbox") {
      element.checked = Boolean(value);
    } else {
      element.value = value;
    }
  });
}

async function save(showMessage = true) {
  state.charset = charset.value;
  state.method = method.value;
  state.hotkeys.toggle = toggleHotkey.value.trim() || readHotkeyChecks("toggle", "Ctrl+Shift");
  state.hotkeys.restoreWord = restoreHotkey.value.trim() || readHotkeyChecks("restore", "Shift+Alt");
  await invoke("save_settings", { settings: state });
  if (showMessage) {
    message.textContent = "Đã lưu";
    setTimeout(() => (message.textContent = ""), 1600);
  }
}

async function refreshEnabled() {
  state.enabled = await invoke("keyboard_enabled");
  enabled.checked = state.enabled;
  statusText.textContent = state.enabled ? "Đang bật" : "Đang tắt";
}

document.querySelectorAll("[data-tab]").forEach((button) => {
  button.addEventListener("click", () => {
    document.querySelectorAll("[data-tab]").forEach((item) => item.classList.toggle("active", item === button));
    document.querySelectorAll(".tab-pane").forEach((pane) => pane.classList.remove("active"));
    document.querySelector(`#tab-${button.dataset.tab}`).classList.add("active");
  });
});

document.addEventListener("change", (event) => {
  const element = event.target;

  if (element === themeToggle) {
    applyTheme(themeToggle.checked);
    return;
  }

  if (!state) return;

  if (element === enabled) {
    // Bật/tắt gõ tiếng Việt áp dụng ngay (không ghi các setting chưa lưu khác).
    state.enabled = enabled.checked;
    statusText.textContent = state.enabled ? "Đang bật" : "Đang tắt";
    void invoke("set_enabled", { enabled: state.enabled });
    return;
  }

  if (element === charset || element === method) {
    state.charset = charset.value;
    state.method = method.value;
    return;
  }

  if (element.dataset.path) {
    setByPath(state, element.dataset.path, element.type === "checkbox" ? element.checked : element.value);
    return;
  }

  if (element.dataset.mod) {
    const hotkey = readHotkeyChecks(element.dataset.mod, element.dataset.mod === "toggle" ? "Ctrl+Shift" : "Shift+Alt");
    if (element.dataset.mod === "toggle") {
      state.hotkeys.toggle = hotkey;
      toggleHotkey.value = hotkey;
    } else {
      state.hotkeys.restoreWord = hotkey;
      restoreHotkey.value = hotkey;
    }
  }
});

document.querySelector("#save").addEventListener("click", () => save(true));
document.querySelector("#close").addEventListener("click", () => invoke("close_window"));
document.querySelector("#exit").addEventListener("click", () => invoke("exit_app"));

applyTheme(localStorage.getItem("vietkb.darkMode") !== "false");
state = await invoke("load_settings");
render();
setInterval(refreshEnabled, 500);
