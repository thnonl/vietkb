<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import Icon from "./Icon.svelte";
  import * as api from "../lib/api";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";

  // macOS only. VietKB cannot see or type keys until the app is ticked in
  // System Settings > Privacy & Security under both Accessibility (post the
  // converted text) and Input Monitoring (see the keys, S-03). The banner shows
  // while either `accessibility_status` or `input_monitoring_status` says no,
  // and its button asks for the one that is missing (the system prompt, which
  // links to System Settings): Accessibility first. macOS shows that prompt once;
  // after the first ask the button opens the right Privacy & Security pane
  // instead (the app opens a pane for the user, it never scripts System
  // Settings: 27-mac-decisions M3).
  //
  // The banner is part of the page: the window follows the page's height (fit.ts
  // runs from App.svelte when the page changes size), so showing or hiding it
  // adds or removes its 80 px (72 + 8 margin) without any size arithmetic here.
  const POLL_MS = 2000;

  let { missing = $bindable(false) }: { missing?: boolean } = $props();

  const native = api.isTauri();
  let busy = $state(false);
  let axMissing = $state(true);
  let imMissing = $state(false);

  // Whether the system prompt was already shown for each permission (remembered
  // across launches; storage may be unavailable, then the prompt is retried).
  type Pane = "accessibility" | "input_monitoring";
  const KEY = (pane: Pane) => `vkb:asked:${pane}`;
  const wasAsked = (pane: Pane): boolean => {
    try {
      return localStorage.getItem(KEY(pane)) === "1";
    } catch {
      return false;
    }
  };
  let asked = $state<Record<Pane, boolean>>({
    accessibility: wasAsked("accessibility"),
    input_monitoring: wasAsked("input_monitoring"),
  });
  const pane = $derived<Pane>(axMissing ? "accessibility" : "input_monitoring");
  const retry = $derived(asked[pane]);

  const title = $derived(
    axMissing && imMissing ? t("ax.both.title") : imMissing ? t("ax.im.title") : t("ax.title"),
  );
  const body = $derived(
    retry
      ? t("ax.retry")
      : axMissing && imMissing
        ? t("ax.both.body")
        : imMissing
          ? t("ax.im.body")
          : t("ax.body"),
  );

  const refresh = async () => {
    if (!platformState.mac) return;
    try {
      // Browser preview has no backend: show the banner so it can be designed.
      const [ax, im] = native
        ? await Promise.all([api.accessibilityStatus(), api.inputMonitoringStatus()])
        : [false, true];
      axMissing = !ax;
      imMissing = !im;
      const now = !ax || !im;
      if (now !== missing) missing = now;
    } catch {
      // Command missing or not answering: show nothing rather than a false alarm.
    }
  };

  const ask = async () => {
    busy = true;
    try {
      if (native) {
        const target = pane;
        if (asked[target]) {
          // The prompt only appears once: go to the pane.
          await api.openPrivacyPane(target);
        } else {
          if (target === "accessibility") await api.requestAccessibility();
          else await api.requestInputMonitoring();
          asked[target] = true;
          try {
            localStorage.setItem(KEY(target), "1");
          } catch {
            // Not remembered: the next launch asks again.
          }
        }
      }
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
    // macOS serves Input Monitoring from a cache and refreshes it in the
    // background (input_monitoring.rs); a refresh that finds a new answer says
    // so here, so a grant made in System Settings shows without a poll's delay.
    let unlisten: UnlistenFn | null = null;
    let disposed = false;
    if (native && platformState.mac) {
      void listen<boolean>("vkb:input-monitoring", () => void refresh())
        .then((stop) => {
          if (disposed) stop();
          else unlisten = stop;
        })
        .catch(() => {});
    }
    return () => {
      disposed = true;
      unlisten?.();
      clearInterval(timer);
      window.removeEventListener("focus", onFocus);
    };
  });
</script>

{#if platformState.mac && missing}
  <div class="axbanner" role="alert">
    <span class="ico"><Icon name="warning" /></span>
    <div class="t">
      <b>{title}</b>
      <span>{body}</span>
    </div>
    <button type="button" class="btn pri" disabled={busy} onclick={ask}>{retry ? t("ax.openSettings") : t("ax.grant")}</button>
  </div>
{/if}
