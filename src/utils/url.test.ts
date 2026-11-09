import { describe, expect, it } from "vitest";
import type { KeyValue } from "@/types";
import { paramsFromUrl, queryOf, urlWithParams } from "./url";

const row = (key: string, value: string, enabled = true, id = key): KeyValue => ({ id, key, value, enabled });

describe("URL ↔ params", () => {
  it("reads the query, decoding values and tolerating half-typed input", () => {
    const p = paramsFromUrl("https://x.dev/a?q=hello%20world&page=2&flag&bad=%E0%A4#frag", []);
    expect(p.map((r) => [r.key, r.value])).toEqual([
      ["q", "hello world"],
      ["page", "2"],
      ["flag", ""],
      ["bad", "%E0%A4"],
    ]);
    expect(queryOf("https://x.dev/a?x=1#y")).toBe("x=1");
  });

  it("keeps ids by position and keeps disabled rows", () => {
    const prev = [row("a", "1", true, "id-a"), row("off", "x", false, "id-off")];
    const p = paramsFromUrl("/p?a=2&b=3", prev);
    expect(p.map((r) => r.id)).toEqual(["id-a", expect.any(String), "id-off"]);
    expect(p[2]).toMatchObject({ key: "off", enabled: false });
  });

  it("writes enabled rows back, keeping {{variables}} readable and the fragment", () => {
    const url = urlWithParams("{{base_url}}/users?old=1#top", [row("q", "a b&c"), row("token", "{{token}}"), row("skip", "1", false), row("", "")]);
    expect(url).toBe("{{base_url}}/users?q=a%20b%26c&token={{token}}#top");
    expect(urlWithParams("https://x.dev/a?z=1", [])).toBe("https://x.dev/a");
  });

  it("round-trips", () => {
    const url = "https://api.dev/v1/items?sort=name&limit=20";
    expect(urlWithParams(url, paramsFromUrl(url, []))).toBe(url);
  });
});
