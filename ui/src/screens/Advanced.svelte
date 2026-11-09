<script lang="ts">
  import Toggle from "../components/Toggle.svelte";
  import Icon from "../components/Icon.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent } from "../lib/window.svelte";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";
  import type { SettingsView, TypingView } from "../lib/bindings";

  const settings = $derived(app.settings);
  const mac = $derived(platformState.mac);

  const setTyping = (patch: Partial<TypingView>) => app.patch({ typing: patch });
  const setSystem = (patch: Partial<SettingsView["system"]>) => app.patch({ system: patch });

  // Config file and logs live in one folder; stdout is lost when LaunchServices
  // starts the app, so this is where a user finds them (S-19).
  const openFolder = async () => {
    if (!api.isTauri()) return;
    try {
      await api.openConfigFolder();
    } catch (failure) {
      app.fail(t("common.openConfig"), failure);
    }
  };

  const resetAll = async () => {
    const before = $state.snapshot(app.settings) as SettingsView | null;
    await app.resetSection("all");
    app.toast("success", t("adv.resetDone"), 10000, {
      label: t("common.undo"),
      run: () => {
        if (!before) return;
        void api.setSettings(before).then(() => app.reload());
      },
    });
  };

  const autostartLabel = $derived(mac ? t("quick.autostart.mac") : t("quick.autostart.win"));
</script>

<div class="screen" data-screen>
  <div class="body" data-body>
    <!-- Gõ: the typing options (SPEC 8.4, switches not checkboxes). -->
    <section class="v4grp">
      <h6 class="v4sh">{t("adv.sec.typing")}</h6>
      <div class="v4row">
        <div class="l"><b>{t("adv.freeTone")}</b></div>
        <Toggle
          checked={settings?.typing.freeTone ?? true}
          label={t("adv.freeTone")}
          onchange={(next) => setTyping({ freeTone: next })} />
      </div>
      <div class="v4row">
        <div class="l"><b>{t("adv.autoHornUo")}</b></div>
        <Toggle
          checked={settings?.typing.autoHornUo ?? true}
          label={t("adv.autoHornUo")}
          onchange={(next) => setTyping({ autoHornUo: next })} />
      </div>
      <div class="v4row">
        <div class="l"><b>{t("adv.autoCapitalize")}</b></div>
        <Toggle
          checked={settings?.typing.autoCapitalize ?? false}
          label={t("adv.autoCapitalize")}
          onchange={(next) => setTyping({ autoCapitalize: next })} />
      </div>
      <div class="v4row">
        <div class="l"><b>{t("adv.jfwz")}</b></div>
        <Toggle
          checked={settings?.typing.jfwzAsConsonant ?? true}
          label={t("adv.jfwz")}
          onchange={(next) => setTyping({ jfwzAsConsonant: next })} />
      </div>
      <div class="v4row" class:dis={!settings?.typing.spellCheck}>
        <div class="l"><b>{t("adv.restoreWrong")}</b></div>
        <Toggle
          checked={settings?.typing.restoreWrongWord ?? false}
          disabled={!settings?.typing.spellCheck}
          label={t("adv.restoreWrong")}
          onchange={(next) => setTyping({ restoreWrongWord: next })} />
      </div>
      <div class="v4row">
        <div class="l"><b>{t("adv.autoEnglish")}</b></div>
        <Toggle
          checked={settings?.typing.autoRestoreEnglish ?? true}
          label={t("adv.autoEnglish")}
          onchange={(next) => setTyping({ autoRestoreEnglish: next })} />
      </div>
      <div class="v4row" class:dis={!settings?.typing.macroEnabled}>
        <div class="l"><b>{t("adv.macroWhenOff")}</b></div>
        <Toggle
          checked={settings?.typing.macroWhenDisabled ?? false}
          disabled={!settings?.typing.macroEnabled}
          label={t("adv.macroWhenOff")}
          onchange={(next) => setTyping({ macroWhenDisabled: next })} />
      </div>
      <div class="v4row">
        <div class="l"><b>{t("adv.macroCase")}</b></div>
        <Toggle
          checked={settings?.typing.macroCaseFollow ?? true}
          label={t("adv.macroCase")}
          onchange={(next) => setTyping({ macroCaseFollow: next })} />
      </div>
    </section>

    <!-- macOS: the clipboard send mode is a Windows injection option (M4). The value
         stays in the config; the Mac backend ignores it, so the group is not shown. -->
    {#if !mac}
      <section class="v4grp">
        <h6 class="v4sh">{t("adv.sec.compat")}</h6>
        <div class="v4row">
          <div class="l"><b>{t("adv.clipboard")}</b></div>
          <Toggle
            checked={settings?.typing.useClipboard ?? false}
            label={t("adv.clipboard")}
            onchange={(next) => setTyping({ useClipboard: next })} />
        </div>
      </section>
    {/if}

    <section class="v4grp">
      <h6 class="v4sh">{t("adv.sec.system")}</h6>
      <div class="v4row">
        <div class="l"><b>{autostartLabel}</b></div>
        <Toggle
          checked={settings?.system.startWithWindows ?? true}
          label={autostartLabel}
          onchange={(next) => setSystem({ startWithWindows: next })} />
      </div>
      <div class="v4row">
        <div class="l"><b>{t("adv.sound")}</b></div>
        <Toggle
          checked={settings?.system.notificationSound ?? true}
          label={t("adv.sound")}
          onchange={(next) => setSystem({ notificationSound: next })} />
      </div>
      {#if app.showUpdateUi}
        <div class="v4row">
          <div class="l"><b>{t("adv.checkUpdates")}</b></div>
          <Toggle
            checked={settings?.system.checkUpdates ?? true}
            label={t("adv.checkUpdates")}
            onchange={(next) => setSystem({ checkUpdates: next })} />
        </div>
      {/if}
      <div class="v4row">
        <div class="l"><b>{t("common.openConfig")}</b></div>
        <button type="button" class="btn" onclick={openFolder}>{t("common.openConfig")}</button>
      </div>
    </section>
  </div>

  <footer class="v4foot" data-foot>
    <button type="button" class="btn ghost" onclick={resetAll}>
      <Icon name="reset" small />
      {t("adv.resetAll")}
    </button>
    <span class="sp"></span>
    <button type="button" class="btn pri" onclick={closeCurrent}>{t("common.close")}</button>
  </footer>
</div>
