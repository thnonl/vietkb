# Team profile: VietKB

Project facts for the global `team-workflow` skill. Read at the start of every VietKB session.

## Sessions and ownership

- **Windows Lead** (Windows session) owns shared code: `vkb-core`, `vkb-platform`, `vkb-config`,
  shared `vkb-app` and `ui`. **Mac Lead** (session "Phiên bản vietkb cho Mac") owns Mac-only
  bugs and Mac features; shared changes stay minimal and go to the Windows Lead.
- Feature branches: `<feature>` (shared code, Windows Lead), `mac-<feature>` rebased on it.
  Mac-gated Rust passes CI (the windows job) before merge.

## Owner rules

- Approvals per action for commit, push, merge, tag, release, Store submission.
- No attribution lines (no Co-Authored-By) in commits or PRs, in every session (owner, 2026-11-03).
- Run the full test set (unit, generated, real-app on every platform) before asking the owner to
  merge (owner, 2026-11-03: "chạy cho đủ test trước khi hỏi merge").
- Reply in Vietnamese, short and plain. Priorities: Vietnamese typed right first ("ưu tiên tiếng
  Việt"), low RAM ("giữ RAM thấp"), no slowdown on the key path.
- Never kill, relaunch or overwrite the owner's installed VietKB without approval; the owner's
  Notepad is off-limits; never modify system or security settings.

## Build and test

- Engine: `cargo test -p vkb-core` (includes `syllable_sweep`, `human_typing`,
  `real_app_scenarios`, `tone_overwrite`, `tone_overwrite_line`, `no_alloc`).
- Workspace: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo clippy -p vkb-app --all-targets --features store -- -D warnings`, `npm --prefix ui run check`.
- D10 English/code evaluation: harness in a scratch folder (`d10-eval/run.py`, Python venv with
  `wordfreq`, `VKB_BIN` = `target\debug\vkb.exe`); compare EN/VI/code rates and every changed row
  with the previous run.
- Rulings: `docs/evkey-clone-work/28-engine-rulings.md` (quote the owner).

## Generated QA

- `crates/vkb-core/tests/syllable_sweep.rs`: every `data/vi.txt` syllable x styles, Telex and VNI;
  `KNOWN` lists excluded keys with reasons.
- `crates/vkb-core/tests/real_app_scenarios.rs`: human scenarios (typos, tone changes, reopen,
  delete-back, fix after the sentence incl. Left + tone overwrite, fixes on other lines,
  multi-line). The model mirrors the platform: plain Left/Right/End go to the engine, every other
  caret mover and chord resets. `REAL_APP_EXPORT=<dir>` writes `human-telex.tsv`/`human-vni.tsv`.
- Words with only Vietnamese-reading checks: `tests/fixtures/d10_not_vietnamese.txt`,
  `data/real_syllables.rs`.

## Real-app QA

Windows: `docs/evkey-clone-work/qa/host-e2e/host-e2e8.ps1` (+ `QN2.cs`), sets `-Set r030 | human |
overwrite`, `-Method telex|vni`, `-KeyDelay`, `-Limit`, `-Skip pass/App`.
- Launch detached via `Start-Process powershell.exe -WindowStyle Hidden`; remove `PSModulePath`
  first (Windows PowerShell 5.1 from pwsh 7 lacks `Get-FileHash` otherwise).
- Focus thieves seen: Claude windows, `hey-look-qa` (another project's QA app), Chrome Remote
  Desktop (`remoting_desktop`). The harness minimizes Claude and `hey-look-qa` and clicks its own
  title bar after raising its window topmost.
- Windows Sandbox drops `KEYEVENTF_UNICODE`: no accented typing there.
- App quirks: VS Code loses the first line typed after it gets the focus (WARM row); VS Code
  mis-orders scripted Ctrl+Left/Left/Ctrl+Backspace (confirmed with unaccented words), so
  fix-after-sentence rows are not graded in VS Code; VS Code text is read from screenshots only
  (zoom before judging ể vs ế); PowerShell `Ctrl+End` deletes to the end of the line.

Mac (`scripts/mac-e2e`, Mac Lead):
- Read `scripts/mac-e2e/README.md` first. `run.sh` guardrails: HID idle >= 60 s, foreground pid
  checked before every key, input only into windows the harness opened (TextEdit, Notes, Stickies,
  Chrome textarea/omnibox, VS Code, Terminal), owner-file manifest before/after, guardrail log.
- Owner present and screen unlocked (front app `loginwindow` = locked); owner told VietKB pauses.
- QA app: release build `--features custom-protocol`, bundle id `vn.vietkb.app`, in
  `qa-build/VietKB.app`, re-signed with the project cert from a throwaway keychain. Never replace
  `/Applications/VietKB.app`. Owner's app: quit with `osascript quit`/SIGTERM, relaunch from the
  same path, exactly one instance. Never run the app binary outside the harness.
- Never click the menu bar by coordinate (the tray menu is not reachable by AX); open Settings via
  the second-instance -> `show_settings` path. Never script System Settings, TCC or `tccutil`.
- Known: `run.sh` `[[ -n $VKB_E2E_NO_IDLE ]]`/`VKB_E2E_CONFIG` fail under `set -u` (fix `${VAR:-}`).

Platform input rules:
- Windows `crates/vkb-windows/src/state.rs` `route`: hotkeys -> chords (reset) -> app/IME/enabled
  -> plain Left/Right/End to the engine -> other caret movers reset -> text keys.
- macOS `crates/vkb-macos` event tap never blocks (no AX, IPC or locks). Left 0x7B, Right 0x7C,
  End 0x77 and Cmd+Right -> engine VK_LEFT/VK_RIGHT/VK_END; Home, Up/Down, PgUp/PgDn, Cmd+Left,
  Option/Ctrl chords and Shift+arrows reset. A bare Shift keeps the word; an input-source switch
  resets. AppKit calls on the main thread; reveal a staged alpha-0 window early when occluded.

## Release and store

- Bump `crates/vkb-app/tauri.conf.json`, `crates/vkb-app/Cargo.toml`, `Cargo.lock`; tag `vX.Y.Z`;
  CI (`release.yml`) builds `VietKB_<v>_x64-setup.exe` and the universal dmg.
- Release notes in Vietnamese (features, fixes, downloads incl. the Store link
  https://apps.microsoft.com/store/detail/9N5TNMLCZNT5?cid=DevShareMCLPCS).
- Store: `scripts/build-store.ps1` -> `target-store\store\VietKB_<v>.0_x64.msix` after the last code
  change; Partner Center product 9N5TNMLCZNT5; the owner accepts terms; the Super hero art is
  removed so the trailer stays in the screenshots row (owner, 2026-11-03).
- macOS signing: self-signed cert CN VietKB, SHA-1 26F051435796E579C526EB1E981BFAB98210064E;
  DR `identifier "vn.vietkb.app" and certificate root = H"26f051435796e579c526eb1e981bfab98210064e"`.
  Never rotate it (TCC grants survive updates only with a stable DR). No notarization. PKCS#12 from
  OpenSSL 3+ needs `-legacy -certpbe PBE-SHA1-3DES -keypbe PBE-SHA1-3DES -macalg sha1`. After a
  release, mount the dmg and check signature, DR and `lipo -archs`.

## CI

- `ci.yml`: build, clippy, test on windows-latest ("the windows job"); macos on macos-latest.
  `gh workflow run ci.yml --ref <branch>`, `gh run watch`. The macos job is sometimes cancelled
  with no failed step: rerun once. `gh run download` can time out: retry.

## Lessons (project)

- 2026-11-02 VNI `To6i40` went raw: a code-shape rule exempted only lowercase; tone digits at a
  word's end are always Vietnamese.
- 2026-11-02 `english_stop_cluster` mid-word broke ~200 words (`bast` = `bát`); English shapes are
  decided at the word break (D10).
- 2026-11-02 Backspace over a coda that promoted `ưo` froze the word: keep the kept keys live.
- 2026-11-02 Mac: ad-hoc signing put the cdhash in the DR (Accessibility re-asked every update);
  fixed with the self-signed project cert and a CI DR check.
- 2026-11-02 Mac: a staged alpha-0 settings window stayed occluded (1.8 s blank); made opaque before
  activating, revealed early when occluded.
- 2026-11-02 Mac: Dev subagents are Sonnet 5.5 medium with a bounded test budget (an Opus Dev
  looped measurements for hours).
- 2026-11-03 Tone overwrite: the UI Automation version was dropped for its cost (Chrome/Electron
  accessibility mode, first-key delay); in-memory only (owner). End forgets the line.
- 2026-11-05 Mac: a fix that trusted the event string under Caps Lock broke real-app TX-C4/VN-C4 (apps follow the flag); ask the owner which platform a bug is on before diagnosing, and let real-app QA settle flag-vs-string guesses.
