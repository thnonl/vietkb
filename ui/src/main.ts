import { mount } from "svelte";
import App from "./App.svelte";
import { detectOS } from "./lib/platform.svelte";
import "./styles/tokens.css";
import "./styles/kit.css";
import "./styles/v4.css";
import "./styles/mac.css";

// Theme before first paint: honour the OS until settings arrive (state.svelte.ts
// overwrites this with the saved override).
document.documentElement.dataset.theme = "system";

// Platform before first paint too (user-agent guess; the `platform` command
// confirms it once the app initialises).
document.documentElement.dataset.platform = detectOS();

const target = document.getElementById("app");
if (!target) throw new Error("missing #app root");

const app = mount(App, { target });

export default app;
