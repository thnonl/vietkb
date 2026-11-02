<script lang="ts">
  import { onMount, tick } from "svelte";
  import Toast from "./components/Toast.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import Quick from "./screens/Quick.svelte";
  import Hotkeys from "./screens/Hotkeys.svelte";
  import Advanced from "./screens/Advanced.svelte";
  import Macros from "./screens/Macros.svelte";
  import About from "./screens/About.svelte";
  import { app } from "./lib/state.svelte";
  import * as api from "./lib/api";
  import { closeCurrent } from "./lib/window.svelte";
  import { fitWindowToContent } from "./lib/fit";
  import { t } from "./lib/lang.svelte";

  const kind = new URLSearchParams(window.location.search).get("win") ?? "main";
  const modal = kind !== "main";

  /** Two animation frames: the first frame that shows the committed content. */
  const twoFrames = () =>
    new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));

  /** Tell the shell the real content is on screen so it can show this window
   *  (main.rs `window_ready`). The shell keeps its own fallback timer; the 50 ms
   *  cap here is for a hidden window, which may run no animation frames at all. */
  const reportPainted = async (): Promise<void> => {
    await tick();
    await Promise.race([twoFrames(), new Promise<void>((resolve) => setTimeout(resolve, 50))]);
    if (api.isTauri()) await api.windowReady().catch(() => {});
  };

  onMount(() => {
    void app.init().then(() => {
      // After the first paint, never before: macOS grows the window if the sheet is
      // taller than the estimated height, and it resizes a window the user sees.
      void reportPainted()
        .then(() => document.fonts.ready)
        .then(() => fitWindowToContent());
      const theme = new URLSearchParams(window.location.search).get("theme");
      if (theme === "light" || theme === "dark") app.applyTheme(theme);
    });
    // A modal closes on Esc as well as its `Đóng` button. A field that captured
    // the key itself (hotkey capture) calls preventDefault, so it keeps Esc.
    const onKey = (event: KeyboardEvent) => {
      if (!modal || event.key !== "Escape" || event.defaultPrevented) return;
      event.preventDefault();
      closeCurrent();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      app.dispose();
    };
  });
</script>

{#if kind === "hotkeys"}
  <Hotkeys />
{:else if kind === "advanced"}
  <Advanced />
{:else if kind === "about"}
  <About />
{:else if kind === "macros"}
  <div class="v4win">
    <TitleBar title={t("win.macros.title")} subtitle="VietKB" showMark={false} showMinimize={false} />
    <main class="sheet">
      {#if app.settings}
        <Macros />
      {/if}
    </main>
  </div>
{:else}
  {#if app.settings}
    <Quick />
  {:else}
    <div class="v4win">
      <TitleBar title="VietKB" />
      <main class="v4sheet">
        <p style="font:400 13px/20px var(--ui);color:var(--t3);padding-top:16px">{t("common.loading")}</p>
      </main>
    </div>
  {/if}
{/if}

<Toast />
