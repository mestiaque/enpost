import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import { useTabs } from "./tabs";

describe("tabs store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("opens next to the active tab and closes to the right neighbour", () => {
    const s = useTabs();
    const a = s.open();
    const c = s.open();
    s.activate(a.id);
    const b = s.open();
    expect(s.tabs.map((t) => t.id)).toEqual([a.id, b.id, c.id]);
    s.close(b.id);
    expect(s.activeId).toBe(c.id);
    s.close(c.id);
    expect(s.activeId).toBe(a.id);
    s.close(a.id);
    expect(s.activeId).toBeNull();
  });

  it("syncs URL and params both ways and marks the tab dirty", () => {
    const s = useTabs();
    const t = s.open();
    s.setUrl(t.id, "https://api.dev/users?page=2");
    expect(t.dirty).toBe(true);
    expect(t.name).toBe("api.dev/users");
    expect(t.draft.params.map((p) => [p.key, p.value])).toEqual([["page", "2"]]);
    const first = t.draft.params[0]!;
    s.setParams(t.id, [{ ...first, value: "3" }, { id: "n", key: "q", value: "x", enabled: true }]);
    expect(t.draft.url).toBe("https://api.dev/users?page=3&q=x");
  });

  it("reorders, duplicates and cycles", () => {
    const s = useTabs();
    const [a, b, c] = [s.open(), s.open(), s.open()];
    s.move(0, 2);
    expect(s.tabs.map((t) => t.id)).toEqual([b.id, c.id, a.id]);
    s.setUrl(b.id, "/x?k=v");
    s.duplicate(b.id);
    const copy = s.tabs[1]!;
    expect(copy.draft.url).toBe("/x?k=v");
    expect(copy.draft.params[0]!.id).not.toBe(b.draft.params[0]!.id);
    expect(s.activeId).toBe(copy.id);
    s.cycle(1);
    expect(s.activeId).toBe(c.id);
    s.closeOthers(c.id);
    expect(s.tabs).toHaveLength(1);
  });
});
