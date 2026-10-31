<script lang="ts">
  import { onMount } from "svelte";
  import Toast from "./components/Toast.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import Quick from "./screens/Quick.svelte";
  import Hotkeys from "./screens/Hotkeys.svelte";
  import Advanced from "./screens/Advanced.svelte";
  import Macros from "./screens/Macros.svelte";
  import About from "./screens/About.svelte";
  import { app } from "./lib/state.svelte";
  import { closeCurrent } from "./lib/window.svelte";
  import { fitWindowToContent } from "./lib/fit";
  import { t } from "./lib/lang.svelte";

  const kind = new URLSearchParams(window.location.search).get("win") ?? "main";
  const modal = kind !== "main";

  onMount(() => {
    void app.init().then(() => {
      // macOS: grow the window if the sheet is taller than the estimated height.
      void document.fonts.ready.then(() => fitWindowToContent());
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
