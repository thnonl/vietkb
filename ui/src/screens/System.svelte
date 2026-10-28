<script lang="ts">
  import AppRulesDialog from "../components/AppRulesDialog.svelte";
  import Icon from "../components/Icon.svelte";
  import Row from "../components/Row.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { charsetShort, methodLabel } from "../lib/catalog";
  import { formatNumber } from "../lib/hotkey";
  import type { AppRuleRow, SettingsView } from "../lib/bindings";

  let { settings }: { settings: SettingsView } = $props();

  const sys = $derived(settings.system);
  const set = (patch: Partial<SettingsView["system"]>) => app.patch({ system: patch });

  let rules = $state<AppRuleRow[]>([]);
  let dialogOpen = $state(false);
  let confirmReset = $state(false);
  let updateMsg = $state<string | null>(null);
  let updateTimer: ReturnType<typeof setTimeout> | undefined;

  const loadRules = async () => {
    if (!api.isTauri()) {
      rules = [
        { process: "WindowsTerminal.exe", enabled: false, charset: null, method: null },
        { process: "Code.exe", enabled: false, charset: null, method: null },
        { process: "acad.exe", enabled: true, charset: "tcvn3", method: null },
      ];
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

  const ruleStatus = (rule: AppRuleRow) => {
    if (rule.enabled === false) return "Tắt tiếng Việt";
    if (rule.charset && rule.method) return "Tuỳ chỉnh";
    if (rule.charset) return `Bảng mã ${charsetShort(rule.charset)}`;
    if (rule.method) return `Kiểu gõ ${methodLabel(rule.method)}`;
    return "Bật tiếng Việt";
  };

  const openFolder = async () => {
    try {
      await api.openConfigFolder();
    } catch (failure) {
      app.fail("Không mở được thư mục", failure);
    }
  };

  const restart = async () => {
    try {
      if (api.isTauri()) await api.restartApp();
      else app.toast("info", "Cần mở lại VietKB");
    } catch (failure) {
      app.fail("Không mở lại được", failure);
    }
  };

  const resetAll = async () => {
    const before = $state.snapshot(app.settings) as SettingsView | null;
    confirmReset = false;
    await app.resetSection("all");
    app.toast("success", "Đã đặt lại tất cả cài đặt", 10000, {
      label: "Hoàn tác",
      run: () => {
        if (!before) return;
        void api.setSettings(before).then(() => app.reload());
      },
    });
  };

  const checkUpdates = () => {
    if (updateTimer) clearTimeout(updateTimer);
    updateMsg = "Chưa kiểm tra được, có thể máy đang mất mạng";
    updateTimer = setTimeout(() => (updateMsg = null), 3000);
  };

  const updateNote = $derived(
    updateMsg ??
      `Đang dùng bản ${app.info?.version ?? "0.1.0"} · chỉ xem có bản mới, không gửi gì khác`,
  );

  const configPath = $derived(app.info?.configPath ?? "%APPDATA%\\VietKB\\config.toml");
</script>

<section class="hang">
  <h3>Khởi động</h3>
  <div class="list">
    <Row label="Khởi động cùng Windows">
      {#snippet note()}Bật máy là gõ được ngay, biểu tượng nằm cạnh đồng hồ{/snippet}
      {#snippet control()}
        <Toggle
          checked={sys.startWithWindows}
          label="Khởi động cùng Windows"
          onchange={(next) => set({ startWithWindows: next })} />
      {/snippet}
    </Row>
    <Row
      label="Chạy với quyền Admin"
      icon="shield"
      link={sys.runAsAdmin ? "Mở lại ngay" : undefined}
      linkBad
      onLink={restart}>
      {#snippet note()}
        {sys.runAsAdmin
          ? "Cần mở lại VietKB, Windows sẽ hỏi xác nhận"
          : "Cần khi gõ vào ứng dụng cũng chạy quyền Admin"}
      {/snippet}
      {#snippet control()}
        <Toggle
          checked={sys.runAsAdmin}
          danger
          label="Chạy với quyền Admin"
          onchange={(next) => set({ runAsAdmin: next })} />
      {/snippet}
    </Row>
  </div>
</section>

<section class="hang">
  <h3>Giao diện</h3>
  <div class="list">
    <Row label="Âm thanh khi bật/tắt">
      {#snippet note()}Kêu “tách” nhỏ mỗi lần bật hoặc tắt tiếng Việt{/snippet}
      {#snippet control()}
        <Toggle
          checked={sys.notificationSound}
          label="Âm thanh khi bật/tắt"
          onchange={(next) => set({ notificationSound: next })} />
      {/snippet}
    </Row>
    <Row label="Ngôn ngữ hiển thị">
      {#snippet note()}Chữ trên cửa sổ và menu của VietKB, không đổi cách gõ{/snippet}
      {#snippet control()}
        <Segmented
          value={sys.language}
          label="Ngôn ngữ hiển thị"
          onchange={(language) => set({ language })}
          options={[
            { value: "vi", label: "Tiếng Việt" },
            { value: "en", label: "English" },
          ]} />
      {/snippet}
    </Row>
    <Row label="Màu nền">
      {#snippet note()}Theo Windows: tự đổi khi Windows chuyển sáng hoặc tối{/snippet}
      {#snippet control()}
        <Segmented
          value={sys.theme}
          label="Màu nền"
          onchange={(theme) => set({ theme })}
          options={[
            { value: "system", label: "Theo Windows" },
            { value: "light", label: "Sáng" },
            { value: "dark", label: "Tối" },
          ]} />
      {/snippet}
    </Row>
  </div>
</section>

<section class="hang">
  <h3>Cập nhật</h3>
  <div class="list">
    <Row label="Tự kiểm tra cập nhật" link="Kiểm tra ngay" onLink={checkUpdates}>
      {#snippet note()}{updateNote}{/snippet}
      {#snippet control()}
        <Toggle
          checked={sys.checkUpdates}
          label="Tự kiểm tra cập nhật"
          onchange={(next) => set({ checkUpdates: next })} />
      {/snippet}
    </Row>
    {#if confirmReset}
      <Row label="Đặt lại mọi cài đặt về mặc định?">
        {#snippet note()}Gõ tắt và danh sách loại trừ được giữ.{/snippet}
        {#snippet control()}
          <button type="button" class="btn sm" onclick={resetAll}>Đặt lại</button>
          <button type="button" class="btn sm ghost" onclick={() => (confirmReset = false)}>Huỷ</button>
        {/snippet}
      </Row>
    {:else}
      <Row
        label="Tệp cài đặt"
        link="Đặt lại tất cả…"
        linkBad
        onLink={() => (confirmReset = true)}>
        {#snippet note()}<span class="path">{configPath}</span>{/snippet}
        {#snippet control()}
          <button type="button" class="btn sm" onclick={openFolder}>
            <Icon name="folder" />
            Mở thư mục
          </button>
        {/snippet}
      </Row>
    {/if}
  </div>
</section>

<section class="hang">
  <h3 class="s">
    Loại trừ ứng dụng
    <small><span class="num">{formatNumber(rules.length)}</span> ứng dụng</small>
    <button type="button" class="link" onclick={() => (dialogOpen = true)}>
      Quản lý
      <Icon name="arrow-right" small />
    </button>
  </h3>
  <div class="list">
    {#if rules.length === 0}
      <div class="xr">
        <span class="md">Chưa loại trừ ứng dụng nào</span>
      </div>
      <div class="xr">
        <span class="md">Hay dùng cho trò chơi và cửa sổ lệnh</span>
      </div>
    {:else}
      {#each rules.slice(0, 3) as rule (rule.process)}
        <div class="xr">
          <span class="tile">{rule.process.slice(0, 1).toUpperCase()}</span>
          <b>{rule.process}</b>
          <span class="md">{ruleStatus(rule)}</span>
        </div>
      {/each}
    {/if}
  </div>
</section>

<AppRulesDialog bind:open={dialogOpen} onchanged={loadRules} />
