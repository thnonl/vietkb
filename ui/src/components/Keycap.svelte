<script lang="ts">
  // A raised keycap in a well — the product's one physical metaphor.
  import { keyLabel } from "../lib/hotkey";

  let {
    text,
    mono = false,
    small = false,
  }: { text: string; mono?: boolean; small?: boolean } = $props();
</script>

<span class="k" class:m={mono} class:sm={small}>{keyLabel(text)}</span>
