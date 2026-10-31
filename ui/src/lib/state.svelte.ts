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
  DownloadOutcome,
  HotkeyField,
  InputView,
  NavId,
  SettingsView,
  StatusView,
  SystemView,
  TypingView,
  UiView,
  UpdateStatus,
} from "./bindings";
import { isLang, type Lang } from "./i18n";
import { langState, t } from "./lang.svelte";
import { isMacOS, platformState } from "./platform.svelte";

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
  version: "0.2.0",
};

const demoInfo: AppInfoView = {
  version: "0.2.0",
  arch: "x86_64",
  configPath: "%APPDATA%\\VietKB\\config.toml",
  hookReady: false,
};

/** Preview mode (plain vite, no Tauri): a stub so About and the badge render. */
const demoUpdate: UpdateStatus = {
  state: "available",
  currentVersion: "0.2.0",
  update: {
    version: "0.9.9",
    tag: "v0.9.9",
    changelog:
      "VietKB 0.9.9 (bản xem trước)\n\n- Huy hiệu cập nhật trên thanh tiêu đề\n- Mục cập nhật trong menu khay hệ thống\n- Cửa sổ Giới thiệu có nhật ký thay đổi\n",
    publishedAt: "2026-10-20T09:00:00Z",
    htmlUrl: "https://github.com/thnonl/vietkb",
    assets: [],
    assetName: null,
    assetUrl: null,
    assetSize: null,
  },
  message: null,
  checkedAt: null,
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
    sendStandard: false,
    useClipboard: false,
    macroEnabled: true,
    macroWhenDisabled: false,
    autoHornUo: true,
    macroCaseFollow: true,
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
  return t("common.unknownError");
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

  /** Update surface: last check result and the busy flag. */
  update = $state<UpdateStatus | null>(null);
  updateBusy = $state<"idle" | "checking" | "downloading" | "installing">("idle");
  updateNote = $state<string | null>(null);

  /** Brush/patch objects merged into a single debounced request. */
  private pending: SettingsPatch = {};
  private timer: ReturnType<typeof setTimeout> | undefined;
  private inFlight: SettingsPatch | null = null;
  private seq = 0;
  private toastSeq = 0;
  private unlisten: UnlistenFn | null = null;
  private themeUnlisten: UnlistenFn | null = null;
  private themeChannel: BroadcastChannel | null = null;
  private settingsUnlisten: UnlistenFn | null = null;
  private settingsChannel: BroadcastChannel | null = null;
  private updateUnlisten: UnlistenFn | null = null;
  private configUnlisten: UnlistenFn | null = null;
  private rulesHash: number | string | null = null;

  /** Bumped when the app-rule list changed behind this window's back (the
   *  per-app exclude hotkey, another window): lists reload on it. */
  rulesRev = $state(0);

  get lang(): Lang {
    const value = this.settings?.system.language;
    return isLang(value) ? value : langState.lang;
  }

  get t() {
    return t;
  }

  /** Mirror `system.language` into the reactive language state. Called after
   *  every place `settings` is assigned or patched. */
  private syncLang(): void {
    const value = this.settings?.system.language;
    if (isLang(value)) langState.set(value);
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

  /** True once a check has found a newer release. Drives the badge and tray. */
  get updateAvailable(): boolean {
    return this.update?.state === "available" && this.update.update !== null;
  }

  get updateVersion(): string | null {
    return this.update?.update?.version ?? null;
  }

  async init(): Promise<void> {
    this.previewMode = !api.isTauri();
    this.startThemeBus();
    try {
      // Not awaited: the user-agent guess is already the initial value, and the
      // settings load must not wait on one more IPC round trip.
      void platformState.init();
      if (this.previewMode) {
        demoSettings.system.language = langState.lang;
        this.settings = demoSettings;
        this.status = demoStatus;
        this.info = demoInfo;
        this.update = demoUpdate;
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
      this.syncLang();
      this.applyTheme(settings.system.theme);
      this.unlisten = await listen<StatusView>("vkb:status", (event) => {
        this.status = event.payload;
        if (this.settings) {
          this.settings.input.enabled = event.payload.enabled;
          this.settings.input.method = event.payload.method;
          this.settings.input.charset = event.payload.charset;
        }
      });

      // Any config write, from any window or from the backend worker (hotkeys),
      // ends in this event: re-read the settings and tell the rule lists.
      this.configUnlisten = await listen<{ rules: number | string }>(
        "vkb:config-changed",
        (event) => {
          if (event.payload.rules !== this.rulesHash) {
            this.rulesHash = event.payload.rules;
            this.rulesRev += 1;
          }
          // A local edit still on its way back wins; its response is newer.
          if (this.inFlight || Object.keys(this.pending).length > 0) return;
          void this.reload();
        },
      );

      // The update state is one object the backend owns: the 5 s startup check,
      // a manual check and the tray all write it. Every window follows the
      // event, so the title-bar badge and the tray item appear without a
      // restart.
      this.update = await api.updateStatus().catch(() => null);
      await listen<UpdateStatus>("vkb:update", (event) => {
        this.update = event.payload;
      })
        .then((unlisten) => {
          this.updateUnlisten = unlisten;
        })
        .catch(() => {});
    } catch (error) {
      this.settings = demoSettings;
      this.status = demoStatus;
      this.info = demoInfo;
      this.toast("error", t("toast.loadConfigFailed", { error: errorText(error) }), 0);
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
    this.settingsUnlisten?.();
    this.settingsUnlisten = null;
    this.settingsChannel?.close();
    this.settingsChannel = null;
    this.updateUnlisten?.();
    this.updateUnlisten = null;
    this.configUnlisten?.();
    this.configUnlisten = null;
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
    if (patch.system?.language) this.syncLang();
    if (patch.input?.enabled !== undefined && this.status) {
      this.status.enabled = patch.input.enabled;
    }
    if (patch.input?.method && this.status) this.status.method = patch.input.method;

    this.pending = mergePatch(this.pending, patch);
    this.broadcastSettings(patch);
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
      if (seq === this.seq && Object.keys(this.pending).length === 0) {
        this.settings = next;
        this.syncLang();
      }
      if (next.input) this.status = { ...(this.status ?? demoStatus), ...next.input };
    } catch (error) {
      this.toast("error", t("toast.saveFailed", { error: errorText(error) }));
      await this.reload(true);
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
      this.toast("info", t("toast.previewReset"));
      return;
    }
    try {
      const next = await api.resetSettings(section);
      this.settings = next;
      this.syncLang();
      this.applyTheme(next.system.theme);
      this.toast("success", t("toast.resetDone"));
    } catch (error) {
      this.toast("error", t("toast.resetFailed", { error: errorText(error) }));
    }
  }

  async reload(force = false): Promise<void> {
    if (this.previewMode) return;
    try {
      const [settings, status] = await Promise.all([api.getSettings(), api.getStatus()]);
      // A local edit made while this was in flight is newer than the answer.
      if (!force && (this.inFlight || Object.keys(this.pending).length > 0)) return;
      this.settings = settings;
      this.status = status;
      this.syncLang();
      // Other windows reload on the same event: no need to re-broadcast.
      this.applyTheme(settings.system.theme, false);
    } catch {
      // Already surfaced by the caller's toast.
    }
  }

  // ------------------------------------------------------------------ update

  /** Ask Rust for the latest release. Never throws: failures become the status
   *  line (`state === "error"`), which is exactly what About shows. */
  async checkForUpdate(): Promise<void> {
    if (this.previewMode) {
      this.update = demoUpdate;
      this.updateNote = t("update.note.preview");
      return;
    }
    this.updateBusy = "checking";
    this.updateNote = null;
    try {
      this.update = await api.checkUpdate();
    } catch (error) {
      const base = this.update ?? demoUpdate;
      this.update = { ...base, state: "error", update: null, message: errorText(error) };
    } finally {
      this.updateBusy = "idle";
    }
  }

  /** `Cập nhật ngay`: download into %TEMP%, then hand off to the installer.
   *  macOS (M5): notify only, so this opens the release page instead. */
  async installUpdate(): Promise<void> {
    if (this.previewMode) {
      this.updateNote = t("update.note.previewNoDownload");
      return;
    }
    if (isMacOS()) {
      try {
        await api.installUpdate();
        this.updateNote = t("update.note.macOpened");
      } catch (error) {
        this.updateNote = errorText(error);
      }
      return;
    }
    this.updateBusy = "downloading";
    this.updateNote = t("update.note.downloading");
    try {
      const done: DownloadOutcome = await api.downloadUpdate();
      this.updateNote = done.verified
        ? t("update.note.verified", { version: done.version })
        : t("update.note.downloaded", { version: done.version });
      this.updateBusy = "installing";
      await api.installUpdate();
    } catch (error) {
      this.updateNote = errorText(error);
    } finally {
      this.updateBusy = "idle";
    }
  }

  applyTheme(theme: string, broadcast = true): void {
    // "system" (the default on both platforms) follows the OS appearance live
    // through CSS; an explicit light/dark is what the toggle stored (M7).
    const next = theme === "light" || theme === "dark" ? theme : "system";
    document.documentElement.dataset.theme = next;
    // Mirror it so the next window can paint the right theme before settings load.
    try {
      localStorage.setItem("vkb:theme", next);
    } catch {
      // Storage unavailable: the window starts on "system" until settings arrive.
    }
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
      const settings = new BroadcastChannel("vkb:settings");
      settings.onmessage = (event: MessageEvent) => this.receiveSettings(event.data);
      this.settingsChannel = settings;
    }
    if (this.previewMode) return;
    void listen<string>("vkb:theme", (event) => this.receiveTheme(event.payload))
      .then((unlisten) => {
        this.themeUnlisten = unlisten;
      })
      .catch(() => {
        // Event bus unavailable: the BroadcastChannel path still covers windows.
      });
    void listen<SettingsPatch>("vkb:settings", (event) => this.receiveSettings(event.payload))
      .then((unlisten) => {
        this.settingsUnlisten = unlisten;
      })
      .catch(() => {
        // Event bus unavailable: the BroadcastChannel path still covers windows.
      });
  }

  /** Push a control change to the other windows so derived rows (for example
   *  `Tự động khôi phục từ gõ sai` behind `Kiểm tra chính tả`) are live. */
  private broadcastSettings(patch: SettingsPatch): void {
    if (this.previewMode) return;
    try {
      this.settingsChannel?.postMessage(patch);
    } catch {
      // Non-cloneable payload: the Tauri event below still carries it.
    }
    void emit("vkb:settings", patch).catch(() => {});
  }

  /** Apply another window's change without echoing it back. */
  private receiveSettings(value: unknown): void {
    if (!value || typeof value !== "object") return;
    const patch = value as SettingsPatch;
    const settings = this.settings;
    if (!settings) return;
    if (patch.input) Object.assign(settings.input, patch.input);
    if (patch.typing) Object.assign(settings.typing, patch.typing);
    if (patch.hotkeys) Object.assign(settings.hotkeys, patch.hotkeys);
    if (patch.system) Object.assign(settings.system, patch.system);
    if (patch.ui) Object.assign(settings.ui, patch.ui);
    if (patch.apps) Object.assign(settings.apps, patch.apps);
    if (patch.system?.theme) this.applyTheme(patch.system.theme, false);
    if (patch.system?.language) this.syncLang();
    if (patch.input?.enabled !== undefined && this.status) {
      this.status.enabled = patch.input.enabled;
    }
    if (patch.input?.method && this.status) this.status.method = patch.input.method;
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
