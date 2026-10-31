// Product catalogues: input methods and hotkey actions.
// Vietnamese product terminology is kept verbatim from EVKey (contract C7).

import { t } from "./lang.svelte";
import { detectOS } from "./platform.svelte";

export interface MethodInfo {
  slug: string;
  label: string;
  /** i18n key of the one-line hint. */
  hintKey: string;
  example: string;
}

export const METHODS: MethodInfo[] = [
  {
    slug: "telex",
    label: "Telex",
    hintKey: "method.telex.hint",
    example: "vieejt → việt",
  },
  {
    slug: "vni",
    label: "VNI",
    hintKey: "method.vni.hint",
    example: "vie6t5 → việt",
  },
];

export const methodLabel = (slug: string): string =>
  METHODS.find((m) => m.slug === slug)?.label ?? slug;

/** Two-line tone-style control (charter: name above, example below).
 *  `modern_tone = true` is kiểu mới — tone on the first vowel: hòa, thúy — as
 *  `vkb_core::tone::target` places it. */
export const TONE_STYLES = [
  { modern: true, nameKey: "tone.modern", example: "hòa, thúy" },
  { modern: false, nameKey: "tone.classic", example: "hoà, thuý" },
] as const;

export type HotkeyGroup = "state" | "tools";

export interface HotkeyAction {
  field: HotkeyActionField;
  /** i18n keys; resolve with `t()` at render time so a language switch applies. */
  labelKey: string;
  hintKey: string;
  group: HotkeyGroup;
  /** `""` means "no shortcut by default". */
  fallback: string;
}

export type HotkeyActionField =
  | "enable"
  | "disable"
  | "toggle"
  | "restoreWord"
  | "appExclude"
  | "macroPanel"
  | "macroToggle"
  | "openSettings"
  | "cycleMethod";

const MAC = detectOS() === "macos";

/** Mirrors `HotkeySet::default()` in crates/vkb-config/src/schema.rs. On macOS
 *  the function-key defaults are empty (27-mac-decisions M2): they are
 *  brightness/media keys there, and F11 is Show Desktop. */
export const DEFAULT_HOTKEYS: Record<HotkeyActionField, string> = {
  enable: MAC ? "" : "F1",
  disable: MAC ? "" : "F2",
  toggle: "Ctrl+Shift",
  restoreWord: "Ctrl+Shift+Z",
  appExclude: MAC ? "" : "F11",
  macroPanel: MAC ? "" : "F8",
  macroToggle: MAC ? "" : "F9",
  openSettings: "Ctrl+Alt+S",
  cycleMethod: "",
};

export const HOTKEY_ACTIONS: HotkeyAction[] = [
  {
    field: "enable",
    labelKey: "hotkey.enable.label",
    hintKey: "hotkey.enable.hint",
    group: "state",
    fallback: "F1",
  },
  {
    field: "disable",
    labelKey: "hotkey.disable.label",
    hintKey: "hotkey.disable.hint",
    group: "state",
    fallback: "F2",
  },
  {
    field: "toggle",
    labelKey: "hotkey.toggle.label",
    hintKey: "hotkey.toggle.hint",
    group: "state",
    fallback: "",
  },
  {
    field: "restoreWord",
    labelKey: "hotkey.restoreWord.label",
    hintKey: "hotkey.restoreWord.hint",
    group: "state",
    fallback: "",
  },
  {
    field: "appExclude",
    labelKey: "hotkey.appExclude.label",
    hintKey: "hotkey.appExclude.hint",
    group: "state",
    fallback: "F11",
  },
  {
    field: "macroPanel",
    labelKey: "hotkey.macroPanel.label",
    hintKey: "hotkey.macroPanel.hint",
    group: "tools",
    fallback: "F8",
  },
  {
    field: "macroToggle",
    labelKey: "hotkey.macroToggle.label",
    hintKey: "hotkey.macroToggle.hint",
    group: "tools",
    fallback: "F9",
  },
  {
    field: "openSettings",
    labelKey: "hotkey.openSettings.label",
    hintKey: "hotkey.openSettings.hint",
    group: "tools",
    fallback: "Ctrl+Alt+S",
  },
  {
    field: "cycleMethod",
    labelKey: "hotkey.cycleMethod.label",
    hintKey: "hotkey.cycleMethod.hint",
    group: "tools",
    fallback: "",
  },
];

export const hotkeyActionLabel = (field: string): string => {
  const action = HOTKEY_ACTIONS.find((a) => a.field === field);
  return action ? t(action.labelKey) : field;
};

/** Safe keys to suggest: system F-keys we never bind by default, in EVKey spirit. */
export const HOTKEY_SUGGESTIONS = [
  "F3",
  "F4",
  "F5",
  "F7",
  "F10",
  "F12",
  "Ctrl+Shift+V",
  "Ctrl+Alt+V",
  "Ctrl+Alt+S",
  "Alt+Z",
  "Ctrl+`",
];
