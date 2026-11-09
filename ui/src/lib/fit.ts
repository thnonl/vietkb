// Window height follows the page (SPEC "B final", section 3): every page's natural
// height, capped at 560 px, and the window is set to it in both directions, on
// macOS and Windows alike. The width is always 560.
//
// Measured on the DOM, not the window: the body's scrollHeight is its full content
// even when the window is shorter, so the answer is the same at any current size.
// Pinned pages (Gõ tắt, `data-fixed`) always ask for the full 560.
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { isNative } from "./window.svelte";

/** Keep equal to `--win-w` / `--win-h` in ui/src/styles/v4.css and MAIN_WIDTH / the 560 cap in main.rs. */
export const WIDTH = 560;
export const MAX_HEIGHT = 560;

/** Height the page needs, in CSS pixels, or null when the page is not on screen. */
export const naturalHeight = (): number | null => {
  const win = document.querySelector<HTMLElement>(".v4win");
  const screen = win?.querySelector<HTMLElement>("[data-screen]");
  const body = screen?.querySelector<HTMLElement>("[data-body]");
  if (!win || !screen || !body) return null;
  if (screen.hasAttribute("data-fixed")) return MAX_HEIGHT;
  const title = win.querySelector<HTMLElement>(".cap-bar")?.offsetHeight ?? 0;
  const strip = win.querySelector<HTMLElement>(".toolbar")?.offsetHeight ?? 0;
  const foot = screen.querySelector<HTMLElement>("[data-foot]")?.offsetHeight ?? 0;
  // The body grows to fill a taller window (footer pinned to the bottom), so read its
  // content height with the grow turned off, then restore it.
  const grow = body.style.flex;
  body.style.flex = "0 1 auto";
  const content = body.scrollHeight;
  body.style.flex = grow;
  return Math.ceil(title + strip + content + foot);
};

const nextFrame = () => new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));

const reported = new Set<string>();
let running = false;
let pending = false;

/** Sets the window to the page's height. A call made while one runs queues one more pass. */
export const fitWindowToContent = async (): Promise<void> => {
  if (!isNative()) return;
  if (running) {
    pending = true;
    return;
  }
  running = true;
  try {
    do {
      pending = false;
      await nextFrame();
      const wanted = naturalHeight();
      if (wanted !== null) {
        const target = Math.min(MAX_HEIGHT, wanted);
        const win = getCurrentWindow();
        const [size, scale] = await Promise.all([win.innerSize(), win.scaleFactor()]);
        const now = size.toLogical(scale);
        if (Math.abs(now.height - target) > 0.5 || Math.abs(now.width - WIDTH) > 0.5) {
          await win.setSize(new LogicalSize(WIDTH, target));
        }
      }
    } while (pending);
  } catch (error) {
    // A window that cannot be resized keeps its size. Say so once per distinct
    // error instead of swallowing it: a silent failure here once hid a broken fit.
    const message = String(error);
    if (!reported.has(message)) {
      reported.add(message);
      console.warn("fitWindowToContent: resize failed", message);
    }
  } finally {
    running = false;
  }
};
