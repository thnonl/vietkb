<script lang="ts">
  import Toggle from "../components/Toggle.svelte";
  import TitleBar from "../components/TitleBar.svelte";
  import Select from "../components/Select.svelte";
  import AppRulesDialog from "../components/AppRulesDialog.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent, openWindow } from "../lib/window.svelte";
  import { METHODS } from "../lib/catalog";
  import type { AppRuleRow } from "../lib/bindings";
  import { formatNumber } from "../lib/hotkey";

  const native = api.isTauri();
  const settings = $derived(app.settings);
  const hotkey = $derived(settings?.hotkeys.toggle ?? "Ctrl+Shift");
  const prettyHotkey = $derived(hotkey.split("+").join(" + "));

  let rules = $state<AppRuleRow[]>([]);
  let dialogOpen = $state(false);

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
      app.fail("Không đọc được quy tắc", failure);
    }
  };

  $effect(() => {
    void loadRules();
  });

  const quit = async () => {
    if (native) await api.quitApp();
  };
</script>

<div class="v4win">
  <TitleBar title="VietKB" />

  <main class="v4sheet">
    <div class="v4pad"></div>

    <div class="v4row main noline">
      <div class="l"><b>Bật tiếng Việt</b><span class="hk">{prettyHotkey}</span></div>
      <div class="c">
        <Toggle
          checked={settings?.input.enabled ?? false}
          label="Bật tiếng Việt"
          onchange={(next) => app.patch({ input: { enabled: next } })} />
      </div>
    </div>

    <div class="v4row">
      <div class="l"><b>Kiểu gõ</b></div>
      <div class="c">
        <Select
          width={96}
          label="Kiểu gõ"
          value={settings?.input.method ?? "telex"}
          options={METHODS.map((method) => ({ value: method.slug, label: method.label }))}
          onchange={(method) => app.patch({ input: { method } })} />
      </div>
    </div>

    <div class="v4sep"></div>

    <div class="v4row noline">
      <div class="l"><b>Đặt dấu kiểu mới</b><span class="h serif">hòa, thúy</span></div>
      <div class="c">
        <Toggle
          checked={settings?.typing.modernTone ?? true}
          label="Đặt dấu kiểu mới"
          onchange={(next) => app.patch({ typing: { modernTone: next } })} />
      </div>
    </div>

    <div class="v4row">
      <div class="l"><b>Kiểm tra chính tả</b></div>
      <div class="c">
        <Toggle
          checked={settings?.typing.spellCheck ?? true}
          label="Kiểm tra chính tả"
          onchange={(next) => app.patch({ typing: { spellCheck: next } })} />
      </div>
    </div>

    <div class="v4row">
      <div class="l"><b>Cho phép gõ tắt</b></div>
      <div class="c">
        <button type="button" class="btn" onclick={() => openWindow("macros")}>
          Bảng gõ tắt…
        </button>
        <Toggle
          checked={settings?.typing.macroEnabled ?? true}
          label="Cho phép gõ tắt"
          onchange={(next) => app.patch({ typing: { macroEnabled: next } })} />
      </div>
    </div>

    <div class="v4row">
      <div class="l">
        <b>Loại trừ ứng dụng</b>
        <span class="h"><span class="num">{formatNumber(rules.length)}</span> ứng dụng</span>
      </div>
      <div class="c">
        <button type="button" class="btn" onclick={() => (dialogOpen = true)}>Quản lý…</button>
      </div>
    </div>

    <footer class="v4foot">
      <button type="button" class="v4lk" onclick={() => openWindow("hotkeys")}>Phím tắt…</button>
      <button type="button" class="v4lk" onclick={() => openWindow("advanced")}>Nâng cao…</button>
      <span class="sp"></span>
      <button type="button" class="btn ghost" onclick={quit}>Thoát VietKB</button>
      <button type="button" class="btn pri" onclick={closeCurrent}>Đóng</button>
    </footer>
  </main>
</div>

<AppRulesDialog bind:open={dialogOpen} onchanged={loadRules} />
