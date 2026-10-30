# 22 — Sound spec: audible toggle, one embedded pair

**Owner (verbatim, 2026-10-30):** "Tôi chỉ muốn có âm thanh khi bật tắt bộ gõ, tôi không yêu cầu thêm
config hay UI mới nào cả."

**Ruling.** The feature is: **the on/off toggle is audible, and ON differs from OFF.** Nothing else is
added — no `[system]` key, no picker, no preview command, no new UI row. This file now specifies only that
reduced feature; the 6-pack + picker design that was built earlier and then cut lives in §9 so its
measurements and audition are not lost and nobody re-adds it by accident.

**Code today.** `crates/vkb-windows/src/sound.rs`: `play_toggle(enabled)` → `plan()` → `Plan::Memory(bytes)`
executed through a private `Player` seam (`Winmm` in production, a stub in tests). Callers: hotkey path
(`lib.rs`), UI switch (`crates/vkb-app/src/commands.rs::toggle_enabled`), tray (`tray.rs`),
`Injection::Beep` → `sound::play_error()` (`state.rs`, `backend.rs`).

## 1. Asset — `go_moc` on/off, the only two files
`assets/sounds/go_moc/on.wav` · `assets/sounds/go_moc/off.wav`. Woodblock: woody attack, sine 1200 Hz on /
800 Hz off, dead by 100 ms, peak −18 dBFS — audible on laptop speakers with headphones off, too short to
fatigue at a few hundred toggles a day.

| | on | off |
|---|---|---|
| tone | sine 1200 Hz + 2nd harmonic −12 dB | sine 800 Hz + 2nd harmonic −14 dB |
| envelope | 2 ms linear attack, `τ` 20 ms decay | 2 ms linear attack, `τ` 18 ms decay |
| duration | 100 ms | 90 ms |
| peak | −18 dBFS | −18 dBFS |

ON and OFF are a **pair**: within 2 dB of each other, same timbre, different pitch. The toggle must never
sound the same in both directions (that was defect `D-SND-03`).

## 2. Format — hard numbers
- Container: RIFF/WAVE, exactly two chunks — `fmt ` (16 B) then `data`; no `LIST`/`fact`/`cue`; 44 B header;
  fmt tag `1`, channels `1`, **22050 Hz**, **16-bit**, block align `2`, byte rate `44100`, `data` even.
- Size: `raw = 44 + 2 × round(22.05 × ms)` → **on 4 454 B**, **off 4 014 B**, pair **8 468 B**.
- Level: peak per §1 ±1 dB, **hard ceiling −6 dBFS**, no clipped sample; block mean subtracted
  (`|mean| ≤ 4 LSB`); fade-in ≥ 2 ms, fade-out 4 ms linear; first and last 2 samples exactly `0`.
- Forbidden: dither, resampling (incl. 44.1 kHz source), MP3/OGG/ADPCM — `SND_MEMORY` takes PCM WAV only.
- Any duration or rate change must be re-derived through the formula above.

## 3. Embedding — `include_bytes!` only
`include_bytes!("../../../assets/sounds/go_moc/on.wav")` / `…/off.wav` in `vkb-windows` → `&'static [u8]`.
No `std::fs`, no `PathBuf`, no install-dir lookup, no user-supplied WAV. A missing or renamed asset is a
**compile error**, never a runtime silence — so there is no "missing file" failure mode to test. Cost:
**8 468 B** in the exe/installer (0.08 % of the 10 485 760 B P7 ceiling).

## 4. Playback — one path, never on the hook thread
```
PlaySoundW(bytes, SND_MEMORY | SND_ASYNC | SND_NODEFAULT)
```
- `SND_MEMORY` plays the `'static` buffer; no alias, no `SND_FILENAME`, no `system` sound scheme — the toggle
  sounds on a machine whose scheme is "No Sounds".
- `SND_ASYNC` returns immediately; the call runs on the existing spawned audio thread, **never on the hook
  thread** and never on the webview/IPC thread.
- No `SND_LOOP`, no `SND_NOSTOP`: a new toggle interrupts the previous sound (a fast ON→OFF must not queue).
- `SND_NODEFAULT` keeps "nothing" as the fallback instead of a random system default.
- Failure (no device, muted endpoint, refused buffer) = **silence**: `let _ =` the `BOOL`, never panic, never
  block, never retry; at most one `debug` line per session, never once per keystroke.
- A garbled WAV is caught at build/test time (§7), not at runtime.

## 5. ON/OFF symmetry — every toggle route, both directions
| Route | ON plays | OFF plays |
|---|---|---|
| hotkey (`Alt+Shift`, `F1`/`F2`) | `on.wav` | `off.wav` |
| UI switch in the main window | `on.wav` | `off.wav` |
| tray `Bật/Tắt tiếng Việt` | `on.wav` | `off.wav` |

Each route passes the **real resulting state**, never a hard-coded `true` (the `lib.rs`/`backend.rs`
`play_toggle(true)` asymmetry was the `D-SND-03` regression). No route may double-play on one toggle.

`Injection::Beep` is **not** a toggle: it plays `sound::play_error()` (the `SystemHand` off alias,
`SND_ALIAS|SND_ASYNC|SND_NODEFAULT`, no pack, no fallback), so an invalid keystroke signals *error* and never
"Vietnamese on". Deliberately asymmetric; silent when the scheme has no sounds, which is accepted because it
is a secondary cue. This is a defect fix (the invalid-keystroke signal was wrong), not a new feature.

## 6. Config and UI — nothing new
- `system.notification_sound` (label `Âm thanh khi bật/tắt`, `Advanced.svelte`) stays **the** master switch,
  unchanged name and meaning: `false` ⇒ no sound on any route.
- No `sound_pack`, no `SoundPack`, no `scheme_silent`, no `preview_sound`, no `soundPack`/`schemeSilent` in
  `SettingsView`, no new row, no `Nghe thử` button. `Nâng cao` stays **560 × 388** (the 424 growth that
  existed only for the picker row reverts).

## 7. Acceptance criteria (machine-checkable)
1. Exactly two assets, `go_moc/{on,off}.wav`, each `RIFF`/`WAVE`, fmt tag 1, ch 1, 22050 Hz, 16-bit, block
   align 2, even `data` — `vkb-windows::sound::tests::packs_are_pcm16_22k_mono`.
2. `data` length equals §2 ±1 sample; each peak within ±1 dB of §1 and the pair within 2 dB —
   `sound::tests::durations_and_peaks_match_spec`.
3. Peak ≤ −6 dBFS, `|mean| ≤ 4 LSB`, first/last 2 samples `0`, fade-in ≥ 2 ms —
   `sound::tests::levels_dc_offset_and_fades`.
4. Only `go_moc` is embedded; no other pack directory is referenced by any `include_bytes!`; no `std::fs` /
   `PathBuf` on the sound path; `vietkb.exe` delta stays under the 120 KiB cap (measured, not assumed).
5. `cargo test -p vkb-windows -p vkb-config` green; hook latency unaffected (P1 `process_key` p99 inside its
   50 µs budget).
6. **Nothing re-added:** `[system]` has no `sound_pack`/`scheme_silent`; `SettingsView.system` has no
   `soundPack`/`schemeSilent`; no `preview_sound` command exists — grep + `cargo test -p vkb-config`.
7. **Symmetry:** for each of the three toggle routes, OFF requests `off.wav` and ON requests `on.wav` (stub
   `Player` seam records the bytes); `Injection::Beep` requests the `SystemHand` alias and never a WAV.
8. **Audible in the real app** (QA, owner listens — not a code reading): master switch on, toggle via hotkey,
   UI switch and tray; ON is audibly higher than OFF, on this machine (scheme `.None`), twice in a row, and
   no distinguishable gap before the sound.
9. **Failure is silent, not fatal:** with the endpoint muted/removed, toggling 20 times produces no panic, no
   error dialog, no keystroke loss, at most one `debug` line.
10. **No new UI:** `Nâng cao` → `Hệ thống` shows the master switch and no sound row; window 560 × 388, no
    clipped row, no scrollbar (screenshot in `design/_shots/`).

## 8. Non-goals (do not add without a new owner ask)
Extra packs, pack picker/`Select`, preview/`Nghe thử`/`preview_sound`, `sound_pack` or any config key,
`schemeSilent` probe and its hint, volume slider, per-event sounds (macro commit, charset switch),
user-supplied WAV files, MP3/OGG, 44.1 kHz, macOS playback, a dedicated audio thread or channel, changes to
`Beep` beyond routing it to `play_error()`.

## 9. History — the 6-pack + picker design, built and then cut (owner ruling 2026-10-30)

**What was built and cut.** Between 2026-10-30 01:06 and 01:29 the team built a seven-choice sound system:
six synthesized WAV packs plus a `system` (Windows Scheme) choice, `system.sound_pack` persisted in
`%APPDATA%\VietKB\config.toml`, a read-only `scheme_silent` probe with a Vietnamese advisory, a full-width
`Kiểu âm thanh` row in `Nâng cao` → `Hệ thống` with a `Select` and `Bật`/`Tắt` preview buttons driven by a
new `preview_sound(pack, enabled: Option<bool>)` command, and the `Nâng cao` window grown 560 × 388 → 560 ×
424 to hold that row. The owner had already heard all six packs in chat (A2c audition page was cancelled —
chat only, `design/sound-audition.html` never existed) and picked **`go_moc`** as the default. Then the owner
cut the whole surface with the line at the top of this file: only "make the toggle audible" was requested, so
the picker, the config key and the preview had **no requester**, and every extra surface is dead weight in
the panel (C9) and in the exe (P7). Everything in the following list was removed; the reason is one line:
*no requester, so no surface.*

- Removed: `[system] sound_pack` and the `SoundPack` enum (`crates/vkb-config/src/schema.rs`),
  `schemeSilent`/`scheme_silent` (Rust view + `ui/src/lib/bindings.ts`), the `preview_sound` command and
  `api.previewSound`, `soundPack` in `SettingsView`/TS, the `Kiểu âm thanh` picker row + hints + `Nghe thử`
  buttons (`ui/src/screens/Advanced.svelte`), the `Nâng cao` height 424 (`crates/vkb-app/src/main.rs:109`,
  back to 388), packs 2–6 assets and their embedded bytes, and the build artifacts `design/sound-data.json`
  + `design/sound-preview.txt` (`scripts/gen-sounds.mjs` keeps `go_moc` only, or is dropped).
- Not lost — the measurement table that justified the cut-to-one default (`raw = 44 + round` per §2 rule):

  | pack | on / off ms | pair raw B | peak | character |
  |---|---|---|---|---|
  | `go_moc` | 100 / 90 | 8 468 | −18 dBFS | woodblock, kept as the ship asset |
  | `click` | 70 / 65 | 6 042 | −18 dBFS | dry filtered click |
  | `hai_not` | 118 / 118 | 10 496 | −16 dBFS | C5→E5 rising on, E5→C5 off |
  | `gio_nuoc` | 110 / 105 | 9 570 | −20 dBFS | soft "plop", quietest |
  | `co_khi` | 110 / 100 | 9 350 | −16 dBFS | camera shutter, two bursts |
  | `eight_bit` | 105 / 95 | 8 908 | −20 dBFS | chiptune square arpeggio |

  The six packs together were 52 834 B (0.5 % of P7); `go_moc` alone is 8 468 B.
- **Removed with the scope cut — do not re-add without an owner ask.** The 15 shipped copy strings (7 labels,
  7 hints, 1 advisory), from the Dev UI report:

  | # | value | label | hint |
  |---|---|---|---|
  | 1 | `go_moc` | `Cổ điển` | `Gõ mộc, dứt khoát, tắt sau 100 ms` |
  | 2 | `click` | `Click gọn` | `Click khô và gọn, không ngân` |
  | 3 | `hai_not` | `Hai nốt` | `Hai nốt: lên khi bật, xuống khi tắt` |
  | 4 | `gio_nuoc` | `Giọt nước` | `Giọt nước mềm, nhỏ nhất bộ` |
  | 5 | `co_khi` | `Cơ học` | `Màn trập máy ảnh, hai tiếng tách` |
  | 6 | `eight_bit` | `8-bit` | `Chiptune vuông, ba nốt lên xuống` |
  | 7 | `system` | `Theo Windows` | `Theo bộ âm thanh Windows của bạn` |

  Advisory: `Windows đang để chế độ không âm thanh · sẽ phát Cổ điển`. These were Dev defaults still awaiting
  Copy review when the surface was cut, so if a picker ever returns the wording work starts over rather than
  resuming. Do not resurrect the strings in `Advanced.svelte`, `ui/` copy tables, or tests.
- Also settled there and still true: `C:\Windows\Media\*.wav` are Microsoft-copyright assets (never ship
  them, hence `system` borrowed the user's scheme); the Windows scheme on this machine is `.None`, so an
  alias-only design is silent by construction — which is why the shipped path is `SND_MEMORY`, not an alias.
