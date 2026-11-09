<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import * as api from "../lib/api";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";

  // macOS only (S-18). Another app (Terminal "Secure Keyboard Entry", a password
  // manager that forgot to let go, a password field) holds secure event input, so
  // VietKB passes every key through unconverted. This is a quiet, non-blocking line
  // on the Chung page, shown only while it is true; nothing asks the user to act.
  //
  // The line is part of the page, so the window follows its height (fit.ts, run
  // from App.svelte): 28 px line + 8 px top margin, the same as `.sechint` in v4.css.
  const POLL_MS = 2000;

  let blocking = $state(false);
  const native = api.isTauri();

  const refresh = async () => {
    if (!native || !platformState.mac) return;
    try {
      const status = await api.getStatus();
      const now = status.secureInputBlocking === true;
      if (now !== blocking) blocking = now;
    } catch {
      // Status not answering: say nothing rather than a false alarm.
    }
  };

  onMount(() => {
    void refresh();
    const timer = setInterval(() => void refresh(), POLL_MS);
    return () => clearInterval(timer);
  });
</script>

{#if platformState.mac && blocking}
  <div class="sechint" role="status">
    <span class="ico"><Icon name="warning" small /></span>
    <span>{t("secure.hint")}</span>
  </div>
{/if}
