// Hotkey helpers shared by the capture field and the hotkey screen.
//
// `canonicalize` mirrors `Hotkey::parse` + `Hotkey::as_str` in
// crates/vkb-config/src/hotkey.rs so the field can show the stored form
// instantly; `parse_hotkey` remains the authority and is called on every change.

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
      if (key !== null) return { canonical: null, error: `Không nhận ra phím: ${token}` };
      const named = NAMED[lower] ?? (lower.startsWith("f") ? null : NAMED[lower]);
      if (named) key = named;
      else if (/^f([1-9]|1[0-9]|2[0-4])$/.test(lower)) key = lower.toUpperCase();
      else if (Array.from(token).length === 1) key = token.toUpperCase();
      else bad = true;
    }
  }

  if (bad) return { canonical: null, error: `Không nhận ra phím: ${trimmed}` };
  const hasModifier = ctrl || alt || shift || win;
  if (!hasModifier && key !== null && !/^F\d+$/.test(key)) {
    return {
      canonical: null,
      error: `Phím tắt cần ít nhất một phím Ctrl/Alt/Shift/Win: ${trimmed}`,
    };
  }
  if (!hasModifier && key === null) {
    return { canonical: null, error: "Phím tắt rỗng" };
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

/** Số lớn kiểu Việt: 148 → 148, 5000 → 5.000. */
export const formatNumber = (value: number): string => value.toLocaleString("vi-VN");

/** Combinations Windows owns; assignment is refused with a calm reason. */
const RESERVED: Record<string, string> = {
  "Win+L": "Windows đã dùng Win + L để khoá máy. Hãy nhấn tổ hợp khác.",
  "Alt+Tab": "Windows đã dùng Alt + Tab để chuyển cửa sổ. Hãy nhấn tổ hợp khác.",
  "Ctrl+Alt+Delete":
    "Windows đã dùng Ctrl + Alt + Del để mở màn hình bảo mật. Hãy nhấn tổ hợp khác.",
  "Ctrl+Shift+Escape":
    "Windows đã dùng Ctrl + Shift + Esc để mở Task Manager. Hãy nhấn tổ hợp khác.",
};

/** Common application shortcuts: allowed, but warned about (27 C#7). */
const COMMON: Record<string, string> = {
  "Ctrl+Z": "Hoàn tác",
  "Ctrl+Y": "Làm lại",
  "Ctrl+Shift+Z": "Làm lại",
  "Ctrl+C": "Sao chép",
  "Ctrl+V": "Dán",
  "Ctrl+X": "Cắt",
  "Ctrl+A": "Chọn tất cả",
  "Ctrl+S": "Lưu",
  "Ctrl+F": "Tìm",
  "Ctrl+Shift+T": "Mở lại tab",
  "Alt+Shift": "Đổi bàn phím của Windows",
  "Win+Space": "Đổi bàn phím của Windows",
  "Ctrl+Space": "Bộ gõ tiếng Trung, Nhật",
};

/** macOS: ⌘ combinations are the app shortcuts everybody has in their fingers,
 *  so they are warned about, never refused. Nothing is reserved. */
const MAC_COMMON: Record<string, string> = {
  "Win+C": "Sao chép",
  "Win+V": "Dán",
  "Win+X": "Cắt",
  "Win+A": "Chọn tất cả",
  "Win+S": "Lưu",
  "Win+F": "Tìm",
  "Win+Z": "Hoàn tác",
  "Win+Shift+Z": "Làm lại",
  "Win+Q": "Thoát ứng dụng",
  "Win+W": "Đóng cửa sổ",
  "Win+Space": "Spotlight",
  "Ctrl+Space": "Đổi bàn phím của macOS",
};

export const reservedReason = (canonical: string): string | null =>
  isMacOS() ? null : (RESERVED[canonical] ?? null);

export const commonWarning = (canonical: string): string | null => {
  const action = (isMacOS() ? MAC_COMMON : COMMON)[canonical];
  if (!action) return null;
  return `${prettyHotkey(canonical)} là phím ${action} ở hầu hết ứng dụng. Vẫn đặt được nếu bạn muốn.`;
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
