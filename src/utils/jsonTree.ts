/**
 * Flattened JSON tree for a virtualised viewer: only expanded nodes produce rows, so a
 * large document renders as a window of rows, never one huge DOM.
 */
export type JsonValue = null | boolean | number | string | JsonValue[] | { [k: string]: JsonValue };

export interface TreeRow {
  /** JSON path, e.g. `$.items[2].name` (also the expand/collapse key). */
  path: string;
  depth: number;
  /** Property name or array index; null for the root. */
  label: string | null;
  kind: "object" | "array" | "string" | "number" | "boolean" | "null";
  /** Child count for containers. */
  size: number;
  value: JsonValue;
  expandable: boolean;
}

const kindOf = (v: JsonValue): TreeRow["kind"] => (v === null ? "null" : Array.isArray(v) ? "array" : typeof v === "object" ? "object" : (typeof v as "string" | "number" | "boolean"));

const childPath = (parent: string, key: string | number) =>
  typeof key === "number" ? `${parent}[${key}]` : /^[A-Za-z_$][\w$]*$/.test(key) ? `${parent}.${key}` : `${parent}[${JSON.stringify(key)}]`;

export function flatten(root: JsonValue, expanded: Set<string>): TreeRow[] {
  const rows: TreeRow[] = [];
  const walk = (v: JsonValue, path: string, depth: number, label: string | null) => {
    const kind = kindOf(v);
    const entries: [string | number, JsonValue][] = kind === "array" ? (v as JsonValue[]).map((x, i) => [i, x]) : kind === "object" ? Object.entries(v as Record<string, JsonValue>) : [];
    rows.push({ path, depth, label, kind, size: entries.length, value: v, expandable: entries.length > 0 });
    if (entries.length && expanded.has(path)) for (const [k, child] of entries) walk(child, childPath(path, k), depth + 1, String(k));
  };
  walk(root, "$", 0, null);
  return rows;
}

/** Paths to expand so the first `levels` levels are open. */
export function expandToDepth(root: JsonValue, levels: number): Set<string> {
  const set = new Set<string>();
  const walk = (v: JsonValue, path: string, depth: number) => {
    if (depth >= levels || v === null || typeof v !== "object") return;
    set.add(path);
    const entries: [string | number, JsonValue][] = Array.isArray(v) ? v.map((x, i) => [i, x]) : Object.entries(v);
    for (const [k, child] of entries) walk(child, childPath(path, k), depth + 1);
  };
  walk(root, "$", 0);
  return set;
}
