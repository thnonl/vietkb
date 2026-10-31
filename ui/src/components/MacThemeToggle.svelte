<script lang="ts">
  import { app } from "../lib/state.svelte";
  import { t } from "../lib/lang.svelte";

  // macOS theme button, next to VI | EN in the top-right row (27-mac-decisions
  // M6). Three states, stored in `system.theme`: follow the system (default),
  // light, dark. Same write path as the Windows button (`app.patch`), which
  // persists, broadcasts to every open window and lets Rust set the native
  // appearance. As on Windows the icon shows what the next click selects, while
  // the tooltip / aria-label name the current state.
  const ORDER = ["system", "light", "dark"] as const;
  type Theme = (typeof ORDER)[number];
  const LABEL_KEY: Record<Theme, string> = {
    system: "titlebar.theme.system",
    light: "titlebar.theme.light",
    dark: "titlebar.theme.dark",
  };
  const normalize = (value: string | undefined): Theme =>
    value === "light" || value === "dark" ? value : "system";

  const theme = $derived(normalize(app.settings?.system.theme));
  const label = $derived(t(LABEL_KEY[theme]));
  const nextTheme = $derived(ORDER[(ORDER.indexOf(theme) + 1) % ORDER.length]);

  const cycle = () => app.patch({ system: { theme: nextTheme } });
</script>

<button
  type="button"
  class="mac-tslot"
  data-no-drag
  aria-label={t("titlebar.theme.aria", { theme: label })}
  title={t("titlebar.theme.title", { theme: label })}
  onclick={cycle}>
  {#if nextTheme === "light"}
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <circle cx="8" cy="8" r="3.1" />
      <path
        d="M8 1.2v1.6M8 13.2v1.6M1.2 8h1.6M13.2 8h1.6M3.2 3.2l1.15 1.15M11.65 11.65l1.15 1.15M12.8 3.2l-1.16 1.16M4.35 11.65L3.2 12.8" />
    </svg>
  {:else if nextTheme === "dark"}
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <path d="M12.4 9.9a5.6 5.6 0 1 1-6.3-8.3 4.6 4.6 0 0 0 6.3 8.3z" />
    </svg>
  {:else}
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <circle cx="8" cy="8" r="5.6" />
      <path class="solid" d="M8 2.4a5.6 5.6 0 0 1 0 11.2z" />
    </svg>
  {/if}
</button>
