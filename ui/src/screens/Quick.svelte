<script lang="ts">
  import { onMount } from "svelte";
  import Toggle from "../components/Toggle.svelte";
  import Select from "../components/Select.svelte";
  import AppRulesDialog from "../components/AppRulesDialog.svelte";
  import AccessibilityBanner from "../components/AccessibilityBanner.svelte";
  import SecureInputHint from "../components/SecureInputHint.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent, openTab } from "../lib/window.svelte";
  import { METHODS } from "../lib/catalog";
  import type { AppRuleRow } from "../lib/bindings";
  import { formatNumber, prettyHotkey as formatHotkey } from "../lib/hotkey";
  import { platformState } from "../lib/platform.svelte";
  import { perms } from "../lib/permissions.svelte";
  import { plural, t } from "../lib/lang.svelte";

  // Chung (SPEC section 8.1): three groups. Group 1 is the input, group 2 the typing
  // extras and the lists, group 3 the system switches.
  const native = api.isTauri();
  const settings = $derived(app.settings);
  const hotkey = $derived(settings?.hotkeys.toggle ?? "Ctrl+Shift");
  const prettyHotkey = $derived(formatHotkey(hotkey));
  const mac = $derived(platformState.mac);
  // Windows: "Khởi động cùng Windows"; macOS: "Mở cùng macOS" (27-mac-decisions M4).
  const autostartLabel = $derived(mac ? t("quick.autostart.mac") : t("quick.autostart.win"));
  let axMissing = $state(perms.missing);
  const macroOn = $derived(settings?.typing.macroEnabled ?? true);

  let rules = $state<AppRuleRow[]>([]);
  let dialogOpen = $state(false);

  // The Run key is the truth, not config.toml: a user who removed the entry in
  // Windows' own startup settings must see the switch off here too.
  let autostart = $state(false);
  // Set once the first answer is in. From then on the switch follows the
  // setting, so flipping the same option in Advanced shows here at once (S-10).
  let autostartReady = $state(false);

  onMount(() => {
    if (!native) {
      autostart = settings?.system.startWithWindows ?? false;
      autostartReady = true;
      return;
    }
    void api
      .autostartState()
      .then((registry) => {
        autostart = registry;
        if (settings && settings.system.startWithWindows !== registry) {
          app.patch({ system: { startWithWindows: registry } });
        }
      })
      .catch(() => {
        autostart = settings?.system.startWithWindows ?? false;
      })
      .finally(() => {
        autostartReady = true;
      });
  });

  $effect(() => {
    const saved = settings?.system.startWithWindows;
    if (autostartReady && saved !== undefined) autostart = saved;
  });

  const setAutostart = async (next: boolean) => {
    if (!native) {
      autostart = next;
      app.patch({ system: { startWithWindows: next } });
      return;
    }
    try {
      // The command writes the Run entry and answers with what the registry
      // now holds, so a failed write leaves the switch showing the truth.
      autostart = await api.setAutostart(next);
      if (settings && settings.system.startWithWindows !== autostart) {
        app.patch({ system: { startWithWindows: autostart } });
      }
    } catch (failure) {
      app.fail(
        mac ? t("quick.autostart.fail.mac") : t("quick.autostart.fail.win"),
        failure,
      );
      autostart = settings?.system.startWithWindows ?? false;
    }
  };

  const loadRules = async () => {
    if (!native) {
      rules = Array.from({ length: 6 }, (_, index) => ({
        process: `app-${index + 1}.exe`,
        enabled: null,
        charset: null,
        method: null,
      }));
      return;
    }
    try {
      rules = await api.listAppRules();
    } catch (failure) {
      app.fail(t("quick.rulesLoadFailed"), failure);
    }
  };

  // Also when the list moved without this window: the per-app exclude hotkey
  // (F11) writes the rule from the backend, other windows edit it too.
  $effect(() => {
    void app.rulesRev;
    void loadRules();
  });

  const quit = async () => {
    if (native) await api.quitApp();
  };
</script>

<div class="screen" data-screen>
  <div class="body" data-body>
    <AccessibilityBanner bind:missing={axMissing} />
    <SecureInputHint />

    <section class="v4grp">
      <div class="v4row">
        <div class="l line">
          <b>{t("quick.enable")}</b>
          <span class="h in">{prettyHotkey}</span>
        </div>
        <div class="c">
          <Toggle
            checked={settings?.input.enabled ?? false}
            label={t("quick.enable")}
            onchange={(next) => app.patch({ input: { enabled: next } })} />
        </div>
      </div>
      <div class="v4row">
        <div class="l"><b>{t("quick.method")}</b></div>
        <div class="c">
          <Select
            width={96}
            label={t("quick.method")}
            value={settings?.input.method ?? "telex"}
            options={METHODS.map((method) => ({ value: method.slug, label: method.label }))}
            onchange={(method) => app.patch({ input: { method } })} />
        </div>
      </div>
    </section>

    <section class="v4grp">
      <div class="v4row">
        <div class="l line">
          <b>{t("quick.modernTone")}</b>
          <span class="h serif in">{t("quick.modernTone.example")}</span>
        </div>
        <div class="c">
          <Toggle
            checked={settings?.typing.modernTone ?? true}
            label={t("quick.modernTone")}
            onchange={(next) => app.patch({ typing: { modernTone: next } })} />
        </div>
      </div>
      <div class="v4row">
        <div class="l"><b>{t("quick.spellCheck")}</b></div>
        <div class="c">
          <Toggle
            checked={settings?.typing.spellCheck ?? false}
            label={t("quick.spellCheck")}
            onchange={(next) => app.patch({ typing: { spellCheck: next } })} />
        </div>
      </div>
      <div class="v4row">
        <div class="l"><b>{t("quick.macroAllow")}</b></div>
        <div class="c">
          <button type="button" class="btn" disabled={!macroOn} onclick={() => openTab("macros")}>
            {t("quick.macroPanel")}
          </button>
          <Toggle
            checked={macroOn}
            label={t("quick.macroAllow")}
            onchange={(next) => app.patch({ typing: { macroEnabled: next } })} />
        </div>
      </div>
      <div class="v4row">
        <div class="l line">
          <b>{t("quick.appExclude")}</b>
          <span class="h in"><span class="num">{formatNumber(rules.length)}</span> {plural("quick.appCount", rules.length)}</span>
        </div>
        <div class="c">
          <button type="button" class="btn" onclick={() => (dialogOpen = true)}>{t("quick.appList")}</button>
        </div>
      </div>
    </section>

    <section class="v4grp">
      <div class="v4row h">
        <div class="l">
          <b>{autostartLabel}</b>
          <span class="h">{mac ? t("quick.autostart.hint.mac") : t("quick.autostart.hint.win")}</span>
        </div>
        <div class="c">
          <Toggle checked={autostart} label={autostartLabel} onchange={setAutostart} />
        </div>
      </div>
      <div class="v4row h">
        <div class="l">
          <b>{t("quick.startHidden")}</b>
          <span class="h">{mac ? t("quick.startHidden.hint.mac") : t("quick.startHidden.hint.win")}</span>
        </div>
        <div class="c">
          <Toggle
            checked={settings?.system.startMinimized ?? false}
            label={t("quick.startHidden")}
            onchange={(next) => app.patch({ system: { startMinimized: next } })} />
        </div>
      </div>
    </section>
  </div>

  <footer class="v4foot" data-foot>
    <button
      type="button"
      class="btn ghost"
      title={mac ? t("quick.quit.title.mac") : t("quick.quit.title.win")}
      aria-label={t("quick.quit")}
      onclick={quit}>{t("quick.quit")}</button>
    <span class="sp"></span>
    <button type="button" class="btn pri" onclick={closeCurrent}>{t("common.close")}</button>
  </footer>
</div>

<AppRulesDialog bind:open={dialogOpen} onchanged={loadRules} />
