// One function per Tauri command. Nothing else in the UI calls `invoke`.
// Command names and payloads are frozen by contract §6; the two commands marked
// "additive" are proposed additions (reported to the Lead, never renamed).

import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfoView,
  AppRuleRow,
  DownloadOutcome,
  MacroRow,
  RunningApp,
  SettingsView,
  StatusView,
  UpdateStatus,
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

/** Additive command: per-app method overrides (contract §6 gap, see impl notes). */
export const setAppRuleOverrides = (
  process: string,
  charset: string | null,
  method: string | null,
) => invoke<void>("set_app_rule_overrides", { process, charset, method });

export const parseHotkey = (input: string) => invoke<string>("parse_hotkey", { input });
export const checkHotkeyConflict = (input: string, ignoreField: string) =>
  invoke<string | null>("check_hotkey_conflict", { input, ignoreField });

export const openConfigFolder = () => invoke<void>("open_config_folder");
export const restartApp = () => invoke<void>("restart_app");
export const appInfo = () => invoke<AppInfoView>("app_info");

/** Additive: the `Run` key itself, read back after every write so the switch
 *  can never claim an autostart Windows does not have. */
export const autostartState = () => invoke<boolean>("autostart_state");
export const setAutostart = (enabled: boolean) =>
  invoke<boolean>("set_autostart", { enabled });

/** `"windows"`, `"macos"` or `"linux"`. */
export const platform = () => invoke<string>("platform");

/** macOS Accessibility grant. `true` always on Windows. */
export const accessibilityStatus = () => invoke<boolean>("accessibility_status");
/** Triggers the system prompt; answers with the status after asking. */
export const requestAccessibility = () => invoke<boolean>("request_accessibility");

/** macOS Input Monitoring grant. `true` always on Windows. */
export const inputMonitoringStatus = () => invoke<boolean>("input_monitoring_status");
/** Triggers the system prompt (once); answers with the status after asking. */
export const requestInputMonitoring = () => invoke<boolean>("request_input_monitoring");

/** Open the Accessibility or Input Monitoring pane of System Settings. */
export const openPrivacyPane = (pane: "accessibility" | "input_monitoring") =>
  invoke<void>("open_privacy_pane", { pane });

/** The hotkey capture field is focused: the hook suspends hotkeys while true. */
export const setHotkeyCapture = (active: boolean) =>
  invoke<void>("set_hotkey_capture", { active });

/** Window management is presentation only; contract §6 commands are untouched. */
export const openWindow = (kind: "hotkeys" | "advanced" | "macros" | "about") =>
  invoke<void>("open_window", { kind });
export const quitApp = () => invoke<void>("quit_app");

// ------------------------------------------------------------------ updates
// Same rule: the update surface is additive. GitHub is queried from Rust, never
// from the page (CSP stays `default-src 'self'`).

export const checkUpdate = () => invoke<UpdateStatus>("check_update");
export const updateStatus = () => invoke<UpdateStatus>("update_status");
export const downloadUpdate = () => invoke<DownloadOutcome>("download_update");
export const installUpdate = () => invoke<void>("install_update");

/** Opens one of the allow-listed links (repo, issues, Ko-fi) in the browser. */
export const openUrl = (url: string) => invoke<void>("open_url", { url });
