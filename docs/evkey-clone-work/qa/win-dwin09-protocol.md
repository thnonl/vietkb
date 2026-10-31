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

### Notepad spell-check / autocorrect strategies (D-WIN-09, D-WIN-11)

Measured 2026-10-31: Win11 Notepad's spell-check TSF service and autocorrect rewrite the word right
after our Backspace + replace burst (macro `qa1` -> `QA ONE` loses case or is garbled; Telex `gox`
-> `go`). Plain human typing is untouched. These strategies change *what* is sent. The acceptance
target is Notepad with **Spelling: Spell check ON and Autocorrect ON** (matrix A of the "Notepad
comparison" block in `60-defects.md`).

| value | what is sent |
|---|---|
| `select-replace` | no Backspaces: `Shift`(`VK_LSHIFT`, scan 0x2A) down, `Left`(extended) x N, `Shift` up (selection), then the text in a second call (VK path, capitals Shift + VK, as `split`) |
| `select-replace-unicode` | the same selection, then the text as `KEYEVENTF_UNICODE` packets in a second call |
| `word-retype` | delete the whole current word and type the whole new word in ONE call: the unchanged head the engine normally leaves alone (`keep_unchanged_head`) is deleted and retyped too |
| `word-retype-split` | as `word-retype`, with the Backspace block as its own call |
| `unicode-after-split-delay-N` (N = 1..50) | Backspace call, >= N ms (injector thread), then the text as `KEYEVENTF_UNICODE` packets |
| `boundary-hold-N` (N = 1..120) | as `split`, but the boundary key (space, tab, Return, punctuation) that follows our text is forwarded >= N ms after it (see below) |

`boundary-hold-N` has three parts: (1) a boundary character that ends our own text (macro
`qa1` -> `QA ONE ` ends with the Space) is split off and sent N ms after the rest; (2) a physical
boundary key (the Space after a word-final edit like `gox` + Space) that arrives sooner than N ms
after our last injection waits at the top of the hook callback; (3) a boundary key the same event
lets through after our text (macro committed with Enter/Tab) is held by sleeping N ms before the
callback returns. `keys.log`: `boundary-hold before|after <us>`, `inject-async ... delay_ms=N`.

Select strategies fall back to the plain `split` (trace `fallback=split`) when a selection is not
safe, so they never select across a line start or grapheme: the engine's word is always on one line
(Enter, caret keys and mouse clicks end it), the selection is used only when N <= the chars of that
word; also required: no combining mark or non-BMP char in the word (Shift+Left moves by grapheme,
Backspace by code point), the user holds no Shift, the text is not empty (an empty text must delete),
and the injection comes from a key the engine just processed (not the `restore_word` chord path,
which has no word context and keeps Backspaces).

Suggested order: `single` (confirms the baseline fails), `split` (confirms current main),
`per-char`, `split-delay-5`, `split-delay-15`, `split-delay-40`, `scancode-shift`,
`capslock-free`, `unicode-after-split`; then, for the Notepad spell-check/autocorrect problem,
`select-replace`, `select-replace-unicode`, `word-retype`, `word-retype-split`,
`unicode-after-split-delay-15`, `unicode-after-split-delay-40`, `boundary-hold-30`,
`boundary-hold-80`.

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

## Acceptance matrix: Notepad, Spell check ON + Autocorrect ON

Fresh Notepad window, Win11 Settings in Notepad: Spelling -> Spell check On, Autocorrect On
(restore the owner's setting afterwards), VietKB dev build with the strategy, typing 30-40 ms/char.
Record ok/total and the wrong strings. Pass bar per row: all correct. Baseline (no strategy,
`6d5fc40`, matrix A): R1 9/20, R2 0/5, R3 0/5, R4 5/5.

| row | input | expected | runs |
|---|---|---|---|
| R1 | VNI macro `qa1` + Space | `QA ONE ` | x20 (10 right after a Telex -> VNI switch, 10 steady) |
| R2 | Telex `gox` + Space | `gõ ` | x5 |
| R3 | the sentence "Tôi đang gõ tiếng Việt" (Telex, one pass) | unchanged sentence | x5 |
| R4 | Telex `dduwowcj` | `được` | x5 |
| R5 | VNI `ba6t5/ta8t1` (and the same as `ta8t1` first) | `bật/tắt` | x10 |
| R6 | VNI `VIE6T5` (all caps, Shift per capital) | `VIỆT` | x10 |

Also run the control (no VietKB, autocorrect ON): typed `go` and `QA ONE` unchanged.

Sanity for a candidate that passes R1-R6: **WordPad**, **Chrome `<textarea>`**, **VS Code** editor
(rows R1-R3 and R5, x5 each), and the **WinForms probe**.

Select strategies behave differently where a selection is not a plain text range, so also run
these for `select-replace` and `select-replace-unicode` (R2, R5 and a macro, x5 each):

* **Excel**, cell in edit mode (F2) and in the formula bar: Shift+Left selects characters in edit
  mode; check that nothing is selected across cells or the cell is not committed.
* **Windows Terminal** (PowerShell prompt): Shift+Left may extend a terminal/mark selection or be
  ignored by the shell line editor (PSReadLine does select, cmd.exe does not). Expect failures; the
  point is to know which prompts break. Also check the text typed after is not lost.

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

Per new strategy (measurement build only; none is on by default):

* `select-replace(-unicode)`: Shift+Left is *caret movement*, not deletion. Apps that ignore it
  (Windows Terminal/cmd, many games, some custom controls, Excel in non-edit mode) turn the burst into
  a duplicated word; apps that reinterpret Shift+Left (word processors with "extend selection" modes,
  RTL text where Left is visually right) select the wrong chars. Typing over a selection is one undo
  step in some apps and two in others. The injected Shift is tagged, but a *physical* Shift pressed
  during the burst changes the result (the strategy refuses when the user already holds Shift).
  Notepad may also run the spell check on the selection change.
* `word-retype`: retypes the head, so capitals in the head use Shift + VK again (D-E2E-06: an
  injected Shift pair around a capital was lost in Notepad); the burst is longer (more flicker, more
  chance of a short send, undo steps grow). `word-retype` in ONE call re-creates the 0.2.6 grouping
  that D-WIN-09 blamed, so `word-retype-split` is the more likely candidate.
* `unicode-after-split-delay-N`: needs the injector thread (see `split-delay-N`); packets after
  Backspaces collapse in Notepad unless the gap is long enough (D-E2E-01), so this may only trade
  one failure for another.
* `boundary-hold-N`: sleeps in the hook callback (<= 120 ms, below the 150 ms wait cap and the OS
  low-level-hook timeout): fast typists see up to N ms of extra latency on a boundary key after an
  edit, and the keys after it are queued behind it. It does nothing if the text service rewrites the
  word *before* the boundary arrives.

* `split-delay-N` runs the text call on a dedicated thread. A physical key that arrives while
  it is pending waits (<= 150 ms) at the top of the hook callback, so typing slower than N ms
  is unaffected but fast typing adds latency of up to N ms per replacement. Not a ship
  candidate as is; it only answers "does a gap help".
* A short send under a strategy is not retried or Shift-repaired (measurement build): treat
  any `sent=a/b` with `a<b` as a failed run and report it.
* The strategy replaces the whole delivery of a `Replace`, including after a `restore_word`
  chord (chord release rides with the Backspace call; Shift re-press with the last text call).
* `select-replace*` and `word-retype*` use the word context built in `state.rs::process`
  (`input_dev::Context`); an injection it does not describe (restore chord path) is planned with no
  context: select strategies degrade to `split`, `word-retype*` retype nothing extra.
* Unit tests for the INPUT grouping are in `crates/vkb-windows/src/input_dev.rs` (Windows-only
  crate: `cargo test -p vkb-windows` on Windows) and for the name parser in
  `crates/vkb-platform/src/inject_strategy.rs`. They were compile-checked only, not run, for
  the Windows crate.

## Mandatory foreground check (added 2026-10-31 after an incident)

During the D-WIN-01 run at aa5e71f, F1 opened Edge help and the QA driver kept sending Telex keys into the owner's Edge/Bing search box ("get help with notepad in windows âưĐâưââââư"). Rule for every run of this protocol:

- Check the foreground HWND **before every single key** (not per batch) and abort the run immediately if it is not a window the agent itself created/opened for the test. Never "re-focus and continue" automatically.
- Never send a key that can open another app or browser (F1, Win, browser keys) in a run that types text afterwards, unless the step is designed for it and the foreground is re-verified.
- After an abort, report which window got focus and what was sent; do not retry until the owner is told.
