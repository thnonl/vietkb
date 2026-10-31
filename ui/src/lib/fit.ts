// macOS: make the window's content area as tall as the page needs.
//
// The Rust side opens each window at the Windows content height with the caption
// swapped for the 28 pt mac caption row (`MAC_CAPTION_PX` in
// crates/vkb-app/src/main.rs). The window uses an overlay title bar, so the
// webview covers the whole frame; fonts and the Accessibility banner can still
// make the sheet taller than that estimate. So the page measures itself: if
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
let pending = false;

/** Resolve once the viewport height differs from `before` (resize event), or after `ms`. */
const viewportChanged = (before: number, ms = 600): Promise<void> =>
  new Promise((resolve) => {
    if (window.innerHeight !== before) return resolve();
    const done = () => {
      window.removeEventListener("resize", onResize);
      clearTimeout(timer);
      resolve();
    };
    const onResize = () => {
      if (window.innerHeight !== before) done();
    };
    const timer = setTimeout(done, ms);
    window.addEventListener("resize", onResize);
  });

/**
 * Grow the window until the sheet's content fits. macOS only; call once the page
 * has rendered. A call made while a fit is running is not dropped: one more
 * pass is queued and runs when the current one ends.
 */
export const fitWindowToContent = async (): Promise<void> => {
  if (!isNative() || document.documentElement.dataset.platform !== "macos") return;
  if (running) {
    pending = true;
    return;
  }
  running = true;
  try {
    do {
      pending = false;
      // Two passes: fonts and late rows can change the height after the first measure.
      for (let pass = 0; pass < 2; pass++) {
        await new Promise((resolve) => requestAnimationFrame(() => resolve(null)));
        const hidden = hiddenHeight();
        if (hidden === 0) break;
        const before = window.innerHeight;
        await resizeBy(hidden);
        // Measure again only once the webview has actually been resized.
        await viewportChanged(before);
      }
    } while (pending);
  } catch {
    // A window that cannot be resized keeps its size.
  } finally {
    running = false;
  }
};
