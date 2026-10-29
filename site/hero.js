(() => {
  const root = document.querySelector('.telex-demo');
  if (!root) return;
  const raw = root.querySelector('.telex-raw');
  const pause = root.querySelector('button');
  const reduceQuery = matchMedia('(prefers-reduced-motion: reduce)');
  const forced = new URLSearchParams(location.search).get('reduce') === '1';
  const freeze = new URLSearchParams(location.search).has('t');
  const CHARS = 'tieengs';
  const STEP = 0.22;              // one keystroke per 220 ms
  const TYPE_END = CHARS.length * STEP;
  const FINAL_AT = 2.0;
  const RESET_AT = 3.5;
  const duration = 4.2;
  let reduced = reduceQuery.matches || forced;
  let paused = false, raf = 0, start = performance.now(), previous = null;
  const finalState = () => { root.dataset.state = 'final'; raw.textContent = ''; };
  const render = (seconds) => {
    if (reduced || paused) { finalState(); return; }
    const t = ((seconds % duration) + duration) % duration;
    if (t >= RESET_AT) {
      root.dataset.state = 'reset';
      raw.textContent = '';
    } else if (t >= FINAL_AT) {
      root.dataset.state = 'final';
      raw.textContent = '';
    } else if (t >= TYPE_END) {
      root.dataset.state = 'typing';
      raw.textContent = CHARS;
    } else {
      root.dataset.state = 'typing';
      raw.textContent = CHARS.slice(0, Math.floor(t / STEP) + 1);
    }
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

