// Hotkey helpers shared by the capture field and the hotkey screen.
//
// `canonicalize` mirrors `Hotkey::parse` + `Hotkey::as_str` in
// crates/vkb-config/src/hotkey.rs so the field can show the stored form
// instantly; `parse_hotkey` remains the authority and is called on every change.

import { langState, t } from "./lang.svelte";
import { isMacOS } from "./platform.svelte";

const NAMED: Record<string, string> = {
  " ": "Space",
  spacebar: "Space",
  enter: "Enter",
  return: "Enter",
  tab: "Tab",
  esc: "Escape",
  escape: "Escape",
  backspace: "Backspace",
  bksp: "Backspace",
  delete: "Delete",
  del: "Delete",
  insert: "Insert",
  ins: "Insert",
  home: "Home",
  end: "End",
  pageup: "PageUp",
  pgup: "PageUp",
  pagedown: "PageDown",
  pgdn: "PageDown",
  up: "Up",
  arrowup: "Up",
  down: "Down",
  arrowdown: "Down",
  left: "Left",
  arrowleft: "Left",
  right: "Right",
  arrowright: "Right",
  backquote: "`",
  minus: "-",
  equal: "=",
  comma: ",",
  period: ".",
  slash: "/",
  semicolon: ";",
  bracketleft: "[",
  backslash: "\\",
  bracketright: "]",
  quote: "'",
};

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "AltGraph", "Meta", "OS"]);

export const isModifierKey = (key: string): boolean => MODIFIER_KEYS.has(key);

export interface CanonicalResult {
  canonical: string | null;
  error: string | null;
}

export const canonicalize = (input: string): CanonicalResult => {
  const trimmed = input.trim();
  if (!trimmed) return { canonical: null, error: null };

  let ctrl = false;
  let alt = false;
  let shift = false;
  let win = false;
  let key: string | null = null;
  let bad = false;

  for (const raw of trimmed.split("+")) {
    const token = raw.trim();
    if (!token) continue;
    const lower = token.toLowerCase();
    if (lower === "ctrl" || lower === "control") ctrl = true;
    else if (lower === "alt") alt = true;
    else if (lower === "shift") shift = true;
    else if (lower === "win" || lower === "windows" || lower === "super" || lower === "meta")
      win = true;
    else {
      if (key !== null) return { canonical: null, error: t("hk.err.unknownKey", { key: token }) };
      const named = NAMED[lower] ?? (lower.startsWith("f") ? null : NAMED[lower]);
      if (named) key = named;
      else if (/^f([1-9]|1[0-9]|2[0-4])$/.test(lower)) key = lower.toUpperCase();
      else if (Array.from(token).length === 1) key = token.toUpperCase();
      else bad = true;
    }
  }

  if (bad) return { canonical: null, error: t("hk.err.unknownKey", { key: trimmed }) };
  const hasModifier = ctrl || alt || shift || win;
  if (!hasModifier && key !== null && !/^F\d+$/.test(key)) {
    return {
      canonical: null,
      // macOS names the modifiers as the keyboard prints them (⌃ ⌥ ⇧ ⌘).
      error: isMacOS()
        ? t("hk.err.needModifier.mac", { key: trimmed })
        : t("hk.err.needModifier", { key: trimmed }),
    };
  }
  if (!hasModifier && key === null) {
    return { canonical: null, error: t("hk.err.empty") };
  }

  const parts: string[] = [];
  if (ctrl) parts.push("Ctrl");
  if (alt) parts.push("Alt");
  if (shift) parts.push("Shift");
  if (win) parts.push("Win");
  if (key) parts.push(key);
  return { canonical: parts.join("+"), error: null };
};

/** Mac symbols for the stored modifier names (27-mac-decisions M2):
 *  Ctrl = ⌃ Control, Alt = ⌥ Option, Shift = ⇧, Win = ⌘ Command. */
const MAC_SYMBOL: Record<string, string> = {
  Ctrl: "⌃",
  Alt: "⌥",
  Shift: "⇧",
  Win: "⌘",
  Backspace: "⌫",
};

/** One key of a stored hotkey as it is drawn on a keycap. */
export const keyLabel = (token: string): string =>
  isMacOS() ? (MAC_SYMBOL[token] ?? token) : token;

/** `Ctrl+Shift` → `Ctrl + Shift` for prose, never for the stored value.
 *  On macOS the symbols run together, as System Settings writes them: `⌃⇧Z`. */
export const prettyHotkey = (value: string): string =>
  isMacOS()
    ? value.split("+").map(keyLabel).join("")
    : value.replaceAll("+", " + ");

/** Grouped digits by UI language: vi 5.000, en 5,000. */
export const formatNumber = (value: number): string =>
  value.toLocaleString(langState.lang === "en" ? "en-US" : "vi-VN");

/** Combinations Windows owns; assignment is refused with a calm reason (i18n keys). */
const RESERVED: Record<string, string> = {
  "Win+L": "hk.reserved.winL",
  "Alt+Tab": "hk.reserved.altTab",
  "Ctrl+Alt+Delete": "hk.reserved.cad",
  "Ctrl+Shift+Escape": "hk.reserved.taskmgr",
};

/** Common application shortcuts: allowed, but warned about (27 C#7). Values are i18n keys. */
const COMMON: Record<string, string> = {
  "Ctrl+Z": "hk.common.undo",
  "Ctrl+Y": "hk.common.redo",
  "Ctrl+Shift+Z": "hk.common.redo",
  "Ctrl+C": "hk.common.copy",
  "Ctrl+V": "hk.common.paste",
  "Ctrl+X": "hk.common.cut",
  "Ctrl+A": "hk.common.selectAll",
  "Ctrl+S": "hk.common.save",
  "Ctrl+F": "hk.common.find",
  "Ctrl+Shift+T": "hk.common.reopenTab",
  "Alt+Shift": "hk.common.winKeyboard",
  "Win+Space": "hk.common.winKeyboard",
  "Ctrl+Space": "hk.common.cjk",
};

/** macOS: app shortcuts the system does not own: allowed, but warned about.
 *  Keys are canonical strings (modifier order Ctrl, Alt, Shift, Win). */
const MAC_COMMON: Record<string, string> = {
  "Win+S": "hk.common.save",
  "Win+F": "hk.common.find",
  "Shift+Win+Z": "hk.common.redo",
};

/** macOS: chords the system or every app's menu owns. The tap cannot reliably
 *  win against them (and the page never receives some), so they are refused
 *  with a reason. Values are i18n keys naming what the chord does. */
const MAC_RESERVED: Record<string, string> = {
  "Win+Q": "hk.common.quit",
  "Win+W": "hk.common.closeWindow",
  "Win+H": "hk.sys.hide",
  "Win+M": "hk.sys.minimize",
  "Win+,": "hk.sys.settings",
  "Win+`": "hk.sys.switchWindows",
  "Win+Space": "hk.common.spotlight",
  "Ctrl+Space": "hk.common.macKeyboard",
  "Ctrl+Alt+Space": "hk.common.macKeyboard",
  "Ctrl+Win+Q": "hk.sys.lockScreen",
  "Ctrl+Win+Space": "hk.sys.characterViewer",
  "Shift+Win+Q": "hk.sys.logOut",
  "Ctrl+Win+F": "hk.sys.fullScreen",
  "Shift+Win+3": "hk.sys.screenshot",
  "Shift+Win+4": "hk.sys.screenshot",
  "Shift+Win+5": "hk.sys.screenshot",
  "Ctrl+Up": "hk.sys.missionControl",
  "Ctrl+Down": "hk.sys.missionControl",
  "Ctrl+Left": "hk.sys.missionControl",
  "Ctrl+Right": "hk.sys.missionControl",
  "Win+C": "hk.common.copy",
  "Win+V": "hk.common.paste",
  "Win+X": "hk.common.cut",
  "Win+Z": "hk.common.undo",
  "Win+A": "hk.common.selectAll",
  F11: "hk.sys.showDesktop",
};

export const reservedReason = (canonical: string): string | null => {
  if (isMacOS()) {
    const action = MAC_RESERVED[canonical];
    return action
      ? t("hk.reserved.mac", { combo: prettyHotkey(canonical), action: t(action) })
      : null;
  }
  const key = RESERVED[canonical];
  return key ? t(key) : null;
};

export const commonWarning = (canonical: string): string | null => {
  const actionKey = (isMacOS() ? MAC_COMMON : COMMON)[canonical];
  if (!actionKey) return null;
  return t("hk.warning", { combo: prettyHotkey(canonical), action: t(actionKey) });
};

/** A hotkey that cannot work on Windows (reserved) or may collide (common app). */
export const hotkeyHazard = (
  canonical: string,
): { kind: "reserved" | "common"; text: string } | null => {
  const reserved = reservedReason(canonical);
  if (reserved) return { kind: "reserved", text: reserved };
  const common = commonWarning(canonical);
  if (common) return { kind: "common", text: common };
  return null;
};

export const modifiersFrom = (event: KeyboardEvent): string => {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Win");
  return parts.join("+");
};

/** Physical keys by `event.code` on macOS, named as the canonical string (and
 *  `Hotkey::parse` in vkb-config) names them. */
const MAC_CODE_TOKENS: Record<string, string> = {
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backslash: "\\",
  Backquote: "`",
  Space: "Space",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  Insert: "Insert",
  Help: "Insert",
};

/** macOS only. The token of the physical key; `null` for a key that cannot be
 *  a hotkey (keypad, ISO/JIS extras: the tap reports other keys for them); or
 *  `undefined` for an unknown code, where `event.key` decides. */
const macKeyFromCode = (code: string): string | null | undefined => {
  const physical = /^(?:Key([A-Z])|Digit([0-9]))$/.exec(code);
  if (physical) return physical[1] ?? physical[2];
  const named = MAC_CODE_TOKENS[code];
  if (named) return named;
  const fn = /^F([1-9]|1[0-9]|20)$/.exec(code);
  if (fn) return `F${fn[1]}`;
  if (code.startsWith("Numpad") || code.startsWith("Intl")) return null;
  return undefined;
};

/** macOS: a keypad or ISO/JIS extra key, which cannot be a hotkey. */
export const isUnusableMacKey = (event: KeyboardEvent): boolean =>
  isMacOS() && macKeyFromCode(event.code ?? "") === null;

/** Key token for a keydown event, or null when the key cannot end a combo. */
export const keyToken = (event: KeyboardEvent): string | null => {
  // macOS (S-08): Option turns letters into composed characters (ß, ∑, dead
  // keys) in `event.key`, so a chord like ⌃⌥S could never be captured. The tap
  // matches physical US key positions anyway, so the physical keycode is the truth.
  // Windows keeps `event.key`: its hook matches layout-based virtual keys.
  if (isMacOS()) {
    const mapped = macKeyFromCode(event.code ?? "");
    if (mapped !== undefined) return mapped;
  }
  const key = event.key;
  if (!key || isModifierKey(key)) return null;
  if (key === "Enter" || key === "Escape" || key === "Backspace" || key === "Tab") return null;
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(key)) return key.toUpperCase();
  const named = NAMED[key.toLowerCase()];
  if (named) return named;
  if (Array.from(key).length === 1) return key.toUpperCase();
  return null;
};

export const comboFromEvent = (event: KeyboardEvent): string | null => {
  const key = keyToken(event);
  if (!key) return null;
  const mods = modifiersFrom(event);
  return mods ? `${mods}+${key}` : key;
};

/** True when the value is one of the documented defaults. */
export const isSameHotkey = (a: string | null, b: string | null): boolean =>
  (a ?? "") === (b ?? "");
