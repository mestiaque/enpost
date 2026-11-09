/** Conversions between the tab's editable draft and a saved request's stored definition. */
import { BODY_TYPES, HTTP_METHODS, type BodyType, type FormRow, type HttpMethod, type KeyValue, type RequestDefinition, type RequestDraft, type SavedRequest } from "@/types";

const kv = (rows: KeyValue[]): RequestDefinition["params"] =>
  rows.map((r) => ({ id: r.id, key: r.key, value: r.value, enabled: r.enabled, ...(r.description ? { description: r.description } : {}) }));

export function draftToDefinition(d: RequestDraft): RequestDefinition {
  return {
    params: kv(d.params),
    headers: kv(d.headers),
    bodyType: d.bodyType,
    body: d.body,
    urlencoded: kv(d.urlencoded),
    form: d.form.map((r) => ({ id: r.id, key: r.key, value: r.value, enabled: r.enabled, kind: r.kind })),
    binaryPath: d.binaryPath,
  };
}

export function savedToDraft(r: SavedRequest): RequestDraft {
  const def = r.definition;
  const method = (HTTP_METHODS as readonly string[]).includes(r.method) ? (r.method as HttpMethod) : "GET";
  const bodyType = (BODY_TYPES as readonly string[]).includes(def.bodyType) ? (def.bodyType as BodyType) : "none";
  const rows = (x: RequestDefinition["params"]): KeyValue[] => x.map((p) => ({ id: p.id, key: p.key, value: p.value, enabled: p.enabled, ...(p.description ? { description: p.description } : {}) }));
  return {
    method,
    url: r.url,
    params: rows(def.params),
    headers: rows(def.headers),
    bodyType,
    body: def.body,
    urlencoded: rows(def.urlencoded),
    form: def.form.map((f): FormRow => ({ id: f.id, key: f.key, value: f.value, enabled: f.enabled, kind: f.kind === "file" ? "file" : "text" })),
    binaryPath: def.binaryPath,
  };
}
