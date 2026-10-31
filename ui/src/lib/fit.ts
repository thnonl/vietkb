// macOS: make the window's content area as tall as the page needs.
//
// The Rust side opens each window at the Windows content height minus the custom
// caption, plus an estimate of the native title bar (`TITLEBAR_PT` in
// crates/vkb-app/src/main.rs). The real title bar height differs between macOS
// releases, and Tauri's `inner_size` on a decorated macOS window reports the
// whole frame, not the area the page can use. So the page measures itself: if
// the sheet's content is taller than the viewport it really has, the window
// grows by the difference and nothing is clipped (D-MAC-02). It never shrinks
// a window.
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { isNative } from "./window.svelte";

/** Window inner height in logical points, as Tauri reports it. */
const innerLogical = async () => {
  const win = getCurrentWindow();
  const [size, scale] = await Promise.all([win.innerSize(), win.scaleFactor()]);
  return size.toLogical(scale);
};

/** Resize the window by `delta` points (keeps width). */
export const resizeBy = async (delta: number): Promise<void> => {
  if (!isNative() || delta === 0) return;
  const inner = await innerLogical();
  await getCurrentWindow().setSize(new LogicalSize(inner.width, inner.height + delta));
};

/** Points of page content hidden below the viewport (0 when everything fits). */
export const hiddenHeight = (): number => {
  const sheet = document.querySelector<HTMLElement>("main.v4sheet, main.sheet");
  if (!sheet) return 0;
  return Math.max(0, Math.ceil(sheet.scrollHeight - sheet.clientHeight));
};

let running = false;

/** Grow the window until the sheet's content fits. macOS only; call once the page has rendered. */
export const fitWindowToContent = async (): Promise<void> => {
  if (!isNative() || document.documentElement.dataset.platform !== "macos" || running) return;
  running = true;
  try {
    // Two passes: fonts and late rows can change the height after the first measure.
    for (let pass = 0; pass < 2; pass++) {
      await new Promise((resolve) => requestAnimationFrame(() => resolve(null)));
      const hidden = hiddenHeight();
      if (hidden === 0) break;
      await resizeBy(hidden);
      await new Promise((resolve) => setTimeout(resolve, 80));
    }
  } catch {
    // A window that cannot be resized keeps its size.
  } finally {
    running = false;
  }
};
