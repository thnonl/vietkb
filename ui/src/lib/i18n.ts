// Minimal i18n for the shell and common controls.
//
// Contract C7: product terminology (kiểu gõ, bảng mã, gõ tắt, …) stays
// Vietnamese at all times. Long explanatory copy is Vietnamese-only in
// Sprint 1 — recorded as a deviation in 70-impl-notes-ui.md.

export type Lang = "vi" | "en";

type Dict = Record<string, string>;

const vi: Dict = {
  "nav.go": "Gõ",
  "nav.macros": "Gõ tắt",
  "nav.hotkeys": "Phím tắt",
  "nav.system": "Hệ thống",

  "app.tagline": "Bộ gõ tiếng Việt",
  "app.theme": "Giao diện",
  "app.theme.system": "Theo hệ thống",
  "app.theme.light": "Sáng",
  "app.theme.dark": "Tối",
  "app.previewMode": "Chế độ xem trước — không có backend. Thay đổi sẽ không được lưu.",

  "common.on": "Bật",
  "common.off": "Tắt",
  "common.save": "Lưu",
  "common.cancel": "Huỷ",
  "common.delete": "Xoá",
  "common.edit": "Sửa",
  "common.add": "Thêm",
  "common.close": "Đóng",
  "common.search": "Tìm kiếm",
  "common.default": "Về mặc định",
  "common.advanced": "Nâng cao",
  "common.copy": "Sao chép",
  "common.copied": "Đã sao chép",
  "common.import": "Nhập tệp",
  "common.export": "Xuất tệp",
  "common.none": "Chưa gán",
  "common.openConfig": "Mở thư mục cấu hình",
  "common.rows": "dòng",
  "common.loading": "Đang tải…",
  "common.retry": "Thử lại",
  "common.resetSection": "Đặt lại mục này",
};

const en: Dict = {
  "nav.go": "Typing",
  "nav.macros": "Macros",
  "nav.hotkeys": "Hotkeys",
  "nav.system": "System",

  "app.tagline": "Vietnamese input method",
  "app.theme": "Appearance",
  "app.theme.system": "Match system",
  "app.theme.light": "Light",
  "app.theme.dark": "Dark",
  "app.previewMode": "Preview mode — no backend attached. Changes are not saved.",

  "common.on": "On",
  "common.off": "Off",
  "common.save": "Save",
  "common.cancel": "Cancel",
  "common.delete": "Delete",
  "common.edit": "Edit",
  "common.add": "Add",
  "common.close": "Close",
  "common.search": "Search",
  "common.default": "Reset to default",
  "common.advanced": "Advanced",
  "common.copy": "Copy",
  "common.copied": "Copied",
  "common.import": "Import file",
  "common.export": "Export file",
  "common.none": "Not set",
  "common.openConfig": "Open config folder",
  "common.rows": "rows",
  "common.loading": "Loading…",
  "common.retry": "Retry",
  "common.resetSection": "Reset this section",
};

const DICTS: Record<Lang, Dict> = { vi, en };

export const isLang = (value: unknown): value is Lang => value === "vi" || value === "en";

/** Translator factory. Unknown keys fall back to Vietnamese, then to the key. */
export const translator = (lang: Lang) => (key: string): string =>
  DICTS[lang][key] ?? DICTS.vi[key] ?? key;
