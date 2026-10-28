// Global UI store (Svelte 5 runes).
//
// Live apply: every `patch` merges into a pending object, updates the local view
// immediately (so controls never lag behind the mouse) and flushes to
// `patch_settings` after 150 ms. A failed flush reloads the authoritative
// settings and surfaces a toast — no Save button anywhere in the app.

import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import * as api from "./api";
import type {
  AppInfoView,
  AppsView,
  HotkeyField,
  InputView,
  NavId,
  SettingsView,
  StatusView,
  SystemView,
  TypingView,
  UiView,
} from "./bindings";
import { isLang, translator, type Lang } from "./i18n";

export interface SettingsPatch {
  input?: Partial<InputView>;
  typing?: Partial<TypingView>;
  hotkeys?: Partial<Record<HotkeyField, string>>;
  system?: Partial<SystemView>;
  ui?: Partial<UiView>;
  apps?: Partial<AppsView>;
}

export type ToastKind = "info" | "success" | "error";

export interface Toast {
  id: number;
  kind: ToastKind;
  text: string;
  action?: { label: string; run: () => void };
}

const DEBOUNCE_MS = 150;

const demoStatus: StatusView = {
  enabled: true,
  method: "telex",
  charset: "unicode",
  mechanism: "hook",
  hookReady: false,
  version: "0.1.0",
};

const demoInfo: AppInfoView = {
  version: "0.1.0",
  arch: "x86_64",
  configPath: "%APPDATA%\\VietKB\\config.toml",
  hookReady: false,
};

const demoSettings: SettingsView = {
  input: { enabled: true, method: "telex", charset: "unicode", mechanism: "hook" },
  typing: {
    modernTone: true,
    freeTone: true,
    jfwzAsConsonant: true,
    autoCapitalize: false,
    spellCheck: true,
    restoreWrongWord: false,
    disableOnNonUsLayout: true,
    suggestionFix1: true,
    suggestionFix2: false,
    metroFix: false,
    sendStandard: false,
    useClipboard: false,
    macroEnabled: true,
    macroWhenDisabled: false,
    autoHornUo: true,
    rawZone: true,
    macroCaseFollow: true,
    sendDelayUs: 0,
  },
  hotkeys: {
    enable: "F1",
    disable: "F2",
    toggle: "Ctrl+Shift",
    restoreWord: "Ctrl+Shift+Z",
    appExclude: "F11",
    macroPanel: "F8",
    macroToggle: "F9",
    openSettings: "Ctrl+Alt+S",
    cycleMethod: null,
    toggleMode: "release",
  },
  system: {
    startWithWindows: true,
    runAsAdmin: false,
    notificationSound: true,
    startMinimized: true,
    language: "vi",
    theme: "system",
    checkUpdates: true,
  },
  ui: { windowWidth: 1040, windowHeight: 720, lastTab: "overview" },
  apps: { rememberPerApp: false },
};

export const errorText = (error: unknown): string => {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Lỗi không xác định";
};

class Store {
  settings = $state<SettingsView | null>(null);
  status = $state<StatusView | null>(null);
  info = $state<AppInfoView | null>(null);
  loading = $state(true);
  previewMode = $state(false);
  toasts = $state<Toast[]>([]);
  /** Total macro count, mirrored into the Gõ tắt tab label and the Gõ tab link. */
  macroCount = $state<number | null>(null);

  /** Brush/patch objects merged into a single debounced request. */
  private pending: SettingsPatch = {};
  private timer: ReturnType<typeof setTimeout> | undefined;
  private inFlight: SettingsPatch | null = null;
  private seq = 0;
  private toastSeq = 0;
  private unlisten: UnlistenFn | null = null;
  private themeUnlisten: UnlistenFn | null = null;
  private themeChannel: BroadcastChannel | null = null;

  get lang(): Lang {
    const value = this.settings?.system.language;
    return isLang(value) ? value : "vi";
  }

  get t() {
    return translator(this.lang);
  }

  get enabled(): boolean {
    return this.settings?.input.enabled ?? false;
  }

  get method(): string {
    return this.settings?.input.method ?? "telex";
  }

  get modernTone(): boolean {
    return this.settings?.typing.modernTone ?? true;
  }

  async init(): Promise<void> {
    this.previewMode = !api.isTauri();
    this.startThemeBus();
    try {
      if (this.previewMode) {
        this.settings = demoSettings;
        this.status = demoStatus;
        this.info = demoInfo;
        this.applyTheme(demoSettings.system.theme);
        return;
      }
      const [settings, status, info] = await Promise.all([
        api.getSettings(),
        api.getStatus(),
        api.appInfo(),
      ]);
      this.settings = settings;
      this.status = status;
      this.info = info;
      this.applyTheme(settings.system.theme);
      this.unlisten = await listen<StatusView>("vkb:status", (event) => {
        this.status = event.payload;
        if (this.settings) {
          this.settings.input.enabled = event.payload.enabled;
          this.settings.input.method = event.payload.method;
          this.settings.input.charset = event.payload.charset;
        }
      });
    } catch (error) {
      this.settings = demoSettings;
      this.status = demoStatus;
      this.info = demoInfo;
      this.toast("error", `Không đọc được cấu hình: ${errorText(error)}`, 0);
    } finally {
      this.loading = false;
    }
  }

  dispose(): void {
    this.unlisten?.();
    this.unlisten = null;
    this.themeUnlisten?.();
    this.themeUnlisten = null;
    this.themeChannel?.close();
    this.themeChannel = null;
    if (this.timer !== undefined) clearTimeout(this.timer);
  }

  // ---------------------------------------------------------------- mutations

  /** Optimistic + debounced persistence. Safe to call on every keystroke. */
  patch(patch: SettingsPatch): void {
    const settings = this.settings;
    if (!settings) return;

    if (patch.input) Object.assign(settings.input, patch.input);
    if (patch.typing) Object.assign(settings.typing, patch.typing);
    if (patch.hotkeys) Object.assign(settings.hotkeys, patch.hotkeys);
    if (patch.system) Object.assign(settings.system, patch.system);
    if (patch.ui) Object.assign(settings.ui, patch.ui);
    if (patch.apps) Object.assign(settings.apps, patch.apps);

    if (patch.system?.theme) this.applyTheme(patch.system.theme);
    if (patch.input?.enabled !== undefined && this.status) {
      this.status.enabled = patch.input.enabled;
    }
    if (patch.input?.method && this.status) this.status.method = patch.input.method;

    this.pending = mergePatch(this.pending, patch);
    this.schedule();
  }

  private schedule(): void {
    if (this.timer !== undefined) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = undefined;
      void this.flush();
    }, DEBOUNCE_MS);
  }

  private async flush(): Promise<void> {
    if (this.previewMode || this.inFlight) {
      if (this.inFlight) this.schedule();
      return;
    }
    const body = this.pending;
    if (Object.keys(body).length === 0) return;
    this.pending = {};
    this.inFlight = body;
    const seq = ++this.seq;
    try {
      const next = await api.patchSettings(body);
      if (seq === this.seq && Object.keys(this.pending).length === 0) this.settings = next;
      if (next.input) this.status = { ...(this.status ?? demoStatus), ...next.input };
    } catch (error) {
      this.toast("error", `Không lưu được thay đổi: ${errorText(error)}`);
      await this.reload();
    } finally {
      this.inFlight = null;
      if (Object.keys(this.pending).length > 0) this.schedule();
    }
  }

  /** Immediate, non-debounced toggle used by the big status switch. */
  async setEnabled(enabled: boolean): Promise<void> {
    this.patch({ input: { enabled } });
  }

  async setMethod(method: string): Promise<void> {
    this.patch({ input: { method } });
  }

  async setTab(tab: NavId): Promise<void> {
    if (this.settings?.ui.lastTab === tab) return;
    this.patch({ ui: { lastTab: tab } });
  }

  async resetSection(section: "typing" | "hotkeys" | "system" | "apps" | "all"): Promise<void> {
    if (this.previewMode) {
      this.toast("info", "Chế độ xem trước: không có gì để đặt lại.");
      return;
    }
    try {
      const next = await api.resetSettings(section);
      this.settings = next;
      this.applyTheme(next.system.theme);
      this.toast("success", "Đã đặt lại mục này về mặc định.");
    } catch (error) {
      this.toast("error", `Không đặt lại được: ${errorText(error)}`);
    }
  }

  async reload(): Promise<void> {
    if (this.previewMode) return;
    try {
      const [settings, status] = await Promise.all([api.getSettings(), api.getStatus()]);
      this.settings = settings;
      this.status = status;
      this.applyTheme(settings.system.theme);
    } catch {
      // Already surfaced by the caller's toast.
    }
  }

  applyTheme(theme: string, broadcast = true): void {
    const next = theme === "light" || theme === "dark" ? theme : "system";
    document.documentElement.dataset.theme = next;
    if (!broadcast) return;
    // Every open window (main panel + secondary) follows immediately.
    this.themeChannel?.postMessage(next);
    if (!this.previewMode) void emit("vkb:theme", next).catch(() => {});
  }

  /** Theme fan-out: one BroadcastChannel for same-origin webviews, one Tauri
   *  event for the whole app. Receivers never re-broadcast. */
  private startThemeBus(): void {
    if (typeof BroadcastChannel !== "undefined") {
      const channel = new BroadcastChannel("vkb:theme");
      channel.onmessage = (event: MessageEvent) => this.receiveTheme(event.data);
      this.themeChannel = channel;
    }
    if (this.previewMode) return;
    void listen<string>("vkb:theme", (event) => this.receiveTheme(event.payload))
      .then((unlisten) => {
        this.themeUnlisten = unlisten;
      })
      .catch(() => {
        // Event bus unavailable: the BroadcastChannel path still covers windows.
      });
  }

  private receiveTheme(value: unknown): void {
    if (typeof value !== "string") return;
    const next = value === "light" || value === "dark" ? value : "system";
    if (this.settings && this.settings.system.theme !== next) {
      this.settings.system.theme = next;
    }
    this.applyTheme(next, false);
  }

  // ------------------------------------------------------------------ toasts

  toast(kind: ToastKind, text: string, ttl = 6000, action?: Toast["action"]): void {
    const id = ++this.toastSeq;
    this.toasts = [...this.toasts, { id, kind, text, action }];
    if (ttl > 0) setTimeout(() => this.dismiss(id), ttl);
  }

  dismiss(id: number): void {
    this.toasts = this.toasts.filter((toast) => toast.id !== id);
  }

  fail(context: string, error: unknown): void {
    this.toast("error", `${context}: ${errorText(error)}`);
  }
}

const mergePatch = (base: SettingsPatch, next: SettingsPatch): SettingsPatch => {
  const out: SettingsPatch = { ...base };
  for (const key of Object.keys(next) as (keyof SettingsPatch)[]) {
    const section = next[key];
    if (!section) continue;
    // Each section is a flat object of scalars, so a shallow merge is correct.
    out[key] = { ...(out[key] as object), ...(section as object) } as never;
  }
  return out;
};

export const app = new Store();
