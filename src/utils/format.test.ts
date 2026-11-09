import { describe, expect, it } from "vitest";
import { formatBytes, formatMs, languageFor, prettyJson, prettyXml, statusColor } from "./format";
import { expandToDepth, flatten, type JsonValue } from "./jsonTree";

describe("format", () => {
  it("sizes and durations", () => {
    expect([formatBytes(512), formatBytes(1536), formatBytes(5 * 1024 * 1024)]).toEqual(["512 B", "1.50 KB", "5.00 MB"]);
    expect([formatMs(87), formatMs(1234), formatMs(75_000)]).toEqual(["87 ms", "1.23 s", "1 min 15 s"]);
  });

  it("status colours and languages", () => {
    expect([statusColor(204), statusColor(301), statusColor(404), statusColor(503)]).toEqual(["text-success", "text-sky-500", "text-warning", "text-danger"]);
    expect([languageFor("application/problem+json"), languageFor("text/html; charset=utf-8"), languageFor("application/xml"), languageFor(null)]).toEqual(["json", "html", "xml", "text"]);
  });

  it("pretty JSON, leaving invalid JSON alone", () => {
    expect(prettyJson('{"a":[1,2]}')).toEqual({ text: '{\n  "a": [\n    1,\n    2\n  ]\n}', ok: true });
    expect(prettyJson("{oops")).toEqual({ text: "{oops", ok: false });
  });

  it("pretty XML", () => {
    expect(prettyXml('<?xml version="1.0"?><a><b x="1">hi</b><c/><d><e>1</e></d></a>')).toBe(
      ['<?xml version="1.0"?>', "<a>", '  <b x="1">hi</b>', "  <c/>", "  <d>", "    <e>1</e>", "  </d>", "</a>"].join("\n"),
    );
  });
});

describe("json tree", () => {
  const doc: JsonValue = { users: [{ name: "Ada", "first-login": null }, { name: "Linus" }], total: 2, ok: true };

  it("shows only expanded nodes, with JSON paths", () => {
    expect(flatten(doc, new Set()).map((r) => r.path)).toEqual(["$"]);
    const rows = flatten(doc, new Set(["$", "$.users", "$.users[0]"]));
    expect(rows.map((r) => r.path)).toEqual(["$", "$.users", "$.users[0]", "$.users[0].name", '$.users[0]["first-login"]', "$.users[1]", "$.total", "$.ok"]);
    expect(rows[1]).toMatchObject({ label: "users", kind: "array", size: 2, depth: 1, expandable: true });
    expect(rows[4]).toMatchObject({ kind: "null", expandable: false });
  });

  it("expands to a depth", () => {
    expect([...expandToDepth(doc, 2)].sort()).toEqual(["$", "$.users"]);
  });
});
