import { mount } from "@vue/test-utils";
import { TooltipProvider } from "reka-ui";
import { describe, expect, it, vi } from "vitest";
import { h } from "vue";
import IconButton from "@/components/ui/IconButton.vue";

describe("IconButton", () => {
  it("passes clicks and classes through its tooltip to the button", async () => {
    const onClick = vi.fn();
    const w = mount({ render: () => h(TooltipProvider, () => h(IconButton, { label: "History", class: "ml-2", onClick }, () => "H")) });
    const btn = w.find('button[aria-label="History"]');
    expect(btn.classes()).toContain("ml-2");
    await btn.trigger("click");
    expect(onClick).toHaveBeenCalledOnce();
  });
});
