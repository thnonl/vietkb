// "Khởi chạy app cùng Windows" / "Khởi chạy app cùng macOS", shared by Chung and Nâng cao.
//
// The OS entry (Windows `Run` value, macOS login item) is the truth, not config.toml:
// a user who removed the entry in the system's own startup settings must see the
// switch off here too. In the one-window UI either page can be the first one shown
// (the last tab is remembered), so the sync runs once at start-up, not when Chung
// mounts, and both pages write through the same command. Mac QA found Chung OFF
// and Nâng cao ON when Nâng cao was shown first and wrote config only (2026-11-09).
import { app } from "./state.svelte";
import * as api from "./api";
import { platformState } from "./platform.svelte";
import { t } from "./lang.svelte";

export const autostart = $state({ on: false, ready: false });

/** Read the OS entry once and bring config.toml in line with it. */
export const syncAutostart = async (): Promise<void> => {
  const saved = app.settings?.system.startWithWindows ?? false;
  if (!api.isTauri()) {
    autostart.on = saved;
    autostart.ready = true;
    return;
  }
  try {
    const os = await api.autostartState();
    autostart.on = os;
    if (app.settings && saved !== os) app.patch({ system: { startWithWindows: os } });
  } catch (failure) {
    console.warn("autostart: could not read the OS entry", String(failure));
    autostart.on = saved;
  } finally {
    autostart.ready = true;
  }
};

/** Write the OS entry; the switch then shows what the OS now holds. */
export const setAutostart = async (next: boolean): Promise<void> => {
  if (!api.isTauri()) {
    autostart.on = next;
    app.patch({ system: { startWithWindows: next } });
    return;
  }
  try {
    autostart.on = await api.setAutostart(next);
    if (app.settings && app.settings.system.startWithWindows !== autostart.on) {
      app.patch({ system: { startWithWindows: autostart.on } });
    }
  } catch (failure) {
    app.fail(
      platformState.mac ? t("quick.autostart.fail.mac") : t("quick.autostart.fail.win"),
      failure,
    );
    autostart.on = app.settings?.system.startWithWindows ?? false;
  }
};
