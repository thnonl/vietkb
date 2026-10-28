// One function per Tauri command. Nothing else in the UI calls `invoke`.
// Types mirror crates/vkb-app/src/commands.rs (serde camelCase).

import { invoke } from "@tauri-apps/api/core";

export interface InputView {
  enabled: boolean;
  method: string;
  charset: string;
  mechanism: string;
}

export interface TypingView {
  modernTone: boolean;
  freeTone: boolean;
  jfwzAsConsonant: boolean;
  autoCapitalize: boolean;
  spellCheck: boolean;
  restoreWrongWord: boolean;
  disableOnNonUsLayout: boolean;
  suggestionFix1: boolean;
  suggestionFix2: boolean;
  metroFix: boolean;
  sendStandard: boolean;
  useClipboard: boolean;
  macroEnabled: boolean;
  macroWhenDisabled: boolean;
}

export interface HotkeyView {
  enable: string | null;
  disable: string | null;
  toggle: string | null;
  restoreWord: string | null;
  appExclude: string | null;
  convertTool: string | null;
  macroPanel: string | null;
  macroToggle: string | null;
  openSettings: string | null;
}

export interface SystemView {
  startWithWindows: boolean;
  runAsAdmin: boolean;
  notificationSound: boolean;
  startMinimized: boolean;
  language: string;
  theme: string;
  checkUpdates: boolean;
}

export interface UiView {
  windowWidth: number;
  windowHeight: number;
  lastTab: string;
}

export interface SettingsView {
  input: InputView;
  typing: TypingView;
  hotkeys: HotkeyView;
  system: SystemView;
  ui: UiView;
}

export interface StatusView {
  enabled: boolean;
  method: string;
  charset: string;
  mechanism: string;
  hookReady: boolean;
  version: string;
}

export interface MacroRow {
  trigger: string;
  expansion: string;
}

export interface AppRuleRow {
  process: string;
  enabled: boolean | null;
  charset: string | null;
  method: string | null;
}

export interface RunningApp {
  process: string;
  title: string;
}

export interface AppInfoView {
  version: string;
  arch: string;
  configPath: string;
  hookReady: boolean;
}

export const getSettings = () => invoke<SettingsView>("get_settings");
export const setSettings = (next: SettingsView) => invoke<SettingsView>("set_settings", { next });
export const patchSettings = (patch: unknown) =>
  invoke<SettingsView>("patch_settings", { patch });
export const resetSettings = (section: string) =>
  invoke<SettingsView>("reset_settings", { section });

export const getStatus = () => invoke<StatusView>("get_status");
export const toggleEnabled = () => invoke<StatusView>("toggle_enabled");
export const setMethod = (method: string) => invoke<StatusView>("set_method", { method });
export const setCharset = (charset: string) => invoke<StatusView>("set_charset", { charset });

export const listMacros = (query = "") => invoke<MacroRow[]>("list_macros", { query });
export const upsertMacro = (trigger: string, expansion: string) =>
  invoke<void>("upsert_macro", { trigger, expansion });
export const deleteMacros = (triggers: string[]) =>
  invoke<void>("delete_macros", { triggers });
export const importMacros = (path: string) => invoke<number>("import_macros", { path });
export const exportMacros = (path: string) => invoke<number>("export_macros", { path });

export const listAppRules = () => invoke<AppRuleRow[]>("list_app_rules");
export const setAppRule = (process: string, enabled: boolean) =>
  invoke<void>("set_app_rule", { process, enabled });
export const deleteAppRule = (process: string) =>
  invoke<void>("delete_app_rule", { process });
export const listRunningApps = () => invoke<RunningApp[]>("list_running_apps");

export const parseHotkey = (input: string) => invoke<string>("parse_hotkey", { input });
export const checkHotkeyConflict = (input: string, ignoreField: string) =>
  invoke<string | null>("check_hotkey_conflict", { input, ignoreField });

export const convertText = (input: string, from: string, to: string) =>
  invoke<string>("convert_text", { input, from, to });

export const openConfigFolder = () => invoke<void>("open_config_folder");
export const appInfo = () => invoke<AppInfoView>("app_info");
