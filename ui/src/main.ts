import { mount } from "svelte";
import App from "./App.svelte";
import { detectOS } from "./lib/platform.svelte";
import "./styles/tokens.css";
import "./styles/kit.css";
import "./styles/v4.css";
import "./styles/mac.css";

// Theme before first paint: the last theme this app applied (mirrored into
// localStorage by state.svelte.ts), else the OS, until settings arrive and
// overwrite it with the saved value. No flash of the wrong theme.
let firstTheme = "system";
// The shell injects the saved theme before this script runs (window_bg.rs).
const injected = (window as unknown as { __VKB_THEME__?: string }).__VKB_THEME__;
if (injected === "light" || injected === "dark" || injected === "system") {
  firstTheme = injected;
} else {
  try {
    const saved = localStorage.getItem("vkb:theme");
    if (saved === "light" || saved === "dark") firstTheme = saved;
  } catch {
    // Storage unavailable: follow the OS.
  }
}
document.documentElement.dataset.theme = firstTheme;

// Platform before first paint too (user-agent guess; the `platform` command
// confirms it once the app initialises).
document.documentElement.dataset.platform = detectOS();

const target = document.getElementById("app");
if (!target) throw new Error("missing #app root");

const app = mount(App, { target });

export default app;
