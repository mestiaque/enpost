import { describe, expect, it } from "vitest";
import type { WorkspaceTree } from "@/types";
import { buildTree, saveTargets } from "./tree";

const t: WorkspaceTree = {
  workspaceId: "w",
  collections: [
    { id: "c1", name: "Users API", description: "" },
    { id: "c2", name: "Billing", description: "" },
  ],
  folders: [
    { id: "f1", collectionId: "c1", parentId: null, name: "Admin" },
    { id: "f2", collectionId: "c1", parentId: "f1", name: "Roles" },
  ],
  requests: [
    { id: "r1", collectionId: "c1", folderId: null, name: "List users", method: "GET", url: "/users", favorite: false },
    { id: "r2", collectionId: "c1", folderId: "f2", name: "Grant role", method: "POST", url: "/roles/grant", favorite: true },
    { id: "r3", collectionId: "c2", folderId: null, name: "Invoices", method: "GET", url: "/invoices", favorite: false },
  ],
};

describe("sidebar tree", () => {
  it("nests folders and requests", () => {
    const v = buildTree(t);
    expect(v.map((c) => c.collection.name)).toEqual(["Users API", "Billing"]);
    expect(v[0]!.requests.map((r) => r.id)).toEqual(["r1"]);
    expect(v[0]!.folders[0]!.folders[0]!.requests.map((r) => r.id)).toEqual(["r2"]);
  });

  it("filters by request name or URL and keeps the path", () => {
    const v = buildTree(t, "grant");
    expect(v.map((c) => c.collection.id)).toEqual(["c1"]);
    expect(v[0]!.requests).toEqual([]);
    expect(v[0]!.folders[0]!.folders[0]!.requests.map((r) => r.id)).toEqual(["r2"]);
    expect(buildTree(t, "/invoices").map((c) => c.collection.id)).toEqual(["c2"]);
  });

  it("a matching collection or folder name keeps its contents", () => {
    expect(buildTree(t, "billing")[0]!.requests.map((r) => r.id)).toEqual(["r3"]);
    const admin = buildTree(t, "admin")[0]!;
    expect(admin.requests).toEqual([]);
    expect(admin.folders[0]!.folders[0]!.requests.map((r) => r.id)).toEqual(["r2"]);
    expect(buildTree(t, "nothing here")).toEqual([]);
  });

  it("lists save targets depth-first", () => {
    expect(saveTargets(t).map((x) => `${"-".repeat(x.depth)}${x.label}`)).toEqual(["Users API", "-Admin", "--Roles", "Billing"]);
  });
});
