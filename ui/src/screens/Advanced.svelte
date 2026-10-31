<script lang="ts">
  import Checkbox from "../components/Checkbox.svelte";
  import Icon from "../components/Icon.svelte";
  import TitleBar from "../components/TitleBar.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent, openWindow } from "../lib/window.svelte";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";
  import type { SettingsView, TypingView } from "../lib/bindings";

  const settings = $derived(app.settings);
  const mac = $derived(platformState.mac);

  const setTyping = (patch: Partial<TypingView>) => app.patch({ typing: patch });
  const setSystem = (patch: Partial<SettingsView["system"]>) => app.patch({ system: patch });

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
</script>

<div class="v4win">
  <TitleBar title={t("win.advanced.title")} subtitle="VietKB" showMark={false} showMinimize={false} />

  <main class="v4sheet">
    <div class="v4sh">{t("adv.sec.typing")}</div>
    <div class="v4grid">
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.freeTone ?? true}
          label={t("adv.freeTone")}
          onchange={(next) => setTyping({ freeTone: next })} />{t("adv.freeTone")}
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.autoHornUo ?? true}
          label={t("adv.autoHornUo")}
          onchange={(next) => setTyping({ autoHornUo: next })} />{t("adv.autoHornUo")}
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.autoCapitalize ?? false}
          label={t("adv.autoCapitalize")}
          onchange={(next) => setTyping({ autoCapitalize: next })} />{t("adv.autoCapitalize")}
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.jfwzAsConsonant ?? true}
          label={t("adv.jfwz")}
          onchange={(next) => setTyping({ jfwzAsConsonant: next })} />{t("adv.jfwz")}
      </div>
      <div class="v4ck" class:dis={!settings?.typing.spellCheck}>
        <Checkbox
          checked={settings?.typing.restoreWrongWord ?? false}
          disabled={!settings?.typing.spellCheck}
          label={t("adv.restoreWrong")}
          onchange={(next) => setTyping({ restoreWrongWord: next })} />{t("adv.restoreWrong")}
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.macroWhenDisabled ?? false}
          label={t("adv.macroWhenOff")}
          onchange={(next) => setTyping({ macroWhenDisabled: next })} />{t("adv.macroWhenOff")}
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.macroCaseFollow ?? true}
          label={t("adv.macroCase")}
          onchange={(next) => setTyping({ macroCaseFollow: next })} />{t("adv.macroCase")}
      </div>
    </div>

    <!-- macOS: the clipboard send mode is a Windows injection option (M4). The
         value stays in the config; the Mac backend ignores it. The window is
         61 px shorter there (main.rs). -->
    {#if !mac}
      <div class="v4sh">{t("adv.sec.compat")}</div>
      <div class="v4grid">
        <div class="v4ck">
          <Checkbox
            checked={settings?.typing.useClipboard ?? false}
            label={t("adv.clipboard")}
            onchange={(next) => setTyping({ useClipboard: next })} />{t("adv.clipboard")}
        </div>
      </div>
    {/if}
    <div class="v4sh">{t("adv.sec.system")}</div>
    <div class="v4grid">
      <div class="v4ck">
        <Checkbox
          checked={settings?.system.startWithWindows ?? true}
          label={mac ? t("quick.autostart.mac") : t("quick.autostart.win")}
          onchange={(next) => setSystem({ startWithWindows: next })} />{mac ? t("quick.autostart.mac") : t("quick.autostart.win")}
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.system.notificationSound ?? true}
          label={t("adv.sound")}
          onchange={(next) => setSystem({ notificationSound: next })} />{t("adv.sound")}
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.system.checkUpdates ?? true}
          label={t("adv.checkUpdates")}
          onchange={(next) => setSystem({ checkUpdates: next })} />{t("adv.checkUpdates")}
      </div>
    </div>
    <footer class="v4foot">
      <button type="button" class="v4lk" onclick={resetAll}>
        <Icon name="reset" small />
        {t("adv.resetAll")}
      </button>
      <button type="button" class="v4lk" onclick={() => openWindow("about")}>
        {t("adv.about")}
      </button>
      <span class="sp"></span>
      <button type="button" class="btn pri" onclick={closeCurrent}>{t("common.close")}</button>
    </footer>
  </main>
</div>
