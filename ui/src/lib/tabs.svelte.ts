// The one settings window's pages. The Rust side asks for a page by name (tray,
// hotkeys, second launch) through the `vkb:tab` event or `take_requested_tab`; the
// page itself switches with the toolbar. The last page is remembered per viewer so a
// window reopened from the tray lands where the user left it (SPEC section 9).

export const TABS = ["general", "hotkeys", "macros", "advanced", "about"] as const;
export type TabId = (typeof TABS)[number];

const KEY = "vkb:tab";

export const isTab = (value: unknown): value is TabId =>
  typeof value === "string" && (TABS as readonly string[]).includes(value);

/** Rust names some pages differently (`system`, `go`); map to a page id, else null. */
export const tabFromKind = (kind: string): TabId | null => {
  switch (kind) {
    case "general":
    case "go":
      return "general";
    case "hotkeys":
      return "hotkeys";
    case "macros":
      return "macros";
    case "advanced":
    case "system":
      return "advanced";
    case "about":
      return "about";
    default:
      return null;
  }
};

const remembered = (): TabId => {
  try {
    const value = localStorage.getItem(KEY);
    if (isTab(value)) return value;
  } catch {
    // Storage blocked or unavailable: start on the first page.
  }
  return "general";
};

class TabState {
  current = $state<TabId>(remembered());

  set(id: TabId): void {
    this.current = id;
    try {
      localStorage.setItem(KEY, id);
    } catch {
      // Not remembered: the next open starts on the first page.
    }
  }
}

export const tabs = new TabState();
