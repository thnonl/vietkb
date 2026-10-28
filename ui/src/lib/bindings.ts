// Hand-written mirror of `crates/vkb-app/src/commands.rs` (serde camelCase).
// Reviewed by hand: there is no codegen step in Sprint 1. Any change here is a
// contract change — see docs/evkey-clone-work/25-contract.md §6.

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
  autoHornUo: boolean;
  rawZone: boolean;
  macroCaseFollow: boolean;
  sendDelayUs: number;
}

/** Every field of `HotkeySet`. Values are canonical strings (`Ctrl+Shift`) or
 *  `null` when the action has no shortcut. */
export interface HotkeyView {
  enable: string | null;
  disable: string | null;
  toggle: string | null;
  restoreWord: string | null;
  appExclude: string | null;
  macroPanel: string | null;
  macroToggle: string | null;
  openSettings: string | null;
  cycleMethod: string | null;
  toggleMode: string;
}

export type HotkeyField = keyof HotkeyView;

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

export interface AppsView {
  rememberPerApp: boolean;
}

export interface SettingsView {
  input: InputView;
  typing: TypingView;
  hotkeys: HotkeyView;
  system: SystemView;
  ui: UiView;
  apps: AppsView;
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

export type Theme = "system" | "light" | "dark";
/** The panel has exactly four tabs (charter C9). */
export type NavId = "go" | "macros" | "hotkeys" | "system";
