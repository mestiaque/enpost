export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v < 10 ? v.toFixed(2) : v < 100 ? v.toFixed(1) : Math.round(v)} ${units[i]}`;
}

export function formatMs(ms: number): string {
  if (ms < 1000) return `${Math.round(ms)} ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(ms < 10_000 ? 2 : 1)} s`;
  return `${Math.floor(ms / 60_000)} min ${Math.round((ms % 60_000) / 1000)} s`;
}

/** Tailwind text colour for an HTTP status. */
export function statusColor(status: number): string {
  if (status >= 500) return "text-danger";
  if (status >= 400) return "text-warning";
  if (status >= 300) return "text-sky-500";
  if (status >= 200) return "text-success";
  return "text-muted";
}

/** Pretty JSON when the text parses; otherwise the text as it is. */
export function prettyJson(text: string): { text: string; ok: boolean } {
  try {
    return { text: JSON.stringify(JSON.parse(text), null, 2), ok: true };
  } catch {
    return { text, ok: false };
  }
}

/** Indent XML/HTML-like markup one element per line; `<a>text</a>` stays on one line. */
export function prettyXml(text: string): string {
  const tokens = text.replace(/>\s+</g, "><").split(/(<[^>]+>)/).filter((s) => s.trim() !== "");
  const out: string[] = [];
  let depth = 0;
  // The last line is an opening tag, possibly followed by its text: `<a>` or `<a>text`.
  let openLine = false;
  for (const raw of tokens) {
    const tok = raw.trim();
    const isTag = tok.startsWith("<");
    const closing = tok.startsWith("</");
    const standalone = isTag && (tok.endsWith("/>") || /^<[!?]/.test(tok));
    if (!isTag) {
      if (openLine) out[out.length - 1] += tok;
      else out.push("  ".repeat(depth) + tok);
      continue;
    }
    if (closing) {
      depth = Math.max(0, depth - 1);
      if (openLine) out[out.length - 1] += tok;
      else out.push("  ".repeat(depth) + tok);
      openLine = false;
      continue;
    }
    out.push("  ".repeat(depth) + tok);
    openLine = !standalone;
    if (!standalone) depth++;
  }
  return out.join("\n");
}

/** The language for a response body, from its content type. */
export function languageFor(contentType: string | null | undefined): "json" | "xml" | "html" | "text" {
  const ct = (contentType ?? "").toLowerCase();
  if (ct.includes("json")) return "json";
  if (ct.includes("html")) return "html";
  if (ct.includes("xml")) return "xml";
  return "text";
}
