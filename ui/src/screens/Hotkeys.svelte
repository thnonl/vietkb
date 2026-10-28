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
  import { canonicalize } from "../lib/hotkey";
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
      return labelFor(camel as HotkeyActionField);
    }
    for (const [other, value] of Object.entries(settings.hotkeys)) {
      if (other !== field && value === canonical) return labelFor(other as HotkeyActionField);
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

  /** The five hotkeys that do not earn a row of their own. Not persisted. */
  const extraRows: { field: HotkeyActionField; label: string; note: string }[] = [
    {
      field: "appExclude",
      label: "Bật / tắt loại trừ ứng dụng",
      note: "Tạm bỏ qua danh sách loại trừ cho cửa sổ đang mở",
    },
    {
      field: "macroPanel",
      label: "Bảng gõ tắt",
      note: "Mở nhanh danh sách gõ tắt để tra hoặc sửa",
    },
    {
      field: "macroToggle",
      label: "Bật / tắt gõ tắt",
      note: "Tắt gõ tắt mà không phải vào cài đặt",
    },
    {
      field: "enable",
      label: "Bật tiếng Việt",
      note: "Chuyển sang gõ tiếng Việt, giữ nguyên kiểu gõ đang chọn",
    },
    {
      field: "disable",
      label: "Tắt tiếng Việt",
      note: "Trả bàn phím về gõ tiếng Anh bình thường",
    },
  ];

  /** Row label of a field, so a warning names it the way the tab does. */
  const labelFor = (field: HotkeyActionField): string =>
    [...rows, ...tools, ...extraRows].find((row) => row.field === field)?.label ??
    hotkeyActionLabel(field);

  /**
   * Conflicts that exist in the *stored* set, not just in a field being edited.
   * `check_hotkey_conflict` on the backend compares canonical strings; this is the
   * same rule applied to all nine fields in one pass, so the tab is truthful about
   * a config that arrived from the file. Unbound (`""`) never conflicts; a clash is
   * symmetric, so both rows get their own label of the other side.
   */
  const storedConflicts = $derived.by(() => {
    const out: Partial<Record<HotkeyActionField, Problem>> = {};
    const byCanonical = new Map<string, HotkeyActionField>();
    for (const field of Object.keys(DEFAULT_HOTKEYS) as HotkeyActionField[]) {
      const raw = (settings.hotkeys[field] ?? "").trim();
      if (!raw) continue;
      const canonical = canonicalize(raw).canonical ?? raw;
      const other = byCanonical.get(canonical);
      if (!other) {
        byCanonical.set(canonical, field);
        continue;
      }
      if (!out[field]) out[field] = { kind: "conflict", text: labelFor(other) };
      if (!out[other]) out[other] = { kind: "conflict", text: labelFor(field) };
    }
    return out;
  });

  /** Live edit problem wins; otherwise the stored-set conflict, if any. */
  const problemFor = (field: HotkeyActionField): Problem =>
    problems[field] ?? storedConflicts[field] ?? null;

  let extraOpen = $state(false);
</script>

<div class="hkbody">
<div class="intro">
  Bấm vào ô, nhấn tổ hợp mới. <Keycap text="Esc" small /> để huỷ,
  <Keycap text="Backspace" small /> để xoá.
</div>

<div class="sh">Chuyển chế độ</div>
{#if extraOpen}
  <div class="list">
    {#each extraRows as row (row.field)}
      <Row label={row.label} short tall={Boolean(problemFor(row.field))}>
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
          {#if problemFor(row.field)}
            <div class="msg">
              <Icon name="warning" small />
              {problemFor(row.field)?.kind === "conflict"
                ? `Trùng với “${problemFor(row.field)?.text}”`
                : problemFor(row.field)?.text}
            </div>
          {/if}
        {/snippet}
      </Row>
    {/each}
    <div class="row s">
      <button type="button" class="link" onclick={() => (extraOpen = false)}>
        ‹ Phím tắt khác
      </button>
      <span class="tag n" style="margin-left:auto">5 phím</span>
    </div>
  </div>
{:else}
  <div class="list">
    {#each rows as row (row.field)}
      <Row label={row.label} tall={Boolean(problemFor(row.field))}>
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
          {#if problemFor(row.field)}
            <div class="msg">
              <Icon name="warning" small />
              {problemFor(row.field)?.kind === "conflict"
                ? `Trùng với “${problemFor(row.field)?.text}”`
                : problemFor(row.field)?.text}
            </div>
          {/if}
        {/snippet}
      </Row>
    {/each}
  </div>
  <button type="button" class="disc" onclick={() => (extraOpen = true)}>
    <Icon name="arrow-right" />
    <b>Phím tắt khác</b>
    <span class="tag n"><span class="num">5</span>&nbsp;phím</span>
  </button>
{/if}

{#if !extraOpen}
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
{/if}

<div class="sh">Công cụ</div>
<div class="list">
  {#each tools as row (row.field)}
    <Row label={row.label} tall={Boolean(problemFor(row.field))}>
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
        {#if problemFor(row.field)}
          <div class="msg">
            <Icon name="warning" small />
            {#if problemFor(row.field)?.kind === "conflict"}
              Trùng với “{problemFor(row.field)?.text}”
            {:else if problemFor(row.field)?.kind === "warn" && row.field === "restoreWord"}
              Cũng là phím Làm lại ở nhiều ứng dụng; không có từ để khôi phục thì vẫn Làm lại
            {:else}
              {problemFor(row.field)?.text}
            {/if}
          </div>
        {/if}
      {/snippet}
    </Row>
  {/each}
</div>
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
