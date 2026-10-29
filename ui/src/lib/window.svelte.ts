import { getCurrentWindow } from "@tauri-apps/api/window";

export const isNative = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const closeCurrent = (): void => {
  if (isNative()) void getCurrentWindow().close();
};

export const openWindow = (kind: "hotkeys" | "advanced" | "macros" | "about"): void => {
  if (isNative()) {
    void import("./api").then((api) => api.openWindow(kind));
    return;
  }
  const url = new URL(window.location.href);
  url.searchParams.set("win", kind);
  window.location.href = url.toString();
};
