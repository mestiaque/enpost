import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { defineComponent, h, nextTick, ref } from "vue";
import Dialog from "@/components/ui/Dialog.vue";

describe("Dialog close", () => {
  it("closes on Escape pressed inside it and leaves the page clickable", async () => {
    const open = ref(true);
    const Host = defineComponent({
      setup: () => () => h("div", [h("button", { id: "outside" }, "x"), h(Dialog, { open: open.value, "onUpdate:open": (v: boolean) => (open.value = v), title: "S" }, () => h("p", "body"))]),
    });
    mount(Host, { attachTo: document.body });
    await nextTick();
    await new Promise((r) => setTimeout(r, 50));
    // Escape pressed inside the dialog (focus is in it) must close it.
    const inside = document.querySelector('[role="dialog"]') as HTMLElement;
    inside.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await new Promise((r) => setTimeout(r, 300));
    expect(open.value).toBe(false);
    expect(document.body.style.pointerEvents).not.toBe("none");
  });
});
