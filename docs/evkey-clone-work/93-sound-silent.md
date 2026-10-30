# 93 — Defect diagnosis: toggle sound silent on the owner's machine

The 6-pack/picker fix proposed in §8 was cut on 2026-10-30 (owner: sound only, no config/UI); the fallback-to-`go_moc` conclusion still holds and is now the shipped behaviour.

**QA/diagnoser:** subagent · **Date:** 2026-10-30 · **Scope:** diagnosis only, no product code edited.
**Owner symptom (verbatim):** "Tôi chọn âm thanh 7. Nhưng hiện tại đang không nghe tiếng đó khi thử
bật/tắt bộ gõ."

Pack 7 in `22-sound-spec.md` §1 = `system` ("Theo Windows") = Windows sound-scheme aliases — the only
pack the shipped code implements. So the report reduces to: **the `system` pack is silent here.**

## Verdict in one paragraph

The audio stack works and the running binary contains the current sound code, but the Windows sound
scheme on this machine is **`.None` = "No Sounds"**. With that scheme every AppEvents event has an
empty `.Current` value, so `PlaySoundW(alias, …|SND_ALIAS|SND_NODEFAULT)` returns `FALSE` for
`SystemAsterisk` and `SystemHand` and the toggle is silent **by design** (`sound.rs:9` documents
exactly that). Independently, packs 1–6 do not exist in the shipped build at all (no
`system.sound_pack` key, no UI picker, no embedded assets), so no other pack choice could have made a
sound either.

---

## 1. Config

Config-dir function: `crates/vkb-config/src/paths.rs:16` `config_dir()` = `dirs::config_dir()/APP_DIR`,
`APP_DIR = "VietKB"` (production) / `"VietKB Dev"` (`dev-channel`), `CONFIG_NAME = "config.toml"`.
⇒ live file **`C:\Users\namth\AppData\Roaming\VietKB\config.toml`** (`%APPDATA%` =
`C:\Users\namth\AppData\Roaming`).

| key | value | note |
|---|---|---|
| `system.notification_sound` | `true` | gating passes; sound is *requested* |
| `input.enabled` | `true` | engine is on |
| `hotkeys.toggle` | `Alt+Shift` (`toggle_mode = "release"`) | owner's toggle chord |
| `hotkeys.enable` / `disable` | `F1` / `F2` | second sound route |
| file readable | yes | mtime `2026-10-30 00:57:31` |
| `system.sound_pack` (or any pack key) | **absent** | no such field exists in `vkb-config` |
| dev-channel file | `C:\Users\namth\AppData\Roaming\VietKB Dev\config.toml`, also `notification_sound = true` | not the one the running exe uses (no `dev-channel` in the installed path) |

Command: `Get-Content "$env:APPDATA\VietKB\config.toml"` (full dump read; macros `m01..m12` and the
`[typing]` block truncated here). `Select-String notification_sound` over `crates/**/*.rs` shows the
only sound config surface is that one boolean (`schema.rs:277`, `commands.rs:552`, `state.rs:174`).

## 2. Sound scheme — the active scheme has no sounds

| query | observed |
|---|---|
| `HKCU:\AppEvents\Schemes` `(default)` | **`.None`** (`packagebase = C:\WINDOWS\media`) |
| `HKCU:\AppEvents\Schemes\Names\*` | only `.Default` = `@mmres.dll,-800` ("Windows Default") and **`.None` = `@mmres.dll,-801` ("No Sounds")** |
| `HKCU:\SOFTWARE\...\AppEvents\Schemes\Apps\.Default\*` | 60 event keys exist (`SystemAsterisk`, `SystemHand`, `SystemNotification`, `SystemExclamation`, `SystemQuestion`, `MenuCommand`, `MenuPopup`, `Notification.*`, …); **no `DefaultBeep` key** |
| `HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppEvents\Schemes\Apps\.Default` | does not exist (all scheme data is per-user) |

Per-event values, `HKCU:\AppEvents\Schemes\Apps\.Default\<event>\.Default` vs `.Current`:

| event | `.Default` (scheme definition) | `.Current` (active) | WAV exists on disk |
|---|---|---|---|
| SystemAsterisk | `C:\WINDOWS\media\Windows Background.wav` | **empty** | yes |
| SystemHand | `C:\WINDOWS\media\Windows Foreground.wav` | **empty** | yes |
| SystemNotification | `…\Windows Background.wav` | **empty** | yes |
| SystemExclamation | `…\Windows Background.wav` | **empty** | yes |
| SystemQuestion | empty | empty | — |
| MenuCommand | empty | empty | — |
| MenuPopup | empty | empty | — |
| DefaultBeep | (key absent) | — | — |

So **0 of 8 aliases resolves to a WAV under the active scheme**; 4 of 8 *would* resolve if the scheme
were "Windows Default", and all of their target WAVs exist (`C:\Windows\Media` holds 70 `.wav` files).

## 3. Does `PlaySoundW` accept the aliases here?

Throwaway harness (PowerShell `Add-Type`, `winmm.dll!PlaySoundW`, `CharSet.Unicode`,
`Marshal.GetLastWin32Error`), flags passed explicitly. Mounted once, never looped.

`SND_ALIAS|SND_SYNC|SND_NODEFAULT` (`0x10000`):

| alias | returned BOOL | GetLastError |
|---|---|---|
| SystemAsterisk | **False** | 87 |
| SystemHand | **False** | 87 |
| SystemNotification | **False** | 87 |
| SystemExclamation | **False** | 87 |
| SystemQuestion | **False** | 87 |
| MenuCommand | **False** | 87 |
| MenuPopup | **False** | 87 |
| `.Default` | **False** | 87 |
| SystemDefault | **False** | 87 |
| DefaultBeep | **False** | 2 |

`SND_ALIAS|SND_SYNC` **without** `SND_NODEFAULT` — the documented fallback to the system default sound —
also returns **False** (err 87), i.e. the fallback target is itself muted by the `.None` scheme.

`SND_FILENAME|SND_SYNC` (proof the audio stack is fine):

| file | BOOL |
|---|---|
| `C:\Windows\Media\Windows Background.wav` | **True** |
| `C:\Windows\Media\Windows Foreground.wav` | **True** |
| `C:\Windows\Media\Windows Ding.wav` | True |
| `C:\Windows\Media\notify.wav` | True |
| `C:\Windows\Media\chimes.wav` | True |

Verdict: alias resolution fails, file playback succeeds. This is an alias/registry problem, not a
device, driver or app problem. `err 87` = `ERROR_INVALID_PARAMETER`, which winmm returns for an
unresolvable alias; the exact code is not load-bearing — the `FALSE` return is.

## 4. Audio stack / session

- Filename playback returns `TRUE` synchronously (table above): the render stack accepts and plays
  audio, and the machine's default endpoint is live.
- 23 render endpoints in `HKLM:\SOFTWARE\...\MMDevices\Audio\Render`, 7 at `DeviceState =
  536870916`/`268435457` (active/plugged) including `Speakers`, `Headphones`, `Realtek HD Audio 2nd
  output`, `NVIDIA Output`.
- **Per-endpoint volume and mute were NOT measured.** They live in each endpoint's `FxProperties`
  blobs and in the default-endpoint `PolicyConfig` store; reading them requires `IAudioEndpointVolume`
  (COM), which the "no extra tooling" constraint excluded. Stated plainly: not measurable with the
  registry alone, so this note does **not** claim the mixer is unmuted. It does not need to be — the
  failing call never reaches the mixer, because no alias resolved to a file.
- The app's own `winmm.dll` and `AudioSes.dll` are loaded in pid 19580, so the process can play audio.

## 5. Which binary is the owner running?

| item | value |
|---|---|
| running process | pid 19580, `vietkb`, path **`C:\Users\namth\AppData\Local\VietKB\vietkb.exe`** |
| started | `2026-10-29 23:58:24` (61 min at measurement time), responding, window `vn.vietkb.app-siw` |
| installed exe | size `8 338 432`, mtime `2026-10-29 22:40:32`, version **`0.2.3`**, SHA-256 `2F896F862B7313A09C679E0AD39472BCA6ED3868811FD51A151401A54800612E` |
| `E:\vietkb\target\release\vietkb.exe` | size `8 370 688`, mtime `2026-10-29 20:23:12`, version `0.2.2`, SHA-256 `87AD893C…ADB3` — **different binary**, not what is running |
| `crates/vkb-windows/src/sound.rs` mtime | `2026-10-29 20:19:27` |
| string probe (Latin-1 over the whole file, aliases are UTF-8 literals) | installed `0.2.3`: `SystemAsterisk=YES`, `SystemHand=YES`, `PlaySoundW=YES`; `SystemNotification/SystemExclamation/DefaultBeep=no`; `target\release` `0.2.2` identical |

⇒ **The running exe does contain the current `sound.rs` code** (both alias literals plus the
`PlaySoundW` import; page-1 UTF-16 probe was a false negative — Rust `&str` literals are UTF-8).
The `sound.rs` mtime is older than both binaries, so no "stale binary" explanation. Pack assets are
**not** in either binary: `go_moc/gio_nuoc/eight_bit/hai_not/co_khi/sound_pack` all absent.

## 6. Reachability per route

Gate: `sound_on` ← `state.rs:174` `self.sound_on = cfg.system.notification_sound` inside
`apply_config` (runs on every config publish) ⇒ **true** here. `Injection::Beep` (`state.rs:504`)
also gated by it.

| route | call chain | plays? | alias |
|---|---|---|---|
| Hotkey `Alt+Shift` (`toggle_mode = "release"`) | `state.rs:461` `Hotkey::Toggle` → `set_enabled(!effective)` → `state.rs:417` `if sound_on` → `sound::play_toggle(enabled)` | **reaches play_toggle, silenced at winmm** | ON `SystemAsterisk` / OFF `SystemHand` (correct symmetry) |
| Hotkey `F1` / `F2` | `state.rs:459/460` `Enable`/`Disable` → same | same | `SystemAsterisk` / `SystemHand` |
| UI switch (`Âm thanh khi bật/tắt` / status toggle) | `commands.rs:696 toggle_enabled` → `commands.rs:719 play_notification` → `lib.rs:97 play_notification_sound` → `sound::play_toggle(true)` | **reaches play_toggle**, silenced | **`SystemAsterisk` for both directions** (asymmetry) |
| Tray menu `Bật/tắt tiếng Việt` | `tray.rs:315` → `commands::toggle_enabled` | same as UI | `SystemAsterisk` only |
| Config change applied to the hook thread | `state.rs:705 sync_global_toggle` updates `last_global` only | **no sound call** | — |
| `Injection::Beep` (other backend path) | `backend.rs:56` `play_toggle(true)` | silenced | `SystemAsterisk` only |

No route is dead in Rust: every one of them calls `play_toggle` with `sound_on = true`. The break is
one layer below Rust.

## 7. Root cause, ranked

1. **Windows active sound scheme is `No Sounds` (`.None`).** Evidence: `HKCU:\AppEvents\Schemes`
   `(default) = .None`; every `Apps\.Default\<event>\.Current` value is empty; all 8 aliases return
   `FALSE` from `PlaySoundW`; the same call with a plain `.wav` filename returns `TRUE`.
   `SND_NODEFAULT` (`sound.rs:18`) turns that into deliberate silence with no user-visible reason.
   This is the whole symptom for pack 7.
2. **No pack other than `system` exists in the shipped build** — so "choose another pack" cannot be a
   workaround. `22-sound-spec.md` §1 defines 7 choices and `assets/sounds/**` holds all 12 WAVs, but
   `vkb-config` has no `sound_pack` field (`schema.rs` grep: only `notification_sound`), no
   `soundPack`/`sound_pack` symbol exists in `ui/src`, and no pack bytes are linked into either exe.
   Pack 7 is therefore not a *choice* the owner could have made in this build; it is the only
   behaviour, and it is the one that is silent here.
3. **Direction asymmetry** (`lib.rs:97` `play_toggle(true)`) — masked while silent, will surface as a
   new bug the moment sound works: turning Vietnamese **off** from the UI/tray plays the **on** sound.
4. **Failure is unobservable** — `sound.rs:15` discards the `BOOL` (`let _ =`) and there is no log
   line, so nothing in the app says "the scheme has no such alias".

## 8. Fix options (no code written; cost/risk for the Lead to pick)

| # | option | cost | risk |
|---|---|---|---|
| **A** | **Ship a real WAV for `system` too**: when the scheme alias fails (`PlaySoundW` → `FALSE`), fall back to playing an embedded `assets/sounds/go_moc/{on,off}.wav` with `SND_FILENAME\|SND_ASYNC`. | **S** — one `play_toggle` body change + 2 `include_bytes!`; assets already exist and are already spec-measured; `play_toggle` keeps its signature, so all 4 call sites are untouched. | Low. Silences the "follow my scheme" promise for users on `No Sounds`, but those users get silence today, so strictly better. Needs the `BOOL` to be captured rather than discarded. |
| **B** | **Fix the machine, not the code**: set the Windows scheme to "Windows Default" (Control Panel → Sound → Sounds → `Windows Default`), or set the two events explicitly. | **XS** for the owner (a 15-second Control Panel change); **0** code. | None for this machine, but the defect stays for every user whose machine is on `No Sounds` — and `No Sounds` is a common deliberate choice on Windows 11. Also QA cannot verify a pack-7 feature on a machine whose scheme makes it silent. |
| **C** | **Land the pack pipeline the spec already defines** (`system.sound_pack` in `vkb-config`, `include_bytes!` of the 6 packs, UI picker with `1…7`, use pack WAVs via `SND_FILENAME`; keep `system` as alias-only with `A`'s fallback): fixes the root cause *and* the "packs 1–6 do not exist" gap in one change. | **M/L** — config schema + default (`go_moc`), `sound.rs` rewrite, `commands.rs` `system.soundPack` patch path, `bindings.ts`/`Settings`, `Advanced.svelte` picker, plus tests. This is `91-round-plan-sound-cases.md` step **A3**, which has not landed. | Medium: touches the config schema (migration/`version` field) and the settings contract; needs the one build slot and a re-verify pass. |
| **D** (any of the above) | **Make the failure visible**: log at `debug` when `PlaySoundW` returns `FALSE` with the alias name. | **XS** — 3 lines. | None. Turns the next "silent toggle" report into a one-line diagnosis. |

Recommended sequence for the Lead: **B now** (unblocks the owner's next smoke test tonight, zero
code, zero build slot), then **C** (which subsumes **A** and **D**), with **D** cheap enough to land
inside **C**.

## 9. What was not measured / limits

- Endpoint volume and mixer mute: not measurable without COM (`IAudioEndpointVolume`) — see §4.
- Only `HKCU` scheme data was read; `HKLM:\...\AppEvents\Schemes\Apps\.Default` does not exist on this
  machine, so there is no machine-wide scheme to cross-check.
- `MessageBeep(MB_OK)` was attempted as a second signal but the `Add-Type` wrapper returned no value;
  it is redundant given the alias table, and it was not retried (no loop).
- The installed `0.2.3` exe cannot be tied to a git commit from the filesystem; it is tied to
  `sound.rs` only through the alias/`PlaySoundW` string probe in §5.
- No `cargo build`, no app start/stop, no config write was performed. Registry reads only; 11
  synchronous `PlaySoundW` calls total (5 of which played a real WAV), no repeats.
