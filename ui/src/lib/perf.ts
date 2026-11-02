// TEMPORARY open-timing probe (branch mac-open-timing, never merged).
//
// Off unless the shell injected `window.__VKB_OPEN_TIMING__ = true` (only when
// the app runs with `VKB_OPEN_TIMING=1`). Marks carry the page's own wall clock
// (`performance.timeOrigin + performance.now()`), so the Rust side can place
// them on its t0 without counting the IPC that delivers them. They are buffered
// and sent in one `perf_marks` call once the first content frame has painted,
// so the probe adds no IPC traffic while the page is loading.

type Internals = {
  invoke: (cmd: string, args?: unknown, options?: unknown) => Promise<unknown>;
};

interface Mark {
  name: string;
  epochMs: number;
  durMs: number | null;
}

const w = (typeof window !== "undefined" ? window : undefined) as
  | (Window & { __VKB_OPEN_TIMING__?: boolean; __TAURI_INTERNALS__?: Internals })
  | undefined;

export const perfOn = w?.__VKB_OPEN_TIMING__ === true;

const buffer: Mark[] = [];
let realInvoke: Internals["invoke"] | null = null;
let flushed = false;

const nowEpoch = () => performance.timeOrigin + performance.now();

export function mark(name: string, durMs: number | null = null, epochMs = nowEpoch()): void {
  if (!perfOn) return;
  buffer.push({ name, epochMs, durMs });
  if (flushed) void send();
}

/** `__TAURI_INTERNALS__.invoke` is a non-writable property, so the probe
 *  cannot wrap every IPC globally. `api.ts` routes its commands through this
 *  instead; event `listen` calls are marked around in `state.svelte.ts`. */
export function timedInvoke<F extends (cmd: string, args?: any, options?: any) => Promise<any>>(
  real: F,
): F {
  if (!perfOn) return real;
  return ((cmd: string, args?: unknown, options?: unknown) => {
    const start = nowEpoch();
    const label = `ipc:${cmd}`;
    const done = () => mark(label, nowEpoch() - start, start);
    const result = real(cmd, args, options);
    result.then(done, done);
    return result;
  }) as F;
}

function wrapInvoke(): void {
  const internals = w?.__TAURI_INTERNALS__;
  if (!perfOn || !internals || realInvoke) return;
  realInvoke = internals.invoke.bind(internals);
}

function navMarks(): void {
  const origin = performance.timeOrigin;
  mark("nav:timeOrigin", null, origin);
  const nav = performance.getEntriesByType("navigation")[0] as PerformanceNavigationTiming | undefined;
  if (nav) {
    const at = (name: string, ms: number) => {
      if (ms > 0) mark(`nav:${name}`, null, origin + ms);
    };
    at("fetchStart", nav.fetchStart);
    at("requestStart", nav.requestStart);
    at("responseStart", nav.responseStart);
    at("responseEnd", nav.responseEnd);
    at("domInteractive", nav.domInteractive);
    at("domContentLoaded", nav.domContentLoadedEventEnd);
    at("loadEventEnd", nav.loadEventEnd);
  }
  for (const entry of performance.getEntriesByType("resource") as PerformanceResourceTiming[]) {
    const file = entry.name.split("/").pop()?.split("?")[0] ?? entry.name;
    mark(`res:${file}`, entry.responseEnd - entry.startTime, origin + entry.startTime);
  }
  for (const entry of performance.getEntriesByType("paint")) {
    mark(`paint:${entry.name}`, null, origin + entry.startTime);
  }
}

async function send(): Promise<void> {
  if (!realInvoke || buffer.length === 0) return;
  const marks = buffer.splice(0, buffer.length);
  try {
    await realInvoke("perf_marks", { marks });
  } catch {
    // Probe only.
  }
}

/** Call once real content is in the DOM: marks the next frame, then the first
 *  task after it (the frame has been handed to the compositor), and flushes. */
export function markPainted(): void {
  if (!perfOn) return;
  requestAnimationFrame(() => {
    mark("raf");
    setTimeout(() => {
      mark("painted");
      navMarks();
      flushed = true;
      void send();
    }, 0);
  });
}

// First module the bundle evaluates (imported first by main.ts).
if (perfOn) {
  wrapInvoke();
  mark("script:start");
}
