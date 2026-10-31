<script lang="ts">
  import CaptureField from "../components/CaptureField.svelte";
  import Icon from "../components/Icon.svelte";
  import Segmented from "../components/Segmented.svelte";
  import TitleBar from "../components/TitleBar.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent } from "../lib/window.svelte";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";
  import { DEFAULT_HOTKEYS, hotkeyActionLabel } from "../lib/catalog";
  import { canonicalize, prettyHotkey } from "../lib/hotkey";
  import type { HotkeyActionField } from "../lib/catalog";
  import type { HotkeyField } from "../lib/bindings";

  const settings = $derived(app.settings);
  const native = api.isTauri();

  // The four actions that earned a row in v4. Everything else stays in
  // config.toml or on the main window; the file is not a feature list.
  const ROWS: { field: HotkeyActionField; labelKey: string }[] = [
    { field: "toggle", labelKey: "hotkeys.row.toggle" },
    { field: "cycleMethod", labelKey: "hotkeys.row.cycleMethod" },
    { field: "restoreWord", labelKey: "hotkeys.row.restoreWord" },
    { field: "openSettings", labelKey: "hotkeys.row.openSettings" },
  ];
  const rows = $derived(ROWS.map((row) => ({ field: row.field, label: t(row.labelKey) })));

  const save = (field: HotkeyActionField, next: string) => {
    app.patch({ hotkeys: { [field]: next } });
  };

  /** Row wording when the action has a row; the catalogue label otherwise. */
  const labelForField = (field: string): string =>
    rows.find((row) => row.field === field)?.label ?? hotkeyActionLabel(field);

  /** Stored form, canonicalised; `config.toml` is a supported editor. */
  const canonicalValue = (value: string | null | undefined): string => {
    if (!value || !value.trim()) return "";
    return canonicalize(value).canonical ?? value.trim();
  };

  const canonicalHotkeys = $derived.by(() => {
    const out: Partial<Record<HotkeyActionField, string>> = {};
    const hotkeys = settings?.hotkeys;
    if (!hotkeys) return out;
    for (const field of Object.keys(hotkeys) as HotkeyField[]) {
      if (field === "toggleMode") continue;
      const value = hotkeys[field];
      out[field as HotkeyActionField] = canonicalValue(typeof value === "string" ? value : null);
    }
    return out;
  });

  /** Every conflicting pair, both sides marked. An unbound hotkey never clashes. */
  const storedConflicts = $derived.by(() => {
    const out: Partial<Record<HotkeyActionField, string>> = {};
    const bound = (Object.entries(canonicalHotkeys) as [HotkeyActionField, string][]).filter(
      ([, value]) => value !== "",
    );
    for (const [field, value] of bound) {
      const others = bound
        .filter(([other, otherValue]) => other !== field && otherValue === value)
        .map(([other]) => labelForField(other));
      if (others.length > 0) {
        out[field] = t("hotkeys.conflict.stored", {
          key: prettyHotkey(value),
          others: others.join(", "),
        });
      }
    }
    return out;
  });

  type Problem = { kind: "conflict" | "error" | "warn"; text: string };
  let live = $state<Partial<Record<HotkeyActionField, Problem | null>>>({});
  /** The other row in a pending (not yet committed) clash, marked alongside. */
  let clashPartner = $state<HotkeyActionField | null>(null);

  const problemFor = (field: HotkeyActionField): Problem | null => {
    const current = live[field];
    if (current) return current;
    const stored = storedConflicts[field];
    return stored ? { kind: "conflict", text: stored } : null;
  };

  const report = (field: HotkeyActionField, problem: Problem | null) => {
    const partner = clashPartner;
    clashPartner = null;
    if (!problem) {
      live = { ...live, [field]: null, ...(partner ? { [partner]: null } : {}) };
      return;
    }
    const text =
      problem.kind === "conflict"
        ? t("hotkeys.conflict.pending", { action: problem.text })
        : problem.text;
    live = { ...live, [field]: { kind: problem.kind, text } };
    if (problem.kind === "conflict") {
      const other = rows.find((row) => row.label === problem.text)?.field ?? null;
      if (other && other !== field) {
        clashPartner = other;
        live = {
          ...live,
          [other]: {
            kind: "conflict",
            text: t("hotkeys.conflict.pending", { action: labelForField(field) }),
          },
        };
      }
    }
  };

  const conflictFor = async (
    field: HotkeyActionField,
    canonical: string,
  ): Promise<string | null> => {
    if (native) {
      const clash = await api.checkHotkeyConflict(canonical, field);
      if (!clash) return null;
      const camel = clash.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());
      return labelForField(camel);
    }
    for (const [other, value] of Object.entries(settings?.hotkeys ?? {})) {
      if (other === "toggleMode" || other === field) continue;
      if (canonicalValue(value as string | null) === canonical) return labelForField(other);
    }
    return null;
  };

  const resetAll = async () => {
    await app.resetSection("hotkeys");
    live = {};
    clashPartner = null;
    app.toast("success", t("hotkeys.resetDone"));
  };
</script>

<div class="v4win">
  <TitleBar title={t("win.hotkeys.title")} subtitle="VietKB" showMark={false} showMinimize={false} />

  <main class="v4sheet">
    <div class="v4scroll">
    <div class="v4intro">
      {t("hotkeys.intro.a")}
      <span class="k sm">Esc</span> {t("hotkeys.intro.esc")}
      <span class="k sm">{platformState.mac ? "⌫" : "Backspace"}</span> {t("hotkeys.intro.del")}
    </div>

    {#each rows as row, index (row.field)}
      {@const problem = problemFor(row.field)}
      <div
        class="v4row hkrow"
        class:noline={index === 0}
        class:tall={problem !== null}>
        <div class="l"><b>{row.label}</b></div>
        <CaptureField
          width={248}
          field={row.field}
          value={settings?.hotkeys[row.field] ?? null}
          label={row.label}
          defaultValue={DEFAULT_HOTKEYS[row.field]}
          conflictText={storedConflicts[row.field] ?? null}
          oncommit={(next) => save(row.field, next)}
          onreset={() => save(row.field, DEFAULT_HOTKEYS[row.field])}
          onproblem={(next) => report(row.field, next)}
          checkConflict={(canonical) => conflictFor(row.field, canonical)} />
        {#if problem}
          <div class="msg" role="status">
            <Icon name="warning" small />
            {problem.text}
          </div>
        {/if}
      </div>
    {/each}

    <div class="v4row hkrow">
      <div class="l"><b>{t("hotkeys.mode")}</b></div>
      <Segmented
        value={settings?.hotkeys.toggleMode ?? "release"}
        label={t("hotkeys.mode")}
        onchange={(mode) => app.patch({ hotkeys: { toggleMode: mode } })}
        options={[
          { value: "release", label: t("hotkeys.mode.release") },
          { value: "press", label: t("hotkeys.mode.press") },
        ]} />
    </div>
    </div>

    <footer class="v4foot">
      <button type="button" class="v4lk" onclick={resetAll}>
        <Icon name="reset" small />
        {t("hotkeys.resetAll")}
      </button>
      <span class="sp"></span>
      <button type="button" class="btn pri" onclick={closeCurrent}>{t("common.close")}</button>
    </footer>
  </main>
</div>
