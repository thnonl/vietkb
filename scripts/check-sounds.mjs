#!/usr/bin/env node
// scripts/check-sounds.mjs — audibility / character sanity for the `go_moc` pair.
// Read-only. Checks that ON sits above OFF (1200 vs 800 Hz), the edge/fade contract
// (first/last 2 samples zero, ≥ 2 ms fade-in, no clipped sample) and the size caps.
//   node.exe scripts/check-sounds.mjs

import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SR = 22050;
const PACK = 'go_moc';

function load(which) {
  const b = readFileSync(join(ROOT, 'assets', 'sounds', PACK, `${which}.wav`));
  const n = b.readUInt32LE(40) / 2;
  const x = new Float64Array(n);
  for (let i = 0; i < n; i++) x[i] = b.readInt16LE(44 + i * 2) / 32768;
  return { x, bytes: b.length, dataLen: b.readUInt32LE(40) };
}

function seg(x, aMs, bMs) {
  return x.slice(Math.round((SR * aMs) / 1000), Math.round((SR * bMs) / 1000));
}

function zcr(s) {
  let c = 0;
  for (let i = 1; i < s.length; i++) if (s[i - 1] <= 0 && s[i] > 0) c++;
  return c / (s.length / SR); // upward crossings per second = Hz for a sine
}

const on = load('on');
const off = load('off');

// go_moc: steady woodblock tone — ON must sit higher than OFF (1200 vs 800 Hz).
const onHz = zcr(seg(on.x, 6, 40));
const offHz = zcr(seg(off.x, 6, 40));
console.log(
  `${PACK}/on  ZCR ${onHz.toFixed(0)} Hz (expect 1200) vs off ${offHz.toFixed(0)} Hz (expect 800) ` +
  (onHz > offHz * 1.3 ? 'ON HIGHER OK' : 'WRONG'),
);
if (!(onHz > offHz * 1.3)) throw new Error('ON must be audibly higher than OFF');

// edges + fade: first/last 2 samples exactly 0, ≥ 2 ms fade-in, no clipped sample.
console.log('');
for (const which of ['on', 'off']) {
  const { x } = which === 'on' ? on : off;
  const n = x.length;
  if (44 + x.length * 2 !== (which === 'on' ? on.bytes : off.bytes)) {
    throw new Error(`${PACK}/${which}: size mismatch`);
  }
  if ((which === 'on' ? on.bytes : off.bytes) > 5300) {
    throw new Error(`${PACK}/${which}: over the 5300 B cap`);
  }
  const firstZero = Math.abs(x[0]) === 0 && Math.abs(x[1]) === 0;
  const lastZero = Math.abs(x[n - 1]) === 0 && Math.abs(x[n - 2]) === 0;
  let clipped = 0;
  for (let i = 0; i < n; i++) if (Math.abs(x[i]) >= 32767 / 32768) clipped++;
  const fi = Math.round((SR * 2) / 1000);
  let pk1 = 0, pk2 = 0;
  for (let i = 0; i < fi; i++) {
    const a = Math.abs(x[i]);
    if (i < fi / 2) pk1 = Math.max(pk1, a); else pk2 = Math.max(pk2, a);
  }
  const fadeOk = Math.abs(x[0]) === 0 && pk2 > pk1 * 1.4;
  console.log(
    `${PACK}/${which}`.padEnd(15) +
    `edges(2/2)=${firstZero && lastZero ? 'ZERO' : 'FAIL'}  ` +
    `fadeIn2ms=${fadeOk ? 'ramp' : 'NO RAMP'} (pk ${pk1.toFixed(3)}→${pk2.toFixed(3)})  ` +
    `clipped=${clipped}`,
  );
  if (!firstZero || !lastZero || !fadeOk || clipped > 0) {
    throw new Error(`${PACK}/${which}: edge/fade/clip contract violated`);
  }
}
console.log(`\n${PACK} pair: ${on.bytes + off.bytes} B (expected 8468)`);
