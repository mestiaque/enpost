import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS } from "@/stores/settings";
import { newDraft } from "@/stores/tabs";
import type { RequestDraft } from "@/types";
import { buildBody, buildHttpRequest } from "./http";

const draft = (p: Partial<RequestDraft>): RequestDraft => ({ ...newDraft(), ...p });
const row = (key: string, value: string, enabled = true) => ({ id: key, key, value, enabled });

describe("building the request for the Rust engine", () => {
  it("sends enabled headers and the settings' options", () => {
    const r = buildHttpRequest(draft({ method: "POST", url: "  https://api.dev/x ", headers: [row("Accept", "json"), row("X-Off", "1", false), row(" ", "blank")] }), { ...DEFAULT_SETTINGS, requestTimeoutMs: 5000, verifyTls: false }, "id-1");
    expect(r).toMatchObject({ requestId: "id-1", method: "POST", url: "https://api.dev/x", headers: [{ name: "Accept", value: "json" }] });
    expect(r.options).toEqual({ timeoutMs: 5000, followRedirects: true, maxRedirects: 10, verifyTls: false });
  });

  it("maps each body type", () => {
    expect(buildBody(draft({ method: "POST", bodyType: "json", body: "{}" }))).toEqual({ type: "raw", contentType: "application/json", text: "{}" });
    expect(buildBody(draft({ method: "PUT", bodyType: "urlencoded", urlencoded: [row("a", "1"), row("b", "2", false)] }))).toEqual({ type: "urlEncoded", fields: [{ name: "a", value: "1", kind: "text" }] });
    expect(buildBody(draft({ method: "POST", bodyType: "form-data", form: [{ ...row("f", "/tmp/a.png"), kind: "file" }] }))).toEqual({ type: "formData", fields: [{ name: "f", value: "/tmp/a.png", kind: "file" }] });
    expect(buildBody(draft({ method: "POST", bodyType: "binary", binaryPath: "" }))).toEqual({ type: "none" });
    expect(buildBody(draft({ method: "POST", bodyType: "binary", binaryPath: "/x.bin" }))).toEqual({ type: "binary", path: "/x.bin" });
  });

  it("never sends a body with GET or HEAD", () => {
    expect(buildBody(draft({ method: "GET", bodyType: "json", body: "{}" }))).toEqual({ type: "none" });
    expect(buildBody(draft({ method: "HEAD", bodyType: "text", body: "x" }))).toEqual({ type: "none" });
  });
});
