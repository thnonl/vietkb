<script lang="ts">
  import CaptureField from "../components/CaptureField.svelte";
  import Icon from "../components/Icon.svelte";
  import Keycap from "../components/Keycap.svelte";
  import Row from "../components/Row.svelte";
  import { app } from "../lib/state.svelte";
  import type { SettingsPatch } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { DEFAULT_HOTKEYS, hotkeyActionLabel } from "../lib/catalog";
  import type { HotkeyActionField } from "../lib/catalog";
  import type { SettingsView } from "../lib/bindings";

  let { settings }: { settings: SettingsView } = $props();

  type Problem = { kind: "conflict" | "error" | "warn"; text: string } | null;
  let problems = $state<Record<string, Problem>>({});

  const presets = [
    { combo: "Ctrl+Shift", note: "mặc định" },
    { combo: "Alt+Z", note: "như UniKey" },
    { combo: "Ctrl+`", note: "" },
  ];

  const save = (field: HotkeyActionField, next: string) => {
    app.patch({ hotkeys: { [field]: next } });
  };

  const conflictFor = async (
    field: HotkeyActionField,
    canonical: string,
  ): Promise<string | null> => {
    if (api.isTauri()) {
      const clash = await api.checkHotkeyConflict(canonical, field);
      if (!clash) return null;
      const camel = clash.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());
      return hotkeyActionLabel(camel as HotkeyActionField);
    }
    for (const [other, value] of Object.entries(settings.hotkeys)) {
      if (other !== field && value === canonical) return hotkeyActionLabel(other);
    }
    return null;
  };

  const resetField = (field: HotkeyActionField, label: string) => {
    const before = settings.hotkeys[field];
    save(field, DEFAULT_HOTKEYS[field]);
    app.toast("success", `Đã đặt lại phím ${label}`, 5000, {
      label: "Hoàn tác",
      run: () => save(field, before ?? ""),
    });
  };

  const resetAll = async () => {
    const before = Object.fromEntries(
      Object.entries({ ...settings.hotkeys }).map(([key, value]) => [key, value ?? ""]),
    ) as NonNullable<SettingsPatch["hotkeys"]>;
    await app.resetSection("hotkeys");
    problems = {};
    app.toast("success", "Đã đặt lại tất cả cài đặt", 10000, {
      label: "Hoàn tác",
      run: () => app.patch({ hotkeys: before }),
    });
  };

  const isModifierOnly = (value: string | null): boolean => {
    const parts = (value ?? "").split("+").filter(Boolean);
    return parts.length > 0 && parts.every((part) => ["Ctrl", "Alt", "Shift", "Win"].includes(part));
  };

  const toggleDesc = $derived(
    isModifierOnly(settings.hotkeys.toggle)
      ? "Ctrl + Shift + T trong trình duyệt vẫn mở lại tab"
      : "Nhấn một lần để bật, nhấn lần nữa để tắt",
  );

  const openDesc = $derived(
    (settings.hotkeys.openSettings ?? "") === DEFAULT_HOTKEYS.openSettings
      ? "Mở cửa sổ này từ bất cứ đâu"
      : "Mặc định là Ctrl + Alt + S",
  );

  const rows: {
    field: HotkeyActionField;
    label: string;
    note: string;
    tall?: boolean;
  }[] = $derived([
    { field: "toggle", label: "Bật/tắt tiếng Việt", note: toggleDesc },
    {
      field: "cycleCharset",
      label: "Đổi bảng mã",
      note: "Unicode → TCVN3 → VNI Windows → Unicode tổ hợp",
    },
    { field: "cycleMethod", label: "Đổi kiểu gõ", note: "Telex → VNI → VIQR" },
  ]);

  const tools: { field: HotkeyActionField; label: string; note: string }[] = $derived([
    {
      field: "restoreWord",
      label: "Khôi phục từ gõ sai",
      note: "Đưa từ vừa gõ về đúng phím đã nhấn: gôgle → google",
    },
    { field: "openSettings", label: "Mở cài đặt", note: openDesc },
    {
      field: "convertTool",
      label: "Chuyển mã chữ đã chép",
      note: "Đổi chữ đã chép sang bảng mã đang dùng, rồi dán như thường",
    },
  ]);
</script>

<div class="intro">
  Bấm vào ô, nhấn tổ hợp mới. <Keycap text="Esc" small /> để huỷ,
  <Keycap text="Backspace" small /> để xoá.
</div>

<div class="sh">Chuyển chế độ</div>
<div class="list">
  {#each rows as row (row.field)}
    <Row label={row.label} tall={Boolean(problems[row.field])}>
      {#snippet note()}{row.note}{/snippet}
      {#snippet control()}
        <CaptureField
          field={row.field}
          value={settings.hotkeys[row.field]}
          label={row.label}
          defaultValue={DEFAULT_HOTKEYS[row.field]}
          oncommit={(next) => save(row.field, next)}
          onreset={() => resetField(row.field, row.label)}
          checkConflict={(canonical) => conflictFor(row.field, canonical)}
          onproblem={(problem) => (problems[row.field] = problem)} />
      {/snippet}
      {#snippet extra()}
        {#if problems[row.field]}
          <div class="msg">
            <Icon name="warning" small />
            {problems[row.field]?.kind === "conflict"
              ? `Trùng với “${problems[row.field]?.text}”`
              : problems[row.field]?.text}
          </div>
        {/if}
      {/snippet}
    </Row>
  {/each}
</div>

<div class="presets">
  Chọn nhanh:
  {#each presets as preset (preset.combo)}
    <button
      type="button"
      class="pre"
      class:on={(settings.hotkeys.toggle ?? "") === preset.combo}
      onclick={() => save("toggle", preset.combo)}>
      {#each preset.combo.split("+") as key, index (key)}
        {#if index > 0}<span class="plus">+</span>{/if}
        <Keycap text={key} small />
      {/each}
      {#if preset.note}<small>{preset.note}</small>{/if}
    </button>
  {/each}
</div>

<div class="sh">Công cụ</div>
<div class="list">
  {#each tools as row (row.field)}
    <Row label={row.label} tall={Boolean(problems[row.field])}>
      {#snippet note()}{row.note}{/snippet}
      {#snippet control()}
        <CaptureField
          field={row.field}
          value={settings.hotkeys[row.field]}
          label={row.label}
          defaultValue={DEFAULT_HOTKEYS[row.field]}
          oncommit={(next) => save(row.field, next)}
          onreset={() => resetField(row.field, row.label)}
          checkConflict={(canonical) => conflictFor(row.field, canonical)}
          onproblem={(problem) => (problems[row.field] = problem)} />
      {/snippet}
      {#snippet extra()}
        {#if problems[row.field]}
          <div class="msg">
            <Icon name="warning" small />
            {#if problems[row.field]?.kind === "conflict"}
              Trùng với “{problems[row.field]?.text}”
            {:else if problems[row.field]?.kind === "warn" && row.field === "restoreWord"}
              Cũng là phím Làm lại ở nhiều ứng dụng; không có từ để khôi phục thì vẫn Làm lại
            {:else}
              {problems[row.field]?.text}
            {/if}
          </div>
        {/if}
      {/snippet}
    </Row>
  {/each}
</div>

<div class="hkfoot">
  <button type="button" class="btn ghost sm" onclick={resetAll} style="margin-left:-10px">
    <Icon name="reset" />
    Đặt lại tất cả
  </button>
  <span class="m">
    Windows đã dùng: <Keycap text="Win" small /><span class="plus">+</span><Keycap text="L" small />
    &nbsp;<Keycap text="Alt" small /><span class="plus">+</span><Keycap text="Tab" small />
    &nbsp;<Keycap text="Ctrl" small /><span class="plus">+</span><Keycap text="Alt" small /><span class="plus">+</span><Keycap text="Del" small />
  </span>
</div>
