<script lang="ts">
  import { onMount } from "svelte";
  import Toast from "./components/Toast.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import Quick from "./screens/Quick.svelte";
  import Hotkeys from "./screens/Hotkeys.svelte";
  import Advanced from "./screens/Advanced.svelte";
  import Macros from "./screens/Macros.svelte";
  import { app } from "./lib/state.svelte";

  const kind = new URLSearchParams(window.location.search).get("win") ?? "main";

  onMount(() => {
    void app.init().then(() => {
      const theme = new URLSearchParams(window.location.search).get("theme");
      if (theme === "light" || theme === "dark") app.applyTheme(theme);
    });
    return () => app.dispose();
  });
</script>

{#if kind === "hotkeys"}
  <Hotkeys />
{:else if kind === "advanced"}
  <Advanced />
{:else if kind === "macros"}
  <div class="v4win">
    <TitleBar title="Bảng gõ tắt" subtitle="VietKB" />
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
        <p style="font:400 13px/20px var(--ui);color:var(--t3);padding-top:16px">Đang tải…</p>
      </main>
    </div>
  {/if}
{/if}

<Toast />
