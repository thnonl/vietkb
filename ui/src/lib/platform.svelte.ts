// Which OS the page runs on, for wording, symbols and window chrome.
//
// The authority is the `platform` Tauri command (`std::env::consts::OS`). Until
// it answers, the user agent gives the same result synchronously, so the first
// paint is already right: WKWebView says "Macintosh", WebView2 says "Windows".
// In a plain browser (design preview) `?platform=macos` forces the Mac layout.

export type OS = "windows" | "macos" | "linux";

const guess = (): OS => {
  if (typeof window === "undefined") return "windows";
  const forced = new URLSearchParams(window.location.search).get("platform");
  if (forced === "macos" || forced === "windows" || forced === "linux") return forced;
  const ua = typeof navigator === "undefined" ? "" : navigator.userAgent;
  if (/Macintosh|Mac OS X/i.test(ua)) return "macos";
  if (/Linux|X11/i.test(ua) && !/Windows/i.test(ua)) return "linux";
  return "windows";
};

/** Synchronous best guess; safe at module load (catalog defaults use it). */
export const detectOS = guess;

const apply = (os: OS) => {
  if (typeof document !== "undefined") document.documentElement.dataset.platform = os;
};

class PlatformState {
  os = $state<OS>(guess());

  get mac(): boolean {
    return this.os === "macos";
  }

  /** Ask Rust once; keeps the guess if the command is not there (browser preview). */
  async init(): Promise<void> {
    apply(this.os);
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return;
    try {
      const { platform } = await import("./api");
      const answer = await platform();
      if (answer === "macos" || answer === "windows" || answer === "linux") {
        this.os = answer;
        apply(answer);
      }
    } catch {
      // Keep the user-agent guess.
    }
  }
}

export const platformState = new PlatformState();

/** Reactive when read inside `$derived` / templates. */
export const isMacOS = (): boolean => platformState.mac;
