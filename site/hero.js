(() => {
  const root = document.querySelector('.telex-demo');
  if (!root) return;
  const raw = root.querySelector('.telex-raw');
  const pause = root.querySelector('button');
  const reduceQuery = matchMedia('(prefers-reduced-motion: reduce)');
  const forced = new URLSearchParams(location.search).get('reduce') === '1';
  const freeze = new URLSearchParams(location.search).has('t');
  // What VietKB shows after each key of "Vieetj": the word changes as you type.
  const STEPS = ['V', 'Vi', 'Vie', 'Viê', 'Viêt', 'Việt'];
  const STEP = 0.32;              // one keystroke per 320 ms
  const RESET_AT = STEPS.length * STEP + 1.6;
  const duration = RESET_AT + 0.5;
  let reduced = reduceQuery.matches || forced;
  let paused = false, raf = 0, start = performance.now();
  const finalState = () => { raw.textContent = STEPS[STEPS.length - 1]; };
  const render = (seconds) => {
    if (reduced || paused) { finalState(); return; }
    const t = ((seconds % duration) + duration) % duration;
    raw.textContent = t >= RESET_AT ? '' : STEPS[Math.min(STEPS.length - 1, Math.floor(t / STEP))];
  };
  const tick = (now) => {
    if (reduced || paused || freeze) return;
    render((now - start) / 1000);
    raf = requestAnimationFrame(tick);
  };
  pause.addEventListener('click', () => {
    paused = !paused;
    pause.setAttribute('aria-pressed', String(paused));
    pause.textContent = paused ? '▶' : 'Ⅱ';
    if (paused) { cancelAnimationFrame(raf); finalState(); }
    else { start = performance.now(); raf = requestAnimationFrame(tick); }
  });
  const motionChange = () => {
    reduced = reduceQuery.matches || forced;
    cancelAnimationFrame(raf);
    if (reduced) finalState();
    else if (!paused && !freeze) { start = performance.now(); raf = requestAnimationFrame(tick); }
  };
  // The hero clip is decorative motion: stop it when motion is reduced.
  const heroVideo = document.querySelector('.hero-video');
  if (heroVideo) {
    const syncVideo = () => {
      if (reduceQuery.matches || forced) {
        heroVideo.removeAttribute('autoplay');
        heroVideo.pause();
      } else {
        heroVideo.play().catch(() => {});
      }
    };
    syncVideo();
    reduceQuery.addEventListener('change', syncVideo);
  }
  reduceQuery.addEventListener('change', motionChange);
  if (freeze) render(Number(new URLSearchParams(location.search).get('t')) || 0);
  else if (reduced) finalState();
  else raf = requestAnimationFrame(tick);
})();


// Types the first headline line the way VietKB does it: each Telex key updates the
// word in place (o + x becomes õ, e + e becomes ê), and the letter that changed flashes.
(() => {
  const root = document.documentElement;
  const live = document.querySelector('.type-live');
  if (!live) return;
  const params = new URLSearchParams(location.search);
  if (matchMedia('(prefers-reduced-motion: reduce)').matches || params.has('t') || params.get('reduce') === '1') {
    root.classList.remove('type-pending');
    return;
  }
  const KEYS = 'Gox tieengs Vieetj';
  // What the line shows after each key in KEYS.
  const SHOWN = ['G', 'Go', 'Gõ', 'Gõ ', 'Gõ t', 'Gõ ti', 'Gõ tie', 'Gõ tiê', 'Gõ tiên', 'Gõ tiêng', 'Gõ tiếng',
    'Gõ tiếng ', 'Gõ tiếng V', 'Gõ tiếng Vi', 'Gõ tiếng Vie', 'Gõ tiếng Viê', 'Gõ tiếng Viêt', 'Gõ tiếng Việt'];
  const caret = document.createElement('span');
  caret.className = 'type-caret';
  const draw = (i) => {
    const text = SHOWN[i], prev = i ? SHOWN[i - 1] : '';
    let at = -1;
    if (text.length === prev.length) at = [...text].findIndex((c, k) => c !== prev[k]);
    live.textContent = '';
    if (at < 0) live.append(text);
    else {
      const flash = document.createElement('span');
      flash.className = 'type-flash';
      flash.textContent = text[at];
      live.append(text.slice(0, at), flash, text.slice(at + 1));
    }
    live.append(caret);
  };
  root.classList.remove('type-pending');
  root.classList.add('type-run');
  live.append(caret);
  let i = 0;
  const step = () => {
    draw(i);
    i += 1;
    if (i < SHOWN.length) setTimeout(step, KEYS[i] === ' ' || KEYS[i - 1] === ' ' ? 280 : 140);
    else setTimeout(() => root.classList.add('type-done'), 400);
  };
  setTimeout(step, 600);
})();

// Compact menu on phones and tablets: a button opens the link panel under the header.
// Escape or a link click closes it. On wide screens the CSS shows the links in the header row.
(() => {
  const btn = document.querySelector('.menu-btn');
  const nav = document.getElementById('site-menu');
  if (!btn || !nav) return;
  const set = (open) => {
    nav.classList.toggle('is-open', open);
    btn.setAttribute('aria-expanded', String(open));
  };
  btn.addEventListener('click', () => set(!nav.classList.contains('is-open')));
  nav.addEventListener('click', (e) => { if (e.target.closest('a')) set(false); });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && nav.classList.contains('is-open')) { set(false); btn.focus(); }
  });
  document.addEventListener('click', (e) => {
    if (!nav.contains(e.target) && !btn.contains(e.target)) set(false);
  });
})();
