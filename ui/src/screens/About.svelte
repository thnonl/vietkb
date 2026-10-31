<script lang="ts">
  import TitleBar from "../components/TitleBar.svelte";
  import { app } from "../lib/state.svelte";
  import * as api from "../lib/api";
  import { platformState } from "../lib/platform.svelte";
  import { t } from "../lib/lang.svelte";
  import { parseChangelog, type Block, type Inline } from "../lib/changelog";
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
        return t("about.status.checking");
      case "none":
        return t("about.status.none");
      case "available":
        return t("about.status.available", { version: status?.update?.version ?? "" });
      case "error":
        return status?.message ?? t("about.status.error");
      default:
        return t("about.status.idle");
    }
  });

  const tone = $derived(
    status?.state === "error" ? "warn" : status?.state === "available" ? "acc" : "dim",
  );

  // With a changelog on screen the story folds into one row to free height;
  // without one the story is the page and has no toggle.
  let storyOpen = $state(false);
  const showStory = $derived(!info || storyOpen);
  const blocks = $derived<Block[]>(info ? parseChangelog(info.changelog) : []);

  const check = () => void app.checkForUpdate();
  const install = () => void app.installUpdate();
  const open = (url: string) => void api.openUrl(url).catch(() => {});
</script>

{#snippet spans(list: Inline[])}
  {#each list as sp}
    {#if sp.kind === "code"}<code>{sp.text}</code>{:else if sp.kind === "bold"}<b>{sp.text}</b>{:else}{sp.text}{/if}
  {/each}
{/snippet}

<div class="v4win">
  <TitleBar title={t("win.about.title")} subtitle="VietKB" showMark={false} showMinimize={false} />

  <main class="about-sheet">
    <div class="about-inner">
      <header class="about-head">
        <img src={markSrc} alt="" width="48" height="48" />
        <b>VietKB</b>
        <span class="about-ver num" title={t("about.versionTitle")}>{version}</span>
      </header>

      <p class="about-lead">{t("about.tagline", { os: platformState.mac ? "macOS" : "Windows" })}</p>

      <section class="about-story" class:folded={!!info}>
        {#if info}
          <h2 class="fold-h">
            <button
              type="button"
              class="about-fold"
              aria-expanded={storyOpen}
              aria-controls="about-story-body"
              onclick={() => (storyOpen = !storyOpen)}
            >
              <span>{t("about.why")}</span>
              <svg class="about-chev" width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
                <path d="M3 4.5 6 7.5 9 4.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </button>
          </h2>
        {:else}
          <h2>{t("about.why")}</h2>
        {/if}
        <div id="about-story-body" hidden={!showStory}>
          <p>{t("about.story1")}</p>
          <p>{t("about.story2")}</p>
          <p>
            {t("about.story3")}
            {#if platformState.mac}
              {t("about.story3.mac")}
            {:else}
              {t("about.story3.win")}
            {/if}
          </p>

          <p class="about-close">{t("about.close")}</p>
        </div>
      </section>

      <div class="about-div"></div>

      <section class="about-update" class:story-open={showStory}>
        <p class="about-status" data-tone={tone} role="status">{statusText}</p>

        <div class="about-actions">
          <button type="button" class="btn" disabled={busy} onclick={check}>
            {app.updateBusy === "checking" ? t("about.status.checking") : t("about.check")}
          </button>
          {#if app.updateAvailable}
            <button type="button" class="btn pri" disabled={busy} onclick={install}>
              {#if app.updateBusy === "downloading"}
                {t("about.downloading")}
              {:else if app.updateBusy === "installing"}
                {t("about.installing")}
              {:else if platformState.mac}
                {t("about.openDownload")}
              {:else}
                {t("about.updateNow")}
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
              <h2>{t("about.changelog", { version: info.version })}</h2>
              {#if info.publishedAt}<span class="num">{info.publishedAt.slice(0, 10)}</span>{/if}
            </div>
            <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
            <div class="about-cl-body" tabindex="0" role="region" aria-label={t("about.changelogAria")}>
              {#each blocks as block}
                {#if block.kind === "heading"}
                  <h3 class="cl-h" data-level={block.level}>{@render spans(block.spans)}</h3>
                {:else if block.kind === "item"}
                  <p class="cl-li">{@render spans(block.spans)}</p>
                {:else}
                  <p class="cl-p">{@render spans(block.spans)}</p>
                {/if}
              {:else}
                <p class="cl-p">{t("about.noNotes")}</p>
              {/each}
            </div>
          </div>
        {/if}
      </section>

      <div class="about-div"></div>

      <footer class="about-foot">
        <div class="about-links">
          <button type="button" class="about-lk" onclick={() => open(REPO)}>{t("about.github")}</button>
          <span class="about-dot">·</span>
          <button type="button" class="about-lk" onclick={() => open(ISSUES)}>{t("about.reportBug")}</button>
          <span class="about-dot">·</span>
          <button type="button" class="about-lk" onclick={() => open(KOFI)}>{t("about.kofi")}</button>
        </div>
        <p class="about-privacy">{t("about.privacy")}</p>
      </footer>
    </div>
  </main>
</div>
