<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import Icon from "./Icon.svelte";
  import * as api from "../lib/api";
  import { platformState } from "../lib/platform.svelte";

  // macOS only. VietKB cannot see or type keys until the app is ticked in
  // System Settings > Privacy & Security > Accessibility. The banner shows
  // while `accessibility_status` says no, and its button asks for the grant
  // (the system prompt, which links to System Settings). The app never opens or
  // scripts System Settings itself (27-mac-decisions M3).
  //
  // The main window is fixed-size, so the banner's height is added to the
  // window (and taken off again once the grant is there) instead of squeezing
  // the rows. Keep these numbers equal to main.rs (`content_height(460.0)` and
  // `BANNER_PX`) and `.axbanner` in styles/mac.css.
  const BASE_CONTENT_PX = 424;
  const AX_BANNER_PX = 80;
  const WIDTH_PX = 480;
  const POLL_MS = 2000;

  let { missing = $bindable(false) }: { missing?: boolean } = $props();

  const native = api.isTauri();
  let busy = $state(false);

  const fit = (needsBanner: boolean) => {
    if (!native) return;
    const height = BASE_CONTENT_PX + (needsBanner ? AX_BANNER_PX : 0);
    void getCurrentWindow()
      .setSize(new LogicalSize(WIDTH_PX, height))
      .catch(() => {});
  };

  const refresh = async () => {
    if (!platformState.mac) return;
    try {
      // Browser preview has no backend: show the banner so it can be designed.
      const granted = native ? await api.accessibilityStatus() : false;
      if (granted === missing) {
        missing = !granted;
        fit(missing);
      }
    } catch {
      // Command missing or not answering: show nothing rather than a false alarm.
    }
  };

  const ask = async () => {
    busy = true;
    try {
      if (native) await api.requestAccessibility();
    } catch {
      // The poll reports the real state.
    } finally {
      busy = false;
      void refresh();
    }
  };

  onMount(() => {
    void refresh();
    const timer = setInterval(() => void refresh(), POLL_MS);
    const onFocus = () => void refresh();
    window.addEventListener("focus", onFocus);
    return () => {
      clearInterval(timer);
      window.removeEventListener("focus", onFocus);
    };
  });
</script>

{#if platformState.mac && missing}
  <div class="axbanner" role="alert">
    <span class="ico"><Icon name="warning" /></span>
    <div class="t">
      <b>Cần quyền Trợ năng</b>
      <span>Chưa cấp quyền thì VietKB chưa gõ được tiếng Việt.</span>
    </div>
    <button type="button" class="btn pri" disabled={busy} onclick={ask}>Cấp quyền…</button>
  </div>
{/if}
