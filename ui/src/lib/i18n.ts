// Pure i18n core (no runes, so a node script can import it).
//
// M6 (27-mac-decisions): the whole UI is bilingual. Strings live in
// locales/vi.ts and locales/en.ts, generated from ui/scripts/strings.py.
// Reactive language state and the global `t` are in lang.svelte.ts.

import { en } from "./locales/en";
import { vi } from "./locales/vi";

export type Lang = "vi" | "en";
export type Params = Record<string, string | number>;

const DICTS: Record<Lang, Record<string, string>> = { vi, en };

export const isLang = (value: unknown): value is Lang => value === "vi" || value === "en";

/** Replace `{name}` placeholders. */
export const format = (text: string, params?: Params): string =>
  params
    ? text.replace(/\{(\w+)\}/g, (whole, name: string) =>
        name in params ? String(params[name]) : whole,
      )
    : text;

/** Look up `key` in `lang`; unknown keys fall back to Vietnamese, then to the key. */
export const translate = (lang: Lang, key: string, params?: Params): string =>
  format(DICTS[lang][key] ?? DICTS.vi[key] ?? key, params);

/** Translator factory. */
export const translator =
  (lang: Lang) =>
  (key: string, params?: Params): string =>
    translate(lang, key, params);
