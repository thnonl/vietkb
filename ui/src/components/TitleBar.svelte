<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { app } from "../lib/state.svelte";
  import { openTab, isNative } from "../lib/window.svelte";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";
  import { tabs } from "../lib/tabs.svelte";
  import { tabLabel } from "../lib/tab-labels";
  import LangToggle from "./LangToggle.svelte";
  import { effectiveTheme } from "../lib/theme.svelte";
  // Product mark: the keycap V (Vietnamese on) from the approved set. The title bar
  // never switches to E; the tray icon carries the on/off state.
  import markSrc from "../assets/keycap-v-20.png";

  // The one settings window's title bar (SPEC sections 3 and 7). macOS: overlay title
  // bar, the traffic lights are native at the left and the current page name sits in
  // the middle. Windows: frameless; logo and "VietKB" at the left, caption buttons at
  // the right. Both carry the update dot, the theme toggle and VI | EN.
  //
  // Tauri 2's data-tauri-drag-region only drags on the exact element and maximises on
  // double click; this window must never maximise, so the bar starts the drag itself.
  const mac = $derived(platformState.mac);

  // Theme cycle: same write path as before (app.patch), broadcast to every open view.
  const ORDER = ["light", "dark"] as const;
  type Theme = (typeof ORDER)[number];
  const LABEL_KEY: Record<Theme, string> = {
    light: "titlebar.theme.light",
    dark: "titlebar.theme.dark",
  };
  const theme = $derived<Theme>(effectiveTheme(app.settings?.system.theme));
  const label = $derived(t(LABEL_KEY[theme]));
  const nextTheme = $derived(ORDER[(ORDER.indexOf(theme) + 1) % ORDER.length]);

  const showUpdate = $derived(app.showUpdateUi && app.updateAvailable);

  const cycleTheme = () => {
    app.patch({ system: { theme: nextTheme } });
  };

  const win = isNative() ? getCurrentWindow() : null;

  const CONTROL = 'button, a, input, select, textarea, label, [role="button"], [data-no-drag]';

  const onPointerDown = (event: MouseEvent) => {
    if (event.button !== 0) return;
    if (event.detail > 1) {
      event.preventDefault();
      return;
    }
    const target = event.target;
    if (target instanceof Element && target.closest(CONTROL)) return;
    if (!win) return;
    event.preventDefault();
    void win.startDragging();
  };

  const minimize = () => void win?.minimize();
  const close = () => void win?.close();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header
  class="cap-bar"
  class:mac
  onmousedown={onPointerDown}
  ondblclick={(event) => event.preventDefault()}>
  {#if mac}
    <span class="ttl">{tabLabel(tabs.current)}</span>
  {:else}
    <span class="mark"><img src={markSrc} alt="" width="14" height="14" /></span>
    <span class="ttl">VietKB</span>
  {/if}
  <span class="drag"></span>
  <div class="wc">
    <!-- The update slot keeps its place when there is no update (hidden), so the
         theme and language toggles never shift. It shows a dot, not an arrow. -->
    <button
      type="button"
      class="tslot upd"
      class:off={!showUpdate}
      aria-hidden={!showUpdate}
      tabindex={showUpdate ? 0 : -1}
      aria-label={showUpdate ? t("titlebar.update.aria") : undefined}
      title={showUpdate ? t("titlebar.update.title", { version: app.updateVersion ?? "" }) : undefined}
      onclick={() => openTab("about")}></button>
    <button
      type="button"
      class="tslot"
      aria-label={t("titlebar.theme.aria", { theme: label })}
      title={t("titlebar.theme.title", { theme: label })}
      onclick={cycleTheme}>
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
    <LangToggle />
    {#if !mac}
      <button
        type="button"
        class="wbtn"
        aria-label={t("titlebar.minimize")}
        title={t("titlebar.minimize")}
        disabled={!win}
        onclick={minimize}>
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5.5h10" /></svg>
      </button>
      <button
        type="button"
        class="wbtn x"
        aria-label={t("titlebar.close")}
        title={t("titlebar.close")}
        disabled={!win}
        onclick={close}>
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M.5.5l9 9M9.5.5l-9 9" /></svg>
      </button>
    {/if}
  </div>
</header>
