// Reactive UI language. `t` reads `langState.lang`, so any template or
// `$derived` that calls it re-renders when the language switches.

import { isLang, translate, type Lang, type Params } from "./i18n";

const KEY = "vkb:lang";

// The saved language is only known after `get_settings` answers. Remember the
// last one locally so an English user does not see a Vietnamese first paint.
const remembered = (): Lang => {
  try {
    const value = localStorage.getItem(KEY);
    if (isLang(value)) return value;
  } catch {
    // Storage unavailable.
  }
  return "vi";
};

class LangState {
  lang = $state<Lang>(remembered());

  set(next: Lang): void {
    if (this.lang !== next) this.lang = next;
    if (typeof document !== "undefined") document.documentElement.lang = next;
    try {
      localStorage.setItem(KEY, next);
    } catch {
      // Storage unavailable.
    }
  }
}

export const langState = new LangState();

export const t = (key: string, params?: Params): string => translate(langState.lang, key, params);

/** Number + unit noun, singular/plural by key suffix `.one` / `.other`. */
export const plural = (base: string, n: number): string =>
  t(`${base}.${n === 1 ? "one" : "other"}`);
