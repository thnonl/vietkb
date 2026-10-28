<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let {
    open = $bindable(false),
    title,
    children,
    footer,
    onclose,
  }: {
    open?: boolean;
    title: string;
    children: Snippet;
    footer?: Snippet;
    onclose?: () => void;
  } = $props();

  let el = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    const node = el;
    if (!node) return;
    if (open && !node.open) node.showModal();
    if (!open && node.open) node.close();
  });

  const close = () => {
    open = false;
    onclose?.();
  };

  const onDialogClose = () => {
    if (!open) return;
    open = false;
    onclose?.();
  };

  const backdrop = (event: MouseEvent) => {
    if (event.target === el) close();
  };
</script>

<dialog class="sheetdlg" bind:this={el} onclose={onDialogClose} aria-label={title} onclick={backdrop}>
  <div class="dlg-head">
    <h2>{title}</h2>
    <button type="button" class="btn ghost icon sm" aria-label="Đóng" onclick={close}>
      <Icon name="close" small />
    </button>
  </div>
  <div class="dlg-body">{@render children()}</div>
  {#if footer}<div class="dlg-foot">{@render footer()}</div>{/if}
</dialog>
