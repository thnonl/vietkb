<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  // Frameless caption shared by all three windows. Tauri 2's
  // `data-tauri-drag-region` only drags when the press lands on the exact
  // element and maximises on double click; the fixed windows must never
  // maximise, so the bar starts the drag itself and ignores the second click.
  let { title, subtitle = "" }: { title: string; subtitle?: string } = $props();

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
  <span style="display:flex">
    <svg width="16" height="16" viewBox="0 0 16 16" aria-label="VietKB" role="img">
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
  <span class="ttl">{title}{#if subtitle}<small>— {subtitle}</small>{/if}</span>
  <span class="drag"></span>
  <div class="wc">
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
