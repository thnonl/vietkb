<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    value,
    options,
    onchange,
    label,
    width,
  }: {
    value: string;
    options: { value: string; label: string }[];
    onchange: (value: string) => void;
    label: string;
    width?: number;
  } = $props();
</script>

<span class="sel-wrap" style={width ? `width:${width}px` : ""}>
  <select
    class="sel"
    aria-label={label}
    style={width ? "width:100%" : ""}
    {value}
    onchange={(event) => onchange((event.currentTarget as HTMLSelectElement).value)}>
    {#each options as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
  <Icon name="chevron" small />
</span>
