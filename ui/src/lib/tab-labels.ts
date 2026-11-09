// Names of the settings window's pages, shown on the toolbar and in the macOS title.
import { t } from "./lang.svelte";
import type { TabId } from "./tabs.svelte";

export const tabLabel = (id: TabId): string => {
  switch (id) {
    case "general":
      return t("tab.general");
    case "hotkeys":
      return t("win.hotkeys.title");
    case "macros":
      return t("tab.macros");
    case "advanced":
      return t("win.advanced.title");
    case "about":
      return t("win.about.title");
  }
};
