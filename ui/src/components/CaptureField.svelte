<script lang="ts">
  import Icon from "./Icon.svelte";
  import Keycap from "./Keycap.svelte";
  import { canonicalize, comboFromEvent, hotkeyHazard, isModifierKey, keyToken, modifiersFrom } from "../lib/hotkey";
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
  }: {
    field: HotkeyActionField;
    value: string | null;
    label: string;
    defaultValue?: string;
    oncommit: (next: string) => void | Promise<void>;
    onreset?: () => void;
    checkConflict?: (canonical: string) => Promise<string | null>;
    onproblem?: (problem: { kind: "conflict" | "error" | "warn"; text: string } | null) => void;
  } = $props();

  let editing = $state(false);
  let draft = $state("");
  let held = $state("");
  let error = $state<string | null>(null);
  let conflict = $state<string | null>(null);
  let warning = $state<string | null>(null);
  let seq = 0;

  const shown = $derived(editing ? held || draft : value ?? "");
  const keys = $derived(shown ? shown.split("+") : []);
  const modified = $derived((value ?? "") !== defaultValue);

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
    if (isModifierKey(key)) {
      held = modifiersFrom(event);
      if (held) draft = held;
      return;
    }
    if (!keyToken(event)) return;
    event.preventDefault();
    held = "";
    const combo = comboFromEvent(event);
    if (!combo) return;
    draft = combo;
    void validate(combo);
  };

  const onkeyup = (event: KeyboardEvent) => {
    if (!editing || !isModifierKey(event.key)) return;
    const next = modifiersFrom(event);
    held = next;
    if (next) draft = next;
    else draft = value ?? "";
  };
</script>

<div
  class="cap"
  class:rec={editing}
  class:warn={conflict !== null || warning !== null}
  class:empty={!editing && !value}>
  <button
    type="button"
    class="capa"
    id={`cap-${field}`}
    aria-label={`${label} — phím tắt hiện tại ${value ?? "chưa đặt"}`}
    onfocus={begin}
    onblur={reset}
    onkeydown={onkeydown}
    onkeyup={onkeyup}>
    {#if editing}
      {#if keys.length === 0}
        <span style="color:var(--t3);font:400 13px/20px var(--ui);padding:0 4px">Nhấn tổ hợp phím…</span>
      {:else}
        {#each keys as key (key)}
          <Keycap text={key} />
          <span class="plus">+</span>
        {/each}
      {/if}
      <span class="caret"></span>
      <span class="end"><Keycap text="Esc" small /> huỷ</span>
    {:else if value}
      {#each keys as key (key)}
        <Keycap text={key} />
      {/each}
      {#if conflict}
        <span class="end"><span class="tag warn"><Icon name="warning" small />Trùng</span></span>
      {:else if warning}
        <span class="end"><span class="tag warn"><Icon name="warning" small />Dễ trùng</span></span>
      {/if}
    {:else}
      Chưa đặt
      <span class="end">Đặt phím</span>
    {/if}
  </button>

  {#if editing}
    <button
      type="button"
      class="btn ghost icon sm"
      aria-label={`Huỷ ghi ${label}`}
      title="Huỷ"
      onmousedown={(event) => event.preventDefault()}
      onclick={reset}>
      <Icon name="close" small />
    </button>
  {:else if modified && onreset}
    <button
      type="button"
      class="btn ghost icon sm"
      aria-label={`Đặt lại mặc định cho ${label}`}
      title="Đặt lại mặc định"
      onclick={onreset}>
      <Icon name="reset" small />
    </button>
  {/if}
</div>
