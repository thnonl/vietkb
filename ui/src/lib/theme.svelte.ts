// The OS appearance, live, and the effective light/dark theme the toggle shows.
//
// `system.theme` is `system` (default), `light` or `dark` (27-mac-decisions M7).
// With `system` the page follows `prefers-color-scheme` through CSS; the toggle
// needs the same answer in script, so it listens to the same media query.

class SystemScheme {
  dark = $state(false);

  constructor() {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") return;
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    this.dark = query.matches;
    query.addEventListener("change", (event) => {
      this.dark = event.matches;
    });
  }
}

export const systemScheme = new SystemScheme();

/** What the windows look like right now: the stored light/dark, or the OS. */
export const effectiveTheme = (stored: string | undefined): "light" | "dark" =>
  stored === "light" || stored === "dark" ? stored : systemScheme.dark ? "dark" : "light";
