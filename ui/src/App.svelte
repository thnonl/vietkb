<script lang="ts">
  import { onMount, tick } from "svelte";
  import Toast from "./components/Toast.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import Toolbar from "./components/Toolbar.svelte";
  import Quick from "./screens/Quick.svelte";
  import Hotkeys from "./screens/Hotkeys.svelte";
  import Macros from "./screens/Macros.svelte";
  import Advanced from "./screens/Advanced.svelte";
  import About from "./screens/About.svelte";
  import { app } from "./lib/state.svelte";
  import * as api from "./lib/api";
  import { closeCurrent, openTab } from "./lib/window.svelte";
  import { fitWindowToContent } from "./lib/fit";
  import { TABS, tabFromKind, tabs } from "./lib/tabs.svelte";
  import { probePermissions } from "./lib/permissions.svelte";
  import { syncAutostart } from "./lib/autostart.svelte";
  import { t } from "./lib/lang.svelte";

  let page = $state<HTMLElement | null>(null);

  /** Two animation frames: the first frame that shows the committed content. */
  const twoFrames = () =>
    new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));

  /** Tell the shell the real content is on screen so it can show this window
   *  (main.rs `window_ready`). The shell keeps its own fallback timer; the 50 ms
   *  cap here is for a hidden window, which may run no animation frames at all.
   *  macOS stages the window at alpha 0 instead of hiding it (macos_shell.rs), so
   *  frames do run there and the report waits for a real painted frame. */
  const reportPainted = async (): Promise<void> => {
    await tick();
    if (document.documentElement.dataset.platform === "macos") await twoFrames();
    else await Promise.race([twoFrames(), new Promise<void>((resolve) => setTimeout(resolve, 50))]);
    if (api.isTauri()) await api.windowReady().catch(() => {});
  };

  onMount(() => {
    let disposed = false;
    let stopTabs: (() => void) | null = null;

    // The tray, the hotkeys and a second launch ask for a page while the window is up.
    if (api.isTauri()) {
      void api
        .onTabRequest((kind) => {
          const id = tabFromKind(kind);
          if (id) openTab(id);
        })
        .then((stop) => {
          if (disposed) stop();
          else stopTabs = stop;
        })
        .catch(() => {});
    }

    void probePermissions()
      .then(() => app.init())
      .then(async () => {
      // The autostart switch shows the OS entry on every page (Chung and Nâng cao).
      void syncAutostart();
      // A page asked for before this page existed (the window was still being built).
      if (api.isTauri()) {
        const kind = await api.takeRequestedTab().catch(() => null);
        const id = kind ? tabFromKind(kind) : null;
        if (id) openTab(id);
      }
      // Fit before the first paint is reported: the window is still staged, so this
      // resize is never seen, and the window is revealed at its final height.
      await tick();
      await fitWindowToContent();
      await reportPainted();
      await document.fonts.ready;
      await fitWindowToContent();
      const theme = new URLSearchParams(window.location.search).get("theme");
      if (theme === "light" || theme === "dark") app.applyTheme(theme);
    });

    // Cmd/Ctrl+W closes the window (VietKB keeps running). Cmd/Ctrl+1..5 pick a page.
    const onKey = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      const key = event.key.toLowerCase();
      if (key === "w") {
        event.preventDefault();
        closeCurrent();
        return;
      }
      const index = Number(event.key) - 1;
      if (Number.isInteger(index) && index >= 0 && index < TABS.length) {
        event.preventDefault();
        openTab(TABS[index]);
      }
    };
    window.addEventListener("keydown", onKey);

    // A click on a row's label flips that row's switch (SPEC 9): the label is text,
    // so this finds the switch in the same row instead of wiring every row.
    const onRowLabel = (event: MouseEvent) => {
      const target = event.target;
      if (!(target instanceof Element) || target.closest("button, a, input, select, textarea")) return;
      const label = target.closest(".v4row > .l, .row > .lbl");
      const sw = label?.parentElement?.querySelector<HTMLButtonElement>('[role="switch"]');
      if (sw && !sw.disabled) sw.click();
    };
    document.addEventListener("click", onRowLabel);
    return () => {
      disposed = true;
      stopTabs?.();
      document.removeEventListener("click", onRowLabel);
      window.removeEventListener("keydown", onKey);
      app.dispose();
    };
  });

  // The window follows the page: whenever the page's height changes (a banner
  // appears, a tab changes, a list grows), fit.ts sets the window to match.
  //
  // `#page` itself always fills the window, so observing it alone never fires on a
  // tab change or a longer list (QA round 3: the window kept its first height). Watch
  // the content instead: every child of the page's body and the footer, re-attached
  // whenever the DOM under the page changes (tab switch, list rows, banners).
  $effect(() => {
    const el = page;
    void tabs.current; // refit on every tab change
    if (!el || typeof ResizeObserver === "undefined") return;
    const sizes = new ResizeObserver(() => void fitWindowToContent());
    const watch = () => {
      sizes.disconnect();
      el.querySelectorAll<HTMLElement>("[data-body] > *, [data-foot], [data-screen] > *").forEach(
        (node) => sizes.observe(node),
      );
    };
    const changes = new MutationObserver(() => {
      watch();
      void fitWindowToContent();
    });
    changes.observe(el, { childList: true, subtree: true });
    watch();
    void tick().then(fitWindowToContent);
    return () => {
      sizes.disconnect();
      changes.disconnect();
    };
  });
</script>

<div class="v4win">
  <TitleBar />
  <Toolbar />
  <div
    id="page"
    class="ct"
    role="tabpanel"
    aria-labelledby={`tab-${tabs.current}`}
    bind:this={page}>
    {#if !app.settings}
      <div class="screen" data-screen>
        <div class="body" data-body>
          <p class="loading">{t("common.loading")}</p>
        </div>
      </div>
    {:else if tabs.current === "general"}
      <Quick />
    {:else if tabs.current === "hotkeys"}
      <Hotkeys />
    {:else if tabs.current === "macros"}
      <Macros />
    {:else if tabs.current === "advanced"}
      <Advanced />
    {:else}
      <About />
    {/if}
  </div>
  <Toast />
</div>
