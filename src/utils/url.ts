import type { KeyValue } from "@/types";

const uid = () => (globalThis.crypto?.randomUUID?.() ?? `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`);

/**
 * Query decoding that tolerates half-typed input and `{{variables}}`: a malformed escape is
 * kept as typed instead of throwing.
 */
function decode(s: string): string {
  try {
    return decodeURIComponent(s.replace(/\+/g, " "));
  } catch {
    return s;
  }
}

/** Encode a key or value, leaving `{{variable}}` placeholders readable. */
function encode(s: string): string {
  return s
    .split(/(\{\{[^}]*\}\})/)
    .map((part) => (/^\{\{[^}]*\}\}$/.test(part) ? part : encodeURIComponent(part)))
    .join("");
}

/** The raw query string of a URL (without `?` and without the fragment). */
export function queryOf(url: string): string {
  const q = url.indexOf("?");
  if (q < 0) return "";
  const rest = url.slice(q + 1);
  const h = rest.indexOf("#");
  return h < 0 ? rest : rest.slice(0, h);
}

/**
 * Params rows for a URL's query. Rows are matched by position to keep their ids and
 * descriptions; disabled rows (not in the URL) are kept after the enabled ones.
 */
export function paramsFromUrl(url: string, previous: KeyValue[]): KeyValue[] {
  const q = queryOf(url);
  const enabledPrev = previous.filter((p) => p.enabled);
  const rows: KeyValue[] = q
    ? q.split("&").map((pair, i) => {
        const eq = pair.indexOf("=");
        const key = decode(eq < 0 ? pair : pair.slice(0, eq));
        const value = eq < 0 ? "" : decode(pair.slice(eq + 1));
        const prev = enabledPrev[i];
        return { id: prev?.id ?? uid(), key, value, enabled: true, ...(prev?.description ? { description: prev.description } : {}) };
      })
    : [];
  return [...rows, ...previous.filter((p) => !p.enabled)];
}

/** The URL with its query replaced by the enabled rows (keys may be empty while typing). */
export function urlWithParams(url: string, params: KeyValue[]): string {
  const hash = url.indexOf("#");
  const fragment = hash < 0 ? "" : url.slice(hash);
  const noFragment = hash < 0 ? url : url.slice(0, hash);
  const q = noFragment.indexOf("?");
  const base = q < 0 ? noFragment : noFragment.slice(0, q);
  const enabled = params.filter((p) => p.enabled && (p.key !== "" || p.value !== ""));
  if (!enabled.length) return base + fragment;
  const query = enabled.map((p) => (p.value === "" ? encode(p.key) : `${encode(p.key)}=${encode(p.value)}`)).join("&");
  return `${base}?${query}${fragment}`;
}
