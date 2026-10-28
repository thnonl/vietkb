<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import Keycap from "../components/Keycap.svelte";
  import Row from "../components/Row.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Select from "../components/Select.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { app } from "../lib/state.svelte";
  import { METHODS, PICKER_CHARSETS, charsetShort, methodLabel } from "../lib/catalog";
  import { formatNumber, prettyHotkey } from "../lib/hotkey";
  import { renderTelex } from "../lib/preview";
  import type { NavId, SettingsView } from "../lib/bindings";

  let { settings, navigate }: { settings: SettingsView; navigate: (tab: NavId) => void } =
    $props();

  const typing = $derived(settings.typing);
  const set = (patch: Partial<SettingsView["typing"]>) => app.patch({ typing: patch });
  const count = $derived(app.macroCount ?? 0);
  const toggleHotkey = $derived(settings.hotkeys.toggle);
  const toggleTitle = $derived(
    toggleHotkey
      ? `Bật/tắt tiếng Việt (${prettyHotkey(toggleHotkey)})`
      : "Bật/tắt tiếng Việt",
  );

  // Thử gõ: raw keystrokes stay in memory only (C5) and clear when the window
  // closes; the strip renders through the local Telex preview.
  let raw = $state("");
  const previewText = $derived(
    settings.input.method === "telex" ? renderTelex(raw, typing.modernTone) : raw,
  );
  const onTryKey = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key === "Backspace") {
      event.preventDefault();
      raw = Array.from(raw).slice(0, -1).join("");
      return;
    }
    if (event.key.length === 1) {
      event.preventDefault();
      raw += event.key;
    }
  };

  let advancedOpen = $state(
    new URLSearchParams(window.location.search).get("advanced") === "1",
  );
  const suggestionValue = $derived(
    typing.suggestionFix2 ? "fix2" : typing.suggestionFix1 ? "fix1" : "off",
  );
  const setSuggestion = (value: string) => {
    set({ suggestionFix1: value === "fix1", suggestionFix2: value === "fix2" });
  };

  const delay = $derived(typing.sendDelayUs);
  const setDelay = (value: number) => {
    const clamped = Math.min(5000, Math.max(0, Number.isFinite(value) ? value : 0));
    set({ sendDelayUs: clamped });
  };
</script>

<div class="status">
  <span title={toggleTitle}>
    <Toggle
      checked={settings.input.enabled}
      label={toggleTitle}
      onchange={(next) => app.patch({ input: { enabled: next } })} />
  </span>
  <span class="lab">Tiếng Việt</span>
  {#if toggleHotkey}
    <span class="hint">
      {#each toggleHotkey.split("+") as key, index (key)}
        {#if index > 0}<span class="plus">+</span>{/if}
        <Keycap text={key} small />
      {/each}
    </span>
  {/if}
  <span class="f">
    <small>Kiểu gõ</small>
    <Select
      width={92}
      label="Kiểu gõ"
      value={settings.input.method}
      options={METHODS.map((method) => ({ value: method.slug, label: method.label }))}
      onchange={(method) => app.patch({ input: { method } })} />
    <small>Bảng mã</small>
    <Select
      width={164}
      label="Bảng mã"
      value={settings.input.charset}
      options={PICKER_CHARSETS.map((charset) => ({ value: charset.slug, label: charset.label }))}
      onchange={(charset) => app.patch({ input: { charset } })} />
  </span>
</div>

{#if advancedOpen}
  <div class="list" style="margin-top:8px">
    <div class="row s">
      <button type="button" class="link" onclick={() => (advancedOpen = false)}>
        ‹ Tuỳ chọn chính
      </button>
      <span class="tag n" style="margin-left:auto">9 tuỳ chọn</span>
    </div>
    <Row short label="Cho phép f j w z làm phụ âm">
      {#snippet note()}Gõ w ra w, không thành ư; hợp khi hay gõ tên nước ngoài{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.jfwzAsConsonant}
          label="Cho phép f j w z làm phụ âm"
          onchange={(next) => set({ jfwzAsConsonant: next })} />
      {/snippet}
    </Row>
    <Row short label="Gõ tắt cả khi tắt tiếng Việt">
      {#snippet note()}Đã tắt tiếng Việt, gõ nth vẫn ra Nguyễn Thu Hà{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.macroWhenDisabled}
          label="Gõ tắt cả khi tắt tiếng Việt"
          onchange={(next) => set({ macroWhenDisabled: next })} />
      {/snippet}
    </Row>
    <Row short label="Viết hoa theo từ tắt">
      {#snippet note()}dc → được, Dc → Được, DC → ĐƯỢC{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.macroCaseFollow}
          label="Viết hoa theo từ tắt"
          onchange={(next) => set({ macroCaseFollow: next })} />
      {/snippet}
    </Row>
    <Row short label="Tắt khi đổi bàn phím khác">
      {#snippet note()}Tự tắt khi Windows dùng bàn phím không phải loại Mỹ (US){/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.disableOnNonUsLayout}
          label="Tắt khi đổi bàn phím khác"
          onchange={(next) => set({ disableOnNonUsLayout: next })} />
      {/snippet}
    </Row>
    <Row short label="Sửa lỗi dấu gợi ý">
      {#snippet note()}Chữ bị lặp ở ô có gợi ý như Chrome, Excel? Thử cách 1 hoặc 2{/snippet}
      {#snippet control()}
        <Segmented
          value={suggestionValue}
          label="Sửa lỗi dấu gợi ý"
          onchange={setSuggestion}
          options={[
            { value: "off", label: "Tắt" },
            { value: "fix1", label: "Cách 1" },
            { value: "fix2", label: "Cách 2" },
          ]} />
      {/snippet}
    </Row>
    <Row short label="Hỗ trợ ứng dụng Metro">
      {#snippet note()}Gõ được trong ứng dụng tải từ Microsoft Store, như Mail{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.metroFix}
          label="Hỗ trợ ứng dụng Metro"
          onchange={(next) => set({ metroFix: next })} />
      {/snippet}
    </Row>
    <Row short label="Gửi chữ qua Clipboard">
      {#snippet note()}Dùng khi ứng dụng bị mất chữ; chữ đã chép trước đó vẫn còn{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.useClipboard}
          label="Gửi chữ qua Clipboard"
          onchange={(next) => set({ useClipboard: next })} />
      {/snippet}
    </Row>
    <Row short label="Độ trễ giữa các phím">
      {#snippet note()}Tăng lên nếu ứng dụng chạy chậm bị lộn hoặc mất chữ{/snippet}
      {#snippet control()}
        <span class="numfield">
          <input
            type="number"
            min="0"
            max="5000"
            step="500"
            value={delay}
            aria-label="Độ trễ giữa các phím, micro giây"
            onchange={(event) => setDelay(Number((event.currentTarget as HTMLInputElement).value))} />
          <span class="unit">µs</span>
        </span>
      {/snippet}
    </Row>
    <Row short label="Kiểu chuyển bật/tắt">
      {#snippet note()}Nhả phím mới chuyển, hoặc đổi ngay khi nhấn.{/snippet}
      {#snippet control()}
        <Segmented
          value={settings.hotkeys.toggleMode}
          label="Kiểu chuyển bật/tắt"
          onchange={(mode) => app.patch({ hotkeys: { toggleMode: mode } })}
          options={[
            { value: "release", label: "Khi nhả phím" },
            { value: "press", label: "Khi nhấn phím" },
          ]} />
      {/snippet}
    </Row>
  </div>
{:else}
  <div class="list" style="margin-top:8px">
    <Row label="Kiểu đặt dấu">
      {#snippet note()}Hai kiểu chỉ khác nhau ở vần oa, oe, uy{/snippet}
      {#snippet control()}
        <Segmented
          tone
          value={typing.modernTone ? "modern" : "classic"}
          label="Kiểu đặt dấu"
          onchange={(value) => set({ modernTone: value === "modern" })}
          options={[
            { value: "modern", label: "Kiểu mới", example: "hoà, thuý" },
            { value: "classic", label: "Kiểu cũ", example: "hòa, thúy" },
          ]} />
      {/snippet}
    </Row>

    <Row label="Gõ dấu ở cuối từ">
      {#snippet note()}<span class="kbd">hoangf</span> → hoàng: gõ dấu sau cả từ cũng được{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.freeTone}
          label="Gõ dấu ở cuối từ"
          onchange={(next) => set({ freeTone: next })} />
      {/snippet}
    </Row>

    <Row label="Tự sửa ưo thành ươ">
      {#snippet note()}<span class="kbd">nguwoif</span> → người, chỉ cần một w{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.autoHornUo}
          label="Tự sửa ưo thành ươ"
          onchange={(next) => set({ autoHornUo: next })} />
      {/snippet}
    </Row>

    <Row label="Kiểm tra chính tả" group>
      {#snippet note()}Chỉ thêm dấu khi ra từ tiếng Việt: search vẫn là search{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.spellCheck}
          label="Kiểm tra chính tả"
          onchange={(next) => set({ spellCheck: next })} />
      {/snippet}
    </Row>

    <Row
      label="Tự động khôi phục từ gõ sai"
      disabled={!typing.spellCheck}
      link={typing.spellCheck ? undefined : "Bật"}
      onLink={() => set({ spellCheck: true })}>
      {#snippet note()}
        {typing.spellCheck
          ? "gôgle tự trở về google khi bạn gõ dấu cách"
          : "Cần bật Kiểm tra chính tả"}
      {/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.restoreWrongWord}
          label="Tự động khôi phục từ gõ sai"
          disabled={!typing.spellCheck}
          onchange={(next) => set({ restoreWrongWord: next })} />
      {/snippet}
    </Row>

    <Row label="Giữ nguyên email, địa chỉ web">
      {#snippet note()}ten@congty.vn, abc.com/xyz gõ sao ra vậy, không thêm dấu{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.rawZone}
          label="Giữ nguyên email, địa chỉ web"
          onchange={(next) => set({ rawZone: next })} />
      {/snippet}
    </Row>

    <Row label="Viết hoa đầu câu">
      {#snippet note()}chào. tôi → chào. Tôi, cả sau ! ? và khi xuống dòng{/snippet}
      {#snippet control()}
        <Toggle
          checked={typing.autoCapitalize}
          label="Viết hoa đầu câu"
          onchange={(next) => set({ autoCapitalize: next })} />
      {/snippet}
    </Row>

    <Row label="Gõ tắt" group>
      {#snippet note()}
        {#if count > 0}
          Gõ <span class="kbd">nth</span> rồi dấu cách → Nguyễn Thu Hà
        {:else}
          Gõ <span class="kbd">dc</span> rồi dấu cách → được
        {/if}
      {/snippet}
      {#snippet control()}
        <button
          type="button"
          class="link"
          title="Mở tab Gõ tắt"
          onclick={() => navigate("macros")}>
          {formatNumber(count)} mục
          <Icon name="arrow-right" small />
        </button>
        <Toggle
          checked={typing.macroEnabled}
          label="Gõ tắt"
          onchange={(next) => set({ macroEnabled: next })} />
      {/snippet}
    </Row>
  </div>

  <button type="button" class="disc" onclick={() => (advancedOpen = true)}>
    <Icon name="arrow-right" />
    <b>Nâng cao</b>
    <span class="tag n"><span class="num">9</span>&nbsp;tuỳ chọn</span>
    <span class="m">f j w z làm phụ âm, sửa lỗi dấu gợi ý, Metro, Clipboard</span>
  </button>
{/if}

<div class="try">
  <span class="lab">Thử gõ</span>
  <input
    class="out"
    value={previewText}
    placeholder="Gõ thử ở đây, chữ không được lưu lại"
    aria-label="Thử gõ"
    spellcheck="false"
    autocomplete="off"
    onkeydown={onTryKey}
    onpaste={(event) => event.preventDefault()} />
  <span class="m">{methodLabel(settings.input.method)} · {charsetShort(settings.input.charset)}</span>
</div>

<style>
  .numfield {
    display: inline-flex;
    align-items: center;
    width: 88px;
    height: 32px;
    padding: 0 8px 0 10px;
    border-radius: 8px;
    background: var(--raised);
    box-shadow: var(--e1), inset 0 0 0 1px var(--line-2);
  }
  .numfield input {
    width: 100%;
    min-width: 0;
    border: 0;
    background: none;
    outline: none;
    font: 400 13px/20px var(--ui);
    color: var(--t1);
    font-variant-numeric: tabular-nums;
  }
  .unit {
    font: 400 12px/18px var(--sm);
    color: var(--t3);
    margin-left: 4px;
  }
</style>
