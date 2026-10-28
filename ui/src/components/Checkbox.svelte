<script lang="ts">
  let {
    checked,
    label,
    disabled = false,
    onchange,
  }: {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onchange: (next: boolean) => void;
  } = $props();
</script>

<button
  type="button"
  class="cb"
  class:on={checked}
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onchange(!checked)}>
  {#if checked}
    <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 6.25 5 8.6 9.5 3.6" /></svg>
  {/if}
</button>
