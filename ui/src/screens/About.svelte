<script lang="ts">
  import TitleBar from "../components/TitleBar.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  // The keycap mark at 48 px: the app icon itself, so About carries the same
  // mark as the title bar and the installer.
  import markSrc from "../assets/keycap-v-128.png";

  const REPO = "https://github.com/thnonl/vietkb";
  const ISSUES = "https://github.com/thnonl/vietkb/issues/new";
  // Ko-fi as a plain link, not their script widget: the widget needs
  // `script-src`/`frame-src` opened to a third party inside a webview that also
  // holds the Tauri IPC bridge, which is a much bigger hole than it is worth.
  const KOFI = "https://ko-fi.com/N4X728S6TT";

  const status = $derived(app.update);
  const info = $derived(status?.update ?? null);
  const busy = $derived(app.updateBusy !== "idle");
  const version = $derived(app.info?.version ?? status?.currentVersion ?? "0.2.0");

  const statusText = $derived.by(() => {
    switch (status?.state ?? "idle") {
      case "checking":
        return "Đang kiểm tra…";
      case "none":
        return "Bạn đang dùng bản mới nhất";
      case "available":
        return `Có bản cập nhật v${status?.update?.version ?? ""}`;
      case "error":
        return status?.message ?? "Không kiểm tra được bản cập nhật.";
      default:
        return "Chưa kiểm tra";
    }
  });

  const tone = $derived(
    status?.state === "error" ? "warn" : status?.state === "available" ? "acc" : "dim",
  );

  const check = () => void app.checkForUpdate();
  const install = () => void app.installUpdate();
  const open = (url: string) => void api.openUrl(url).catch(() => {});
</script>

<div class="v4win">
  <TitleBar title="Giới thiệu" subtitle="VietKB" showMark={false} showMinimize={false} />

  <main class="about-sheet">
    <div class="about-inner">
      <header class="about-head">
        <img src={markSrc} alt="" width="48" height="48" />
        <b>VietKB</b>
        <span class="about-ver num" title="Bảng mã cũ (TCVN3, VNI Windows, VISCII) và VIQR không có trong VietKB.">{version}</span>
      </header>

      <p class="about-lead">Bộ gõ tiếng Việt cho Windows. Nhẹ, nhanh, và không làm phiền bạn.</p>

      <section class="about-story">
        <h2>Vì sao có VietKB</h2>
        <p>
          Những bộ gõ đang có vẫn gõ đúng, nhưng mang theo cả một thời phần mềm cũ: bảng mã 8-bit,
          hàng chục tuỳ chọn, nhiều tab.
        </p>
        <p>
          VietKB có mặt vì phần dùng hằng ngày thì ít, phần phải mang theo thì nhiều. Ở đây chỉ có
          Telex, VNI, gõ tắt và loại trừ ứng dụng, không có mục nào để bạn phải đoán.
        </p>
        <p>
          Miễn phí, mã nguồn mở, không quảng cáo, không ghi lại nội dung gõ. Cả bộ gõ nằm trong
          khay, tốn 14 MB; bộ cài khoảng 3 MB; mở cửa sổ mất khoảng 0,3 giây.
        </p>

        <p class="about-close">Một bộ gõ tốt là bộ gõ bạn không phải nghĩ tới.</p>
      </section>

      <div class="about-div"></div>

      <section class="about-update">
        <p class="about-status" data-tone={tone} role="status">{statusText}</p>

        <div class="about-actions">
          <button type="button" class="btn" disabled={busy} onclick={check}>
            {app.updateBusy === "checking" ? "Đang kiểm tra…" : "Kiểm tra cập nhật"}
          </button>
          {#if app.updateAvailable}
            <button type="button" class="btn pri" disabled={busy} onclick={install}>
              {#if app.updateBusy === "downloading"}
                Đang tải…
              {:else if app.updateBusy === "installing"}
                Đang cập nhật…
              {:else}
                Cập nhật ngay
              {/if}
            </button>
          {/if}
        </div>

        {#if app.updateNote}
          <p class="about-progress">{app.updateNote}</p>
        {/if}

        {#if info}
          <div class="about-changelog">
            <div class="about-sec-head">
              <h2>Nhật ký thay đổi — v{info.version}</h2>
              {#if info.publishedAt}<span class="num">{info.publishedAt.slice(0, 10)}</span>{/if}
            </div>
            <pre>{info.changelog.trim() || "Bản phát hành không kèm mô tả."}</pre>
          </div>
        {/if}
      </section>

      <div class="about-div"></div>

      <footer class="about-foot">
        <div class="about-links">
          <button type="button" class="about-lk" onclick={() => open(REPO)}>GitHub</button>
          <span class="about-dot">·</span>
          <button type="button" class="about-lk" onclick={() => open(ISSUES)}>Báo lỗi</button>
          <span class="about-dot">·</span>
          <button type="button" class="about-lk" onclick={() => open(KOFI)}>Ko-fi</button>
        </div>
        <p class="about-privacy">Không quảng cáo. Không thu thập nội dung bạn gõ.</p>
      </footer>
    </div>
  </main>
</div>
