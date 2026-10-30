#!/usr/bin/env node
// scripts/gen-sounds.mjs — deterministic generator for the toggle sound pair.
//
// Contract: docs/evkey-clone-work/22-sound-spec.md (§1 asset, §2 format, §3 embedding).
// Only `go_moc` ships — one embedded pair, no config key, no picker. Numbers come from
// the spec; this script only implements them. No dependencies, idempotent: same bytes
// on every run.
//
//   node.exe scripts/gen-sounds.mjs
//
// Writes assets/sounds/go_moc/{on,off}.wav (2 files, 8 468 B), then re-reads both from
// disk and prints the measured table, failing if any cap is exceeded.

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SR = 22050;

const msToSamples = (ms) => Math.round((SR * ms) / 1000);
const dbToLin = (db) => Math.pow(10, db / 20);

// ---------------------------------------------------------------- generator (§1/§2)

// linear 2 ms attack then exp(-t/tau) decay.
function hitEnv(tLocal, tauMs, attackMs) {
  const at = attackMs / 1000;
  let e = Math.exp(-tLocal / (tauMs / 1000));
  if (tLocal < at) e *= tLocal / at;
  return e;
}

// §1 go_moc: sine f0 + 2nd harmonic at harmDb, linear attack, exp decay.
function goMoc(ms, f0, harmDb, tauMs, attackMs = 2) {
  const n = msToSamples(ms);
  const out = new Float64Array(n);
  const a2 = dbToLin(harmDb);
  for (let i = 0; i < n; i++) {
    const t = i / SR;
    const env = hitEnv(t, tauMs, attackMs);
    out[i] = env * (Math.sin(2 * Math.PI * f0 * t) + a2 * Math.sin(2 * Math.PI * 2 * f0 * t));
  }
  return out;
}

// ---------------------------------------------------------------- post / quantise

function finish(buf, peakDb) {
  const n = buf.length;
  const fi = Math.max(2, msToSamples(2));   // fade-in >= 2 ms (§2)
  const fo = msToSamples(4);                // fade-out 4 ms (§2)
  for (let i = 0; i < fi; i++) buf[i] *= i / fi;
  for (let i = 0; i < fo; i++) buf[n - 1 - i] *= i / fo;

  let mean = 0;
  for (let i = 0; i < n; i++) mean += buf[i];
  mean /= n;
  for (let i = 0; i < n; i++) buf[i] -= mean;   // DC removal (§2)

  let peak = 0;
  for (let i = 0; i < n; i++) peak = Math.max(peak, Math.abs(buf[i]));
  const target = dbToLin(peakDb);
  const g = peak > 0 ? target / peak : 0;

  const pcm = new Int16Array(n);
  for (let i = 0; i < n; i++) {
    const v = Math.round(buf[i] * g * 32767);
    if (v > 32767 || v < -32768) throw new Error('clipping — normaliser bug');
    pcm[i] = v;
  }
  pcm[0] = 0; pcm[1] = 0;                       // first/last 2 samples exactly 0 (§2)
  pcm[n - 1] = 0; pcm[n - 2] = 0;
  return pcm;
}

function wavBytes(pcm) {
  const dataLen = pcm.length * 2;
  const b = Buffer.alloc(44 + dataLen);
  b.write('RIFF', 0, 'ascii');
  b.writeUInt32LE(36 + dataLen, 4);
  b.write('WAVE', 8, 'ascii');
  b.write('fmt ', 12, 'ascii');
  b.writeUInt32LE(16, 16);
  b.writeUInt16LE(1, 20);          // PCM
  b.writeUInt16LE(1, 22);          // mono
  b.writeUInt32LE(SR, 24);
  b.writeUInt32LE(SR * 2, 28);     // byte rate
  b.writeUInt16LE(2, 32);          // block align
  b.writeUInt16LE(16, 34);         // bits
  b.write('data', 36, 'ascii');
  b.writeUInt32LE(dataLen, 40);
  for (let i = 0; i < pcm.length; i++) b.writeInt16LE(pcm[i], 44 + i * 2);
  return b;
}

// ---------------------------------------------------------------- the single pair (§1)

const PACK = {
  id: 'go_moc',
  name_vi: 'Cổ điển',
  character_vi: 'woodblock: woody attack, 1200 Hz on / 800 Hz off, dead by 100 ms',
  peak: -18,
  on: { ms: 100, build: () => goMoc(100, 1200, -12, 20) },
  off: { ms: 90, build: () => goMoc(90, 800, -14, 18) },
};

// ---------------------------------------------------------------- write

const soundRoot = join(ROOT, 'assets', 'sounds');
const dir = join(soundRoot, PACK.id);
mkdirSync(dir, { recursive: true });

const written = [];
for (const which of ['on', 'off']) {
  const spec = PACK[which];
  const pcm = finish(spec.build(), PACK.peak);
  const bytes = wavBytes(pcm);
  const expected = 44 + 2 * msToSamples(spec.ms);
  if (bytes.length !== expected) {
    throw new Error(`${PACK.id}/${which}: ${bytes.length} B, spec formula ${expected} B`);
  }
  const path = join(dir, `${which}.wav`);
  writeFileSync(path, bytes);
  written.push({ which, path });
}

// ---------------------------------------------------------------- measurement / caps (§2)

function measure(path) {
  const b = readFileSync(path);
  const riff = b.toString('ascii', 0, 4);
  const wave = b.toString('ascii', 8, 12);
  const fmt = b.toString('ascii', 12, 16);
  const tag = b.readUInt16LE(20);
  const ch = b.readUInt16LE(22);
  const sr = b.readUInt32LE(24);
  const byteRate = b.readUInt32LE(28);
  const align = b.readUInt16LE(32);
  const bits = b.readUInt16LE(34);
  const dataLen = b.readUInt32LE(40);
  if (riff !== 'RIFF' || wave !== 'WAVE' || fmt !== 'fmt ' || b.toString('ascii', 36, 40) !== 'data') {
    throw new Error(`${path}: bad chunk layout`);
  }
  if (b.length !== 44 + dataLen) throw new Error(`${path}: size mismatch`);
  if (tag !== 1 || ch !== 1 || sr !== SR || bits !== 16 || align !== 2 || byteRate !== SR * 2) {
    throw new Error(`${path}: bad fmt`);
  }
  const n = dataLen / 2;
  let peak = 0;
  let sum = 0;
  for (let i = 0; i < n; i++) {
    const s = b.readInt16LE(44 + i * 2);
    peak = Math.max(peak, Math.abs(s));
    sum += s;
  }
  const mean = sum / n;
  const peakDb = 20 * Math.log10(peak / 32768);
  return { bytes: b.length, sr, bits, ch, durMs: (n / SR) * 1000, peakDb, mean };
}

const pad = (s, w) => String(s).padStart(w);
const padE = (s, w) => String(s).padEnd(w);
console.log(
  padE('file', 24) + pad('bytes', 7) + pad('sr', 7) + pad('bits', 5) + pad('ch', 4) +
  pad('dur ms', 8) + pad('peak dBFS', 11) + pad('DC LSB', 9),
);
let total = 0;
for (const w of written) {
  const m = measure(w.path);
  total += m.bytes;
  const name = `${PACK.id}/${w.which}.wav`;
  console.log(
    padE(name, 24) + pad(m.bytes, 7) + pad(m.sr, 7) + pad(m.bits, 5) + pad(m.ch, 4) +
    pad(m.durMs.toFixed(1), 8) + pad(m.peakDb.toFixed(2), 11) + pad(m.mean.toFixed(2), 9),
  );
  if (m.bytes > 5300) throw new Error(`${name}: ${m.bytes} B over the 5300 B cap`);
  if (m.peakDb > -6) throw new Error(`${name}: peak above -6 dBFS`);
  if (Math.abs(m.mean) > 4) throw new Error(`${name}: DC offset ${m.mean} LSB`);
}
console.log(`TOTAL raw ${total} B (expected 8468, cap 5300/file)`);
if (total > 60000) throw new Error('group cap exceeded');
