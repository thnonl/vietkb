# 81 — P2 measurement procedure: keystroke → injected character

**Budget (charter §3, P2):** p95 ≤ 8 ms, p99 ≤ 15 ms.
**Scope:** what this document measures, how to produce a valid session, how to read
the numbers, and what counts as a fail. It is written so any stream can run it
without asking the owner.

**Status when written (2026-10-28):** the probe is implemented in `vkb-windows`,
but `vkb-app` does **not** start the Windows backend yet (`commands::HOOK_READY`
is hardcoded `false`, "the Windows backend lives in another stream this sprint").
A session started from the app therefore records **0 samples** and is invalid, not
a pass. P2 stays *unmeasured* until that stream installs the hook. §7 tells the
tester how to recognise exactly this state.

---

## 1. What the number is (and is not)

The probe in `crates/vkb-windows/src/perf.rs` records, per keystroke:

```
start  = the guard rails at the top of the WH_KEYBOARD_LL callback are past,
         i.e. the earliest point of the key we actually handle
end    = the moment the injection call returns (SendInput, one batched array)
sample = end - start, in microseconds
```

One sample per keystroke **that produced an injected character**. Keys the engine
swallows without injecting (modifiers, shortcuts, caret movers) produce no row —
they emit no character, so latency is undefined for them.

Not included, by construction:

* the OS dispatch from the driver to the hook (`start` is already inside it);
* `SendInput` → the target app's paint → the pixel on screen;
* clipboard-fallback pastes (opt-in, off by default; see charter C4).

So a passing number is *necessary but not sufficient* for a "feels instant" app,
and a failing number is unambiguous: the part this project owns is too slow.

## 2. Preconditions

* Windows 10 1809+ / 11, x64, **AC power**, power plan *High performance*.
* Display scaling **100 %**, one monitor, 1920 × 1080 or larger.
* A **release** build. The dev profile (`opt-level = 1`) is not the shipping
  binary and must never be quoted as a P2 result.

  ```powershell
  cd E:\vietkb
  cargo build --release -p vkb-app --features custom-protocol
  ```

* The binary under test must install the keyboard hook. Check §7 before typing.
* No other keyboard tool running: EVKey/UniKey, PowerToys Keyboard Manager,
  AutoHotkey scripts, RDP sessions, on-screen keyboards.
* US keyboard layout active (`0x0409`). The header line of the output records the
  layout the probe started under; a `vi-VN` header invalidates the run.

## 3. Launch with the probe on

`VKB_PERF=1` is read **once at startup** and cached in an atomic; it cannot be
switched on later. Start the app from PowerShell so the variable is inherited and
so the log has somewhere to go (the release build is a GUI-subsystem process with
no console of its own):

```powershell
$env:VKB_PERF = '1'
cd E:\vietkb
.\target\release\vietkb.exe *> "$env:TEMP\vkb-perf-A.log"
```

In the log, before touching anything, confirm the two lines:

```
INFO vkb_windows::perf: perf probe active (VKB_PERF=1) capacity=16384 build=release version=0.1.0 layout="0x0409 (en-US)"
INFO vkb_app: VietKB starting ... hook_ready=false
```

`hook_ready=false` in that second line is the normal startup value (the hook is
installed after the window exists). Confirm the hook separately — the status strip
of the settings window, or `get_status` — before typing. If the hook never becomes
ready, stop: the session would be invalid (§7).

## 4. Session A — Notepad

1. Open **Notepad** (plain `notepad.exe`, not Notepad++/VS Code).
2. Warm up for **30 s**: type any text at a natural pace (e.g. repeat
   `xin chào việt nam`), then select all and delete. This fills caches, IME and
   font paths, and gives the probe its startup outliers.
3. Type the fixed paragraph in §8 **once**, at a natural, steady pace. Do not
   stop to correct typos; do not use the mouse; do not switch windows. Corrections
   add legitimate samples, but stopping/starting makes the two apps incomparable.
4. Leave the text in the document. Do **not** save.
5. Quit with the tray item **“Thoát VietKB”** (not Task Manager, not the window's
   X — closing the window only hides it). The quit path writes the TSV and the
   summary line.

Expected volume: ~2,300 samples (2,256 characters; one sample per keystroke that
injects, and every tone mark costs an extra `Replace` injection).

## 5. Session B — Chrome

Repeat §3–§4 with these substitutions:

* close Notepad first; start Chrome fresh with **one** tab;
* open `data:text/html,<textarea style="width:100%;height:90vh"></textarea>` and
  click once inside the textarea. A blank textarea keeps Chrome's autocomplete,
  history and page scripts out of the measurement while still exercising
  Chromium's real input pipeline (the point of this session);
* same 30 s warm-up, same paragraph, same tray quit.

## 6. Compute p50/p95/p99 from the TSV

Files land in `%APPDATA%\VietKB\perf-<UTC timestamp>.tsv`. Header lines start with
`#`; every other non-empty line is one sample in microseconds, in chronological
order.

```powershell
$file = Get-ChildItem "$env:APPDATA\VietKB\perf-*.tsv" |
        Sort-Object LastWriteTime | Select-Object -Last 1

Get-Content $file.FullName | Where-Object { $_ -like '#*' }   # provenance + the app's own verdict

$us = Get-Content $file.FullName |
      Where-Object { $_ -match '^\d+$' } |
      Select-Object -Skip 20 |            # isolation rule: drop the first 20 samples
      ForEach-Object { [int]$_ } |
      Sort-Object

$n = $us.Count
function P([int]$p) { $us[[math]::Ceiling($p / 100 * $n) - 1] }

[pscustomobject]@{
  file = $file.Name
  count = $n
  p50_us = P 50
  p95_us = P 95
  p99_us = P 99
  max_us = $us[-1]
} | Format-List
```

`P` is a nearest-rank percentile: `$us` is already sorted, so index
`ceil(p/100 × n) − 1` is the smallest sample at or above that percentile.

The same numbers are already in the log, written at exit by the probe itself:

```
INFO vkb_windows::perf: P2 keystroke -> injection latency count=3120 p50_us=210 p95_us=1180 p99_us=2410 max_us=5300 dropped=0 path=... verdict=PASS
```

Use the PowerShell snippet as the authority (it applies `-Skip 20`); the log line
is the quick verdict and must agree with it. If they disagree, the run included
samples the snippet dropped — recompute and trust the snippet.

## 7. Isolation rules and invalidation

A session is **invalid** (discard the file, do not report it) when any of these is
true:

| Rule | Why |
|---|---|
| Run on AC power, High performance plan, no battery saver | CPU parking shows up as a +5–20 ms tail |
| Nothing else running: no browsers other than the one under test, no Teams/Slack/Discord, no sync client, no screen recorder, no OBS, no VPN, no Windows Update | each adds scheduling and filter-driver latency |
| No Defender full scan in progress (real-time protection stays **on** — it is the shipping condition) | scan I/O blocks the injection thread |
| 30 s warm-up before the measured paragraph | cold caches dominate the first seconds |
| Discard the first 20 samples (`-Skip 20`) | hook install, first foreground probe, font/IME caches |
| Mouse untouched during the paragraph | a click re-probes the foreground app (1.5 s TTL) |
| No debug/trace logging during the run | `RUST_LOG=debug` puts file I/O on the hook thread; the probe must measure the shipping path |
| Log and TSV on the local disk, not a network share | write stalls |
| `count ≥ 100` after the skip | fewer samples cannot support a p99 |
| Header says `build=release`, `keyboard_layout: 0x0409` | dev builds and other layouts measure a different product |

Two states that are **not** a fail, but mean the run is worthless:

* **0 data rows**, or `verdict=insufficient-samples`, or no `perf probe active`
  line → the binary never installed the hook, so nothing was recorded. This is
  the current `vkb-app` state (§0): the Windows backend is not wired into the app
  yet. Fix the wiring, then re-run; do not report a number.
* `verdict=FAIL` with `count < 100` → still invalid, but suspicious enough to
  re-run before concluding anything.

## 8. Fixed paragraph (501 syllables, 2,256 characters)

Vietnamese is counted by syllable, so this block is 501 whitespace-separated tokens — the fixed input for both apps, every run, unchanged. Changing it invalidates comparison with earlier runs.

Type exactly this, in this order, once per session.

> Buổi sáng ở Hà Nội bắt đầu rất sớm, khi mặt trời còn chưa lên khỏi những mái
> nhà thấp thoáng sau rặng cây. Tiếng rao của người bán bánh cuốn, bánh mì, phở
> vang lên trong từng ngõ nhỏ, hòa với tiếng chuông xe đạp và tiếng còi xe máy.
> Những hàng cây bên hồ vẫn còn đọng sương, vài cụ già chậm rãi tập thể dục, vài
> bạn trẻ chạy bộ dọc con đường lát gạch. Không khí se lạnh, trong lành và dễ chịu
> vô cùng.
>
> Ở phố cổ, những ngôi nhà cũ với mái ngói rêu phong nằm san sát nhau. Mỗi căn
> nhà đều có một câu chuyện riêng: có nhà làm nghề tranh dân gian, có nhà đúc
> đồng, có nhà chỉ mở một quán trà nhỏ đã mấy chục năm. Người Hà Nội quen với nhịp
> sống chậm rãi ấy, quen với việc ngồi bên vỉa hè nhâm nhi ly cà phê sữa đá và
> nhìn dòng người qua lại. Có người bảo rằng, chính sự chậm rãi ấy làm nên nét
> duyên riêng của thành phố này.
>
> Trưa đến, nắng bắt đầu gay gắt hơn. Những quán ăn nhỏ ven đường đông nghịt
> khách, mùi bún chả, mùi nem rán, mùi canh cua thoang thoảng trong gió. Ai cũng
> vội vã, nhưng ai cũng cố dành vài phút để ăn cho xong bữa trưa rồi quay lại với
> công việc. Tiếng nói cười rộn rã, tiếng bát đĩa chạm nhau lách cách, tất cả tạo
> nên một bức tranh rất đời thường mà cũng rất sống động.
>

> Tối đến, thành phố khoác lên mình tấm áo khác hẳn ban ngày. Đèn đường vàng óng,
> những quán cà phê mở nhạc nhỏ, các con phố đi bộ đông người dạo chơi. Có những
> nhóm bạn trẻ ngồi hát với nhau bên hồ, có những gia đình dắt trẻ nhỏ đi ăn kem.
> Tiếng đàn, tiếng hát, tiếng cười hòa quyện vào nhau, làm cho đêm Hà Nội vừa
> nhộn nhịp vừa bình yên.
>
> Người ta nói nhiều về sự thay đổi của thành phố: nhà cao hơn, đường rộng hơn,
> xe nhiều hơn. Nhưng có những thứ vẫn vậy, như gánh hàng rong, như quán nước chè
> dưới gốc cây, như thói quen ngồi kể cho nhau nghe những câu chuyện nhỏ trong
> ngày. Những điều giản dị ấy giữ cho thành phố một nhịp thở riêng, khó lẫn với
> bất cứ nơi nào khác.
>

> Tôi vẫn thường nghĩ về những buổi sáng ấy mỗi khi đi xa. Ở nơi khác, thành phố
> có thể hiện đại hơn, tiện nghi hơn, nhưng không có tiếng rao quen thuộc, không
> có mùi cốm, không có cái se lạnh đầu mùa. Có lẽ vì thế mà mỗi lần trở về, tôi
> lại thấy lòng mình nhẹ đi, như vừa được ai đó vỗ về sau một chặng đường dài. Có những buổi chiều tôi chỉ ngồi im nghe tiếng phố, và thấy lòng mình bình yên.

Coverage: all tone marks (`sắc huyền hỏi ngã nặng`), `ă â ê ô ơ ư đ`, upper case in
`Hà Nội`/`Việt`, punctuation, digits absent on purpose (no shift/`send_standard`
noise), and enough syllables per word to exercise `Replace` (multi-backspace)
injections, which are the slowest single injection in the batch path.

Run it **three times per app**; report all three and take the verdict from the
**worst** p95 and the **worst** p99 across the six files (conservative, and the
charter measures "this machine", not "a lucky run").

## 9. Fail criteria, and what to do

**PASS** requires, in both apps (Notepad and Chrome), worst of three runs:

* `p95_us ≤ 8000`
* `p99_us ≤ 15000`
* `count ≥ 100` after `-Skip 20`
* header `build=release`, `keyboard_layout: 0x0409`

Anything else is **FAIL** — including a pass in Notepad and a fail in Chrome. The
budget is a hard gate (charter §3): a change that breaks it is a defect, not a
tradeoff, so the number goes into `docs/evkey-clone-work/80-verify-report.md`
verbatim with the date, the run files and the machine.

If the numbers exceed the budget, in this order:

1. **Re-run once** to rule out a one-off (Defender, a background update).
   Three consecutive fails is a real finding; one is an anecdote.
2. **Check what the distribution looks like**, not just p95/p99:
   `$us | Group-Object { [int]($_ / 1000) } | Sort-Object Name` (millisecond
   buckets). A fat body means per-keystroke work (allocation, engine, injection);
   a spike at ~1.5 s periodicity means the foreground probe; a spike every 60 ms
   means the IME composition check (`IME_TTL`); a gap every 30 s means the hook
   refresh (`HOOK_REFRESH`).
3. **Rule out the environment** before touching code: another hooking tool,
   clipboard fallback on (`typing.use_clipboard` — the paste path has a fixed
   delay and is not the default path), `typing.send_standard` on, 150 % scaling,
   the exe on a network share, Defender "Controlled folder access".
4. **Bisect inside the path** with the probe itself: it already splits the path at
   `SendInput` return. If the p99 is high while `count` is right, the cost is in
   the hook→engine→build→inject sequence, not in the target app. The known
   candidates in this codebase are, in order of suspicion: one `Vec` allocated per
   injection in `input::build_segment`, `foreground::probe` (process name +
   layout) on its 1.5 s TTL, `ime::composition_active` every 60 ms, and the hook
   refresh every 30 s.
5. **Do not** relax the budget, do not average percentiles across runs, and do not
   quote a dev build. Record the fail, open a defect, keep the TSVs.

## 10. Where the probe lives (for whoever picks this up)

| Path | Role |
|---|---|
| `crates/vkb-windows/src/perf.rs` | the whole probe: flag, ring, stats, TSV + log report |
| `crates/vkb-windows/src/hook.rs` | `init_from_env()` in `install()`; the single `if perf::enabled()` at the top of `keyboard_proc` |
| `crates/vkb-windows/src/input.rs` | `perf::end()` immediately after `SendInput` returns |
| `crates/vkb-app/src/main.rs` | arm at startup, flush on tray quit and after the event loop |

Off-path cost, per keystroke, when `VKB_PERF` is unset: one atomic load
plus the branch it guards. No clock read, no lock, no allocation. When the probe
is on: two QPC reads (`Instant::now` + `elapsed`), one uncontended mutex around a `Vec` reserved
to 16,384 samples at startup, one `u32` push; overflow is dropped, never grown.
