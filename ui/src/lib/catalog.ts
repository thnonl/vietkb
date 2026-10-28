// Product catalogues: charsets, input methods, hotkey actions.
// Vietnamese product terminology is kept verbatim from EVKey (contract C7).

export interface CharsetInfo {
  slug: string;
  label: string;
  /** Short name for the strip chip and the Thử gõ hint. */
  short: string;
  hint: string;
  tone: string;
  /** Offered in the panel pickers (decision 27 C#5/#6: six verified charsets). */
  picker: boolean;
  /** Convertible by `convert_text` in Sprint 1. */
  convertible: boolean;
}

export const CHARSETS: CharsetInfo[] = [
  {
    slug: "unicode",
    label: "Unicode dựng sẵn",
    short: "Unicode",
    hint: "Dùng cho hầu hết ứng dụng: Word, Zalo, trình duyệt",
    tone: "Tiếng Việt",
    picker: true,
    convertible: true,
  },
  {
    slug: "unicode_nfd",
    label: "Unicode tổ hợp",
    short: "Unicode tổ hợp",
    hint: "Chỉ chọn khi phần mềm yêu cầu",
    tone: "Tiếng Việt",
    picker: true,
    convertible: true,
  },
  {
    slug: "tcvn3",
    label: "TCVN3 (ABC)",
    short: "TCVN3",
    hint: "Văn bản cũ dùng phông .VnTime, .VnArial",
    tone: "TiÕng ViÖt",
    picker: true,
    convertible: true,
  },
  {
    slug: "vni_windows",
    label: "VNI Windows",
    short: "VNI Windows",
    hint: "Văn bản cũ dùng phông VNI-Times",
    tone: "Tieáng Vieät",
    picker: true,
    convertible: true,
  },
  {
    slug: "viqr",
    label: "VIQR",
    short: "VIQR",
    hint: "Dấu viết bằng ký hiệu: việt thành vie^.t",
    tone: "Tieengs Vieetj",
    picker: true,
    convertible: true,
  },
  {
    slug: "viscii",
    label: "VISCII",
    short: "VISCII",
    hint: "Văn bản cũ dùng phông VISCII",
    tone: "Tiếng Việt",
    picker: true,
    convertible: false,
  },
  {
    // Hidden from every picker (27 C#5): kept for config compatibility only.
    slug: "utf8",
    label: "UTF-8",
    short: "UTF-8",
    hint: "Chỉ dành cho tệp UTF-8",
    tone: "Tiếng Việt",
    picker: false,
    convertible: true,
  },
  {
    // Hidden until byte tables are verified (27 C#6).
    slug: "bk_hcm1",
    label: "BK HCM1",
    short: "BK HCM1",
    hint: "Chưa hỗ trợ trong bản này",
    tone: "Tiếng Việt",
    picker: false,
    convertible: false,
  },
  {
    slug: "vps",
    label: "VPS",
    short: "VPS",
    hint: "Chưa hỗ trợ trong bản này",
    tone: "Tiếng Việt",
    picker: false,
    convertible: false,
  },
];

export const PICKER_CHARSETS = CHARSETS.filter((charset) => charset.picker);

export const charsetLabel = (slug: string): string =>
  CHARSETS.find((c) => c.slug === slug)?.label ?? slug;

export const charsetShort = (slug: string): string =>
  CHARSETS.find((c) => c.slug === slug)?.short ?? slug;

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
  {
    slug: "viqr",
    label: "VIQR",
    hint: "Gõ dấu bằng ký tự: a? → ả, a( → ă, dd → đ.",
    example: "vie^.t5 → việt",
  },
];

export const methodLabel = (slug: string): string =>
  METHODS.find((m) => m.slug === slug)?.label ?? slug;

/** Two-line tone-style control (charter: name above, example below).
 *  `modern_tone = true` is kiểu mới — hoà, thuý — per 20-spec §B.1. */
export const TONE_STYLES = [
  { modern: true, name: "Kiểu mới", example: "hoà, thuý" },
  { modern: false, name: "Kiểu cũ", example: "hòa, thúy" },
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
  | "convertTool"
  | "macroPanel"
  | "macroToggle"
  | "openSettings"
  | "cycleCharset"
  | "cycleMethod";

/** Mirrors `HotkeySet::default()` in crates/vkb-config/src/schema.rs. */
export const DEFAULT_HOTKEYS: Record<HotkeyActionField, string> = {
  enable: "F1",
  disable: "F2",
  toggle: "Ctrl+Shift",
  restoreWord: "Ctrl+Shift+Z",
  appExclude: "F11",
  convertTool: "F6",
  macroPanel: "F8",
  macroToggle: "F9",
  openSettings: "Ctrl+Alt+S",
  cycleCharset: "",
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
    field: "convertTool",
    label: "Công cụ chuyển mã",
    hint: "Mở hộp thoại chuyển bảng mã cho văn bản đang bôi đen.",
    group: "tools",
    fallback: "F6",
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
    field: "cycleCharset",
    label: "Đổi bảng mã",
    hint: "Chuyển vòng Unicode → TCVN3 → VNI Windows → Unicode tổ hợp.",
    group: "tools",
    fallback: "",
  },
  {
    field: "cycleMethod",
    label: "Đổi kiểu gõ",
    hint: "Chuyển vòng Telex → VNI → VIQR.",
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
