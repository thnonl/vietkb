<script lang="ts">
  import Icon from "./Icon.svelte";
  import { app } from "../lib/state.svelte";
  import { t } from "../lib/lang.svelte";
</script>

<div class="toasts" aria-live="polite" aria-atomic="false">
  {#each app.toasts as toast (toast.id)}
    <div
      class="toast"
      class:ok={toast.kind === "success"}
      class:err={toast.kind === "error"}
      role={toast.kind === "error" ? "alert" : "status"}>
      <span class="gi">
        <Icon
          name={toast.kind === "error" ? "warning" : toast.kind === "success" ? "check" : "info"}
          small />
      </span>
      <span class="tx">{toast.text}</span>
      {#if toast.action}
        <button
          type="button"
          class="link"
          onclick={() => {
            toast.action?.run();
            app.dismiss(toast.id);
          }}>{toast.action.label}</button>
      {/if}
      <button type="button" class="x" aria-label={t("toast.close")} onclick={() => app.dismiss(toast.id)}>
        <Icon name="close" small />
      </button>
    </div>
  {/each}
</div>
