import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { h } from "vue";
import Dialog from "@/components/ui/Dialog.vue";

describe("Dialog", () => {
  it("renders its content when open, centred by a flex wrapper", async () => {
    mount(Dialog, { props: { open: true, title: "Settings" }, slots: { default: () => h("p", { id: "body" }, "hello") }, attachTo: document.body });
    await new Promise((r) => setTimeout(r, 0));
    const body = document.getElementById("body");
    expect(body?.textContent).toBe("hello");
    expect(document.querySelector('[role="dialog"]')?.parentElement?.className).toContain("items-center");
  });
});
