<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  // One setting row: 46 px, label over one-line note, control right-aligned.
  let {
    label,
    note,
    control,
    extra,
    link,
    onLink,
    linkBad = false,
    disabled = false,
    tall = false,
    group = false,
    short = false,
    icon,
  }: {
    label: string;
    note?: Snippet;
    control: Snippet;
    extra?: Snippet;
    link?: string;
    onLink?: () => void;
    linkBad?: boolean;
    disabled?: boolean;
    tall?: boolean;
    group?: boolean;
    short?: boolean;
    icon?: "shield" | "info" | "warning";
  } = $props();
</script>

<div class="row" class:gb={group} class:dis={disabled} class:tall class:s={short}>
  <div class="lbl">
    <b>{#if icon}<Icon name={icon} small />{/if}{label}</b>
    {#if note || link}
      <span>
        {#if note}{@render note()}{/if}
        {#if link}
          {#if note} · {/if}
          <button
            type="button"
            class="lnk"
            class:bad={linkBad}
            onclick={onLink}
            style="background:none;padding:0;font:inherit;text-align:left">{link}</button>
        {/if}
      </span>
    {/if}
    {#if extra}{@render extra()}{/if}
  </div>
  <div class="ctl">{@render control()}</div>
</div>
