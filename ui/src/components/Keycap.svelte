<script lang="ts">
  // A raised keycap in a well — the product's one physical metaphor.
  // `mono` keycaps show literal text (macro triggers), never Mac symbols.
  import { keyLabel } from "../lib/hotkey";

  let {
    text,
    mono = false,
    small = false,
  }: { text: string; mono?: boolean; small?: boolean } = $props();
</script>

<span class="k" class:m={mono} class:sm={small}>{mono ? text : keyLabel(text)}</span>
