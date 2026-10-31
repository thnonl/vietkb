<script lang="ts">
  import { app } from "../lib/state.svelte";
  import { langState, t } from "../lib/lang.svelte";
  import type { Lang } from "../lib/i18n";

  // `VI | EN` quick switch. Same write path as the theme button: `app.patch`
  // persists `system.language`, broadcasts it to every open window, and the
  // Rust side rebuilds the tray menu from the saved value.
  const OPTIONS: { id: Lang; shortKey: string; nameKey: string }[] = [
    { id: "vi", shortKey: "lang.vi.short", nameKey: "lang.vi.name" },
    { id: "en", shortKey: "lang.en.short", nameKey: "lang.en.name" },
  ];

  const choose = (id: Lang) => {
    if (id === langState.lang) return;
    app.patch({ system: { language: id } });
  };
</script>

<span class="langtg" role="group" aria-label={t("lang.group")} data-no-drag>
  {#each OPTIONS as option (option.id)}
    <button
      type="button"
      class:on={langState.lang === option.id}
      aria-pressed={langState.lang === option.id}
      lang={option.id}
      title={t(option.nameKey)}
      aria-label={t(option.nameKey)}
      onclick={() => choose(option.id)}>{t(option.shortKey)}</button>
  {/each}
</span>
