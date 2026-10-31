<script lang="ts">
  import { app } from "../lib/state.svelte";
  import { t } from "../lib/lang.svelte";
  import { effectiveTheme } from "../lib/theme.svelte";

  // macOS theme button, next to VI | EN in the top-right row. Same behaviour as
  // the Windows title-bar button (27-mac-decisions M7): it shows the EFFECTIVE
  // light/dark state (the OS appearance while `system.theme` is `system`), and a
  // click stores the opposite as an explicit light/dark through `app.patch`,
  // which persists, broadcasts to every open window and lets Rust set the native
  // appearance. The icon shows what the next click selects; the tooltip and
  // aria-label name the current state.
  const theme = $derived(effectiveTheme(app.settings?.system.theme));
  const label = $derived(t(theme === "light" ? "titlebar.theme.light" : "titlebar.theme.dark"));
  const nextTheme = $derived(theme === "light" ? "dark" : "light");

  const toggle = () => app.patch({ system: { theme: nextTheme } });
</script>

<button
  type="button"
  class="mac-tslot"
  data-no-drag
  aria-label={t("titlebar.theme.aria", { theme: label })}
  title={t("titlebar.theme.title", { theme: label })}
  onclick={toggle}>
  {#if nextTheme === "light"}
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <circle cx="8" cy="8" r="3.1" />
      <path
        d="M8 1.2v1.6M8 13.2v1.6M1.2 8h1.6M13.2 8h1.6M3.2 3.2l1.15 1.15M11.65 11.65l1.15 1.15M12.8 3.2l-1.16 1.16M4.35 11.65L3.2 12.8" />
    </svg>
  {:else}
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <path d="M12.4 9.9a5.6 5.6 0 1 1-6.3-8.3 4.6 4.6 0 0 0 6.3 8.3z" />
    </svg>
  {/if}
</button>
