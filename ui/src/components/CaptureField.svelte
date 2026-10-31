<script lang="ts">
  import { onDestroy } from "svelte";
  import * as api from "../lib/api";
  import Icon from "./Icon.svelte";
  import Keycap from "./Keycap.svelte";
  import { t } from "../lib/lang.svelte";
  import { canonicalize, comboFromEvent, hotkeyHazard, keyToken } from "../lib/hotkey";
  import type { HotkeyActionField } from "../lib/catalog";

  // The capture well: focus, press a chord, Enter commits, Esc cancels,
  // Backspace clears. Shows the exact canonical string the backend stores.
  let {
    field,
    value,
    label,
    defaultValue = "",
    oncommit,
    onreset,
    checkConflict,
    onproblem,
    conflictText = null,
    width = 276,
  }: {
    field: HotkeyActionField;
    value: string | null;
    label: string;
    defaultValue?: string;
    oncommit: (next: string) => void | Promise<void>;
    onreset?: () => void;
    checkConflict?: (canonical: string) => Promise<string | null>;
    onproblem?: (problem: { kind: "conflict" | "error" | "warn"; text: string } | null) => void;
    /** Conflict already present in the stored config, detected on mount. */
    conflictText?: string | null;
    width?: number;
  } = $props();

  let editing = $state(false);
  let draft = $state("");
  let held = $state("");
  let error = $state<string | null>(null);
  let conflict = $state<string | null>(null);
  let warning = $state<string | null>(null);
  let seq = 0;

  // While this field is recording, the keyboard hook must not match hotkeys or
  // swallow keys, or the chord being entered would fire its action instead of
  // reaching the field (spec C.6 step 2, S-07). Only transitions are sent.
  let capturing = false;
  $effect(() => {
    const now = editing;
    if (now === capturing || !api.isTauri()) return;
    capturing = now;
    void api.setHotkeyCapture(now).catch(() => {});
  });
  // The backend lets a capture lapse after ~30 s; renew it while focused.
  $effect(() => {
    if (!editing || !api.isTauri()) return;
    const timer = setInterval(() => {
      // A hidden or unfocused webview must not re-arm what the backend cleared.
      if (document.hasFocus()) void api.setHotkeyCapture(true).catch(() => {});
    }, 10_000);
    return () => clearInterval(timer);
  });
  onDestroy(() => {
    if (capturing && api.isTauri()) void api.setHotkeyCapture(false).catch(() => {});
  });

  // Physical modifiers held since the capture began. A chord made only of
  // modifiers commits when the last one is released, so `Ctrl+Shift` — the
  // toggle default — can be entered like any other chord.
  const MODIFIER_TOKENS: Record<string, string> = {
    Control: "Ctrl",
    Alt: "Alt",
    Shift: "Shift",
    Meta: "Win",
    OS: "Win",
    AltGraph: "Alt",
  };
  const MODIFIER_ORDER = ["Ctrl", "Alt", "Shift", "Win"];
  /** Modifiers physically down right now (live display + when the run ends). */
  let pressed = new Set<string>();
  /** Every modifier pressed during this run; survives earlier releases. */
  let run: string[] = [];
  /** A real key already completed the chord: later modifier keyups must not
   *  overwrite it. Cleared by begin/reset/Backspace. */
  let latched = false;
  const order = (tokens: string[]) =>
    MODIFIER_ORDER.filter((mod) => tokens.includes(mod)).join("+");
  const chordOf = () => order([...pressed]);
  const runChord = () => order(run);

  const shown = $derived(editing ? held || draft : value ?? "");
  const keys = $derived(shown ? shown.split("+") : []);
  const modified = $derived((value ?? "") !== defaultValue);
  const clash = $derived(conflict ?? conflictText);

  const setProblem = () => {
    onproblem?.(
      conflict
        ? { kind: "conflict", text: conflict }
        : error
          ? { kind: "error", text: error }
          : warning
            ? { kind: "warn", text: warning }
            : null,
    );
  };

  const reset = () => {
    editing = false;
    draft = "";
    held = "";
    pressed = new Set();
    run = [];
    latched = false;
    error = null;
    conflict = null;
    warning = null;
    onproblem?.(null);
    seq++;
  };

  const begin = () => {
    editing = true;
    draft = value ?? "";
    held = "";
    pressed = new Set();
    run = [];
    latched = false;
    error = null;
    conflict = null;
    warning = null;
    onproblem?.(null);
    seq++;
  };

  const validate = async (candidate: string) => {
    const mine = ++seq;
    if (!candidate.trim()) {
      draft = "";
      error = null;
      conflict = null;
      warning = null;
      setProblem();
      return;
    }
    const local = canonicalize(candidate);
    if (local.error) {
      error = local.error;
      conflict = null;
      setProblem();
      return;
    }
    let canonical = local.canonical ?? candidate;
    const hazard = hotkeyHazard(canonical);
    if (hazard?.kind === "reserved") {
      error = hazard.text;
      conflict = null;
      warning = null;
      setProblem();
      return;
    }
    warning = hazard?.kind === "common" ? hazard.text : null;
    if (checkConflict) {
      try {
        const clash = await checkConflict(canonical);
        if (mine !== seq) return;
        conflict = clash;
        canonical = local.canonical ?? canonical;
      } catch (failure) {
        if (mine !== seq) return;
        error = failure instanceof Error ? failure.message : String(failure);
        conflict = null;
        warning = null;
        setProblem();
        return;
      }
    }
    draft = canonical;
    error = null;
    setProblem();
  };

  const commit = async () => {
    const clean = draft.trim();
    if (clean && (error || conflict)) return;
    try {
      await oncommit(clean);
      reset();
    } catch (failure) {
      error = failure instanceof Error ? failure.message : String(failure);
      setProblem();
    }
  };

  /** Commit a chord the moment it is complete (modifier-only release). */
  const settle = async (candidate: string) => {
    await validate(candidate);
    if (!draft.trim() || error || conflict) return;
    await commit();
  };

  const onkeydown = (event: KeyboardEvent) => {
    if (!editing) return;
    const key = event.key;
    if (key === "Escape") {
      event.preventDefault();
      reset();
      return;
    }
    if (key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      void commit();
      return;
    }
    if (key === "Backspace" || key === "Delete") {
      event.preventDefault();
      draft = "";
      held = "";
      pressed = new Set();
      run = [];
      latched = false;
      error = null;
      conflict = null;
      warning = null;
      setProblem();
      return;
    }
    if (key === "Tab") {
      reset();
      return;
    }
    const token = MODIFIER_TOKENS[key];
    if (token) {
      if (latched) return;
      event.preventDefault();
      pressed.add(token);
      if (!run.includes(token)) run = [...run, token];
      held = chordOf();
      if (held) draft = held;
      return;
    }
    if (!keyToken(event)) return;
    event.preventDefault();
    // A real key ends the modifier run: later keyups must not commit it.
    latched = true;
    pressed = new Set();
    run = [];
    held = "";
    const combo = comboFromEvent(event);
    if (!combo) return;
    draft = combo;
    void validate(combo);
  };

  const onkeyup = (event: KeyboardEvent) => {
    if (!editing || latched) return;
    const token = MODIFIER_TOKENS[event.key];
    if (!token) return;
    pressed.delete(token);
    held = chordOf();
    if (pressed.size > 0) {
      draft = held;
      return;
    }
    // Last modifier released: the whole run is the chord.
    const candidate = runChord();
    run = [];
    if (!candidate.includes("+")) {
      // A lone modifier is not a shortcut; keep the stored value visible.
      draft = value ?? "";
      held = "";
      return;
    }
    held = "";
    draft = candidate;
    void settle(candidate);
  };
</script>

<div
  class="cap"
  style={`width:${width}px`}
  class:rec={editing}
  class:warn={clash !== null || warning !== null}
  class:empty={!editing && !value}>
  <button
    type="button"
    class="capa"
    id={`cap-${field}`}
    aria-label={t("capture.aria", { label, value: (editing ? shown : value) || t("capture.unset") })}
    onfocus={begin}
    onblur={reset}
    onkeydown={onkeydown}
    onkeyup={onkeyup}>
    {#if editing}
      {#if keys.length === 0}
        <span style="color:var(--t3);font:400 13px/20px var(--ui);padding:0 4px">{t("capture.prompt")}</span>
      {:else}
        {#each keys as key (key)}
          <Keycap text={key} />
          <span class="plus">+</span>
        {/each}
      {/if}
      <span class="caret"></span>
      <span class="end"><Keycap text="Esc" small /> {t("capture.cancelHint")}</span>
    {:else if value}
      {#each keys as key (key)}
        <Keycap text={key} />
      {/each}
      {#if clash}
        <span class="end"><span class="tag warn"><Icon name="warning" small />{t("capture.clash")}</span></span>
      {:else if warning}
        <span class="end"><span class="tag warn"><Icon name="warning" small />{t("capture.mayClash")}</span></span>
      {/if}
    {:else}
      {t("capture.empty")}
      <span class="end">{t("capture.setKey")}</span>
    {/if}
  </button>

  {#if editing}
    <button
      type="button"
      class="btn ghost icon sm"
      aria-label={t("capture.cancelAria", { label })}
      title={t("common.cancel")}
      onmousedown={(event) => event.preventDefault()}
      onclick={reset}>
      <Icon name="close" small />
    </button>
  {:else if modified && onreset}
    <button
      type="button"
      class="btn ghost icon sm"
      aria-label={t("capture.resetAria", { label })}
      title={t("capture.reset")}
      onclick={onreset}>
      <Icon name="reset" small />
    </button>
  {/if}
</div>
