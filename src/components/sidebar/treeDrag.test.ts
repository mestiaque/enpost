import { describe, expect, it } from "vitest";
import { besideTarget, type Dragged } from "./treeDrag";

const d = (index: number, folderId: string | null = null): Dragged => ({ kind: "request", id: "x", collectionId: "c", folderId, index });

describe("tree drop targets", () => {
  it("accounts for the item leaving its old place when moving down", () => {
    expect(besideTarget(d(0), "c", null, 2, "after").index).toBe(2); // [x a b] → [a b x]
    expect(besideTarget(d(0), "c", null, 2, "before").index).toBe(1); // → [a x b]
    expect(besideTarget(d(3), "c", null, 0, "before").index).toBe(0); // moving up: no shift
  });
  it("does not shift when coming from another parent", () => {
    expect(besideTarget(d(0, "f"), "c", null, 2, "after").index).toBe(3);
  });
});
