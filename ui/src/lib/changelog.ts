// Light markdown for release notes. The body comes from GitHub, so it is
// untrusted text: this parser only ever produces plain strings in a typed
// block list, and the view renders them as Svelte text nodes (no {@html}).

export type Inline =
  | { kind: "text"; text: string }
  | { kind: "code"; text: string }
  | { kind: "bold"; text: string };

export type Block =
  | { kind: "heading"; level: 2 | 3; spans: Inline[] }
  | { kind: "item"; spans: Inline[] }
  | { kind: "para"; spans: Inline[] };

/** `SHA256: <hex>` machine line, same rule as `expected_sha256` in update.rs. */
function isShaLine(line: string): boolean {
  const trimmed = line.trim().replace(/^[*\-# ]+/, "").trim();
  const plain = trimmed.replace(/[`*]/g, "");
  const at = plain.indexOf(":");
  if (at < 0) return false;
  if (plain.slice(0, at).trim().toLowerCase() !== "sha256") return false;
  const hex = plain.slice(at + 1).trim().replace(/^["' ]+|["' ]+$/g, "");
  return /^[0-9a-f]{64}$/i.test(hex);
}

function isFullChangelogLine(line: string): boolean {
  const plain = line.trim().replace(/^[*\-# ]+/, "").replace(/\*/g, "").trim();
  return /^full changelog\s*:/i.test(plain);
}

export function parseInline(src: string): Inline[] {
  const out: Inline[] = [];
  const re = /`([^`]+)`|\*\*([^*]+)\*\*/g;
  let last = 0;
  for (let m = re.exec(src); m; m = re.exec(src)) {
    if (m.index > last) out.push({ kind: "text", text: src.slice(last, m.index) });
    if (m[1] !== undefined) out.push({ kind: "code", text: m[1] });
    else out.push({ kind: "bold", text: m[2] });
    last = m.index + m[0].length;
  }
  if (last < src.length) out.push({ kind: "text", text: src.slice(last) });
  return out;
}

export function parseChangelog(body: string): Block[] {
  const blocks: Block[] = [];
  let para: string[] = [];
  const flush = () => {
    if (para.length) blocks.push({ kind: "para", spans: parseInline(para.join(" ")) });
    para = [];
  };
  for (const raw of body.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || isShaLine(line) || isFullChangelogLine(line)) {
      flush();
      continue;
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line);
    if (h) {
      flush();
      blocks.push({ kind: "heading", level: h[1].length <= 2 ? 2 : 3, spans: parseInline(h[2]) });
      continue;
    }
    const li = /^[-*]\s+(.*)$/.exec(line);
    if (li) {
      flush();
      blocks.push({ kind: "item", spans: parseInline(li[1]) });
      continue;
    }
    para.push(line);
  }
  flush();
  return blocks;
}
