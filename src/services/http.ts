/**
 * Turns the request being edited into the typed request the Rust engine executes, and runs
 * it. No request is ever sent from the webview itself.
 */
import { listen } from "@tauri-apps/api/event";
import type { BodyType, FormField, HeaderPair, HttpProgress, HttpRequest, KeyValue, RequestBody, RequestDraft, Settings } from "@/types";

const RAW_TYPES: Partial<Record<BodyType, string>> = { json: "application/json", xml: "application/xml", text: "text/plain" };

const active = <T extends KeyValue>(rows: T[]): T[] => rows.filter((r) => r.enabled && r.key.trim() !== "");

export function buildBody(d: RequestDraft): RequestBody {
  // GET and HEAD carry no body (servers and proxies may reject one).
  if (d.method === "GET" || d.method === "HEAD") return { type: "none" };
  const raw = RAW_TYPES[d.bodyType];
  if (raw !== undefined) return { type: "raw", contentType: raw, text: d.body };
  switch (d.bodyType) {
    case "urlencoded":
      return { type: "urlEncoded", fields: active(d.urlencoded).map((r): FormField => ({ name: r.key, value: r.value, kind: "text" })) };
    case "form-data":
      return { type: "formData", fields: active(d.form).map((r): FormField => ({ name: r.key, value: r.value, kind: r.kind })) };
    case "binary":
      return d.binaryPath ? { type: "binary", path: d.binaryPath } : { type: "none" };
    default:
      return { type: "none" };
  }
}

export function buildHttpRequest(d: RequestDraft, s: Settings, requestId: string): HttpRequest {
  const headers: HeaderPair[] = active(d.headers).map((h) => ({ name: h.key.trim(), value: h.value }));
  return {
    requestId,
    method: d.method,
    url: d.url.trim(),
    headers,
    body: buildBody(d),
    options: { timeoutMs: s.requestTimeoutMs, followRedirects: s.followRedirects, maxRedirects: s.maxRedirects, verifyTls: s.verifyTls },
  };
}

/** One listener for all progress events, routed to whoever waits for that request id. */
const waiting = new Map<string, (p: HttpProgress) => void>();
let listening: Promise<unknown> | null = null;

export function onProgress(requestId: string, fn: (p: HttpProgress) => void): () => void {
  listening ??= listen<HttpProgress>("http-progress", (e) => waiting.get(e.payload.requestId)?.(e.payload));
  waiting.set(requestId, fn);
  return () => waiting.delete(requestId);
}
