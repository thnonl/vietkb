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

/** macOS: ⌘ combinations are the app shortcuts everybody has in their fingers,
 *  so they are warned about, never refused. Nothing is reserved. */
const MAC_COMMON: Record<string, string> = {
  "Win+C": "hk.common.copy",
  "Win+V": "hk.common.paste",
  "Win+X": "hk.common.cut",
  "Win+A": "hk.common.selectAll",
  "Win+S": "hk.common.save",
  "Win+F": "hk.common.find",
  "Win+Z": "hk.common.undo",
  "Win+Shift+Z": "hk.common.redo",
  "Win+Q": "hk.common.quit",
  "Win+W": "hk.common.closeWindow",
  "Win+Space": "hk.common.spotlight",
  "Ctrl+Space": "hk.common.macKeyboard",
};

export const reservedReason = (canonical: string): string | null => {
  if (isMacOS()) return null;
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

/** Key token for a keydown event, or null when the key cannot end a combo. */
export const keyToken = (event: KeyboardEvent): string | null => {
  // macOS (S-08): Option turns letters into composed characters (ß, ∑, dead
  // keys) in `event.key`, so a chord like ⌃⌥S could never be captured. The tap
  // matches physical US key positions anyway, so the physical code is the truth.
  // Windows keeps `event.key`: its hook matches layout-based virtual keys.
  if (isMacOS()) {
    const physical = /^(?:Key([A-Z])|Digit([0-9]))$/.exec(event.code ?? "");
    if (physical) return physical[1] ?? physical[2];
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
