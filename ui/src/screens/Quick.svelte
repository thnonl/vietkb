<script lang="ts">
  import { onMount } from "svelte";
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

  // The Run key is the truth, not config.toml: a user who removed the entry in
  // Windows' own startup settings must see the switch off here too.
  let autostart = $state(false);

  onMount(() => {
    if (!native) {
      autostart = settings?.system.startWithWindows ?? false;
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
      });
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
      app.fail("Không đổi được khởi động cùng Windows", failure);
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
        <button type="button" class="btn" onclick={() => (dialogOpen = true)}>Danh sách ứng dụng…</button>
      </div>
    </div>

    <div class="v4row sys">
      <div class="l">
        <b>Khởi động cùng Windows</b>
        <span class="h">Chạy ngay khi đăng nhập Windows</span>
      </div>
      <div class="c">
        <Toggle checked={autostart} label="Khởi động cùng Windows" onchange={setAutostart} />
      </div>
    </div>

    <div class="v4row sys">
      <div class="l">
        <b>Chạy ẩn</b>
        <span class="h">Mở lên không hiện cửa sổ, chỉ nằm trong khay</span>
      </div>
      <div class="c">
        <Toggle
          checked={settings?.system.startMinimized ?? false}
          label="Chạy ẩn"
          onchange={(next) => app.patch({ system: { startMinimized: next } })} />
      </div>
    </div>

    <footer class="v4foot">
      <button type="button" class="v4lk" onclick={() => openWindow("hotkeys")}>Phím tắt…</button>
      <button type="button" class="v4lk" onclick={() => openWindow("advanced")}>Nâng cao…</button>
      <span class="sp"></span>
      <button
        type="button"
        class="btn ghost"
        title="Thoát hẳn, tắt luôn biểu tượng trong khay"
        aria-label="Thoát hẳn"
        onclick={quit}>Thoát hẳn</button>
      <button type="button" class="btn pri" onclick={closeCurrent}>Đóng</button>
    </footer>
  </main>
</div>

<AppRulesDialog bind:open={dialogOpen} onchanged={loadRules} />
