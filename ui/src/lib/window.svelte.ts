import { getCurrentWindow } from "@tauri-apps/api/window";
import { tabs, type TabId } from "./tabs.svelte";

export const isNative = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Closes the one settings window (VietKB keeps running; the tray brings it back). */
export const closeCurrent = (): void => {
  if (isNative()) void getCurrentWindow().close();
};

/** Shows a page of the settings window. Pages are not windows any more. */
export const openTab = (id: TabId): void => {
  tabs.set(id);
};
