<script lang="ts">
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";
  import Row from "./Row.svelte";
  import Select from "./Select.svelte";
  import Toggle from "./Toggle.svelte";
  import { app, errorText } from "../lib/state.svelte";
  import { METHODS } from "../lib/catalog";
  import * as api from "../lib/api";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";
  import type { AppRuleRow, RunningApp } from "../lib/bindings";

  // C10: the rule editor needs more room than the panel, so it is a dialog.
  let {
    open = $bindable(false),
    onchanged,
  }: { open?: boolean; onchanged?: () => void } = $props();

  let mode = $state<"rules" | "pick">("rules");
  let rules = $state<AppRuleRow[]>([]);
  let running = $state<RunningApp[]>([]);
  let query = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);

  const local = !api.isTauri();
  const DEMO_RULES: AppRuleRow[] = [
    { process: "game.exe", enabled: false, charset: null, method: null },
    { process: "notepad.exe", enabled: true, charset: "unicode", method: "telex" },
  ];

  const loadRules = async () => {
    if (local) {
      rules = [...DEMO_RULES];
      return;
    }
    try {
      rules = await api.listAppRules();
      error = null;
    } catch (failure) {
      error = errorText(failure);
    }
  };

  const loadRunning = async () => {
    busy = true;
    if (local) {
      running = [
        { process: "chrome.exe", title: "Google Chrome" },
        { process: "zalo.exe", title: "Zalo" },
        { process: "code.exe", title: "Visual Studio Code" },
        { process: "excel.exe", title: "Microsoft Excel" },
      ];
      busy = false;
      return;
    }
    try {
      running = await api.listRunningApps();
      error = null;
    } catch (failure) {
      error = errorText(failure);
    } finally {
      busy = false;
    }
  };

  $effect(() => {
    if (open) void loadRules();
  });

  $effect(() => {
    if (open && mode === "pick") void loadRunning();
  });

  const candidates = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    const seen = new Set<string>();
    const out: RunningApp[] = [];
    for (const entry of running) {
      const key = entry.process.trim().toLowerCase();
      if (!key || seen.has(key)) continue;
      if (needle && !key.includes(needle) && !entry.title.toLowerCase().includes(needle)) continue;
      seen.add(key);
      out.push(entry);
    }
    return out.sort((a, b) => a.process.localeCompare(b.process));
  });

  const setEnabled = async (rule: AppRuleRow, next: boolean) => {
    rule.enabled = next;
    if (local) return;
    try {
      await api.setAppRule(rule.process, next);
      onchanged?.();
    } catch (failure) {
      app.fail(t("rules.saveFailed"), failure);
    }
  };

  const setOverride = async (rule: AppRuleRow, value: string) => {
    rule.method = value || null;
    if (local) return;
    try {
      await api.setAppRuleOverrides(rule.process, rule.charset, rule.method);
      onchanged?.();
    } catch (failure) {
      app.fail(t("rules.overrideFailed"), failure);
      await loadRules();
    }
  };

  const removeRule = async (rule: AppRuleRow) => {
    rules = rules.filter((item) => item.process !== rule.process);
    if (local) return;
    try {
      await api.deleteAppRule(rule.process);
      onchanged?.();
    } catch (failure) {
      app.fail(t("rules.deleteFailed"), failure);
      await loadRules();
    }
  };

  const addApp = async (entry: RunningApp) => {
    const process = entry.process.trim();
    if (!process || rules.some((rule) => rule.process.toLowerCase() === process.toLowerCase()))
      return;
    if (local) {
      rules = [...rules, { process, enabled: false, charset: null, method: null }];
      return;
    }
    try {
      await api.setAppRule(process, false);
      onchanged?.();
      await loadRules();
    } catch (failure) {
      app.fail(t("rules.addFailed"), failure);
    }
  };
</script>

<Modal bind:open title={mode === "rules" ? t("rules.title") : t("rules.pickTitle")}>
  {#if mode === "rules"}
    <Row label={t("rules.auto")}>
      {#snippet note()}{t("rules.auto.note")}{/snippet}
      {#snippet control()}
        <Toggle
          checked={app.settings?.apps.rememberPerApp ?? false}
          label={t("rules.auto")}
          onchange={(next) => app.patch({ apps: { rememberPerApp: next } })} />
      {/snippet}
    </Row>

    {#if error}
      <p class="err">{error}</p>
    {:else if rules.length === 0}
      <p class="empty">{t("rules.empty")}</p>
    {:else}
      <ul class="rules">
        {#each rules as rule (rule.process)}
          <li>
            <div class="head">
              <span class="proc">{rule.process}</span>
              <Toggle
                checked={rule.enabled !== false}
                label={t("rules.enableIn", { app: rule.process })}
                onchange={(next) => setEnabled(rule, next)} />
              <button
                type="button"
                class="btn ghost icon sm danger"
                aria-label={t("rules.delete", { app: rule.process })}
                title={t("rules.delete.title")}
                onclick={() => removeRule(rule)}>
                <Icon name="trash" />
              </button>
            </div>
            <div class="overrides">
              <label class="mini">
                <span>{t("rules.method")}</span>
                <Select
                  value={rule.method ?? ""}
                  label={t("rules.methodFor", { app: rule.process })}
                  options={[{ value: "", label: t("rules.methodDefault") }, ...METHODS.map((m) => ({ value: m.slug, label: m.label }))]}
                  onchange={(value) => setOverride(rule, value)} />
              </label>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  {:else}
    <div class="input" style="margin: 4px 0 8px">
      <Icon name="search" />
      <input
        type="search"
        placeholder={t("rules.search")}
        bind:value={query}
        aria-label={t("rules.searchAria")} />
    </div>
    {#if busy}
      <p class="empty">{t("rules.searching")}</p>
    {:else if candidates.length === 0}
      <p class="empty">{t("rules.noneFound")}</p>
    {:else}
      <ul class="apps">
        {#each candidates as entry (entry.process)}
          {@const added = rules.some(
            (rule) => rule.process.toLowerCase() === entry.process.toLowerCase(),
          )}
          <li>
            <button
              type="button"
              class="app"
              class:added
              class:mac={platformState.mac}
              disabled={added}
              onclick={() => addApp(entry)}
              title={added
                ? t("rules.addedTitle", { app: entry.title || entry.process })
                : t("rules.addTitle", { app: entry.title || entry.process })}>
              <span class="proc">{entry.process}</span>
              <span class="title">{added && !platformState.mac ? t("rules.added") : entry.title}</span>
              <Icon name={added ? "check" : "plus"} />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}

  {#snippet footer()}
    {#if mode === "rules"}
      <button type="button" class="btn ghost" onclick={() => (mode = "pick")} style="margin-right:auto">
        <Icon name="plus" />
        {t("rules.pickButton")}
      </button>
    {:else}
      <button type="button" class="btn ghost" onclick={() => (mode = "rules")} style="margin-right:auto">
        {t("rules.back")}
      </button>
    {/if}
    <button type="button" class="btn pri" onclick={() => (open = false)}>{t("common.close")}</button>
  {/snippet}
</Modal>

<style>
  .err {
    font: 400 12px/18px var(--sm);
    color: var(--bad);
    padding: 8px 0;
  }
  .empty {
    font: 400 13px/20px var(--ui);
    color: var(--t3);
    padding: 8px 0;
  }
  .rules,
  .apps {
    list-style: none;
    max-height: 336px;
    overflow-y: auto;
    margin: 0 -4px;
  }
  .rules > li {
    padding: 6px 4px;
    border-bottom: 1px solid var(--line-1);
  }
  .rules > li:last-child {
    border-bottom: 0;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .proc {
    flex: 1;
    font: 500 13px/20px var(--mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .overrides {
    display: grid;
    grid-template-columns: minmax(0, 220px);
    gap: 8px;
    padding: 4px 0 2px;
  }
  .mini {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font: 400 12px/18px var(--sm);
    color: var(--t3);
  }
  .mini :global(.sel-wrap),
  .mini :global(.sel) {
    width: 100%;
  }
  .app {
    display: grid;
    grid-template-columns: 132px minmax(0, 1fr) auto;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 36px;
    padding: 0 8px;
    border-radius: 7px;
    text-align: left;
    color: var(--t1);
  }
  .app:hover:not(:disabled) {
    background: var(--hover);
  }
  .app.added {
    background: var(--acc-tint);
  }
  .app:disabled {
    color: var(--t3);
    cursor: default;
  }
  /* macOS: the app name leads, the bundle id (the rule key) follows in mono. */
  .app.mac {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr) auto;
  }
  .app.mac .title {
    order: -1;
    font: 500 13px/20px var(--ui);
    color: var(--t1);
  }
  .app.mac .proc {
    font: 400 12px/18px var(--mono);
    color: var(--t3);
  }
  .app.mac:disabled .title {
    color: var(--t3);
  }
  .app .title {
    font: 400 12px/18px var(--sm);
    color: var(--t3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
