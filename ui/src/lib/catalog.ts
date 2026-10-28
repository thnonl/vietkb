// Product catalogues: input methods and hotkey actions.
// Vietnamese product terminology is kept verbatim from EVKey (contract C7).

export interface MethodInfo {
  slug: string;
  label: string;
  hint: string;
  example: string;
}

export const METHODS: MethodInfo[] = [
  {
    slug: "telex",
    label: "Telex",
    hint: "Gõ chữ cái để thêm dấu: aa → â, ow → ơ, s → dấu sắc.",
    example: "vieejt → việt",
  },
  {
    slug: "vni",
    label: "VNI",
    hint: "Gõ số để thêm dấu: a1 → á, a6 → â, o7 → ơ.",
    example: "vie6t5 → việt",
  },
];

export const methodLabel = (slug: string): string =>
  METHODS.find((m) => m.slug === slug)?.label ?? slug;

/** Two-line tone-style control (charter: name above, example below).
 *  `modern_tone = true` is kiểu mới — tone on the first vowel: hòa, thúy — as
 *  `vkb_core::tone::target` places it. */
export const TONE_STYLES = [
  { modern: true, name: "Kiểu mới", example: "hòa, thúy" },
  { modern: false, name: "Kiểu cũ", example: "hoà, thuý" },
] as const;

export type HotkeyGroup = "state" | "tools";

export interface HotkeyAction {
  field: HotkeyActionField;
  label: string;
  hint: string;
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

/** Mirrors `HotkeySet::default()` in crates/vkb-config/src/schema.rs. */
export const DEFAULT_HOTKEYS: Record<HotkeyActionField, string> = {
  enable: "F1",
  disable: "F2",
  toggle: "Ctrl+Shift",
  restoreWord: "Ctrl+Shift+Z",
  appExclude: "F11",
  macroPanel: "F8",
  macroToggle: "F9",
  openSettings: "Ctrl+Alt+S",
  cycleMethod: "",
};

export const HOTKEY_ACTIONS: HotkeyAction[] = [
  {
    field: "enable",
    label: "Bật tiếng Việt",
    hint: "Chuyển sang chế độ gõ tiếng Việt, giữ nguyên kiểu gõ đang chọn.",
    group: "state",
    fallback: "F1",
  },
  {
    field: "disable",
    label: "Tắt tiếng Việt",
    hint: "Trả bàn phím về gõ tiếng Anh bình thường.",
    group: "state",
    fallback: "F2",
  },
  {
    field: "toggle",
    label: "Chuyển bật / tắt",
    hint: "Một phím duy nhất để đổi trạng thái, tiện khi chỉ muốn một phím tắt.",
    group: "state",
    fallback: "",
  },
  {
    field: "restoreWord",
    label: "Phục hồi từ gõ sai",
    hint: "Khôi phục đúng các phím bạn vừa gõ khi từ bị nhận sai.",
    group: "state",
    fallback: "",
  },
  {
    field: "appExclude",
    label: "Bật / tắt loại trừ ứng dụng",
    hint: "Tạm bỏ qua danh sách loại trừ cho cửa sổ đang mở.",
    group: "state",
    fallback: "F11",
  },
  {
    field: "macroPanel",
    label: "Bảng gõ tắt",
    hint: "Mở nhanh danh sách gõ tắt để tra hoặc sửa.",
    group: "tools",
    fallback: "F8",
  },
  {
    field: "macroToggle",
    label: "Bật / tắt gõ tắt",
    hint: "Tắt gõ tắt mà không phải vào cài đặt.",
    group: "tools",
    fallback: "F9",
  },
  {
    field: "openSettings",
    label: "Mở cài đặt",
    hint: "Mở cửa sổ này từ bất cứ đâu.",
    group: "tools",
    fallback: "Ctrl+Alt+S",
  },
  {
    field: "cycleMethod",
    label: "Đổi kiểu gõ",
    hint: "Chuyển qua lại Telex ↔ VNI.",
    group: "tools",
    fallback: "",
  },
];

export const hotkeyActionLabel = (field: string): string =>
  HOTKEY_ACTIONS.find((a) => a.field === field)?.label ?? field;

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
