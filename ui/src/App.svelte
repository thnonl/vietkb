<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TabStrip from "./components/TabStrip.svelte";
  import Toast from "./components/Toast.svelte";
  import Go from "./screens/Go.svelte";
  import Hotkeys from "./screens/Hotkeys.svelte";
  import Macros from "./screens/Macros.svelte";
  import System from "./screens/System.svelte";
  import * as api from "./lib/api";
  import { app } from "./lib/state.svelte";
  import { charsetShort, methodLabel } from "./lib/catalog";
  import { formatNumber } from "./lib/hotkey";
  import type { NavId } from "./lib/bindings";

  const TABS: { id: NavId; label: string }[] = [
    { id: "go", label: "Gõ" },
    { id: "macros", label: "Gõ tắt" },
    { id: "hotkeys", label: "Phím tắt" },
    { id: "system", label: "Hệ thống" },
  ];

  const LEGACY: Record<string, NavId> = {
    overview: "go",
    typing: "go",
    charset: "go",
    apps: "system",
    about: "system",
  };

  let tab = $state<NavId>("go");
  let restored = $state(false);

  // Deep link for reviews and for future "open this tab" tray/hotkey actions:
  // ?tab=macros keeps the restored last tab from overriding it.
  const requested = new URLSearchParams(window.location.search).get("tab");
  if (requested && (TABS as { id: string }[]).some((item) => item.id === requested)) {
    tab = requested as NavId;
    restored = true;
  }

  const native = api.isTauri();
  const win = native ? getCurrentWindow() : null;

  onMount(() => {
    void app.init().then(() => {
      const theme = new URLSearchParams(window.location.search).get("theme");
      if (theme === "light" || theme === "dark") app.applyTheme(theme);
      if (api.isTauri()) {
        void api
          .listMacros("")
          .then((rows) => (app.macroCount = rows.length))
          .catch(() => {});
      } else {
        app.macroCount = 8;
      }
    });
    return () => app.dispose();
  });

  $effect(() => {
    const saved = app.settings?.ui.lastTab;
    if (restored || !saved) return;
    restored = true;
    const ids = TABS.map((item) => item.id) as string[];
    if (ids.includes(saved)) tab = saved as NavId;
    else if (LEGACY[saved]) tab = LEGACY[saved];
  });

  const navigate = (next: NavId) => {
    tab = next;
    void app.setTab(next);
  };

  // Frameless window: the tab strip is the title bar. Tauri 2's
  // `data-tauri-drag-region` only drags when the press lands on the exact
  // element carrying it, and any child with its own handler swallows it — and
  // it maximizes on double click. So the strip starts the drag itself, skips
  // presses that began on a control, and ignores the second click of a pair.
  const CONTROL = 'button, a, input, select, textarea, label, [role="button"], [data-no-drag]';

  const onStripPointerDown = (event: MouseEvent) => {
    if (event.button !== 0) return;
    // detail > 1 is the second press of a double click. The window is fixed
    // size (charter C9), so a double click on the strip must do nothing —
    // never maximize or restore.
    if (event.detail > 1) {
      event.preventDefault();
      return;
    }
    const target = event.target;
    // Controls keep their own click: the drag never starts underneath them.
    if (target instanceof Element && target.closest(CONTROL)) return;
    if (!win) return;
    event.preventDefault();
    void win.startDragging();
  };

  const onStripDoubleClick = (event: MouseEvent) => event.preventDefault();

  const onKey = (event: KeyboardEvent) => {
    if (!event.ctrlKey || event.altKey || event.metaKey) return;
    const index = Number(event.key);
    if (index >= 1 && index <= TABS.length) {
      event.preventDefault();
      navigate(TABS[index - 1].id);
    }
  };

  const stateChip = $derived(
    tab === "go" || !app.settings
      ? null
      : {
          enabled: app.settings.input.enabled,
          detail: `${methodLabel(app.settings.input.method)} · ${charsetShort(app.settings.input.charset)}`,
          ontoggle: () => app.patch({ input: { enabled: !app.settings?.input.enabled } }),
        },
  );
</script>

<svelte:window onkeydown={onKey} />

<div class="win">
  <!-- The strip is the frameless window's drag handle; presses that start on a
       control are ignored, so the header itself stays non-interactive. -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <header
    class="strip"
    onmousedown={onStripPointerDown}
    ondblclick={onStripDoubleClick}>
    <span class="mark">
      <svg width="18" height="18" viewBox="0 0 16 16" aria-label="VietKB" role="img">
        <rect width="16" height="16" rx="4.5" style="fill:var(--acc)" />
        <path
          d="M6.05 5.35c0-1.2.9-2.05 2.05-2.05 1.15 0 2 .8 2 1.85 0 .9-.6 1.4-1.35 1.75-.45.2-.65.5-.65.95v.55"
          fill="none"
          stroke="#fff"
          stroke-width="1.45"
          stroke-linecap="round"
          stroke-linejoin="round" />
        <path d="M4.75 11.75h6.5" stroke="#fff" stroke-width="1.45" stroke-linecap="round" />
      </svg>
    </span>

    <TabStrip
      tabs={TABS.map((item) =>
        item.id === "macros" && app.macroCount !== null
          ? { ...item, count: formatNumber(app.macroCount ?? 0) }
          : item,
      )}
      active={tab}
      onselect={navigate}
      state={stateChip} />

    <div class="wc">
      <button
        type="button"
        class="wbtn"
        aria-label="Thu nhỏ"
        title="Thu nhỏ"
        disabled={!native}
        onclick={() => void win?.minimize()}>
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5.5h10" /></svg>
      </button>
      <button
        type="button"
        class="wbtn close"
        aria-label="Đóng, VietKB vẫn chạy"
        title="Đóng, VietKB vẫn chạy"
        disabled={!native}
        onclick={() => void win?.close()}>
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M.5.5l9 9M9.5.5l-9 9" /></svg>
      </button>
    </div>
  </header>

  <main class="sheet">
    {#if !app.settings}
      <p style="font:400 13px/20px var(--ui);color:var(--t3);padding-top:16px">
        {app.t("common.loading")}
      </p>
    {:else if tab === "go"}
      <Go settings={app.settings} {navigate} />
    {:else if tab === "macros"}
      <Macros />
    {:else if tab === "hotkeys"}
      <Hotkeys settings={app.settings} />
    {:else if tab === "system"}
      <System settings={app.settings} />
    {/if}
  </main>
</div>

<Toast />
