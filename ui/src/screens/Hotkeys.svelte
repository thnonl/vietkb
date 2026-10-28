<script lang="ts">
  import CaptureField from "../components/CaptureField.svelte";
  import Icon from "../components/Icon.svelte";
  import Segmented from "../components/Segmented.svelte";
  import TitleBar from "../components/TitleBar.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent } from "../lib/window.svelte";
  import { DEFAULT_HOTKEYS, hotkeyActionLabel } from "../lib/catalog";
  import type { HotkeyActionField } from "../lib/catalog";

  const settings = $derived(app.settings);
  const native = api.isTauri();

  // The six actions that earned a row in v4. Everything else stays in
  // config.toml or on the main window; the file is not a feature list.
  const rows: { field: HotkeyActionField; label: string }[] = [
    { field: "toggle", label: "Bật/tắt tiếng Việt" },
    { field: "cycleCharset", label: "Đổi bảng mã" },
    { field: "cycleMethod", label: "Đổi kiểu gõ" },
    { field: "restoreWord", label: "Khôi phục từ gõ sai" },
    { field: "openSettings", label: "Mở cài đặt" },
    { field: "convertTool", label: "Chuyển mã chữ đã chép" },
  ];

  const save = (field: HotkeyActionField, next: string) => {
    app.patch({ hotkeys: { [field]: next } });
  };

  const conflictFor = async (
    field: HotkeyActionField,
    canonical: string,
  ): Promise<string | null> => {
    if (native) {
      const clash = await api.checkHotkeyConflict(canonical, field);
      if (!clash) return null;
      const camel = clash.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());
      return hotkeyActionLabel(camel);
    }
    for (const [other, value] of Object.entries(settings?.hotkeys ?? {})) {
      if (other !== field && value === canonical) return hotkeyActionLabel(other);
    }
    return null;
  };

  const resetAll = async () => {
    await app.resetSection("hotkeys");
    app.toast("success", "Đã đặt lại tất cả phím tắt");
  };
</script>

<div class="v4win">
  <TitleBar title="Phím tắt" subtitle="VietKB" />

  <main class="v4sheet">
    <div class="v4intro">
      Bấm vào ô, nhấn tổ hợp mới.
      <span class="k sm">Esc</span> để huỷ,
      <span class="k sm">Backspace</span> để xoá.
    </div>

    {#each rows as row, index (row.field)}
      <div class="v4row hkrow" class:noline={index === 0}>
        <div class="l"><b>{row.label}</b></div>
        <CaptureField
          width={248}
          field={row.field}
          value={settings?.hotkeys[row.field] ?? null}
          label={row.label}
          defaultValue={DEFAULT_HOTKEYS[row.field]}
          oncommit={(next) => save(row.field, next)}
          onreset={() => save(row.field, DEFAULT_HOTKEYS[row.field])}
          checkConflict={(canonical) => conflictFor(row.field, canonical)} />
      </div>
    {/each}

    <div class="v4row hkrow">
      <div class="l"><b>Kiểu chuyển bật/tắt</b></div>
      <Segmented
        value={settings?.hotkeys.toggleMode ?? "release"}
        label="Kiểu chuyển bật/tắt"
        onchange={(mode) => app.patch({ hotkeys: { toggleMode: mode } })}
        options={[
          { value: "release", label: "Khi nhả phím" },
          { value: "press", label: "Khi nhấn phím" },
        ]} />
    </div>

    <footer class="v4foot">
      <button type="button" class="v4lk" onclick={resetAll}>
        <Icon name="reset" small />
        Đặt lại tất cả
      </button>
      <span class="sp"></span>
      <button type="button" class="btn pri" onclick={closeCurrent}>Đóng</button>
    </footer>
  </main>
</div>
