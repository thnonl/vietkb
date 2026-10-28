<script lang="ts">
  let {
    value,
    options,
    onchange,
    label,
    tone = false,
  }: {
    value: string;
    options: { value: string; label: string; example?: string }[];
    onchange: (value: string) => void;
    label: string;
    tone?: boolean;
  } = $props();
</script>

<span class="seg" class:tone role="group" aria-label={label}>
  {#each options as option (option.value)}
    <button
      type="button"
      class:on={value === option.value}
      aria-pressed={value === option.value}
      onclick={() => onchange(option.value)}>
      {option.label}
      {#if option.example}<i>·</i><em>{option.example}</em>{/if}
    </button>
  {/each}
</span>
