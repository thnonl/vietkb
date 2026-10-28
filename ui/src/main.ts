import { mount } from "svelte";
import App from "./App.svelte";
import "./styles/tokens.css";
import "./styles/kit.css";
import "./styles/v4.css";

// Theme before first paint: honour the OS until settings arrive (state.svelte.ts
// overwrites this with the saved override).
document.documentElement.dataset.theme = "system";

const target = document.getElementById("app");
if (!target) throw new Error("missing #app root");

const app = mount(App, { target });

export default app;
