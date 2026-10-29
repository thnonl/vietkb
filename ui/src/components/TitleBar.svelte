<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { app } from "../lib/state.svelte";
  // Product mark, identical in every window: the keycap `V` (Vietnamese on)
  // from the approved set (design/logo11/keycap/v-20.png). The title bar never
  // switches to `E` — the tray icon carries the on/off state.
  import mark from "../assets/keycap-v-20.png";

  // Frameless caption shared by all three windows. Tauri 2's
  // `data-tauri-drag-region` only drags when the press lands on the exact
  // element and maximises on double click; the fixed windows must never
  // maximise, so the bar starts the drag itself and ignores the second click.
  let { title, subtitle = "" }: { title: string; subtitle?: string } = $props();

  // Theme cycle — same write path as the Advanced screen (`app.patch`), which
  // persists `settings.system.theme` and broadcasts it to every open window.
  const ORDER = ["system", "light", "dark"] as const;
  type Theme = (typeof ORDER)[number];
  const LABEL: Record<Theme, string> = { system: "Theo Windows", light: "Sáng", dark: "Tối" };
  const normalize = (value: string | undefined): Theme =>
    value === "light" || value === "dark" ? value : "system";

  const theme = $derived(normalize(app.settings?.system.theme));
  const label = $derived(LABEL[theme]);

  const cycleTheme = () => {
    const next = ORDER[(ORDER.indexOf(theme) + 1) % ORDER.length];
    app.patch({ system: { theme: next } });
  };

  const win = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
    ? getCurrentWindow()
    : null;

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
<header class="cap-bar" onmousedown={onPointerDown} ondblclick={(event) => event.preventDefault()}>
  <span class="mark">
    <img src={mark} alt="VietKB" width="20" height="20" />
  </span>
  <span class="ttl">{title}{#if subtitle}<small>— {subtitle}</small>{/if}</span>
  <span class="drag"></span>
  <div class="wc">
    <button
      type="button"
      class="tslot"
      aria-label={`Giao diện: ${label}. Bấm để đổi`}
      title={`Giao diện: ${label} — bấm để đổi`}
      onclick={cycleTheme}>
      {#if theme === "system"}
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <circle cx="8" cy="8" r="5.4" />
          <path class="solid" d="M8 2.6a5.4 5.4 0 0 1 0 10.8z" />
        </svg>
      {:else if theme === "light"}
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
    <button
      type="button"
      class="wbtn"
      aria-label="Thu nhỏ"
      title="Thu nhỏ"
      disabled={!win}
      onclick={minimize}>
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5.5h10" /></svg>
    </button>
    <button
      type="button"
      class="wbtn x"
      aria-label="Đóng, VietKB vẫn chạy"
      title="Đóng, VietKB vẫn chạy"
      disabled={!win}
      onclick={close}>
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M.5.5l9 9M9.5.5l-9 9" /></svg>
    </button>
  </div>
</header>
