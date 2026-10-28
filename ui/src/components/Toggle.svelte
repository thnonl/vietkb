<script lang="ts">
  let {
    checked,
    label,
    danger = false,
    disabled = false,
    onchange,
  }: {
    checked: boolean;
    label: string;
    danger?: boolean;
    disabled?: boolean;
    onchange: (next: boolean) => void;
  } = $props();
</script>

<button
  type="button"
  class="tg"
  class:on={checked}
  class:bad={danger}
  class:dis={disabled}
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onchange(!checked)}></button>
