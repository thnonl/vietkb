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
  sendStandard: boolean;
  useClipboard: boolean;
  macroEnabled: boolean;
  macroWhenDisabled: boolean;
  autoHornUo: boolean;
  macroCaseFollow: boolean;
  rawZone?: boolean; // reserved, not implemented
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
  /** macOS: secure event input is making the tap pass keys through right now. */
  secureInputBlocking?: boolean;
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

export type Theme = "light" | "dark";
/** The panel has exactly four tabs (charter C9). */
export type NavId = "go" | "macros" | "hotkeys" | "system";

// ---------------------------------------------------------------- updates

export type UpdateState = "idle" | "checking" | "none" | "available" | "error";

export interface UpdateAsset {
  name: string;
  url: string;
  size: number;
}

/** One GitHub release that is newer than this build. */
export interface UpdateInfo {
  version: string;
  tag: string;
  changelog: string;
  publishedAt: string | null;
  htmlUrl: string;
  assets: UpdateAsset[];
  assetName: string | null;
  assetUrl: string | null;
  assetSize: number | null;
}

export interface UpdateStatus {
  state: UpdateState;
  currentVersion: string;
  update: UpdateInfo | null;
  /** Readable reason for `state === "error"` (network, rate limit, …). */
  message: string | null;
  checkedAt: number | null;
}

export interface DownloadOutcome {
  path: string;
  version: string;
  /** `false` when the release notes carried no `SHA256:` line. */
  verified: boolean;
}
