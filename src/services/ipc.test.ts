import { describe, expect, it } from "vitest";
import { hexToRgbTriplet } from "@/composables/useTheme";
import { IpcError, inTauri, toIpcError } from "./ipc";

describe("ipc errors", () => {
  it("maps backend payloads, strings and errors to IpcError", () => {
    const e = toIpcError({ kind: "validation", message: "Accent must be a colour like #3b82f6" });
    expect(e).toBeInstanceOf(IpcError);
    expect([e.kind, e.message]).toEqual(["validation", "Accent must be a colour like #3b82f6"]);
    expect(toIpcError("boom").kind).toBe("internal");
    expect(toIpcError(new Error("x")).message).toBe("x");
  });

  it("knows it is not inside Tauri in tests", () => {
    expect(inTauri()).toBe(false);
  });
});

describe("theme", () => {
  it("converts accent colours to CSS token triplets", () => {
    expect(hexToRgbTriplet("#3b82f6")).toBe("59 130 246");
    expect(hexToRgbTriplet("blue")).toBeNull();
  });
});
