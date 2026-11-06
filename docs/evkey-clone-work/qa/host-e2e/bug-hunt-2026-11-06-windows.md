# Bug hunt 2026-11-06: Windows and shared code (Windows Lead)

Owner (2026-11-06): "Tôi không muốn có những bug liên quan đến việc gõ cơ bản như vậy diễn ra nữa. Hãy cho team workflow review lại code và tìm những điểm có thể xảy ra bug, verify những nghi ngờ bằng cách test thật với ở cả Windows mà Mac."

Head b7161a3 (0.3.4). Candidate `target\release\vietkb.exe` built from head with `--features custom-protocol`, SHA-256 95CB8519B97ED094…, 8 774 144 B.
Mac half: `docs/evkey-clone-work/qa/mac/bug-hunt-2026-11-06.md` (Mac Lead).

## Real-app run (host-e2e8 `-Set hunt`, Chrome textarea + Windows Terminal, secondary screen)

Rows: `hunt-telex.tsv`, `hunt-vni.tsv`. Results: `results8-hunt-telex.tsv`, `results8-hunt-vni.tsv`.
The harness gained a `{WAIT:ms}` token (no input, lets the 1.5 s Caps Lock re-read run).

| row | keys | expected | Chrome | Terminal |
|---|---|---|---|---|
| TX HC1 | Caps on, wait 2.5 s, `vieejt`, wait, `dduwowngf` | VIỆT ĐƯỜNG | PASS | PASS |
| TX HC2 | Caps on/off, wait 2.5 s, `vieejt` `dduwowngf` | việt đường | PASS | PASS |
| TX HC3 | Caps on, Shift held `vieejt`, then `Tieengs` | việt tIẾNG | FAIL `tIEENGS` | FAIL `tIEENGS` |
| TX HC4 | Caps toggled thrice, waits 1.7 s | VIỆT NĂM việt | PASS | PASS |
| TX HE1 | `tuowwn` | tuown | FAIL `tươn` | FAIL `tươn` |
| TX HE2 | `thuoww` | thuow | FAIL `thuơ` | FAIL `thuơ` |
| VNI HC1/HC2 | as above | | PASS | PASS |
| VNI HC4 | | VIỆT NĂM việt | PASS | NOT RUN (abort) |
| VNI HE1 | `muo77` | muo7 | FAIL `mươ` | NOT RUN (abort) |

The VNI Terminal pass aborted on a physical mouse move (guard worked). Teardown clean: config and Run key identical, clipboard restored, owner VietKB relaunched from `%LOCALAPPDATA%\VietKB\vietkb.exe`, lock released.

The Caps Lock re-read from 60b706a holds in real apps: no revert after the 1.5 s poll (W5 poll-revert refuted).

## Findings

| id | sev | area | finding | evidence | status |
|---|---|---|---|---|---|
| E1 | P1 | vkb-core `method.rs:402-422` (Telex), `612-629` (VNI) | A second horn key on an open or `qu` `uơ` is swallowed or horns `u` instead of escaping: `tuowwn` = `tươn`, `thuoww` = `thuơ`, `quoiwwf` = `quời`, VNI `muo77` = `mươ`. D3 escape says the doubled key gives the raw keys. | engine CLI + real app (Chrome, Terminal) | CONFIRMED |
| HC3 | P2, needs ruling | vkb-core `engine.rs:591-615` (D10 code shape) | Caps Lock on + Shift on the first letter gives `tIEENGS`, `nAMF`, `vIEJT` raw: a capital after a lowercase is read as `iPhone`/`getValue`. EVKey/Unikey users get `tIẾNG`. The platform knows Caps Lock is on, so the capital is not deliberate. | engine CLI + real app | CONFIRMED (behaviour), ruling needed |
| E2 | P2 | vkb-core `spell.rs:264-283` | The spell check ignores marks: `daaw` = `dă`, `dauw` = `daư`, `coaiw` = `coăi`, `ddeeuw` = `đêư` instead of raw (D5). | engine CLI | CONFIRMED (unit) |
| W2 | P2 | vkb-windows `keys.rs:153-166` | `drop_released` only clears modifiers. A Ctrl/Alt key-down missed while the hook is stalled or re-installed (every 30 s, `hook.rs:46`) is never repaired: Ctrl+S then types a tone and the injected Backspaces run as Ctrl+Backspace. | scratch unit test | CONFIRMED (unit), real NOT RUN (needs a hook stall) |
| W1 | P2 | vkb-windows `input.rs:304-316` | A Replace with a capital sends a generic VK_SHIFT down/up when the shadow says Shift is up; if the shadow missed the user's Shift, the user's Shift is released. | scratch unit test | CONFIRMED (unit), real NOT RUN |
| W3 | P3 | vkb-windows `state.rs:535-553`, `1044-1053` | An input-language switch (Alt+Shift, Win+Space) does not end the word; macOS does. | scratch unit test | CONFIRMED (unit), real NOT RUN (only en-US installed) |
| W4 | P3 | vkb-windows `state.rs:1141`, `keys.rs:245` | Characters come from a US map whatever the layout (AZERTY, Dvorak); the doc promises a `GetKeyboardLayout == 0x0409` probe that does not exist. | code + unit | CONFIRMED (unit), real NOT RUN (installing a layout changes system settings) |
| E4 | P3 | vkb-core `engine.rs:2061-2069` | A chord clears the line but not the word; masked today because both platforms reset the engine on a chord. | engine run | latent |
| W6 | P3 | vkb-windows `state.rs:959-966` | Two transient `SendInput` = 0 in a row disable Vietnamese for that app until the focus moves. | code only | NOT VERIFIED |
| W7 | P3 | vkb-windows | Sticky Keys latched Shift is invisible to the shadow; rewritten text loses its capital. | code only | NOT VERIFIED |
| E3 | question | vkb-core `method.rs:449-460` | `aaw` = `ă` but `oow` stays raw. | engine CLI | owner question |

Checked clean: planner vs plain text apply (6 000 random Telex/VNI streams, zero divergence), all 4 048 `vi.txt` syllables, mixed case `TIEENGS`/`Vieejt`, `Đ` cases, Backspace and retype across a space, overflow, macros, own-injection tagging, mouse click and Alt/Ctrl/Win+Tab mid-word, Win+L stuck Win.

## Fix run (branch `typing-hunt`, owner: "Sửa và push")

Fixed: E1 (`method.rs`, Telex and VNI), W2 (`keys.rs` `os_pressed`/`adopt_pressed`, `state.rs` `resync_with_os`: missed Ctrl/Alt/Win/Shift presses adopted at the 1.5 s re-read; a newly found Ctrl/Alt/Win makes the key a chord), W1 (`state.rs` `held()`: an injection reads the OS Shift when the stream has none). Unit tests added for each. Gates: `cargo test --workspace` (debug and release), clippy `-D warnings` workspace and `--features store`: all green.

Candidate SHA-256 6783D81B…, 8 774 656 B. Real app, Chrome + Terminal on the secondary screen:
- hunt Telex: all PASS except HC3 (open ruling); HE1/HE2 now `tuown` / `thuow`.
- hunt VNI: all PASS (HE1 `muo7`).
- human Telex (first 12 scenarios): all PASS in Chrome; Terminal PASS on the single-line rows.
- human VNI: aborted on a physical mouse click (guard), NOT RUN.
Teardown clean each time; owner VietKB relaunched, lock released.

## Round 2 (owner: "Fix luôn bug đó đi, đây là round fix bug nên có bug là phải fix")

Fixed on `typing-hunt` ba53569, CI run 42176941872 green (6/6 jobs):
- Owner bug: with `restore_wrong_word` on, VNI `vi22` + space + BS BS BS + `2` gave `vi2`; now `vì` (Telex `viff`, `tooo`; VNI `to66`, `ba11` the same). Sweep test `reopen_escaped_tail.rs`: 46 cases, 22 failed before the fix.
- HC3 (ruling recorded): `KeyInput::caps_lock`; Caps Lock + Shift `Tieengs` = `tIẾNG`. Test `caps_lock_shift.rs`.
- E2: vowel-mark spell check (`daư`, `coăi`, `đêư` stay raw; `giới`, `giữ`, `hươu`, `quân` unchanged); open `ă` at the word end restores with `restore_wrong_word`.
- W3: input-language switch ends the word (layout re-read after a modifier-only chord). W4: non-letter keys on a non-US layout use `ToUnicodeEx` (flag 4) with that layout.
- D10 eval run-033 vs run-032: EN 10k 97.46 -> 97.49 %, VI 97.59 -> 97.61 %, code 99.5 % unchanged; every changed row an improvement except `phêrô` (wrong before and after).

Real app, candidate SHA-256 ED3A3956…: human Telex 23/23 PASS (Chrome + Terminal), human VNI Chrome 14/14 PASS. The hunt sets and VNI Terminal aborted (the owner moved the mouse, then a game took the foreground): NOT RUN, to rerun when the desktop is free. W3/W4 real-app NOT RUN (only en-US installed; adding a layout changes system settings).

Round 3 rerun (same candidate ED3A3956…, 2026-11-06 13:56-14:09): hunt Telex 23/23 PASS in Chrome and Terminal, incl. HC3 `tIẾNG`, HR1 `viff` + space + BS×3 + `f` = `vì`, HR2 `tô`, HS1/HS2 raw, HE1/HE2. hunt VNI and human VNI Terminal aborted again (a physical click, then Torchlight Infinite in front): NOT RUN on Windows. Mac real app on ba53569 ran the VNI rows (vi22 = vì, to66 = tô, ba11 = bá, muo77 = muo7): PASS.

Round 4, VNI only (same candidate, 14:39-14:50): hunt VNI 17/19 PASS in Chrome and Terminal: HR1 `vi22` + space + BS×3 + `2` = `vì`, HR2 `tô`, HR3 `bá`, HE1 `muo7`, Caps rows. The two HC3 FAILs are a fixture fault: the row held Shift over `vie65t`, so the digits typed `^%` (`vie^%t`), as on any keyboard; the `tIẾNG` half passed in both apps. Row corrected to `{CAPS_ON}Tie6ng1{CAPS_OFF}` = `tIẾNG`. human VNI 23/23 PASS (Chrome 14, Terminal 9). Edge took the focus once and the harness recovered.
