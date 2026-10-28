<script lang="ts">
  import type { NavId } from "../lib/bindings";

  // Tabs live in the caption strip. On tabs other than Gõ, the strip also
  // carries the single live state chip (handover §2.5: never show it twice).
  let {
    tabs,
    active,
    onselect,
    state,
  }: {
    tabs: { id: NavId; label: string; count?: string }[];
    active: NavId;
    onselect: (id: NavId) => void;
    state?: { enabled: boolean; detail: string; ontoggle: () => void } | null;
  } = $props();
</script>

<nav class="tabs" aria-label="Mục cài đặt">
  {#each tabs as tab (tab.id)}
    <button
      type="button"
      class="tab"
      class:on={active === tab.id}
      aria-current={active === tab.id ? "page" : undefined}
      onclick={() => onselect(tab.id)}>
      {tab.label}
      {#if tab.count !== undefined}<span class="c num">{tab.count}</span>{/if}
    </button>
  {/each}
</nav>

{#if state}
  <button
    type="button"
    class="st"
    aria-pressed={state.enabled}
    aria-label={state.enabled ? "Bấm để tắt tiếng Việt" : "Bấm để bật tiếng Việt"}
    title={state.enabled ? "Bấm để tắt tiếng Việt" : "Bấm để bật tiếng Việt"}
    onclick={() => state.ontoggle()}>
    <span class="dot" class:off={!state.enabled}></span>
    {#if state.enabled}
      <b>Tiếng Việt</b>
      <span>· {state.detail}</span>
    {:else}
      <b style="color:var(--t2)">Đã tắt tiếng Việt</b>
    {/if}
  </button>
{/if}
