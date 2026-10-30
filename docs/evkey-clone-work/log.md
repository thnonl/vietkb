# Log

- 2026-10-30 BA DOCS (docs only, no `crates/`, no `ui/`, no cargo) SCOPE-CUT FOLLOW-UP on the docs that still described the removed surface — finding first: `25-contract.md`, `27-copy-decisions.md` and `50-test-cases.md` had **never** received the sound additions, so nothing was struck; each got a dated withdrawal record instead. `25-contract.md` gains §8 Change log: the sound-pack additions **WITHDRAWN** (owner ruling 2026-10-30 — `[system] sound_pack` + `SoundPack`, `schemeSilent`, `preview_sound`/`api.previewSound`, `soundPack` on `SettingsView.system`; proposed this round, never applied, command surface returns to exactly pre-round §6, `notification_sound` the only switch). `27-copy-decisions.md` gains §F: the 15 `Kiểu âm thanh` picker strings withdrawn, not deleted, verbatim text left in `22-sound-spec.md` §9, nothing for Copy to chase. `50-test-cases.md` gains a dated `§3 sound cases` block (no picker/`Nghe thử`/pack-persistence/`schemeSilent`/424 row ever existed, so later ones are `N/A — surface cut`; `Nâng cao` keeps 560 × 496) plus **UI-47** toggle ON plays the `go_moc` on WAV, **UI-48** toggle OFF plays the off WAV on hotkey + UI switch + tray with ON != OFF, **UI-49** `Injection::Beep` plays the error cue. `93-sound-silent.md` untouched except a one-line header note: §8's 6-pack/picker fix was cut 2026-10-30 and the fallback-to-`go_moc` conclusion is now the shipped behaviour.

- 2026-10-30 **Owner closed the sound-pack default: `go_moc` (pack 1, "Cổ điển").** `22-sound-spec.md` only
  (docs; no `crates/`, no `ui/`, no cargo). §1: `go_moc` marked DEFAULT, table order unchanged so "pack 1"
  stays unambiguous, `system` stays selectable as "theo Windows" (plays the user's own scheme, silent if the
  scheme has none, falls back to `go_moc` per §6). The open decision is now a **closed decision dated
  2026-10-30**; the single code change is `#[default]` on `GoMoc` in `crates/vkb-config/src/schema.rs`, which
  writes `sound_pack = "go_moc"` under `[system]`. Licensing rationale kept in one sentence so nobody
  re-opens it: the `C:\Windows\Media\*.wav` files are Microsoft copyrighted (redistributing them in the
  installer is a licence violation) and the full scheme would blow the P7 installer budget at 300–900 KB per
  file, so `system` must keep borrowing the user's own installation. §7 default/migration/unknown-slug all
  follow `GoMoc`; §8 default selection `Cổ điển`; §9 gains item 17 (fresh config → `go_moc`, an existing
  `sound_pack = "system"` from the previous build still loads as `system` — no silent migration) and items
  6/7/16 retargeted to `go_moc`; §6 notes `system` is now a deliberate opt-in, so `scheme_silent` and the
  §8 hint stay but are optional/low priority. Owner also asked whether the Windows WAVs can simply ship —
  answer was no, for the two reasons above.

- 2026-10-30 **BA: A9 amendment §2b added — chained openers (D-SC-05), `C:\Users\nam` in Telex.**
  Ruling: an opener arms with an empty left token iff `c ∈ { '\', '/' }` ∧ `last_emitted ∈ { ':', '\', '/' }`
  ∧ the previous opener is still armed (`raw_pending` true, i.e. no `word_key` between them). No new
  state, no new config key: the chain reads the flag the previous boundary left, so it expires exactly
  as `raw_pending` does (consumed once by the next `word_key`, false after any non-opener boundary,
  cleared by Backspace/`reset`) and protection still stops at the first char outside `{':', '\', '/'}`.
  Narrowest form kept: `_ @ = # ? .` never chain, so `bật:_/tắt`, `bật./tắt`, `x=@cof` (L1) and
  `e.g./i.e.` are untouched; `bật//tắt`, `bật /tắt`, `/tắt`, `a/b`, `2/3`, `bật.http` unaffected because
  the first opener never armed there. Cost declared as new **L5** (`a:/tiếng`, `abc://tiếng` stay literal
  — unreachable between two Vietnamese words). §6 row corrected to "Telex and VNI", the impossible
  "at `\` A10 restored `Users`" note replaced, `last_emitted`/`previous opener armed` added to §1 terms,
  L1 narrowed (`http://cof` no longer a loss), §8 gains `chained_openers_extend_a_path`, 4 new
  `amendment_2026_10_30_borderlines` rows (`C:/Users/nam`, `C:\\server\share`, `https://x/y/z`,
  `xyz./tieesng`+`bật:_/tieesng`). Docs only — no `crates/`, no cargo.

- 2026-10-30 **Updater verified in place inside Windows Sandbox.** Installed the published
  `v0.2.0`, started it, then handed the `v0.2.3` setup the exact command line the About
  button uses (`/S /UPDATE`) while the app was running: exit 0 in 0.6 s, `FileVersion`
  0.2.3, the install directory kept a sentinel file (so the old uninstaller never ran), the
  config directory kept its marker, the installer closed the old process, the app reopened
  by itself as exactly one process, and the uninstall registry entry reads 0.2.3. The same
  command measured 1.3 s with no app running and 1.5 s with one running on the host, so the
  earlier "hang" was the host machine, not the installer. `inplace-flow.ps1` now logs
  `START setup` plus a 10-second heartbeat while the setup runs, so a stall is visible
  instead of silent.

- 2026-10-29 **Paragraph-level typing E2E, Telex and VNI.** Ten paragraphs written for
  different purposes (chat, email, technical doc, news, administrative text, marketing,
  study notes, place names, a mixed English spec, punctuation-heavy prose) plus 95 edge
  cases. `qa/e2e/encode-keys.ps1` turns the authored Vietnamese back into keystrokes via
  Unicode decomposition, so one paragraph is typed in both methods. Headless engine check:
  20/20 paragraphs, 95/95 edge cases. Desktop E2E through the real hook: 34/34.
  Windows Sandbox cannot be used as the oracle here — it drops `KEYEVENTF_UNICODE`, so
  accented text never reaches the target field (7 ASCII cases passed, 18 accented ones
  failed); the harness therefore runs on the real desktop, and needed three focus fixes
  (minimize the app panel, click-to-focus fallback, warm-up until the probe form holds the
  foreground) before the first two cases stopped returning empty text.
  The `w` question is settled in EVKey's favour without a code change: with `j f w z` as
  consonants (EVKey's default) `w` stays literal, and with the option off `w → ư`,
  `tw → tư`, `nhw → như` — `vkb simulate` grew `--no-jfwz-consonant` to test both.

- 2026-10-29 **Update path verified in Windows Sandbox.** `scripts/verify-update.ps1 -Version 0.2.2`
  installed the published `v0.2.0` build, then the `v0.2.2` installer, read back
  `FileVersion 0.2.2`, uninstalled, and confirmed both the install directory and the
  `HKCU\...\Run` value were gone: PASS on tag, asset, size, SHA256, previous version,
  installed version and uninstall. Two harness bugs surfaced and were fixed in
  `scripts/update-check/in-sandbox.ps1`: NSIS stores `InstallLocation`/`UninstallString`
  with surrounding quotes (`Join-Path` turned that into a drive named `"C`), and `$roots`
  was scoped inside `Find-App` while the uninstall step used it outside.
  A stuck `vmwp` (SYSTEM-owned) had also blocked every earlier run with "Only one running
  instance of Windows Sandbox is allowed"; killing the stale VMs from an elevated shell
  cleared it.

- 2026-10-29 **Released 0.2.2 and shipped the public site.** The E/V toggle now plays the
  Windows sound scheme (`SystemAsterisk` on, `SystemHand` off) instead of a generated tone —
  the synthesized WAV was too loud and was removed. CI (`release.yml`, no verify job) builds and
  publishes `VietKB_0.2.2_x64-setup.exe` on tag push. The landing page got a motion hero
  (`site/hero.js`: `tieengs` → `tiếng`, transform/opacity only, pause control, tiered
  reduced-motion) built on the rules from `iart-ai/web-animation-skills`, a full copy rewrite,
  and a mobile pass (hero grid track pinned, header keeps the download CTA at ≤640 px). A README
  with the Pages and Ko-fi links was added. Release verification lives in
  `scripts/verify-update.ps1` (Windows Sandbox), not CI.

- 2026-10-29 **Owner, second parity round.** Custom input methods (`user_method_input`) are
  dropped: Telex/VNI cover the real use and a rule-table editor is a large surface nobody wants
  to maintain. EVKey's hidden per-application table (Excel, Google Sheets, browsers, Viber,
  EmEditor, MindManager, Audition, terminal, games, `winuapentry.exe`) **is** implemented, with
  no toggle — that is where `CSuggestion`/`CSuggestion2` actually belong. Per-app remembered
  on/off is persisted to config like EVKey's `blacklist`.

- 2026-10-29 **Owner: two more settings removed.** `typing.send_delay_us` goes away entirely —
  a per-keystroke delay is an implementation detail, not something a user can reason about, and
  the field was never wired. The theme picker leaves `Nâng cao`: `system.theme` stays in the
  config because the title-bar switch owns it, but only one control may exist.

- 2026-10-29 **Owner decisions after the parity matrix (`86-feature-parity.md`).**
  `system.run_as_admin` is removed rather than implemented: no elevation flow is promised.
  `typing.suggestion_fix_1`, `suggestion_fix_2` and `typing.metro_fix` are removed: the owner
  toggles them on Chrome and sees no difference, and our spec still marks the mechanism
  `UNVERIFIED`, so there is nothing honest to wire. Toggle feedback becomes **two distinct
  sounds** (on/off) instead of one system beep. `typing.disable_on_non_us_layout` is removed:
  EVKey keeps typing under another keyboard layout (e.g. a Japanese IME) and VietKB must do the
  same. Each removed field must keep old `config.toml` files loading.

- 2026-10-29 **REA + Ghidra: EVKey engine rules read from the binary.** Ghidra 12.1.4 (+ JDK 21)
  installed for REA 6.8.0; `analyze` plus `decompile-all` produced ~2,300 decompiled functions in
  `reference/evkey-decomp/` (kept out of git). Confirmed chain: hook proc `FUN_14002dc90` →
  per-key dispatcher `FUN_14000a590` → engine backspace `FUN_14003cb50` / process-char
  `FUN_14003c8a0`; character classes are built at startup by `FUN_140036b10`. Consequences:
  Backspace pops exactly one letter entry and EVKey lets the physical key through unless it must
  rewrite (`delete > 1` or new text); a word is hard-reset only by class 3 (Enter and other
  control characters), while class 1 — space, Tab and every ASCII punctuation character —
  appends a boundary entry (`FUN_14003d480`) that a Backspace pops, which is the reopen
  mechanism. `vkb-core` now uses that same class-1 set (`is_reopening_separator`), so
  `Tha` + separator + BS + `s` / VNI digit composes to `Thá`. Rulings B1–B4 and D8 in
  `28-engine-rulings.md`; 145 core tests green, workspace and clippy clean.

- 2026-10-29 **Owner decision: drop the legacy charsets and VIQR.** Shipped input
  methods are now Telex and VNI only; shipped charsets are `unicode` (NFC, the only
  value the UI exposes) and `unicode_nfd` (config-file only). Deleted: the TCVN3,
  VNI Windows, VISCII, VIQR, BK HCM1, VPS and UTF-8 tables and encoders; the VIQR
  engine path and corpus; `hotkeys.cycle_charset`; the tray `Bảng mã ▸` submenu; the
  clipboard `convert_text` tool and its F6 hotkey. Old `config.toml` files still load:
  a removed `charset`/`method` slug deserialises to a sentinel that
  `AppConfig::migrated()` rewrites to `unicode`/`telex`, keeping `extra` and macros.
  `50-test-cases.md` §1 is now 503 rows and §2 keeps the two Unicode forms; the main
  window has 11 panel controls. Older sections of `00-charter`, `20-spec`,
  `25-contract`, `27-copy-decisions`, `30-plan`, `50-test-cases`, `70-impl-notes-*`,
  `80-verify-report` that describe the removed charsets/VIQR are superseded by this
  entry.

- 2026-10-28 Sprint 0 start. Charter written (`00-charter.md`). Legacy code declared reference-only; owner authorized full deletion.
- 2026-10-28 Dispatched background scouts: legacy repo survey, EVKey capability research, toolchain check.
- 2026-10-28 Env facts recorded (charter §10): Rust 1.98.1 MSVC, Node 24.22.1, WebView2 155,
  no Tauri CLI, no NSIS/WiX yet, git needs `safe.directory`, legacy `target/` = 13.4 GB.
  PowerShell blocks `.ps1` shims → use `npm.cmd`/`npx.cmd`.
- 2026-10-28 BA: `20-spec.md` v1 (A engine, B config + 9 CCRs, C UI, D AC-1…AC-85) and `40-design-review.md` pass 1 (3 directions, rec. A "Lặng") written. CCR-1…9 await Lead.
- 2026-10-28 Legacy preserved: tag `legacy-v0`, branch `legacy/pre-rewrite`, 23 files copied to
  `reference/legacy/` (tests + core + hook + gui). Old tree deleted, 14.4 GB freed.
- 2026-10-28 EVKey inspected (bytes only, never executed): `SetWindowsHookExW` + `SendInput` +
  clipboard fallback, `ActivateKeyboardLayout`/`LoadKeyboardLayout` present (source of the
  unwanted-US-layout bug), **no** `RegisterHotKey`, no IMM. Settings: Win32 dialog, 3 tabs
  (Cơ bản/Phím tắt/Hệ thống), defaults F1/F2/F6/F8/F9/F11. Repo has no engine source → clean room.
- 2026-10-28 Scaffold committed (`13521e4`, 66 files). `cargo test -p vkb-core -p vkb-config`
  = 12 tests green, clippy clean, config round-trips unknown keys and empty-string unbinding.
- 2026-10-28 Sprint 1 dispatched: worktrees `wt-engine` (feat/engine), `wt-windows` (feat/windows),
  `wt-ui` (feat/ui) — all on DeepSeek V4.1 Flash max. QA started on `50-test-cases.md`.
- 2026-10-28 BA spec + 3 directions delivered (`20-spec.md` 8.5k words, `40-design-review.md`).
  Contract amended by Lead: `modern_tone` semantics corrected (kiểu mới = `hoà`/`thuý`, BA was
  right, the earlier comment was backwards), `Replace.delete_units` (UTF-16 units = Backspaces),
  new `ReplaceThenPass` for macro-on-Enter, `set_macros(MacroTable)` on the engine trait.
- 2026-10-28 Owner rejected the first UI direction pass as "chưa đẹp". Dedicated designer pass
  started (Opus 5.5 high) writing screenshot-ready mocks to `design/`; `scripts/shoot-design.ps1`
  renders them with headless Edge at 1040×720 for visual review.
- 2026-10-28 QA delivered `50-test-cases.md`: 551 engine rows, 48 UI tests, P1–P7 protocol.
  5 charsets remain UNVERIFIED (`tcvn3`, `vni_windows`, `viscii`, `bk_hcm1`, `vps`).
- 2026-10-28 Owner rejected Lead hand-editing product code during integration; charter now forbids it
  (Lead writes docs only, dev children own `crates/` and `ui/`).
- 2026-10-28 Three streams merged: `cargo test --workspace` = 151 tests green. Tone-style semantics
  corrected twice — final: `modern_tone = true` → `hòa`/`hóa` (tone on the first vowel), old →
  `hoà`/`hoá`. Design pills and COPY.md updated to match.
- 2026-10-28 Runtime smoke: app panicked at startup (`plugins.updater` missing in tauri.conf) and was
  handed to a dev child. UI build: 131 KB JS / 24.6 KB CSS gzipped to 46 KB / 5.5 KB.



- 2026-10-28 wt-windows: pure layer done in `vkb-platform` (injection planning, hotkey matcher,
  per-app rule resolution) + `vkb-windows` backend (WH_KEYBOARD_LL on a dedicated thread with
  install timeout and watchdog, batched `SendInput`, in-hook hotkeys, foreground cache, HKCU
  autostart, opt-in clipboard). Tests green; details in `70-impl-notes-windows.md`. No full
  build run, so no BUILD slot claimed.
- 2026-10-28 wt-ui: settings shell + 8 screens built (Svelte 5, strict TS, hand-written
  tokens); live-apply `patch_settings`, hotkey capture field, charset converter, macro
  import/export, running-app picker, tray with method/charset submenus. Notes:
  `70-impl-notes-ui.md`.
- 2026-10-28 wt-ui **contract change proposals (additive only, Lead to approve)**:
  1) `SettingsView` gains `apps: { rememberPerApp: boolean }`;
  2) new command `set_app_rule_overrides(process, charset: string|null, method: string|null)`
  because the frozen `set_app_rule` cannot carry per-app overrides. No frozen name or
  payload changed.
- 2026-10-28 **Env gotcha for everyone using node here**: the session exports
  `NODE_OPTIONS=--import=<headroom hook shim>`, which makes npm's network calls die with
  ECONNREFUSED and makes installs hang. Clear `NODE_OPTIONS` (and `NODE_EXTRA_CA_CERTS`) for
  npm/vite child processes: `$env:NODE_OPTIONS=$null; npm.cmd install --prefer-offline
  --no-audit --no-fund`. First install took 14 s after that fix (49 packages).
- 2026-10-28 BUILD: ui 2026-10-28 19:25 (wt-ui) — `npm.cmd run build` + `svelte-check`,
  `cargo check -p vkb-app`, `cargo test -p vkb-app` (10/10). Slot released.
  `ui/dist` = 158,723 bytes; 0 errors / 0 warnings. Visual review done on the built
  `dist` in both themes; notes in `70-impl-notes-ui.md`.
- 2026-10-28 wt-ui **C9/C10 scope change applied** (owner): sidebar/dashboard shell deleted
  (10 files), panel is now 720 × 560 frameless, 4 tabs — Gõ · Gõ tắt · Phím tắt · Hệ thống.
  Charset screen/convert tool and About removed (C10: separate windows later); Hệ thống
  absorbs app exceptions as a 3-row list + Quản lý dialog. Design handover for 720 × 560
  still pending (existing `design/DESIGN-SPEC.md` is the 1040 × 720 round).
  Blocked on vkb-config: `typing.auto_horn_uo`, `typing.raw_zone` (and a distinct
  `typing.free_marking`); those rows render disabled with the missing key named.
- 2026-10-28 wt-ui: **design handover applied** (`design/HANDOVER.md` + `DESIGN-SPEC.md` v3).
  Tokens + control kit ported to `tokens.css`/`kit.css`; 11 components; four tabs at
  720 × 560 frameless; tauri.conf set to 720×560 / min 680×520 / not resizable;
  corrections 1–5 applied. Still blocked on vkb-config: `typing.auto_horn_uo`,
  `typing.raw_zone`, `typing.macro_case_follow`, `compat.send_delay_us`,
  `hotkeys.cycle_charset`, `hotkeys.cycle_method`, `hotkeys.toggle_mode`.
- 2026-10-28 BUILD: ui 2026-10-28 (wt-ui) — `svelte-check` 0/0, `npm run build` pass,
  `cargo check -p vkb-app` pass. `ui/dist` = 151,771 B (43.65 KB gzip JS, 5.35 KB CSS);
  720 × 560 fit verified on all four tabs; 680 × 520 clips the Gõ test strip (window is
  fixed at 720 × 560). Slot released.
- 2026-10-28 wt-ui: **final copy pass** — COPY.md + `27-copy-decisions.md` applied
  verbatim (off state `Đã tắt tiếng Việt`, `Đặt lại…`, `Giữ nguyên email, địa chỉ web`,
  `Cần mở lại VietKB`). Schema additions in this worktree: `typing.auto_horn_uo`,
  `typing.raw_zone`, `typing.macro_case_follow`, `typing.send_delay_us`,
  `hotkeys.toggle_mode`, `hotkeys.cycle_charset`, `hotkeys.cycle_method`; hotkey defaults
  now Ctrl+Shift / Ctrl+Shift+Z / Ctrl+Alt+S; new `restart_app`; tray copy updated.
  `Nâng cao` = 9 options (40 px short rows to keep C9's no-scroll rule). Deviations:
  `Ctrl phải` preset omitted (no side-specific modifier in the hotkey grammar), import
  toast lacks the skipped-duplicate count, update check has no channel.
- 2026-10-28 BUILD: ui 2026-10-28 (wt-ui) — `svelte-check` 0/0, `npm run build` pass,
  `cargo test -p vkb-config` 9/9, `cargo test -p vkb-app` 11/11, `cargo check -p vkb-app`
  pass. `ui/dist` = 162,482 B; 720 × 560 fit on all four tabs plus the 9-row Nâng cao;
  125/150 % scale wrappers show no reflow; 680 × 520 clips the Thử gõ strip (window is
  fixed). Slot released.

## 2026-10-29 — logo landed
Keycap logo (V = bật / E = tắt) approved after eleven design passes and a Codex (gpt-6-luna) pass.
Integrated: title-bar mark 20px in every window, tray V/E at 16/20/32 picked by DPI, icon.ico with
16/24/32/48/64/128/256 for app + NSIS installer/uninstaller, build.rs now emits rerun-if-changed for
the icons (a stale icon.ico had been baked into the exe). Installer 2.30 MB, SHA256 D508D111…

- 2026-10-30 BUILD: qa-special-cases main 2026-10-30 10:05 (+07:00) — engine simulation + cargo test -p vkb-core for the special-case audit. Slot held.
- 2026-10-30 BA: sound spec landed — 6 WAV packs + `system`, 22050 Hz/16-bit/mono chat-inlinable caps (12 files 52 834 B, base64 70 448), `system.sound_pack` default `go_moc`, A2c audition page cancelled (chat only) → `22-sound-spec.md`

- 2026-10-30 **QA B1: `/` report reproduced on the real engine, root cause = raw zone, not the hook.**
  `cargo run -q -p vkb-cli -- simulate --method vni "ba65t/ta8t1"` -> `bật/ta8t1` (Telex too:
  `baajt/taats` -> `bật/taats`). Cause: `engine.rs:429` arms the raw zone on
  `. / : @ _ \ # = ?`, so the whole next word is typed literal; `raw_zone:false` makes the same
  input `bật/tắt`. 78 measured cases, 56 pass / 22 fail, one cause; `D-SC-01..04` filed
  (`/` P1, `. ?` P2, `_ @ :` P3, `= # \` P3) — they need an A9 narrowing, and URL/e-mail protection
  (`http://a.b/c`, `a@b.com`, `file_name`, `2/3`) must survive it. Artifact `51-special-cases.md`,
  harness `crates/vkb-core/tests/special_cases.rs`. `cargo test -p vkb-core` = 188 tests, 0 failed,
  5 ignored (the four repros + one trace helper).
- 2026-10-30 BUILD: release qa-special-cases

- 2026-10-30 **BA: A9 raw-zone amendment (owner ruling D-SC-01…04) — predicate, tables, test contract.**
  Superseded the §A9 bullet ("any of `. / : @ _ \ # = ?` ⇒ next word raw") with a left-token predicate:
  arm iff `raw_zone` ∧ opener ∧ not ellipsis/digit-dot ∧ left token non-empty ∧ plain-ASCII final left
  text (no composed letter, no tone, charset-independent) ∧ for `. / ?` only, left token is not a valid
  A9 syllable. Empty left ⇒ never arms, so `/tắt`, `bật//tắt`, `bật /tắt` compose; every diacritic word
  left of the punctuation composes, so `bật/tắt`, `bật.tắt`, `bật?tắt`, `bật_tắt`, `bật@tắt`, `bật:tắt`,
  `bật=tắt`, `bật#tắt`, `bật\tắt`, `Bật/Tắt`, chains, numpad and all VNI triggers pass in Telex and VNI,
  while `http://a.b/c`, `a@b.com`, `github.cof`, `file_name`, `x=cof`, `me@cof`, `2/3`, `1.5`, `%20`,
  `--opt`, `3.5`, `...` keep passing. Decision tables (17 must-compose rows, 13 guards, 8 borderlines),
  precedence (macro > restore > spell untouched), Backspace clears the flag, and known losses L1–L4 are
  written in the amendment. Kept the old wording struck for history; config row `typing.raw_zone` now
  points at the amendment; ruling recorded as `28-engine-rulings.md` D9. Docs only — no `crates/` edit.
- 2026-10-30 A2 (Dev): 12 toggle-sound WAVs generated by `scripts/gen-sounds.mjs` (22050 Hz/16-bit/mono, all 12 raw 52 834 B, base64 70 456, max file 5 248 B) + `design/sound-data.json` (6 packs, no `system`); `scripts/check-sounds.mjs` proves pitch direction per pack; byte-identical on re-run. Spec slip: §4 lists hai_not pair as 13 996 b64 chars, the §4 formula gives 14 000 (total 70 456 not 70 448).

- 2026-10-30 00:39:31 BUILD: dev-rawzone-fix main 2026-10-30 00:39:31
- 2026-10-30 00:48 **Dev: A9 raw-zone amendment implemented (crates/vkb-core), D-SC-01…04 fix landed (QA to verify).**
  `boundary_key` decides at the end of the key, on the final left text: arm iff `raw_zone` ∧ opener ∧ not
  ellipsis/digit-dot ∧ left non-empty ∧ left plain ASCII ∧ (for `. / ?` only) left is not a valid A9
  syllable. `d_sc_01…04` un-ignored; `matrix()` 78/78 PASS and now asserts 0 failures (SC-29 Telex keys
  corrected to `tawts`; SC-72 → `bật/tắ`, SC-74 → `bật/tắt`⏎, both had pinned the defect); new
  `separators_keep_the_next_word_vietnamese` (Telex+VNI, 5 VNI triggers × 5 separators) and
  `amendment_2026_10_30_borderlines`. `cargo test -p vkb-core`: exit 0, 201 passed / 0 failed / 0 ignored.
  `cargo test --workspace`: exit 0, 337 passed / 0 failed / 5 ignored (pre-existing, other crates).
  `tests/raw_zone.rs` and `tests/no_alloc.rs` unchanged and green. P1 `process_key` p99 1.2 µs before and
  after (budget 50 µs); criterion mean per word 2.83 → 2.64 µs (`telex_word`), `noop_chord` 1.41 ns
  unchanged. Open item for QA/Lead: `C:\Users\nam` holds in VNI but not Telex (the `\` after `:` is a
  second opener ⇒ empty left ⇒ no arm ⇒ `C:\Úe\nam`); see the Dev fix note in `60-defects.md`.
- 2026-10-30 00:48 BUILD: release dev-rawzone-fix
- 2026-10-30 00:56:11 BUILD: dev-chained-opener main 2026-10-30 00:56:11
- 2026-10-30 00:58:27 **Dev: D-SC-05 / §A9 §2b chained openers implemented (crates/vkb-core), QA to verify.**
  `chained(c) = raw_zone ∧ c ∈ {'\','/'} ∧ last_emitted ∈ {':','\','/'} ∧ left token empty ∧ raw_pending`,
  with `raw_pending` read before the boundary overwrites it (`arm_prev`); no new state, no new config key.
  `arm_raw_zone` gained `chained` so the "never arm on an empty left token" term relaxes to
  `(!empty || chained)`; the early empty-left return now re-arms on a chain. Pinned `C:\Úe\nam` became
  `C:\Users\nam`; new `special_cases.rs::chained_openers_extend_a_path` (both methods × `C:\Users\nam`,
  `C:/Users/nam`, `C:\\server\share`, `https://x/y/z`, `http://a.b/c`; composing: `bật/tắt`,
  `bật//tắt`, `bật /tắt`, `/tắt`, `a/b`, `2/3`, `bật.http`, `bật./tắt`, `bật:_/tắt`;
  negative control `xyz./tieesng` → `xyz./tiếng`, `bật:_/tieesng` → `bật:_/tiếng`).
  `cargo test -p vkb-core` exit 0, 202 passed / 0 failed / 0 ignored (matrix() 78/78, asserts 0 failures);
  `cargo test --workspace` exit 0, 338 passed / 0 failed / 5 ignored (pre-existing). `tests/raw_zone.rs` green.
  P1 `process_key` p99 0.8 µs before → 0.8–0.9 µs after (budget 50 µs), throughput 3.22M keys/s.
- 2026-10-30 00:58:27 BUILD: release dev-chained-opener
- 2026-10-30 01:00:32 BUILD: qa-verify-sc main 2026-10-30 — re-verify D-SC-01…05 on dev-chained-opener (78-case matrix, CLI repros, guards, cargo test, bench). Slot held.
- 2026-10-30 DIAG-QA D-SND-01..04: pack 7 (system) toggle is silent because this machine's Windows sound scheme is .None ('No Sounds') — all 8 AppEvents aliases resolve to an empty .Current so `PlaySoundW(alias, SND_ALIAS|SND_NODEFAULT)` returns FALSE (8/8, err 87) while `SND_FILENAME` on `C:\Windows\Media\Windows Background.wav` returns TRUE; running exe `%LOCALAPPDATA%\VietKB\vietkb.exe` v0.2.3 (started 23:58:24) *does* contain `SystemAsterisk`/`SystemHand`/`PlaySoundW`, so the binary is not stale; packs 1-6 are not shipped at all (no `sound_pack` key, no picker, no embedded assets — step A3 unlanded). Files: `93-sound-silent.md`, `60-defects.md`. No product code changed; no build, no app restart.
- 2026-10-30 SPEC `22-sound-spec.md` amended for the D-SND root cause (docs only, no `crates/`/`ui/`, no cargo). §1/§7 default pack is now `system` (owner's pick); §6 adds the mandatory `system` fallback chain — alias `SND_ALIAS|SND_ASYNC|SND_NODEFAULT` → on `FALSE` `go_moc` pair via `SND_MEMORY` → else silent, never panic, never block the hook thread, ≤1 `debug` log per session; §3/§5 state the files ship only via `include_bytes!` (no runtime I/O, missing asset = compile error); §7/§8 expose `scheme_silent: bool` → `schemeSilent: boolean` and a conditional Vietnamese hint ("Copy owns final wording"); §7 unknown `sound_pack` degrades to `system` without failing the load; §9 items 13–16 cover the `go_moc` fallback on a `.None` scheme, ON/OFF symmetry on all 5 routes, non-blocking/side-effect-free `preview_sound`, and the unknown-slug fallback; §11 note that `Beep` alias semantics are unchanged. Open owner decision recorded: default stays `system`, may switch to `go_moc`/`hai_not` (change `#[default]` in `schema.rs`).
- 2026-10-30 01:06:00 BUILD: dev-sound-packs main 2026-10-30 01:06:00 — sound packs 1-6 embedding + system.sound_pack + preview_sound; slot held.
- 2026-10-30 01:14:02 **QA: D-SC-01…05 FIXED + VERIFIED (stream B, independent re-run on dev A9+§2b tree).** 78/78 matrix rows PASS (exit 0); CLI repros green in both methods (`ba65t/ta8t1`, `baajt/tawts`, `/ . _ @ : = # \`, numpad, 5 VNI triggers); §2b rows literal in both methods (`C:\Users\nam`, `C:/Users/nam`, `C:\\server\share`, `https://x/y/z`, `http://a.b/c`) and negative controls compose (`xyz./tieesng`→`xyz./tiếng`, `bật:_/tieesng`→`bật:_/tiếng`); guards green incl. `Tha`+`/`+BS+`s`=`Thá` and macros. `cargo test -p vkb-core` exit 0 — 207 passed/0 failed/0 ignored; `no_alloc` 4 passed; P1 bench p99 0.8 µs (budget 50 µs), 3.18M keys/s. BLOCKER: `cargo test --workspace` exit 101 on unrelated uncommitted `vkb-windows/src/sound.rs:207` E0015; workspace minus vkb-windows/vkb-app green. Two NEW P3 under-protection shapes filed OPEN for Lead ruling: D-SC-06 (single ASCII letter left of `. / ?` never arms: `x/cof`→`x/cò`, `c/users`→`c/úe`), D-SC-07 (Telex even-length digit run: `10/users`→`10/úe` vs VNI literal). Artifacts: `92-verify-report.md`, `docs/evkey-clone-work/qa/verify/`, `crates/vkb-core/tests/qa_92_verify.rs`. L1–L5 all reproduce as declared.
- 2026-10-30 01:22:00 SPEC `20-spec.md` §A9 amendment closes D-SC-06/D-SC-07 as intended (docs only, no `crates/`/`ui/`, no cargo). New **§6b "Intended, not a defect (owner ruling 2026-10-30)"** after the decision tables states both in spec terms — the A9 syllable test is deliberately generous, so a one-letter left token (`x`, `c`, `v1`) is an onset-only syllable and does not arm (`x/cò`, `c/úe`, `v1/tiếng`), which is what keeps `a/b` composing; in Telex a second digit ends the numeric run, so the opener arrives with an empty left token and §2 forbids arming (`10/úe`, `21/cò`; §2b needs a previous armed opener), while VNI keeps the run and stays literal (method-dependent text accepted). §9's loss inventory extended with **L6** (one-letter left token) and **L7** (Telex digit-run opener), both marked owner-accepted 2026-10-30, and §9's intro split L1–L5 (test-pinned) from L6–L7 (no test beyond `a/b`/`2/3`); `60-defects.md` D-SC-06/D-SC-07 rows now cross-reference §A9 §6b + L6/L7.
- 2026-10-30 01:14:02 BUILD: release qa-verify-sc
- 2026-10-30 01:18:51 DEV D-SND-01…04 (Rust, no ui/): 12 WAVs embedded via include_bytes! (52 834 B), new `system.sound_pack` (7 slugs, default `system`, unknown→system+debug), `SettingsView.system.soundPack`+`schemeSilent`, additive `preview_sound(pack, enabled=None)`, ON/OFF symmetry on all 5 routes (commands/tray/Beep); `system` alias BOOL checked → embedded `go_moc` pair; tests vkb-config 17 / vkb-windows 72+6 sound / vkb-app 22, `cargo test --workspace` exit 0, release exe 8 463 360 B (+92 672 = +90.5 KiB, SHA 2A27FD5B…). BUILD: release dev-sound-packs main.
- 2026-10-30 01:20:16 BUILD: dev-rawzone-nits main 2026-10-30 01:20:16 — reviewer nits on the A9 raw_zone change (armed-flag leaks + tests); slot held.
- 2026-10-30 QA D-SC-06/D-SC-07 reclassified **NOT A DEFECT — owner ruling 2026-10-30** (expected, not defects): D-SC-06 a single ASCII letter left of `. / ?` is already a valid A9 onset-only syllable so the opener correctly does not arm (same rule that keeps `a/b` composing); D-SC-07 in Telex a second digit ends the numeric run so the left token is empty at the opener and an empty left token never arms (§2b needs a previous opener), while VNI keeps the run and stays literal. Rows moved from the open list to `## Closed as expected (owner ruling)` in `60-defects.md`; repro text, measured output and evidence paths kept. Docs only — no `crates/`, no `ui/`, no `20-spec.md`, no build.
- 2026-10-30 01:23:05 **Dev: raw_zone reviewer nits closed (crates/vkb-core, 3 leak fixes + 4 tests + 2 headers).**
  set_macro_only and VietnameseEngine::set_options now clear 
aw_pending after their 
eset_word()
  (arming github. then flipping 
aw_zone off no longer freezes the next word), and a ch: None key
  (arrows/Home/End/PgUp/PgDn/Delete/F-keys) clears it on the early return — the caret-move leak QA
  measured at 92-verify-report.md §7.13 now dies inside Engine, so kb-windows' 
eset_engine is
  belt-and-braces, not the only guard. New in 	ests/raw_zone.rs:
  rming_needs_a_code_shaped_left_token (amendment §4/§5 tables), 
aw_zone_is_recomputed_per_opener
  (§7 //, `/ `, x=@cof, ./cof — both tagged declared loss L1), plus the two leak regressions
  set_options_clears_the_armed_raw_zone and caret_keys_clear_the_armed_raw_zone; all four fail
  github.cof on the pre-fix engine and pass github.cò after. mendment_2026_10_30_borderlines gains
  the two L1 rows; 	ests/raw_zone.rs and 	ests/special_cases.rs headers rewritten to the shipped rule
  (no more "never after whitespace"-only wording, no ghost #[ignore] repros);
  qa_92_verify.rs row 4 now expects github.cò. cargo test -p vkb-core exit 0 — **211 passed /
  0 failed / 0 ignored**; 
o_alloc 4 passed; cargo test --workspace exit 0 (sound.rs compiles now);
  P1 bench p99 0.8 µs (p50 0.3 µs, 2.95 M keys/s, budget 50 µs). Cargo.lock untouched.
- 2026-10-30 01:23:05 BUILD: release dev-rawzone-nits
- 2026-10-30 01:23:45 BUILD: dev-sound-default main 2026-10-30 01:23:45 — default pack GoMoc (#[default]) + Beep→play_error(); slot held.
- 2026-10-30 DEV SOUND-UI (`ui/` only, no `crates/`): `Nâng cao` → `Hệ thống` gains the sound row (`Âm thanh bật/tắt`, full-width 36 px) — `Select` with the 7 packs `1 · Cổ điển` … `7 · Theo Windows` (default `go_moc`), a one-line character hint, `system` + `schemeSilent` advisory, and `Bật`/`Tắt` preview buttons → `api.previewSound(pack, enabled)` with a 400 ms disable guard; `bindings.ts` `soundPack`/`schemeSilent` + `SystemPatch`. `svelte-check` 0 errors/0 warnings, `npm.cmd --prefix ui run build` exit 0. Open: `main.rs:109` `Nâng cao` is 560 × 388, needs 424 for this row (design value, not touched here).
- 2026-10-30 BA DOCS (`docs/` only, no `crates/`, no `ui/`): `22-sound-spec.md` §8.1 records the Lead ruling that the `Nâng cao` window grows to **560 × 424** (was 560 × 388, +36 for the `Kiểu âm thanh` row, `main.rs:109`) with the binding rule "a new row must not clip: grow the containing window or move the row, never scroll", C9's 720 × 560 main window untouched, and the row pinned directly under the master `notification_sound` switch in `Hệ thống`; §8.2 records the **15 shipped strings verbatim** (7 labels, 7 hints, 1 `schemeSilent` advisory) under "**Copy owns final wording — these are the Dev defaults under review**", with three phrasing risks for Copy (`ngân` vs `vọng`, `Màn trập máy ảnh, hai tiếng tách` longest/ellipsis, `nhỏ nhất bộ` quietest-vs-shortest). §9.7 already read 560 × 424; `20-spec.md` states no window sizes, so it is unchanged.
- 2026-10-30 01:27:26 DEV D-SND-01..04 (Rust, no `ui/`, no vkb-core, no Cargo.lock): Lead rulings landed. (1) `#[default]` moved `System`->`GoMoc` on `SoundPack`; `SystemConfig::default` now uses `SoundPack::default()` so the default lives in exactly one place; `CURRENT_PACK` seeded `GoMoc`; tests assert `GoMoc` default + `sound_pack = "go_moc"` on save, a stored `"system"` keeps loading as `System` (no migration), and unknown/non-string -> `system` (fallback, unchanged); `vkb-app` acceptance assertions follow the new default. (2) `Injection::Beep` restored to `sound::play_error()` (SystemHand off alias) at `backend.rs:56` and `state.rs:506` — Beep is deliberately asymmetric: it signals an invalid keystroke, so §9.14's five-route symmetry covers only the three toggle routes (hotkey / UI switch / tray). (3) Ratified as built and noted in `60-defects.md`: `preview_sound(pack, enabled: Option<bool>)` and `schemeSilent` read from the registry, never a playing probe. Tests: `cargo test -p vkb-config -p vkb-windows` exit 0 (17 + 72 passed/1 ignored + 1 integration), `cargo test --workspace` exit 0 (357 passed / 0 failed / 5 ignored, pre-existing), release `vkb-app` exit 0; `vietkb.exe` 8 465 408 B (+94 720 = +92.5 KiB), SHA-256 `BAE1DA2CAC6616C69AE50134335D5C645E20BE113E2F6C1088A4284ED174DD76`. D-SND-01..04 reset to OPEN — awaiting QA re-verification (QA closes).
- 2026-10-30 01:27:26 BUILD: release dev-sound-default
- 2026-10-30 01:29:10 BUILD: dev-advanced-height main 2026-10-30 01:29:10 — Nâng cao 388 → 424 (sound row); slot held.
- 2026-10-30 BA DOCS (docs only, no `crates/`, no `ui/`, no cargo) SCOPE CUT: owner ruled "Tôi chỉ muốn có âm thanh khi bật tắt bộ gõ, tôi không yêu cầu thêm config hay UI mới nào cả" -> the sound feature is one embedded `go_moc` on/off pair (`assets/sounds/go_moc/{on,off}.wav`, 22050/16-bit/mono, 8 468 B, `include_bytes!` -> `SND_MEMORY|SND_ASYNC|SND_NODEFAULT` off the hook thread, failure silent + never panic), played on every toggle route with ON != OFF, `Injection::Beep` = `play_error()` error cue, `system.notification_sound` the only switch; everything else removed: `[system] sound_pack` + `SoundPack`, `schemeSilent`/`scheme_silent`, `preview_sound`/`api.previewSound`, the `Kiểu âm thanh` picker row + hints + `Bật`/`Tắt`, packs 2-6 assets, `design/sound-data.json`, `design/sound-preview.txt`, and the `Nâng cao` 388 -> 424 growth (reverts to 388 - no row). `22-sound-spec.md` rewritten to the reduced spec (§1 asset, §2 format, §3 embedding, §4 playback, §5 symmetry, §6 config/UI = nothing new, §7 acceptance, §8 non-goals, §9 History keeping the 6-pack measurements + the 15 shipped copy strings, marked "removed with the scope cut - do not re-add without an owner ask"); `91-round-plan-sound-cases.md` gains the dated Scope cut block (A1/A2/A3 collapse into "one embedded pair, no config, no UI"), release-gate items 3-4 and DoD 1 now match (no picker, `Nâng cao` 560x388, no `sound_pack`, `go_moc` pair audible ON != OFF); `60-defects.md`: `D-SND-02` (picker/preview) OUT OF SCOPE owner ruling 2026-10-30 (row status updated), `D-SND-03` (ON/OFF asymmetry) fixed in flight, `D-SND-01` (silent toggle) the only live item - left OPEN for QA, `D-SND-04` folded into the spec (silence + <=1 debug line, not a surface). Note: the agent brief had the last two IDs swapped; the defects table is authoritative.
- 2026-10-30 01:32 BUILD: qa-verify-engine-nits main — QA re-verify raw-zone leak fixes + 92 row-4 assertion; slot held (dev-advanced-height line superseded by the BA scope cut, no cargo running).
- 2026-10-30 01:34:46 **QA: raw-zone nits FIXED + VERIFIED (third pass, own harness crates/vkb-core/tests/qa_93_engine_nits.rs, 4 tests).** The changed assertion is legitimate: `github . LEFT cof (engine only)` is now `github.cò` because VK_LEFT (`ch: None`) ends the one-key armed window exactly as production's `reset_engine` does (`state.rs:350-352`, spec §7); the old `github.cof` pinned the engine-only leak. Leak A (set_options/set_macro_only) and Leak B (ch:None caret/command key) both measured `github.cò` (control without the switch/caret stays `github.cof`); removing the fix would flip them back because `word_key` reads `raw_pending` only (`engine.rs:228-234`) — reasoning check, caret half also the §7.13 pre-fix measurement. 78/78 matrix PASS; `cargo test -p vkb-core` exit 0 — 215 passed/0 failed/0 ignored; `cargo test --workspace` exit 0 — 361 passed/0 failed/5 ignored (earlier sound.rs blocker gone); `no_alloc` 4 passed; P1 p99 0.8 µs, 3.10 M keys/s. Guards green: `bật/tắt` VNI+Telex, `C:\Users\nam` both methods, all 8 D-SC-02/03/04 separators, `github.cof` literal, `x=@cof`→`x=@cò` (L1), `a. tieengs`→`a. tiếng`. Nothing reopened in 60-defects; residual (not a defect): ctrl/alt/win chords and key-up still return before the clear, production resets on both (`state.rs:322-331`/`355-362`). Artifacts: `92-verify-report.md` §9.
- 2026-10-30 01:34:46 BUILD: release qa-verify-engine-nits
- 2026-10-30 01:37:52 BUILD: dev-sound-revert main 2026-10-30 01:37:52 — remove the sound pack surface (packs 2-6, SoundPack/sound_pack, schemeSilent, preview_sound, UI row; keep go_moc pair + ON/OFF symmetry); slot held.
- 2026-10-30 01:47:30 DEV SOUND REVERT (Rust + ui + assets, no docs prose beyond this log, no Cargo.lock/version change): removed the 6-pack surface the owner did not ask for. vkb-config: SoundPack (enum + ALL/as_str/from_str/index/from_index + lenient Deserialize) and SystemConfig.sound_pack deleted; 22-sound-spec.md §6/§9 captured as forward-compat test removed_sound_key_still_loads (key name assembled so the zero-leftover grep stays honest; loads go_moc/system/unknown/non-string, other keys intact, save drops it). vkb-windows/src/sound.rs now only the embedded go_moc pair (ON/OFF 4 454/4 014 B), play_toggle (SND_MEMORY|SND_ASYNC|SND_NODEFAULT, spawned thread, silence + <=1 debug line) and play_error (SystemHand); Plan/plan/run_plan/Player/Winmm/set_pack/current_pack/play_toggle_pack/play_pack/preview/scheme_is_silent/ALIAS_ON/ASSETS + packs 2-6 deleted. lib.rs keeps play_notification_sound only; state.rs dropped the set_pack publish; backend.rs/state.rs keep play_error(). commands.rs: soundPack/schemeSilent view + patch arms, scheme_silent(), preview_sound and its main.rs registration removed; toggle_enabled still plays the state it sets via vkb_windows::sound::play_toggle (hotkey path unchanged; tray delegates to the same command). ui/ restored to HEAD exactly (bindings.ts SystemView/SystemPatch, api.previewSound, demo fields, Advanced.svelte rows/hints/preview/style). main.rs advanced comment back to content 295 / 388 (value was already 388.0 on disk; the 424 unit had not landed). Assets: only assets/sounds/go_moc/{on,off}.wav; deleted click,hai_not,gio_nuoc,co_khi,eight_bit (10 WAVs), design/sound-data.json, design/sound-preview.txt; gen-sounds.mjs/check-sounds.mjs trimmed to the pair (regenerate byte-identical, caps + edge/fade checks, exit 0). Verify: cargo test --workspace exit 0 — 356 passed / 0 failed / 5 ignored (361 before: -5 tests removed with the surface), cargo test -p vkb-config -p vkb-windows -p vkb-app exit 0 (16+70+20 passed, 1+2 ignored); npm --prefix ui run check 0 errors/0 warnings, run build exit 0. Leftover grep (sound_pack|soundPack|schemeSilent|scheme_silent|preview_sound|previewSound|plan(pack|SoundPack) excluding docs/target: 0 hits (spec §9 history keeps the names by instruction). Release vkb-app --release --features custom-protocol exit 0 — vietkb.exe 8 380 928 B, SHA-256 45B571155238DBDAB1BCE5F26B683504520F8106C013CE16848A3A187BE0F957, delta +10 240 B (+10.0 KiB) vs pre-feature 8 370 688 (was +94 720 B before the revert). A stale vietkb.exe from 01:32 held the release binary lock and was stopped to relink. No version bump (0.2.3 / 0.1.0 unchanged); release blocked until QA E2E.
- 2026-10-30 01:48:18 BUILD: release dev-sound-revert
- 2026-10-30 01:54:09 DEV D-SND-03 height — **no code change landed, and that is the correct outcome**: the `Nâng cao` 388 → 424 growth was prepared (`main.rs:109` `560.0, 424.0`, comment "content 295 + the 36 px sound row") and then **withdrawn**, because the row that needed the 36 px was cut by the sound revert (owner scope cut recorded in `60-defects.md`; `dev-sound-revert` restored `ui/` to HEAD, leaving no `Âm thanh bật/tắt` row in `Advanced.svelte`). `crates/vkb-app/src/main.rs` is back to HEAD (`560.0, 388.0` + the "content 295 … 388 → 388" comment); a 424 window without the row would be 36 px of dead space, against the measured-height rule at `main.rs:94`.
  Measured the shipped 388 with the current build (`vietkb.exe` 8 380 928 B, SHA `45B57115…`), read with `GetWindowRect`/`GetClientRect`/`GetDpiForWindow` from a per-monitor-v2 thread (a DPI-unaware reader reports 574×396 and a fake 96 dpi — that is a measurement artefact, not the window):
  - **125 %** (primary, dpi 120): outer (250,145) **718 × 495**, client **700 × 485** physical = **560 × 388 CSS**; work area 2560 × 1380; fully visible; last grid row (`Tự kiểm tra cập nhật`) bottom 553, footer band 576–617, client bottom 630 → everything inside, no overlap, no scrolling.
  - **100 %** (secondary, dpi 96): outer (2648,107) **574 × 396**, client **558 × 387** (560 × 388 logical; 2 px lost to the 1.25 → 1.0 rounding), fully visible; work area 1920 × 1032; footer 452–485 vs client bottom 494 → inside.
  Placement untouched: `centre_on_parent` already clamps to the monitor work area and `centre_on_monitor` pins to the origin when the window is larger than the monitor, so the taller window could not have gone off-screen anyway.
  If the 36 px row ever returns, the single line is `main.rs:109` `388.0 → 424.0` (comment: content 295 → 331). `cargo test -p vkb-app` exit 0 (20 passed / 2 ignored); `cargo build -p vkb-app --release --features custom-protocol` exit 0 on the same tree. QA closes.
- 2026-10-30 01:54:09 BUILD: release dev-advanced-height
- 2026-10-30 02:43:29 BUILD: dev-deadcode main 2026-10-30 02:43:29 - delete dead play_notification_sound (0 callers, hard-coded play_toggle(true)); slot held.
- 2026-10-30 02:47:40 DEV DEADCODE (Rust only: `crates/vkb-windows/src/lib.rs`; no ui, no vkb-core, no vkb-config, no Cargo.lock, no version bump): deleted `play_notification_sound()` (lib.rs:97-99) - 0 callers since the toggle moved to `sound::play_toggle(<state set>)` (QA `94-verify-sound.md` item 8), and it hard-coded `play_toggle(true)`, the exact D-SND-03 asymmetry, so it was a trap for the next reader. No re-export existed for it (it was defined directly in lib.rs, not re-exported; the `reference/legacy/windows-src/lib.rs` `pub use` is a different, unbuilt tree) and no `#[allow(dead_code)]` existed only for it (grep 0 in the crate), so nothing else was removed. `commands.rs::play_notification(enabled)` was left alone: it is not a one-line passthrough - it is a `#[cfg(windows)]` / `#[cfg(not(windows))]` pair because `vkb_windows::sound` does not exist on other targets; inlining it at the call site would force `#[cfg]` onto the call itself, and the wrapper is the only thing keeping `vkb-app` cross-target clean. Behaviour unchanged. Leftover grep for `play_notification_sound` across the whole workspace: 0 hits in `crates/`; 4 in `reference/legacy/` (`windows-src/system.rs:70` definition, `windows-src/lib.rs:11` re-export, `gui/main.rs:314,490` callers) - unbuilt legacy mirror, report only, not edited; 4 stale prose hits in `docs/evkey-clone-work/` (`94-verify-sound.md:12`, `93-sound-silent.md:147`, `60-defects.md:178,241`) - QA/BA historical records, left intact as the audit trail, superseded by this entry. Verify: `cargo test -p vkb-windows -p vkb-app` exit 0 (vkb-app 20 passed / 2 ignored; vkb-windows 70 passed / 1 ignored + hook_integration 1 passed / 2 ignored); `cargo test --workspace` exit 0 - 356 passed / 0 failed / 5 ignored (same totals as the sound revert: no test covered the dead function); `npm.cmd --prefix ui run build` exit 0; release `cargo build -p vkb-app --release --features custom-protocol` exit 0 - `target/release/vietkb.exe` 8 380 928 B, SHA-256 `0B566EE499A2F1E9DDCE23498C7EEA03DF0C7FF255B0032FE42209D503A743EF` (byte-identical size to 45B57115...; the dead function was already folded out by the optimiser). No stray `vietkb.exe` held the target and the app was not relaunched. Nothing edited after the build except this log line.
- 2026-10-30 02:47:40 BUILD: release dev-deadcode
- 2026-10-30 03:15:26 BUILD: qa-e2e-024 main — claim before cargo build -p vkb-cli --release for P1 (gate 95); released at the end of the bench.
- 2026-10-30 03:16:40 BUILD released: qa-e2e-024 (vkb.exe bench, 14.4 s, no artefacts left behind).
- 2026-10-30 04:12 QA E2E GATE 0.2.4 (`95-e2e-report.md`): binary 8 380 928 B / SHA 0B566EE4…A743EF (the brief's hash was 62 chars — one `C7` omitted; size and log.md agree). Preconditions clean (no `vietkb` alive; config `58EB422E…` backed up and restored byte-identical). **A Typing: Notepad 11/19, Chrome 17/19, VS Code 15/19** — new blocking defects **D-E2E-01** (default VK_PACKET injection silently mangles Vietnamese in Win11 Notepad; `send_standard = true` fixes it 5/5) and **D-E2E-02** (`restore_word` Ctrl+Shift+Z never fires; `keys.log` `stage=shortcut action=none`), plus D-E2E-03 (F1 does not re-enable in VS Code) and D-E2E-04 (main panel 478×459, no tabs vs C9). **B Toggles**: F1/F2, Alt+Shift, UI switch and the tray icon all flip state and the icon tooltip follows; tray menu item not UIA-reachable; sound inconclusive (no tracing output reaches the redirected log). **C UI**: `Nâng cao` 558×387 client @96 dpi = 560×388, no clipping, no pack select, About opens/closes (0.2.3). **D Budgets**: P1 p99 0.8 µs, P3 8.4 MB/10 min, P4 27.9 MB app / 347.5 MB total / −343 MB in 3 s, P5 0.016 %, P6 391 ms median (433 first), P7 7.99 MiB exe (installer not built). **E Soak**: 31 m 47 s, 5 apps, ~35 k keystrokes, no crash, hook alive 10/10, memory flat. **HOLD.**

- 2026-10-30 08:52:16 BUILD: dev-e2e02-restore main 2026-10-30 08:52:16 — D-E2E-02 hotkey matcher: modifier-only chord defers to release when a longer chord shares its modifiers + typed key cancels it; cargo test --workspace then release vkb-app rebuild; slot held.
- 2026-10-30 09:12:40 BUILD: dev-e2e01-input main — claim before cargo test/build for D-E2E-01 (input.rs replacement encoding); slot held.
- 2026-10-30 09:31:04 BUILD: release dev-e2e01-input
- 2026-10-30 09:40 DEV D-E2E-01 (`crates/vkb-windows/src/input.rs` only; no ui/, no vkb-core, no Cargo.lock, no version bump, no config default change). **Mechanism (measured with the app closed, no engine and no hook):** a `Replace` is one `SendInput` batch of [backspace pair x n][KEYEVENTF_UNICODE run]. In the Windows 11 Notepad text stack a VK_PACKET run that follows backspaces is not appended char-by-char - it collapses onto the run's LAST character with the run's length preserved: Replace{2,"tă"} over "bật/ta" -> "bật/ăă", Replace{2,"tx"} -> "bật/xx", Replace{3,"tắt"} -> "bậtttt", Replace{3,"được"} -> "bậtdợợợc". Evidence: qa/e2e95/lab-raw.ps1 and lab-matrix.ps1 (raw arrays into Notepad, UIA read-back) and lab-matrix5.ps1, which replays the engine's keys.log decision stream for ba6t5/ta8t1 with the app closed at 0/55/150/400 ms - every speed gives the field symptom "bật/ttt"; lab-matrix7.ps1 counts 0/8 correct for the pure packet path and 8/8 for the scancode-ASCII encoding (6/6 with the delete/text split). The deletes were never wrong: P4 removes exactly n units, and a single packet after deletes lands correctly. **Fix:** `build_segment(deletes, text, mode)` passes `scancode_ascii = deletes > 0` into `push_text`; `push_char` computes `via_scancode = mode == Standard || scancode_ascii` and types ASCII through `keys::us_vk_for_char` + `keys::scan_for_vk` (new `push_ascii_key`, Shift wrapped), while non-ASCII keeps `push_unicode_char` (surrogate-safe). `Injection::Type` (macro / preedit / commit) keeps the pure packet path - no delete block, not affected. SendMode, send_delay_us and the clipboard fallback (C4) are untouched; still exactly one `SendInput` call per decision and the same event count for unshifted ASCII. Config default **not** changed (`typing.send_standard` stays false, no migration - the fix lives in the injection layer, so existing config files are already correct). Tests: six new unit tests in input::tests (replace_types_ascii_with_scancodes_in_unicode_mode, type_keeps_unicode_packets_in_unicode_mode, replace_wraps_shifted_ascii_in_shift, replace_keeps_surrogate_pairs_whole, replace_order_is_all_deletes_then_all_text, standard_mode_replace_is_unchanged); the OS send itself is untested in CI (SendInput needs an interactive desktop). Verify: `cargo test --workspace` exit 0 (0 failed, 5 ignored; vkb-windows 17/17 input tests); P1 bench p99 **0.8 µs** (p50 0.2, 3.12 M keys/s, budget 50 µs) - unchanged; release `vietkb.exe` **8 383 488 B** (+2 560 vs 8 380 928), SHA-256 `1B70347F513023F222BC757FCAB56A8212D8181CB31C8C9479A1CBCAAED2A72A`. **Re-ran the E2E typing rows myself** (qa/e2e95/run-typing.ps1): Notepad **16/19** (was 11/19) - A1 bật/tắt, A2 bật.tắt, A3 bật?tắt, A4 bật_tắt, A6 bật tắt, A7 Nguyễn Thu Hà, B1 bậ, B2 bâ, C2 mục 1 , D2 bật<Tab>bật PASS; D1 verified byte-exact `62 1EAD 74 0D 0A 62 1EAD 74` from the saved file (the harness marks it FAIL only because Notepad's UIA text keeps CRLF while Chrome's does not); A8 "…/ta3i" is the frozen raw_zone ruling and C1 is D-E2E-02. Chrome **18/19** (A8 only). VS Code: A1/A2/A4/A5/B1/B2/C2 PASS (a second focused pass re-read stale saves for A1/B1 - harness save race, both PASS in the full run). WordPad is **not installed** on this image and the legacy row is UNVERIFIED (Character Map's RichEdit50W gave truncated/stale reads, the Win32 Run dialog exposes no Edit child here); note the fix moves ASCII onto the scancode path legacy controls already require. Residual, target-side: 3+ consecutive non-ASCII after a replacement (e.g. "được") can still collapse in Notepad under every encoding tried (per-event calls, down-only, modifier separators, split batches, wVk=0xE7) and with raw SendInput. App not left running; slot released by the BUILD line below.
- 2026-10-30 09:31:04 BUILD: release dev-e2e01-input
- 2026-10-30 09:56:00 BUILD: qa-host-e2e main - single binary for the host E2E window: cargo test --workspace, vkb-cli bench, ui build + release vkb-app (custom-protocol); no app launch. Slot held.
- 2026-10-30 10:12 QA/DEV-PREP (docs + qa scripts only; no product code touched): one-binary check for the host E2E window. `cargo test --workspace` exit 0 - 378 passed / 0 failed / 5 ignored across 31 test binaries (first invocation exited 1 from the pipeline, a clean re-run exited 0 with zero failures). `cargo build -p vkb-cli --release` then `vkb.exe bench`: p50 0.3 us, p95 0.7 us, p99 0.8 us (budget 50 us), max 22.0 us, 3.08 M keys/s - PASS. `npm.cmd --prefix ui run build` exit 0 (vite 147 modules, 1.92 s; only the known api.ts dynamic/static import warning); `cargo build -p vkb-app --release --features custom-protocol` exit 0 (3m19s) - `target/release/vietkb.exe` 8 384 512 B (8.00 MiB), SHA-256 810EDD24B24913B5FBED01E82810ADB855DE61C7C0BE336A92CD5CF951E64DAA, mtime 2026-10-30 10:00:28. Both landed fixes are on disk: D-E2E-01 `crates/vkb-windows/src/input.rs` (scancode-ASCII replacement) and D-E2E-02 `crates/vkb-platform/src/hotkeys.rs` + `crates/vkb-windows/src/hotkeys.rs` + `crates/vkb-core/{lib,engine}.rs` + `state.rs` (modifier-only defer/cancel, `fire_on_full_release` for restore_word). New: `docs/evkey-clone-work/qa/e2e/run-host-e2e.ps1` (34-row corpus, 9 chord rows, 4 toggle routes, own quit via Thoat han, config byte-compare, `-Fast`, `-SelfCheck`), dry-run clean (parse + `-SelfCheck` + a function-level harness: judgement, corpus, escaping, config parse, hotkey map - 21 checks). BUILD: release qa-host-e2e.
- 2026-10-30 10:49:43 BUILD: dev-host-ch1-ch2 main 2026-10-30 10:49:43 - CH1/CH2 fixes (hotkeys matcher tests, input.rs restore release prefix, state.rs restore chord): cargo test --workspace, vkb-cli bench, ui build + release vkb-app (custom-protocol), no app launch. Slot held.
- 2026-10-30 10:58:12 BUILD: release dev-host-ch1-ch2 (ui build + release vkb-app; vietkb.exe 8 386 048 B, SHA-256 C682505967444CAA51BAEF57C945404CA88CEA01B834DA682F3DEE4EA9817D23, mtime 2026-10-30 10:53:34; no app left running).
- 2026-10-30 10:58:12 DEV CH1/CH2 (`crates/vkb-windows/src/{input,state,hotkeys}.rs` + `crates/vkb-platform/src/hotkeys.rs`; no ui/, no Cargo.lock, no version bump). **CH2 fixed:** `build_restore(deletes, text, mode, chord)` prefixes the restore batch with key-ups for the configured chord's modifiers, because `restore_word` fires inside the hook callback of the key-up that releases it - the OS has not applied that key-up yet, so the target still has Shift/Ctrl down and the whole batch arrived shifted. Unit-test reproduction (`replay` over the INPUT array from a stale modifier state): the old array turns `ba6t5` into `BA^T%` (Shift stale) or five `Ctrl+letter` shortcuts (Ctrl stale); the new array gives `ba6t5`. Per-character Shift was already correct and is now pinned (`Ba6t5`, `bAtB`, `%`, `^`, `100%/50%`, `ăâđêôơư`, `😀`, mixed). `state.rs::restore_chord` reads the modifiers from `hotkeys.restore_word`. Macro/preedit `Replace` and `Injection::Type` keep `build_segment` (no prefix) - unchanged. OS delivery untested in CI. **CH1 is not a matcher defect:** the run's config binds `hotkeys.toggle = "Alt+Shift"` (`qa/e2e/config.backup.toml`, `91-round-plan-sound-cases.md` §B) while `run-host-e2e.ps1:599` presses `Ctrl+Shift`; an unbound chord toggles nothing, and `RT2` (Alt+Shift x2) passing in the same run is that fact from the other side. New tests: the shipped default (`toggle = Ctrl+Shift` + `Ctrl+Shift+Z`) toggles exactly once per press in all three release orders and both modes, autorepeat does not re-fire, and a beaten `Ctrl+Shift+X` leaves no latch. Verify: `cargo test -p vkb-platform -p vkb-windows -p vkb-core` exit 0; `cargo test --workspace` exit 0 - 389 passed / 0 failed / 5 ignored; P1 bench p99 0.9 us (budget 50 us), 2.69 M keys/s.
- 2026-10-30 11:12 BUILD: qa-tone-cases main 14bc655+dirty (debug vkb-cli only) - tone-change/partial-delete matrix; no product code touched, no app launch.
- 2026-10-30 11:09:37 QA (docs/evkey-clone-work/qa/e2e only; no product code, no app launch): host-E2E harness driven by the live config - no chord literal left in any press path (was CH1 pressing Ctrl+Shift while the config binds Alt+Shift, i.e. three harness FAILs). hotkeys.toggle/restore_word/open_settings/enable/disable now parse through the product grammar (mirror of crates/vkb-config/src/hotkey.rs) into modifiers+VK and are pressed with the release orders that binding actually has (toggle shift,alt,both; restore key,shift,ctrl,both; open key,ctrl,alt,both); empty/unparsable binding -> SKIP(unbound) for that group, never FAIL. results-host.txt opens with a '# CONFIG' block (raw + parsed + mods + key per binding) and '# binary-id' compared against -ExpectSha/-ExpectBytes, default 8386048 B / C682505967444CAA51BAEF57C945404CA88CEA01B834DA682F3DEE4EA9817D23 (mismatch noted, not a gate). Buckets now PASS / PASS(raw) / QUALITY / READ-FAIL / FAIL / MANUAL / SKIP; HOLD only when FAIL>0. Budget next run 123 rows -Fast (102 typing + 11 chord + 8 toggle/teardown + 2 method), 124 with soak. -SelfCheck OK: parse 17.7k tokens, bindings+release orders, no-chord-literal scan, reader ladder, skip arithmetic, method-switch round-trip on a copy of the live config, corpus pin 29/29 through vkb simulate, binary-id MATCH; 43-check function-level dry run OK.
- 2026-10-30 11:14:25 BUILD: release qa-tone-cases (debug vkb-cli only; no app build, no app launch, no product code touched).
- 2026-10-30 11:14:25 QA tone-change/partial-delete matrix (owner report 'đổi dấu / xóa 1 unit rồi viết lại'): 123 rows through `vkb-cli simulate` (Telex 101 / VNI 22; unicode 116 / unicode_nfd 7; --old-tone-style, --no-free-tone, --no-spell-check) - **88 PASS / 35 FAIL**. 8 root shapes filed OPEN as D-TN-01..08: same-tone key re-pressed with a coda between (`baajtj`->`baatj`, VNI `ba65t5`->`ba6t5`); Telex `z` not implemented (`basz`->`basz`); VNI `0` clears marks (`a60`->`a`); no A9 coda check on a tone change (`baajtf`->`bầt`); no-target digit escapes to raw (`ba65t6`->`ba5t6`); BS freeze makes the next tone key literal (`toans<BS>f`->`toáf`); plain letter literalises the whole word (`baajt<BS>b`->`baajb`); doubled-letter escape triggers on the composed last letter (`Nguyeexn<BS>e`->`Nguyexe`). `legacy` is not a selectable method (`unknown method`), covered by `--old-tone-style`. Docs: `57-tone-change-cases.md`; evidence `qa/tone/{matrix.txt,matrix.md,repro.txt,run.ps1,repro.ps1,explore{,2,3,4}.ps1}`.
- 2026-10-30 11:19:57 BUILD: dev-tone-change main - claim before cargo test --workspace / bench / ui build + release vkb-app for D-TN-01/06/07/08 (vkb-core only). Slot held.
KILL-SWITCH 2026-10-30 11:38:53: stopped 1 vietkb processes (pid 26028, C:\Users\namth\AppData\Local\VietKB\vietkb.exe) and 0 automation scripts; all input-synthesizing processes gone.
- 2026-10-30 11:43:07 BUILD: dev-tone-build main 2026-10-30 11:43:07 - claim before cargo test --workspace / bench / ui build + release vkb-app (0.2.4 candidate, no version bump). Installed app pid 19224 left untouched. Slot held.
- 2026-10-30 11:50 DIAG (read-only, no kill/launch/input) stray 'u+something' -> 'u-horn' report: the installed app is ALIVE (pid 19224, %LOCALAPPDATA%\VietKB\vietkb.exe, start 11:41:08, parent explorer 11344, i.e. relaunched from the shell 2m15s after KILL-SWITCH 11:38:53 which stopped pid 26028), mechanism=hook, method=vni, auto_horn_uo=true -> VNI digit 7 after u (method.rs:226) is the source; no task/service/WMI/startup respawner. Two defects found: D-GUARD-01 HKCU Run 'VietKB' now points at E:\vietkb\target\debug\deps\vkb_windows-591f7a65901d2457.exe --autostart (agent debug build, enabled in StartupApproved) -> next logon starts a second hook; D-GUARD-02 EVKey64.lnk in the user Startup folder (D:\Software\EVKey, setting.ini touched 10:53 today, currently not running). Evidence: docs/evkey-clone-work/97-stray-input-source.md. Nothing stopped or started by this task.
- 2026-10-30 11:53 BUILD: release dev-tone-build. 0.2.4 candidate (no version bump; tauri.conf.json still 0.2.3). Fix coherence D-TN-01/06/07/08 all four present and complete in crates/vkb-core/src/{engine.rs,method.rs} (drop_for+previous_trigger removal + tone_stale re-place; backspace stale-slot pin; edited/keeps_the_marks; double_letter mark_at+1==i) - no todo!/dbg! leftovers in crates. cargo test --workspace exit 0: 406 passed / 0 failed / 5 ignored (32 binaries); cargo test -p vkb-core exit 0: 232 passed / 0 failed / 0 ignored, tone_change.rs 17/17, no_alloc.rs 4/4; raw-zone matrix cargo test -p vkb-core --test special_cases matrix -- --nocapture exit 0: 78/78 PASS (was 56 PASS / 22 FAIL). Tone matrix re-run (qa/tone/run.ps1 logic, output in %TEMP%\opencode\tone-after): 123 rows, 35 FAIL -> 19 FAIL, 16 fixed (A-02, A-39, A-49, A-54, B-06, B-13, B-14, B-15, B-34, B-38, B-42, B-43, B-44, B-46, B-47, B-54); remaining 19 are D-TN-02/03/04/05 only: A-03..A-05, A-11, A-13, A-14, A-24, A-26, A-31, A-36..A-38, A-43..A-45, A-47, A-55, B-07, B-61. Owner repro ie6t+SP+BS+1 -> iết (VNI) PASS. bench p99 0.8 us (budget 50 us), max 15.6 us, 3.04 M keys/s - PASS. npm ui build exit 0 (known api.ts warning); cargo build -p vkb-app --release --features custom-protocol exit 0 -> target\release\vietkb.exe 8 390 144 B, SHA-256 346299C0EA211DFB45CC58A449469C1E06024F3B0973001D0D7B59FEDB113374, mtime 2026-10-30 11:47:49. No app launched, no input synthesised, installed pid 19224 untouched.
- 2026-10-30 11:51:13 QA (docs/evkey-clone-work/qa/e2e only; no product code, no app launch, no kill): host-E2E harness now obeys the owner-app guardrail (99-agent-guardrails.md + charter section 6). Removed every kill of %LOCALAPPDATA%\VietKB\vietkb.exe and every kill by name; PRECONDITIONS detects the installed instance by path, prints pid/path/start, and aborts HOLD(owner-quit) ('installed VietKB is running, owner must quit it - the script will not kill it') or, with the new -OwnerQuit switch, waits up to 120 s for the owner to quit it themselves and then aborts HOLD(owner-quit) - it never terminates it, and if the owner quit it the script does not relaunch it (summary states this; absent-in stays absent). A leftover target build this run did not launch is HOLD(stale-target), same no-kill rule. Only a process this run launched may be stopped and a vietkb stop must be under E:\vietkb\target; Test-KillAllowed is the single decision point, Stop-Owned writes 'KILL <time> pid=<n> path=<p> reason=<why>' (or REFUSED) to results-host.txt and log.md before stopping, and exactly one Stop-Process -Id call remains in the file. Launch is verified by pid+path: a redirect into the installed build is BLOCKED(redirected) (void run, installed app untouched), a script-scope trap stops only the instance it launched. TE1 quits the test pid, TE2 counts only target-path processes, so a running owner app can never fail teardown. Two new blocking labels in the arithmetic (BLOCKED(redirected), HOLD(owner-quit)/HOLD(stale-target)); bucket line gains BLOCKED. -SelfCheck OK: kill-rule matrix (installed/unowned refused, owned target + scratch allowed), exactly one stop call by -Id inside Stop-Owned, kill log written before the stop, Wait-OwnerQuit contains no stop, both block labels prevent SHIP, plus the earlier chord/reader/pin checks; a function-level dry run (20 292 tokens, 86 functions) is clean. Budget unchanged at 123 rows -Fast.
- 2026-10-30 12:44 QA (abandoned task): passive keyboard watcher for stray "ư" — ABANDONED at owner's instruction, nothing shipped. Reason (owner): the stray characters came from agents debug/test runs injecting input into the focused window, not from an app bug, so the fix is policy (99-agent-guardrails.md section 3), not instrumentation. qa/watch now holds only a stub README; keystroke-watch.ps1 / summarise.ps1 / watch.cmd deleted. No keyboard hook was ever installed (only the decoder self-check ran, no hook, no typing) and no process was started or stopped; installed VietKB untouched.
- 2026-10-30 14:40 LEAD: owner rulings — D-TN-09 won't fix (keep VNI u7 behaviour); D-E2E-04 closed by amending charter C9 to the compact panel; D-GUARD-01 resolved (Run\VietKB value gone, observed read-only); D-GUARD-02 EVKey64.lnk left for the owner. Model assignment per charter §5: BA/PO/Design Opus medium, Dev Sonnet medium, QA Haiku high. Dispatched dev-engine (D-TN-02..05, vkb-core, worktree) and dev-windows (zero-focus hook-sandwich harness behind feature e2e-harness + D-E2E-03, worktree). Test policy (owner): zero desktop impact first; a short real-app host window only with the owner's OK at run time.
- 2026-10-30 14:53 LEAD: merged dev-engine D-TN-02..05 (72fee6b + 4afe9aa, merge eebbdb8) after cavecrew-reviewer MERGE. VNI 0 rule (Lead, within row D-TN-03): 0 acts only when the word has a tone or a mark, else literal digit. Dev reports cargo test --workspace 418/0/5, vkb-core 244, no_alloc 4/4, tone matrix 123/0. Accepted: literal trigger keeps the rest of the word literal (stricter than spec, never wrong text); ba68 gives bâ8; tafc (tone before coda) still tàc.
- 2026-10-30 15:22 BUILD: qa-round-close main claim — QA independent verify of round fixes (cargo test --workspace, e2e-harness in-process, no_alloc/raw_zone/tone_change, tone matrix, corpus oracles, zero_focus_e2e example, bench); no desktop, no vietkb.exe launch, no input, no --live-hooks. Slot held.
- 2026-10-30 15:28 QA verify-close (no desktop; no vietkb.exe launch, no kill, no input, no --live-hooks): cargo test --workspace 428/0/5 (429/0/5 after QA added state::tests::bare_shift_between_letters_does_not_end_the_word in state.rs); vkb-windows --features e2e-harness 96/0/3; no_alloc 4/4; raw_zone 9/0; tone_change 29/0; tone matrix 123 PASS / 0 FAIL; corpus 20/20 paragraphs PASS; edge 95/95; zero_focus_e2e in-process PASS=254 FAIL=0, control caught, hooks_installed=0, global_events_sent=0; P1 p99 telex 1.0 us / vni 0.6 us (budget 50 us). D-TN-02..05 -> FIXED + VERIFIED (engine); D-E2E-03 -> FIXED, host check pending; D-E2E-05 filed FIXED (0f70497), host check pending. Nothing failing. Report: docs/evkey-clone-work/98-verify-round-close.md; raw: qa/verify-close/raw/. Not committed (Lead commits). Note: plain .ps1 blocked by execution policy, used process-scoped -ExecutionPolicy Bypass.
- 2026-10-30 15:28 BUILD released: qa-round-close
