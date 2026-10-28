// Live preview approximator for the overview screen.
//
// IMPORTANT: this is NOT the engine. `vkb-core` owns Telex/VNI semantics
// (owned by another stream in Sprint 1). This small renderer exists so the
// overview field can show something truthful while the real engine lands, and
// its limits are stated in the UI. It handles: aa/ee/oo doubling, aw/ow/uw,
// dd, the five tone keys (s f r x j), z to strip, and modern/old tone
// placement. It intentionally keeps EVKey's behaviour of interpreting a lone
// tone key ("test" → "tét") because that is what the app will do too.

const TONE_KEYS = "sfrxjz";

type ToneName = "acute" | "grave" | "hook" | "tilde" | "dot";

const TONE_FOR_KEY: Record<string, ToneName> = {
  s: "acute",
  f: "grave",
  r: "hook",
  x: "tilde",
  j: "dot",
};

/** base letter -> tone name -> composed letter (lowercase). */
const TONE_MAP: Record<string, Partial<Record<ToneName, string>>> = {
  a: { acute: "á", grave: "à", hook: "ả", tilde: "ã", dot: "ạ" },
  ă: { acute: "ắ", grave: "ằ", hook: "ẳ", tilde: "ẵ", dot: "ặ" },
  â: { acute: "ấ", grave: "ầ", hook: "ẩ", tilde: "ẫ", dot: "ậ" },
  e: { acute: "é", grave: "è", hook: "ẻ", tilde: "ẽ", dot: "ẹ" },
  ê: { acute: "ế", grave: "ề", hook: "ể", tilde: "ễ", dot: "ệ" },
  i: { acute: "í", grave: "ì", hook: "ỉ", tilde: "ĩ", dot: "ị" },
  o: { acute: "ó", grave: "ò", hook: "ỏ", tilde: "õ", dot: "ọ" },
  ô: { acute: "ố", grave: "ồ", hook: "ổ", tilde: "ỗ", dot: "ộ" },
  ơ: { acute: "ớ", grave: "ờ", hook: "ở", tilde: "ỡ", dot: "ợ" },
  u: { acute: "ú", grave: "ù", hook: "ủ", tilde: "ũ", dot: "ụ" },
  ư: { acute: "ứ", grave: "ừ", hook: "ử", tilde: "ữ", dot: "ự" },
  y: { acute: "ý", grave: "ỳ", hook: "ỷ", tilde: "ỹ", dot: "ỵ" },
};

const VOWELS = new Set("aăâeêioôơuưy");
const MARKED = new Set("ăâêôơư");
const PLAIN: Record<string, string> = {
  ă: "a",
  â: "a",
  ê: "e",
  ô: "o",
  ơ: "o",
  ư: "u",
};

const isAsciiLetter = (ch: string): boolean => /[A-Za-z]/.test(ch);

const keepCase = (source: string, lower: string): string =>
  source === source.toUpperCase() && source !== source.toLowerCase()
    ? lower.toUpperCase()
    : lower;

const stripTone = (word: string): string =>
  word.normalize("NFD").replace(/[\u0300-\u036f]/g, "");

/**
 * Turn one raw Telex token into Vietnamese.
 * Returns the token unchanged when it already contains Vietnamese letters.
 */
export function renderTelexWord(raw: string, modernTone: boolean): string {
  if (raw.length === 0) return raw;
  if (/[^\x00-\x7F]/.test(raw)) return raw;
  if (raw.length > 32) return raw;

  const chars = raw.split("");
  let toneKey: ToneName | null = null;
  let toneAt = -1;

  // Find a tone key that a Vietnamese syllable could accept: at the end, or
  // followed by a short Vietnamese final consonant cluster, with a vowel before it.
  for (let i = chars.length - 1; i >= 1; i--) {
    const lower = chars[i].toLowerCase();
    if (!TONE_KEYS.includes(lower)) continue;
    const prefix = chars.slice(0, i).join("");
    if (![...prefix.toLowerCase()].some((c) => VOWELS.has(c))) continue;
    const suffix = chars.slice(i + 1).join("").toLowerCase();
    const finalOk = suffix === "" || /^[cchmnpt]{1,3}$/.test(suffix);
    if (!finalOk) continue;
    toneAt = i;
    toneKey = lower === "z" ? null : TONE_FOR_KEY[lower] ?? null;
    break;
  }

  const letters = toneAt >= 0 ? chars.slice(0, toneAt).concat(chars.slice(toneAt + 1)) : chars;
  const stripOnly = toneAt >= 0 && chars[toneAt].toLowerCase() === "z";

  const out: string[] = [];
  for (let i = 0; i < letters.length; i++) {
    const ch = letters[i];
    const lower = ch.toLowerCase();
    const lastIdx = out.length - 1;
    const last = lastIdx >= 0 ? out[lastIdx] : "";
    const lastLower = last.toLowerCase();

    if (isAsciiLetter(ch)) {
      if (lower === "w") {
        // Telex w: ă / ơ / ư, and uo + w -> ươ.
        if (lastLower === "o" && out[lastIdx - 1]?.toLowerCase() === "u") {
          out[lastIdx - 1] = keepCase(out[lastIdx - 1], "ư");
          out[lastIdx] = keepCase(last, "ơ");
        } else if (lastLower === "a") out[lastIdx] = keepCase(last, "ă");
        else if (lastLower === "o") out[lastIdx] = keepCase(last, "ơ");
        else if (lastLower === "u") out[lastIdx] = keepCase(last, "ư");
        else out.push(keepCase(ch, "ư"));
        continue;
      }
      if ((lower === "a" || lower === "e" || lower === "o") && lastLower === lower) {
        out[lastIdx] = keepCase(last, lower === "a" ? "â" : lower === "e" ? "ê" : "ô");
        continue;
      }
      if (lower === "d" && lastLower === "d") {
        out[lastIdx] = keepCase(last, "đ");
        continue;
      }
      out.push(ch);
    } else {
      out.push(ch);
    }
  }

  let word = out.join("");
  if (stripOnly) return stripTone(word).replace(/[ăâêôơư]/g, (c) => PLAIN[c] ?? c);
  if (!toneKey) return word;

  const plainChars = word.split("");
  const cluster: number[] = [];
  for (let i = 0; i < plainChars.length; i++) {
    if (VOWELS.has(plainChars[i].toLowerCase())) cluster.push(i);
    else if (cluster.length > 0) break;
  }
  if (cluster.length === 0) return word;

  let target = cluster[cluster.length - 1];
  const marked = cluster.filter((i) => MARKED.has(plainChars[i].toLowerCase()));
  if (marked.length > 0) {
    target = marked[marked.length - 1];
  } else if (cluster.length === 2) {
    target = modernTone ? cluster[0] : cluster[1];
  } else if (cluster.length >= 3) {
    target = cluster[1];
  }

  const base = plainChars[target];
  const composed = TONE_MAP[base.toLowerCase()]?.[toneKey];
  if (!composed) return word;
  plainChars[target] = keepCase(base, composed);
  word = plainChars.join("");
  return word;
}

/** Render a whole line: letters are Telex, everything else passes through. */
export function renderTelex(input: string, modernTone: boolean): string {
  const parts = input.split(/([^\p{L}]+)/u);
  return parts
    .map((part) => {
      if (!part) return part;
      if (/^\p{L}+$/u.test(part)) return renderTelexWord(part, modernTone);
      return part;
    })
    .join("");
}
