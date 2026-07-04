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
const defaults = document.querySelector("#defaults");
const openProcessPicker = document.querySelector("#openProcessPicker");
const processDialog = document.querySelector("#processDialog");
const closeProcessDialog = document.querySelector("#closeProcessDialog");
const doneProcessDialog = document.querySelector("#doneProcessDialog");
const addTypedProcess = document.querySelector("#addTypedProcess");
const processSearch = document.querySelector("#processSearch");
const processLoading = document.querySelector("#processLoading");
const processOptions = document.querySelector("#processOptions");
const excludedAppChips = document.querySelector("#excludedAppChips");
const excludedAppsEnabled = document.querySelector("#excludedAppsEnabled");
let knownProcesses = [];
let loadingProcesses = false;

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

function normalizeProcessName(value) {
  const name = value.trim().split(/[\\/]/).pop().toLowerCase();
  return name && !name.endsWith(".exe") ? `${name}.exe` : name;
}

function excludedAppNames() {
  return Object.entries(state.apps || {})
    .filter(([, config]) => config.enabled === false)
    .map(([name]) => name)
    .sort();
}

function addExcludedApp(value) {
  const name = normalizeProcessName(value);
  if (!name) return;
  state.apps ??= {};
  state.apps[name] = { ...(state.apps[name] || {}), enabled: false };
  processSearch.value = "";
  renderExcludedApps();
}

function removeExcludedApp(name) {
  if (!state.apps?.[name]) return;
  const next = { ...state.apps[name] };
  delete next.enabled;
  if (Object.keys(next).length) {
    state.apps[name] = next;
  } else {
    delete state.apps[name];
  }
  renderExcludedApps();
}

function renderExcludedApps() {
  const enabled = Boolean(state.system?.excludedApps);
  const selected = new Set(excludedAppNames());
  openProcessPicker.disabled = !enabled;
  excludedAppChips.classList.toggle("disabled", !enabled);
  excludedAppChips.innerHTML = "";
  for (const name of selected) {
    const chip = document.createElement("button");
    chip.type = "button";
    chip.className = "chip";
    chip.disabled = !enabled;
    chip.textContent = `${name} ×`;
    chip.addEventListener("click", () => removeExcludedApp(name));
    excludedAppChips.append(chip);
  }
  if (!selected.size) {
    const empty = document.createElement("span");
    empty.className = "muted";
    empty.textContent = enabled ? "Chưa chọn process" : "Danh sách loại trừ đang tắt";
    excludedAppChips.append(empty);
  }
  if (!enabled && !processDialog.hidden) hideProcessDialog();
  if (!processDialog.hidden) renderProcessOptions();
}

function renderProcessOptions() {
  const query = processSearch.value.trim().toLowerCase();
  const selected = new Set(excludedAppNames());
  const matches = knownProcesses
    .filter((name) => !selected.has(name) && (!query || name.includes(query)))
    .slice(0, 80);
  const typed = normalizeProcessName(processSearch.value);
  processOptions.innerHTML = "";
  if (typed && !selected.has(typed) && !matches.includes(typed)) {
    appendProcessOption(`Thêm “${typed}”`, typed);
  }
  for (const name of matches) appendProcessOption(name, name);
}

function appendProcessOption(label, value) {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = label;
  button.addEventListener("click", () => addExcludedApp(value));
  processOptions.append(button);
}

async function loadProcesses() {
  if (loadingProcesses) return;
  loadingProcesses = true;
  processLoading.hidden = false;
  processOptions.hidden = true;
  try {
    knownProcesses = await invoke("list_processes");
  } finally {
    loadingProcesses = false;
    processLoading.hidden = true;
    processOptions.hidden = false;
  }
  renderProcessOptions();
}

async function ensureProcessesLoaded() {
  if (!knownProcesses.length) await loadProcesses();
  renderProcessOptions();
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
  renderExcludedApps();
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
    if (element === excludedAppsEnabled) renderExcludedApps();
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

defaults.addEventListener("click", async () => {
  state = await invoke("default_settings");
  render();
  message.textContent = "Đã khôi phục mặc định (chưa lưu)";
  setTimeout(() => (message.textContent = ""), 1600);
});

function hideProcessDialog() {
  processDialog.hidden = true;
}

openProcessPicker.addEventListener("click", async () => {
  if (!state.system?.excludedApps) return;
  processDialog.hidden = false;
  processSearch.focus();
  await loadProcesses();
});

closeProcessDialog.addEventListener("click", hideProcessDialog);
doneProcessDialog.addEventListener("click", hideProcessDialog);
addTypedProcess.addEventListener("click", () => addExcludedApp(processSearch.value));
processDialog.addEventListener("click", (event) => {
  if (event.target === processDialog) hideProcessDialog();
});

processSearch.addEventListener("focus", () => void ensureProcessesLoaded());
processSearch.addEventListener("input", () => void ensureProcessesLoaded());
processSearch.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    addExcludedApp(processSearch.value);
  }
  if (event.key === "Escape") {
    hideProcessDialog();
  }
});

document.querySelector("#save").addEventListener("click", () => save(true));
document.querySelector("#close").addEventListener("click", () => invoke("close_window"));
document.querySelector("#exit").addEventListener("click", () => invoke("exit_app"));

applyTheme(localStorage.getItem("vietkb.darkMode") !== "false");
state = await invoke("load_settings");
render();
setInterval(refreshEnabled, 500);
