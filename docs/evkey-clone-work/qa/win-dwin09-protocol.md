# D-WIN-09 measurement protocol (injection strategies)

For the Windows Lead. Goal: find which way of handing a `Replace { deletes, text }` to
`SendInput` makes Win11 Notepad type the VNI macro `qa1` -> `QA ONE` every time (today 25-50%,
identical emitted sequence for hits and misses; WinForms 20/20). Related: D-E2E-01.

## Build and enable

* Build the **dev channel**: `cargo build --release -p vkb-app --features dev-channel` (the
  feature forwards to `vkb-windows/dev-channel`). A build without it never reads the variable.
* Set the variable in the shell that launches the dev exe, then start it (it is read once, at
  the first replacement, so restart the app to change strategy):

  ```powershell
  $env:VKB_INJECT_STRATEGY = "per-char"   # one of the names below
  $env:VKB_DEBUG_KEYS = "1"               # strategy + timings in keys.log
  .\VietKB-dev.exe
  ```
* Unset or unparsable variable = the shipped behaviour (`keys.log`: `inject-strategy IGNORED`).
* Applies only to a `Replace` in the default Unicode mode on the hook path. `send_standard`,
  plain typing, clipboard mode and the alternate `Injector` sink are unchanged.

## Strategies

| value | what is sent |
|---|---|
| `single` | control: Backspaces + text in ONE call (0.2.6 behaviour) |
| `split` | current main: Backspace call, then text call (text keeps Shift + VK) |
| `split-delay-N` (N = 1..50) | as `split`, text call >= N ms later, from a dedicated injector thread (see risks) |
| `per-char` | Backspace call, then each character (Shift-wrapped press / packet pair) as its own call |
| `scancode-shift` | as `split`; Shift = `VK_LSHIFT` + scan 0x2A + `KEYEVENTF_SCANCODE`; letters VK + scan |
| `unicode-after-split` | Backspace call, then the whole text (capitals too) as `KEYEVENTF_UNICODE` packets |
| `capslock-free` | as `split`; Shift = plain `VK_LSHIFT` + scan 0x2A (no `VK_SHIFT`, no scancode flag) |

Suggested order: `single` (confirms the baseline fails), `split` (confirms current main),
`per-char`, `split-delay-5`, `split-delay-15`, `split-delay-40`, `scancode-shift`,
`capslock-free`, `unicode-after-split`.

## Per-strategy run (Notepad, Win11, one fresh window, VNI on, typing 30-40 ms/char)

Clear the document between rows. Record correct/total.

1. `qa1` + Space, macro `qa1` -> `QA ONE`, expected `QA ONE ` -- x20: 10 immediately after a
   Telex -> VNI switch (each of the 10: switch to Telex, switch to VNI, type), 10 steady state.
2. `ba6t5/ta8t1` x10, expected `bật/tắt`.
3. `VIEETJ` x10 (VNI all-caps `VIỆT`: type `VIE6T5` in VNI; use the Telex form only if the
   current method is Telex), expected `VIỆT`.
4. `dduwowcj` x10 (Telex), expected `được`.
5. The sentence row from `95-e2e-report.md` (mixed words with tones, one pass), x3.

Pass bar: 20/20 for row 1 and 0 wrong in rows 2-5 for a strategy to be a candidate.

## Sanity (candidate strategies only)

Type rows 1, 2 and 5 once (x5 each) in **WordPad**, in a **Chrome `<textarea>`**, and in a
WinForms probe (the 20/20 reference). A strategy that fixes Notepad but breaks any of these is
rejected.

## What to collect

* Result counts per strategy/row (table: strategy x row -> ok/total, plus the wrong strings).
* `keys.log` for one failing and one passing run of the best strategy. Lines to look at:
  * `inject strategy=.. deletes=.. total=..ev sent=a/b total_us=.. [Nev @us +us sent=..] ..` per injection:
    per-call offsets and durations.
  * `inject-async ..` (delay strategies): when the delayed call actually went out and its result.
  * `inject-wait Nus` / `inject-wait TIMEOUT`: a physical key held back by a pending delayed call.
    TIMEOUT means the wait cap (150 ms) was hit; ordering was then not guaranteed.
* Whether `sent=` is ever short (`a<b`): that would also trigger the app step-down.

## Risks and notes

* `split-delay-N` runs the text call on a dedicated thread. A physical key that arrives while
  it is pending waits (<= 150 ms) at the top of the hook callback, so typing slower than N ms
  is unaffected but fast typing adds latency of up to N ms per replacement. Not a ship
  candidate as is; it only answers "does a gap help".
* A short send under a strategy is not retried or Shift-repaired (measurement build): treat
  any `sent=a/b` with `a<b` as a failed run and report it.
* The strategy replaces the whole delivery of a `Replace`, including after a `restore_word`
  chord (chord release rides with the Backspace call; Shift re-press with the last text call).
* Unit tests for the INPUT grouping are in `crates/vkb-windows/src/input_dev.rs` (Windows-only
  crate: `cargo test -p vkb-windows` on Windows) and for the name parser in
  `crates/vkb-platform/src/inject_strategy.rs`. They were compile-checked only, not run, for
  the Windows crate.
