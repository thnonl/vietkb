<script lang="ts">
  import { app } from "../lib/state.svelte";
  import { tabs, TABS, type TabId } from "../lib/tabs.svelte";
  import { tabLabel } from "../lib/tab-labels";
  import { t } from "../lib/lang.svelte";

  // The page switcher (SPEC section 7): a classic Preferences toolbar, five tabs with a
  // glyph and a label. One Tab stop for the strip (roving tabindex). Left and Right move
  // and select (wrapping); Home and End jump to the ends. Ctrl/Cmd+1..5 is handled by the
  // shell (App.svelte).

  // Glyphs, 24-unit grid drawn at 20 px with a 1.5 stroke: settings, keyboard, bolt,
  // sliders, info. Inline paths, no icon package.
  const GLYPH: Record<TabId, string> = {
    general:
      // 8-tooth gear centred on (12, 12): teeth r 6.2 -> 8.8, hub r 2.8. The old path
      // had its tooth ring offset from the hub (owner saw it, 2026-11-09).
      "M10.50 5.98L10.70 3.30A8.8 8.8 0 0 1 13.30 3.30L13.50 5.98A6.2 6.2 0 0 1 15.19 6.69L17.23 4.93A8.8 8.8 0 0 1 19.07 6.77L17.31 8.81A6.2 6.2 0 0 1 18.02 10.50L20.70 10.70A8.8 8.8 0 0 1 20.70 13.30L18.02 13.50A6.2 6.2 0 0 1 17.31 15.19L19.07 17.23A8.8 8.8 0 0 1 17.23 19.07L15.19 17.31A6.2 6.2 0 0 1 13.50 18.02L13.30 20.70A8.8 8.8 0 0 1 10.70 20.70L10.50 18.02A6.2 6.2 0 0 1 8.81 17.31L6.77 19.07A8.8 8.8 0 0 1 4.93 17.23L6.69 15.19A6.2 6.2 0 0 1 5.98 13.50L3.30 13.30A8.8 8.8 0 0 1 3.30 10.70L5.98 10.50A6.2 6.2 0 0 1 6.69 8.81L4.93 6.77A8.8 8.8 0 0 1 6.77 4.93L8.81 6.69A6.2 6.2 0 0 1 10.50 5.98ZM14.8 12a2.8 2.8 0 1 1-5.6 0 2.8 2.8 0 0 1 5.6 0z",
    hotkeys: "M3.5 7.5h17v9h-17zM6.5 10.5h.01M9.5 10.5h.01M12.5 10.5h.01M15.5 10.5h.01M7 13.5h10",
    macros: "M13.5 3 5.5 14h6l-1 7 8-11h-6z",
    advanced: "M4 7h9M17 7h3M4 17h3M11 17h9M15 5v4M9 15v4",
    about: "M12 20.5a8.5 8.5 0 1 0 0-17 8.5 8.5 0 0 0 0 17zM12 11v5.5M12 7.9v.01",
  };

  const buttons: Partial<Record<TabId, HTMLElement>> = {};

  const select = (id: TabId, focus = false) => {
    tabs.set(id);
    if (focus) buttons[id]?.focus();
  };

  const step = (from: TabId, delta: number): TabId => {
    const at = TABS.indexOf(from);
    return TABS[(at + delta + TABS.length) % TABS.length];
  };

  const onKey = (event: KeyboardEvent, id: TabId) => {
    switch (event.key) {
      case "ArrowRight":
        event.preventDefault();
        select(step(id, 1), true);
        break;
      case "ArrowLeft":
        event.preventDefault();
        select(step(id, -1), true);
        break;
      case "Home":
        event.preventDefault();
        select(TABS[0], true);
        break;
      case "End":
        event.preventDefault();
        select(TABS[TABS.length - 1], true);
        break;
    }
  };

  const updateDot = $derived(app.showUpdateUi && app.updateAvailable);
</script>

<div class="toolbar" role="tablist" aria-label={t("win.settings.label")}>
  {#each TABS as id, index (id)}
    <button
      bind:this={buttons[id]}
      type="button"
      role="tab"
      id={`tab-${id}`}
      class="tab"
      class:dot={id === "about" && updateDot}
      aria-selected={tabs.current === id}
      aria-controls="page"
      tabindex={tabs.current === id ? 0 : -1}
      title={`${tabLabel(id)} (${index + 1})`}
      onclick={() => select(id)}
      onkeydown={(event) => onKey(event, id)}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d={GLYPH[id]} /></svg>
      <span>{tabLabel(id)}</span>
    </button>
  {/each}
</div>
