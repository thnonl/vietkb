<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import Keycap from "../components/Keycap.svelte";
  import Modal from "../components/Modal.svelte";
  import { app, errorText } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { closeCurrent } from "../lib/window.svelte";
  import { formatNumber } from "../lib/hotkey";
  import type { MacroRow } from "../lib/bindings";

  const DEMO: MacroRow[] = [
    { trigger: "ctdd", expansion: "Công ty TNHH Đông Dương" },
    { trigger: "dc", expansion: "được" },
    { trigger: "dchi", expansion: "địa chỉ" },
    { trigger: "hcm", expansion: "Hồ Chí Minh" },
    { trigger: "kg", expansion: "không" },
    { trigger: "mail", expansion: "ha.nth@dongduong.com.vn" },
    { trigger: "mst", expansion: "Mã số thuế" },
    { trigger: "nth", expansion: "Nguyễn Thu Hà" },
  ];

  let rows = $state<MacroRow[]>([]);
  let total = $state(0);
  let query = $state("");
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let selected = $state<string[]>([]);
  let busy = $state(false);
  let menuFor = $state<string | null>(null);

  const local = $derived(!api.isTauri());
  const macroOn = $derived(app.settings?.typing.macroEnabled ?? true);

  const refresh = async () => {
    if (local) {
      const needle = query.trim().toLowerCase();
      rows = DEMO.filter(
        (row) =>
          !needle ||
          row.trigger.toLowerCase().includes(needle) ||
          row.expansion.toLowerCase().includes(needle),
      );
      total = DEMO.length;
      loading = false;
      app.macroCount = total;
      return;
    }
    try {
      rows = await api.listMacros(query.trim());
      total = query.trim() ? (await api.listMacros("")).length : rows.length;
      app.macroCount = total;
      loadError = null;
    } catch (failure) {
      loadError = errorText(failure);
    } finally {
      loading = false;
    }
  };

  $effect(() => {
    query;
    const timer = setTimeout(() => void refresh(), 120);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    query;
    selected = [];
    menuFor = null;
  });

  let searchEl = $state<HTMLInputElement | null>(null);
  const onKey = (event: KeyboardEvent) => {
    if (event.ctrlKey && event.key.toLowerCase() === "f") {
      event.preventDefault();
      searchEl?.focus();
      return;
    }
    if (event.key === "Escape") {
      if (menuFor) menuFor = null;
      else if (selected.length > 0) selected = [];
    }
  };

  // ------------------------------------------------------------------ editor
  let editorOpen = $state(false);
  let editTrigger = $state("");
  let editExpansion = $state("");
  let originalTrigger = $state<string | null>(null);
  let formError = $state<string | null>(null);

  const openNew = () => {
    originalTrigger = null;
    editTrigger = "";
    editExpansion = "";
    formError = null;
    editorOpen = true;
  };

  const openEdit = (row: MacroRow) => {
    menuFor = null;
    originalTrigger = row.trigger;
    editTrigger = row.trigger;
    editExpansion = row.expansion;
    formError = null;
    editorOpen = true;
  };

  const saveMacro = async () => {
    const trigger = editTrigger.trim();
    const expansion = editExpansion.trim();
    if (!trigger) return void (formError = "Nhập từ tắt, ví dụ dc");
    if (/\s/.test(trigger)) return void (formError = "Từ tắt viết liền, không có dấu cách");
    if (Array.from(trigger).length > 32)
      return void (formError = `Từ tắt dài tối đa 32 ký tự, đang có ${Array.from(trigger).length}`);
    if (!expansion)
      return void (formError = "Phần thay thế không được để trống — muốn bỏ thì xoá cả mục.");
    if (Array.from(expansion).length > 500)
      return void (formError = `Nội dung dài tối đa 500 ký tự, đang có ${Array.from(expansion).length}`);

    const renaming = Boolean(originalTrigger && originalTrigger !== trigger);
    if (renaming && local) return void (formError = "Chế độ xem trước: không đổi tên được.");
    if (local) {
      const existing = DEMO.findIndex((row) => row.trigger === trigger);
      if (existing >= 0) DEMO[existing] = { trigger, expansion };
      else DEMO.push({ trigger, expansion });
      editorOpen = false;
      void refresh();
      return;
    }
    busy = true;
    try {
      await api.upsertMacro(trigger, expansion);
      if (renaming && originalTrigger) await api.deleteMacros([originalTrigger]);
      editorOpen = false;
      await refresh();
    } catch (failure) {
      formError = errorText(failure);
    } finally {
      busy = false;
    }
  };

  const doDelete = async (targets: MacroRow[]) => {
    if (targets.length === 0) return;
    menuFor = null;
    const triggers = targets.map((row) => row.trigger);
    selected = [];
    if (local) {
      for (const trigger of triggers) {
        const index = DEMO.findIndex((item) => item.trigger === trigger);
        if (index >= 0) DEMO.splice(index, 1);
      }
      void refresh();
      return;
    }
    try {
      await api.deleteMacros(triggers);
      await refresh();
      app.toast("info", `Đã xoá ${formatNumber(targets.length)} mục gõ tắt`, 5000, {
        label: "Hoàn tác",
        run: () => {
          for (const row of targets) void api.upsertMacro(row.trigger, row.expansion);
          setTimeout(() => void refresh(), 200);
        },
      });
    } catch (failure) {
      app.fail("Không xoá được", failure);
    }
  };

  const duplicate = async (row: MacroRow) => {
    menuFor = null;
    let candidate = `${row.trigger}2`;
    let index = 2;
    while (rows.some((item) => item.trigger === candidate) && index < 99) {
      index += 1;
      candidate = `${row.trigger}${index}`;
    }
    if (local) {
      DEMO.push({ trigger: candidate, expansion: row.expansion });
      void refresh();
      return;
    }
    try {
      await api.upsertMacro(candidate, row.expansion);
      await refresh();
    } catch (failure) {
      app.fail("Không nhân bản được", failure);
    }
  };

  const toggleSelect = (trigger: string) => {
    selected = selected.includes(trigger)
      ? selected.filter((item) => item !== trigger)
      : [...selected, trigger];
  };

  const selectedRows = $derived(rows.filter((row) => selected.includes(row.trigger)));

  // ------------------------------------------------------------ file transfer
  const importFile = async () => {
    if (local) return void app.toast("info", "Cần backend để nhập tệp.");
    try {
      const { open: openDialog } = await import("@tauri-apps/plugin-dialog");
      const path = await openDialog({
        multiple: false,
        directory: false,
        title: "Chọn tệp gõ tắt",
        filters: [{ name: "Tệp gõ tắt", extensions: ["txt", "tsv"] }],
      });
      if (typeof path !== "string") return;
      const count = await api.importMacros(path);
      app.toast("success", `Đã nhập ${formatNumber(count)} mục`);
      await refresh();
    } catch (failure) {
      app.fail("Không nhập được tệp", failure);
    }
  };

  const exportFile = async () => {
    if (local) return void app.toast("info", "Cần backend để xuất tệp.");
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        title: "Lưu danh sách gõ tắt",
        defaultPath: "vietkb-go-tat.tsv",
        filters: [{ name: "Tệp gõ tắt", extensions: ["tsv", "txt"] }],
      });
      if (!path) return;
      const count = await api.exportMacros(path);
      const name = path.split(/[\\/]/).pop() ?? path;
      app.toast("success", `Đã xuất ${formatNumber(count)} mục ra ${name}`);
    } catch (failure) {
      app.fail("Không xuất được tệp", failure);
    }
  };

  const addSample = async (sample: MacroRow) => {
    if (local) {
      if (!DEMO.some((row) => row.trigger === sample.trigger)) DEMO.push(sample);
      void refresh();
      return;
    }
    try {
      await api.upsertMacro(sample.trigger, sample.expansion);
      await refresh();
      app.toast("success", "Đã thêm 1 mục");
    } catch (failure) {
      app.fail("Không thêm được mục", failure);
    }
  };

  const seedExamples = async () => {
    const samples: MacroRow[] = [
      { trigger: "dc", expansion: "được" },
      { trigger: "ko", expansion: "không" },
      { trigger: "vn", expansion: "Việt Nam" },
    ];
    const add = async (item: MacroRow) => {
      if (local) {
        if (!DEMO.some((row) => row.trigger === item.trigger)) DEMO.push(item);
        return;
      }
      await api.upsertMacro(item.trigger, item.expansion);
    };
    try {
      for (const item of samples) await add(item);
      if (local) void refresh();
      else await refresh();
    } catch (failure) {
      app.fail("Không thêm được ví dụ", failure);
    }
  };
</script>

<svelte:window onkeydown={onKey} />

<div class="tb">
  <div class="input">
    <Icon name="search" />
    <input
      bind:this={searchEl}
      bind:value={query}
      type="search"
      placeholder="Tìm từ tắt hoặc nội dung"
      aria-label="Tìm từ tắt hoặc nội dung"
      spellcheck="false" />
    <span class="kk"><Keycap text="Ctrl" small /><span class="plus">+</span><Keycap text="F" small /></span>
  </div>
  <button type="button" class="btn" onclick={importFile}>
    <Icon name="download" />
    Nhập từ tệp…
  </button>
  <button type="button" class="btn" onclick={exportFile} disabled={total === 0}>
    <Icon name="upload" />
    Xuất ra tệp…
  </button>
  <button type="button" class="btn pri" onclick={openNew}>
    <Icon name="plus" />
    Thêm
  </button>
</div>

<div class="tbl">
  {#if loading}
    <div class="tr"><span style="grid-column:1/-1;padding-left:16px" class="muted">Đang tải…</span></div>
  {:else if loadError}
    <div class="tr"><span style="grid-column:1/-1;padding-left:16px;color:var(--bad)">{loadError}</span></div>
  {:else if rows.length === 0 && !query.trim()}
    <div class="empty">
      <div class="illustration" aria-hidden="true">
        <Keycap text="dc" mono />
        <span class="plus">+</span>
        <Keycap text="Space" />
        <Icon name="arrow-right" />
        <span class="sample">được</span>
      </div>
      <b>Chưa có gõ tắt nào</b>
      <p>Gõ dc rồi bấm dấu cách sẽ ra “được”. Thêm vài mục mẫu để thử:</p>
      <div class="samples">
        {#each [
          { trigger: "dc", expansion: "được" },
          { trigger: "ko", expansion: "không" },
          { trigger: "vn", expansion: "Việt Nam" },
        ] as sample (sample.trigger)}
          <button type="button" class="sample-row" onclick={() => addSample(sample)}>
            <Keycap text={sample.trigger} mono />
            <span>→</span>
            <span>{sample.expansion}</span>
          </button>
        {/each}
      </div>
      <div class="inline">
        <button type="button" class="btn pri" onclick={seedExamples}>Thêm cả 3 mục</button>
        <button type="button" class="btn" onclick={importFile}>Nhập từ tệp…</button>
      </div>
    </div>
  {:else if rows.length === 0}
    <div class="tr">
      <span style="grid-column:1/-1;padding-left:16px">Không có mục nào khớp “{query}”</span>
    </div>
    <div class="tr">
      <span style="grid-column:1/-1;padding-left:16px">
        <button
          type="button"
          class="link"
          onclick={() => { editTrigger = query.trim(); editExpansion = ""; originalTrigger = null; formError = null; editorOpen = true; }}>
          {Array.from(query.trim()).length > 9 ? "Thêm mục này" : `Thêm “${query.trim()}”`}
        </button>
      </span>
    </div>
  {:else if selected.length > 0}
    <div class="bulk">
      <span class="n">
        <button
          type="button"
          class="cb mix"
          aria-label="Bỏ chọn tất cả"
          onclick={() => (selected = [])}>
          <svg viewBox="0 0 12 12"><path d="M3 6h6" /></svg>
        </button>
        <span>Đã chọn <span class="num">{formatNumber(selected.length)}</span> <small>trong <span class="num">{formatNumber(total)}</span></small></span>
      </span>
      <button
        type="button"
        class="btn ghost sm"
        disabled={selected.length !== 1}
        onclick={() => selectedRows[0] && openEdit(selectedRows[0])}>
        <Icon name="edit" small />
        Sửa
      </button>
      <button type="button" class="btn ghost sm" onclick={exportFile}>
        <Icon name="upload" small />
        Xuất…
      </button>
      <button type="button" class="btn ghost sm danger" onclick={() => doDelete(selectedRows)}>
        <Icon name="trash" small />
        Xoá
      </button>
      <span class="vsep"></span>
      <button type="button" class="btn ghost sm" onclick={() => (selected = [])}>
        Bỏ chọn <Keycap text="Esc" small />
      </button>
    </div>
  {:else if !macroOn}
    <div class="bulk">
      <span class="n" style="font-weight:400">
        Gõ tắt đang tắt, các mục chưa có tác dụng ·
        <button type="button" class="lnk" onclick={() => app.patch({ typing: { macroEnabled: true } })}>Bật</button>
      </span>
    </div>
  {:else}
    <div class="tr head">
      <span class="c0"></span>
      <span class="c1">Từ tắt</span>
      <span>Nội dung</span>
      <span></span>
    </div>
  {/if}

  {#if !loading && !loadError && rows.length > 0}
    <div class="tblbody">
      {#each rows as row (row.trigger)}
        <div class="tr" class:on={selected.includes(row.trigger)}>
          <span class="c0">
            <button
              type="button"
              class="cb"
              class:on={selected.includes(row.trigger)}
              aria-label={`Chọn ${row.trigger}`}
              aria-pressed={selected.includes(row.trigger)}
              onclick={() => toggleSelect(row.trigger)}>
              {#if selected.includes(row.trigger)}
                <svg viewBox="0 0 12 12"><path d="M2.5 6.25 5 8.75l4.5-4.5" /></svg>
              {/if}
            </button>
          </span>
          <span class="c1"><Keycap text={row.trigger} mono /></span>
          <span class="ex">
            <em>{row.expansion}</em>
            {#if row.expansion.includes("\n")}<span class="tag n">+1 dòng</span>{/if}
          </span>
          <span class="acts">
            <button
              type="button"
              class="btn ghost icon sm"
              aria-label={`Sửa ${row.trigger}`}
              title="Sửa"
              onclick={() => openEdit(row)}>
              <Icon name="edit" />
            </button>
            <button
              type="button"
              class="btn ghost icon sm"
              aria-label={`Tuỳ chọn khác cho ${row.trigger}`}
              title="Tuỳ chọn khác"
              onclick={() => (menuFor = menuFor === row.trigger ? null : row.trigger)}>
              <Icon name="more" />
            </button>
          </span>
          {#if menuFor === row.trigger}
            <div class="rowmenu" role="menu">
              <button type="button" role="menuitem" onclick={() => duplicate(row)}>Nhân bản</button>
              <button type="button" role="menuitem" class="danger" onclick={() => doDelete([row])}>Xoá</button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<div class="mfoot">
  <span>
    {#if total === 1 && !query.trim()}
      Có 1 mục
    {:else if query.trim()}
      <span class="num">{formatNumber(rows.length)}</span> kết quả trong
      <span class="num">{formatNumber(total)}</span> mục · cuộn để xem hết
    {:else}
      <span class="num">{formatNumber(total)}</span> mục · cuộn để xem hết
    {/if}
  </span>
  <button
    type="button"
    class="btn ghost sm"
    title="Đóng, VietKB vẫn chạy"
    onclick={closeCurrent}>
    Đóng
  </button>
</div>

<Modal
  bind:open={editorOpen}
  title={originalTrigger ? "Sửa từ tắt" : "Thêm từ tắt"}
  onclose={() => (formError = null)}>
  <div class="form">
    <label class="field">
      <span>Từ tắt</span>
      <input
        class="input"
        bind:value={editTrigger}
        placeholder="ví dụ: dc"
        maxlength="32"
        spellcheck="false" />
      <span class="hint">Từ tắt viết liền, không có dấu cách.</span>
    </label>
    <label class="field">
      <span>Nội dung</span>
      <textarea
        class="input"
        bind:value={editExpansion}
        placeholder="ví dụ: được · Enter để xuống dòng"
        spellcheck="false"></textarea>
      <span class="hint">{editExpansion.length} ký tự</span>
    </label>
    {#if formError}
      <p class="form-error" role="alert">{formError}</p>
    {/if}
  </div>
  {#snippet footer()}
    <button type="button" class="btn ghost" onclick={() => (editorOpen = false)}>Huỷ</button>
    <button type="button" class="btn pri" onclick={saveMacro} disabled={busy}>
      {busy ? "Đang lưu…" : "Lưu"}
    </button>
  {/snippet}
</Modal>

<style>
  .tr.head {
    height: 36px;
    font: 600 12px/18px var(--sm);
    color: var(--t3);
    box-shadow: inset 0 -1px 0 var(--line-2);
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    gap: 6px;
    /* Fills the table box the window is sized for — a fixed minimum left a
     * band under the illustration once the window matched its content. */
    flex: 1 1 auto;
    padding: 18px 24px;
  }
  .illustration {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
  }
  .illustration :global(.i) {
    color: var(--t4);
    margin: 0 4px;
  }
  .illustration .sample {
    font: 400 22px/30px var(--serif);
    color: var(--t1);
  }
  .empty b {
    font: 600 16px/24px var(--ui);
  }
  .empty p {
    max-width: 420px;
    font: 400 13px/20px var(--ui);
    color: var(--t2);
  }
  .samples {
    display: flex;
    gap: 8px;
    margin: 4px 0 6px;
  }
  .sample-row {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 34px;
    padding: 0 10px;
    border-radius: 8px;
    box-shadow: inset 0 0 0 1px var(--line-2);
    font: 400 13px/20px var(--ui);
    color: var(--t2);
  }
  .sample-row:hover {
    background: var(--hover);
    color: var(--t1);
  }
  .empty .inline {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }
  .tr {
    position: relative;
  }
  /* The table keeps its header pinned and scrolls the body, so all 12 rows
   * are reachable at 720 × 560 with the scrollbar always visible. */
  .tbl {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    min-height: 0;
    /* 9 px instead of the kit's 10: the sheet's 16 px bottom pad plus this
     * margin is what makes the window height land exactly on the 4 px grid
     * (see `macros` in crates/vkb-app/src/main.rs). */
    margin-bottom: 9px;
  }
  .tblbody {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-gutter: stable;
    overscroll-behavior: contain;
  }
  .tblbody::-webkit-scrollbar {
    width: 10px;
  }
  .tblbody::-webkit-scrollbar-track {
    background: transparent;
  }
  .tblbody::-webkit-scrollbar-thumb {
    background: var(--line-3);
    border: 2px solid transparent;
    background-clip: padding-box;
    border-radius: 6px;
  }
  .tblbody::-webkit-scrollbar-thumb:hover {
    background: var(--t3);
    background-clip: padding-box;
  }
  .rowmenu {
    position: absolute;
    right: 8px;
    top: 38px;
    z-index: 20;
    width: 148px;
    padding: 4px;
    border-radius: 10px;
    background: var(--float);
    box-shadow: var(--e2);
    display: flex;
    flex-direction: column;
  }
  .rowmenu button {
    height: 30px;
    padding: 0 10px;
    border-radius: 7px;
    text-align: left;
    font: 500 13px/20px var(--ui);
    color: var(--t1);
  }
  .rowmenu button:hover {
    background: var(--hover);
  }
  .rowmenu button.danger {
    color: var(--bad);
  }
  .lnk {
    padding: 0;
    color: var(--acc-text);
    font: 500 12px/18px var(--sm);
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 8px 0 4px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font: 500 13px/20px var(--ui);
    color: var(--t2);
  }
  .field .input {
    padding: 0 10px;
    color: var(--t1);
    font: 400 13px/20px var(--ui);
  }
  textarea.input {
    display: block;
    height: 72px;
    padding: 8px 10px;
    resize: vertical;
    color: var(--t1);
  }
  .hint {
    font: 400 12px/18px var(--sm);
    color: var(--t3);
  }
  .form-error {
    font: 400 12px/18px var(--sm);
    color: var(--bad);
  }
  .muted {
    color: var(--t3);
    font: 400 13px/20px var(--ui);
  }
</style>
