import { onBeforeUnmount, onMounted } from "vue";
import { useSettings } from "@/stores/settings";
import { useTabs } from "@/stores/tabs";
import { useUi } from "@/stores/ui";

export interface Shortcut {
  keys: string;
  label: string;
}

/** Shown in the command palette and in Settings → Shortcuts. */
export const SHORTCUTS: Shortcut[] = [
  { keys: "Ctrl+K", label: "Command palette" },
  { keys: "Ctrl+P", label: "Go to an open tab" },
  { keys: "Ctrl+T", label: "New request tab" },
  { keys: "Ctrl+W", label: "Close tab" },
  { keys: "Ctrl+Tab", label: "Next tab (Shift: previous)" },
  { keys: "Ctrl+B", label: "Show or hide the sidebar" },
  { keys: "Ctrl+,", label: "Settings" },
  { keys: "Ctrl+Enter", label: "Send request (Enter in the URL bar too)" },
  { keys: "Ctrl+S", label: "Save request (Phase 3)" },
];

/** Not built yet: say so instead of silently doing nothing. */
export function notYet(what: string, phase: number) {
  useUi().toast({ kind: "info", title: `${what} arrives in Phase ${phase}`, detail: "See docs/ROADMAP.md" });
}

export function useShortcuts() {
  const ui = useUi();
  const tabs = useTabs();
  const settings = useSettings();
  const onKey = (e: KeyboardEvent) => {
    const ctrl = e.ctrlKey || e.metaKey;
    if (!ctrl) return;
    const k = e.key.toLowerCase();
    let handled = true;
    if (k === "k") ui.openPalette("commands");
    else if (k === "p") ui.openPalette("tabs");
    else if (k === ",") ui.openSettings();
    else if (ui.dialog || ui.confirmReq) handled = false;
    else if (k === "t") tabs.open();
    else if (k === "w") tabs.activeId && tabs.close(tabs.activeId);
    else if (k === "tab") tabs.cycle(e.shiftKey ? -1 : 1);
    else if (k === "b") void settings.update({ sidebarCollapsed: !settings.settings.sidebarCollapsed });
    else if (k === "enter") tabs.activeId && void tabs.send(tabs.activeId);
    else if (k === "s") notYet("Saving requests", 3);
    else handled = false;
    if (handled) e.preventDefault();
  };
  onMounted(() => window.addEventListener("keydown", onKey));
  onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
}
