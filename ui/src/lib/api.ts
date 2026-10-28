// One function per Tauri command. Nothing else in the UI calls `invoke`.
// Command names and payloads are frozen by contract §6; the two commands marked
// "additive" are proposed additions (reported to the Lead, never renamed).

import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfoView,
  AppRuleRow,
  MacroRow,
  RunningApp,
  SettingsView,
  StatusView,
} from "./bindings";

export type * from "./bindings";

export const isTauri = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

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
export const deleteAppRule = (process: string) => invoke<void>("delete_app_rule", { process });
export const listRunningApps = () => invoke<RunningApp[]>("list_running_apps");

/** Additive command: per-app charset/method overrides (contract §6 gap, see impl notes). */
export const setAppRuleOverrides = (
  process: string,
  charset: string | null,
  method: string | null,
) => invoke<void>("set_app_rule_overrides", { process, charset, method });

export const parseHotkey = (input: string) => invoke<string>("parse_hotkey", { input });
export const checkHotkeyConflict = (input: string, ignoreField: string) =>
  invoke<string | null>("check_hotkey_conflict", { input, ignoreField });

export const convertText = (input: string, from: string, to: string) =>
  invoke<string>("convert_text", { input, from, to });

export const openConfigFolder = () => invoke<void>("open_config_folder");
export const restartApp = () => invoke<void>("restart_app");
export const appInfo = () => invoke<AppInfoView>("app_info");
