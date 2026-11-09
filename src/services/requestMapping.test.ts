import { describe, expect, it } from "vitest";
import type { RequestDraft, SavedRequest } from "@/types";
import { draftToDefinition, savedToDraft } from "./requestMapping";

describe("draft ↔ saved request", () => {
  it("round-trips every field", () => {
    const d: RequestDraft = {
      method: "PATCH",
      url: "https://api.dev/users/1?x=1",
      params: [{ id: "p", key: "x", value: "1", enabled: true }],
      headers: [{ id: "h", key: "Accept", value: "application/json", enabled: false, description: "why" }],
      bodyType: "form-data",
      body: "{}",
      urlencoded: [{ id: "u", key: "a", value: "b", enabled: true }],
      form: [{ id: "f", key: "file", value: "/tmp/a.png", enabled: true, kind: "file" }],
      binaryPath: "/tmp/b.bin",
    };
    const saved: SavedRequest = { id: "r", collectionId: "c", folderId: null, name: "Patch", description: "", method: d.method, url: d.url, definition: draftToDefinition(d), favorite: false, updatedAt: 0 };
    expect(savedToDraft(saved)).toEqual(d);
  });

  it("opens unknown methods and body types safely", () => {
    const saved: SavedRequest = {
      id: "r", collectionId: "c", folderId: null, name: "x", description: "", method: "PURGE", url: "", favorite: false, updatedAt: 0,
      definition: { params: [], headers: [], bodyType: "graphql", body: "", urlencoded: [], form: [{ id: "f", key: "k", value: "v", enabled: true, kind: "weird" }], binaryPath: "" },
    };
    const d = savedToDraft(saved);
    expect([d.method, d.bodyType, d.form[0]!.kind]).toEqual(["GET", "none", "text"]);
  });
});
