import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import type { KeyValue } from "@/types";
import KeyValueEditor from "./KeyValueEditor.vue";

describe("KeyValueEditor", () => {
  it("adds a row when the empty last row is typed into, keeping its id", async () => {
    const w = mount(KeyValueEditor, { props: { modelValue: [] as KeyValue[], label: "Headers" } });
    const trailingKey = w.findAll('input[aria-label="Key"]').at(-1)!;
    await trailingKey.setValue("Accept");
    const emitted = w.emitted("update:modelValue")!.at(-1)![0] as KeyValue[];
    expect(emitted).toHaveLength(1);
    expect(emitted[0]).toMatchObject({ key: "Accept", value: "", enabled: true });
    // Parent passes the new rows back: the row keeps the id the input had, plus a new empty row.
    await w.setProps({ modelValue: emitted });
    expect(w.findAll('input[aria-label="Key"]')).toHaveLength(2);
  });

  it("edits, disables and removes rows", async () => {
    const rows: KeyValue[] = [{ id: "1", key: "a", value: "1", enabled: true }];
    const w = mount(KeyValueEditor, { props: { modelValue: rows, label: "Params" } });
    await w.find('input[aria-label="Value"]').setValue("2");
    expect((w.emitted("update:modelValue")!.at(-1)![0] as KeyValue[])[0]!.value).toBe("2");
    await w.find('input[type="checkbox"]').setValue(false);
    expect((w.emitted("update:modelValue")!.at(-1)![0] as KeyValue[])[0]!.enabled).toBe(false);
    await w.find('button[aria-label="Remove a"]').trigger("click");
    expect(w.emitted("update:modelValue")!.at(-1)![0]).toEqual([]);
  });
});
