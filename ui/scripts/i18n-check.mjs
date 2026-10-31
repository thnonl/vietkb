// i18n consistency test (run: npm --prefix ui run test:i18n).
// Node 22.6+ strips the TypeScript types of the locale files on import.
//   1. every vi key has an en key and vice versa
//   2. no empty strings
//   3. {placeholders} match between vi and en
//   4. every literal key used in the source exists
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, "..", "src");
const { vi } = await import("../src/lib/locales/vi.ts");
const { en } = await import("../src/lib/locales/en.ts");

const errors = [];
const keys = new Set([...Object.keys(vi), ...Object.keys(en)]);
const holders = (text) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort().join(",");

for (const key of keys) {
  if (!(key in vi)) errors.push(`missing in vi: ${key}`);
  if (!(key in en)) errors.push(`missing in en: ${key}`);
  if (key in vi && !vi[key].trim()) errors.push(`empty vi: ${key}`);
  if (key in en && !en[key].trim()) errors.push(`empty en: ${key}`);
  if (key in vi && key in en && holders(vi[key]) !== holders(en[key]))
    errors.push(`placeholder mismatch: ${key}`);
}

const files = [];
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) {
      if (name !== "locales") walk(path);
    } else if (/\.(svelte|ts)$/.test(name)) files.push(path);
  }
};
walk(src);

for (const file of files) {
  const text = readFileSync(file, "utf8");
  const used = [
    ...text.matchAll(/\bt\(\s*"([a-zA-Z][\w.]*)"/g),
    ...text.matchAll(/\b\w*Key:\s*"([a-zA-Z][\w.]*)"/g),
    ...text.matchAll(/"((?:hk|hotkey|method|tone)\.[\w.]+)"/g),
  ].map((m) => m[1]);
  for (const key of used) if (!keys.has(key)) errors.push(`unknown key ${key} in ${file}`);
  for (const m of text.matchAll(/plural\(\s*"([\w.]+)"/g))
    for (const suffix of ["one", "other"])
      if (!keys.has(`${m[1]}.${suffix}`)) errors.push(`unknown plural ${m[1]}.${suffix} in ${file}`);
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`i18n ok: ${keys.size} keys, vi and en match`);
