<script lang="ts">
  import Checkbox from "../components/Checkbox.svelte";
  import Icon from "../components/Icon.svelte";
  import Select from "../components/Select.svelte";
  import TitleBar from "../components/TitleBar.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent, openWindow } from "../lib/window.svelte";
  import type { SettingsView, TypingView } from "../lib/bindings";

  const settings = $derived(app.settings);

  const setTyping = (patch: Partial<TypingView>) => app.patch({ typing: patch });
  const setSystem = (patch: Partial<SettingsView["system"]>) => app.patch({ system: patch });

  const suggestion = $derived(
    settings?.typing.suggestionFix2
      ? "fix2"
      : settings?.typing.suggestionFix1
        ? "fix1"
        : "off",
  );
  const setSuggestion = (value: string) =>
    setTyping({ suggestionFix1: value === "fix1", suggestionFix2: value === "fix2" });

  const delay = $derived(settings?.typing.sendDelayUs ?? 0);
  const setDelay = (value: number) => {
    const clamped = Math.min(5000, Math.max(0, Number.isFinite(value) ? value : 0));
    setTyping({ sendDelayUs: clamped });
  };

  const resetAll = async () => {
    const before = $state.snapshot(app.settings) as SettingsView | null;
    await app.resetSection("all");
    app.toast("success", "Đã đặt lại tất cả cài đặt", 10000, {
      label: "Hoàn tác",
      run: () => {
        if (!before) return;
        void api.setSettings(before).then(() => app.reload());
      },
    });
  };
</script>

<div class="v4win">
  <TitleBar title="Nâng cao" subtitle="VietKB" showMark={false} showMinimize={false} />

  <main class="v4sheet">
    <div class="v4sh">Gõ</div>
    <div class="v4grid">
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.freeTone ?? true}
          label="Gõ dấu ở cuối từ"
          onchange={(next) => setTyping({ freeTone: next })} />Gõ dấu ở cuối từ
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.autoHornUo ?? true}
          label="Tự sửa ưo thành ươ"
          onchange={(next) => setTyping({ autoHornUo: next })} />Tự sửa ưo thành ươ
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.autoCapitalize ?? false}
          label="Viết hoa đầu câu"
          onchange={(next) => setTyping({ autoCapitalize: next })} />Viết hoa đầu câu
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.rawZone ?? true}
          label="Giữ nguyên email, địa chỉ web"
          onchange={(next) => setTyping({ rawZone: next })} />Giữ nguyên email, địa chỉ web
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.jfwzAsConsonant ?? true}
          label="Cho phép f j w z làm phụ âm"
          onchange={(next) => setTyping({ jfwzAsConsonant: next })} />Cho phép f j w z làm phụ âm
      </div>
      <div class="v4ck" class:dis={!settings?.typing.spellCheck}>
        <Checkbox
          checked={settings?.typing.restoreWrongWord ?? false}
          disabled={!settings?.typing.spellCheck}
          label="Tự động khôi phục từ gõ sai"
          onchange={(next) => setTyping({ restoreWrongWord: next })} />Tự động khôi phục từ gõ sai
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.macroWhenDisabled ?? false}
          label="Vẫn dùng gõ tắt khi đã tắt tiếng Việt"
          onchange={(next) => setTyping({ macroWhenDisabled: next })} />Vẫn dùng gõ tắt khi đã tắt tiếng Việt
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.macroCaseFollow ?? true}
          label="Viết hoa theo từ tắt"
          onchange={(next) => setTyping({ macroCaseFollow: next })} />Viết hoa theo từ tắt
      </div>
    </div>

    <div class="v4sh">Tương thích</div>
    <div class="v4grid">
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.metroFix ?? false}
          label="Hỗ trợ ứng dụng từ Microsoft Store"
          onchange={(next) => setTyping({ metroFix: next })} />Hỗ trợ ứng dụng từ Microsoft Store
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.useClipboard ?? false}
          label="Gửi chữ qua Clipboard"
          onchange={(next) => setTyping({ useClipboard: next })} />Gửi chữ qua Clipboard
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.typing.disableOnNonUsLayout ?? true}
          label="Tắt khi chuyển bố cục bàn phím khác"
          onchange={(next) => setTyping({ disableOnNonUsLayout: next })} />Tắt khi chuyển bố cục bàn phím khác
      </div>
    </div>
    <div class="v4sr">
      <span class="lb">Sửa lỗi đặt dấu</span>
      <Select
        width={96}
        label="Sửa lỗi đặt dấu"
        value={suggestion}
        options={[
          { value: "off", label: "Tắt" },
          { value: "fix1", label: "Cách 1" },
          { value: "fix2", label: "Cách 2" },
        ]}
        onchange={setSuggestion} />
      <span class="sp"></span>
      <span class="lb">Độ trễ giữa các phím</span>
      <span class="v4num">
        <input
          type="number"
          min="0"
          max="5000"
          step="500"
          value={delay}
          aria-label="Độ trễ giữa các phím, micro giây"
          onchange={(event) => setDelay(Number((event.currentTarget as HTMLInputElement).value))} />
        <small>µs</small>
      </span>
    </div>

    <div class="v4sh">Hệ thống</div>
    <div class="v4grid">
      <div class="v4ck">
        <Checkbox
          checked={settings?.system.startWithWindows ?? true}
          label="Khởi động cùng Windows"
          onchange={(next) => setSystem({ startWithWindows: next })} />Khởi động cùng Windows
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.system.runAsAdmin ?? false}
          label="Chạy bằng quyền quản trị"
          onchange={(next) => setSystem({ runAsAdmin: next })} />Chạy bằng quyền quản trị
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.system.notificationSound ?? true}
          label="Âm thanh khi bật/tắt"
          onchange={(next) => setSystem({ notificationSound: next })} />Âm thanh khi bật/tắt
      </div>
      <div class="v4ck">
        <Checkbox
          checked={settings?.system.checkUpdates ?? true}
          label="Tự kiểm tra cập nhật"
          onchange={(next) => setSystem({ checkUpdates: next })} />Tự kiểm tra cập nhật
      </div>
    </div>
    <div class="v4sr">
      <span class="lb">Màu nền</span>
      <Select
        width={132}
        label="Màu nền"
        value={settings?.system.theme ?? "system"}
        options={[
          { value: "system", label: "Theo Windows" },
          { value: "light", label: "Sáng" },
          { value: "dark", label: "Tối" },
        ]}
        onchange={(theme) => setSystem({ theme })} />
    </div>

    <footer class="v4foot">
      <button type="button" class="v4lk" onclick={resetAll}>
        <Icon name="reset" small />
        Đặt lại tất cả…
      </button>
      <button type="button" class="v4lk" onclick={() => openWindow("about")}>
        Giới thiệu…
      </button>
      <span class="sp"></span>
      <button type="button" class="btn pri" onclick={closeCurrent}>Đóng</button>
    </footer>
  </main>
</div>
