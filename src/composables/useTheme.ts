import { onScopeDispose, watchEffect } from "vue";
import { useSettings } from "@/stores/settings";

/** "#3b82f6" → "59 130 246" (the format of the CSS colour tokens). */
export function hexToRgbTriplet(hex: string): string | null {
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex);
  return m ? m.slice(1, 4).map((h) => parseInt(h, 16)).join(" ") : null;
}

/** Applies theme, accent, motion and editor font settings to the document. */
export function useTheme() {
  const store = useSettings();
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => {
    const s = store.settings;
    const root = document.documentElement;
    const dark = s.theme === "dark" || s.theme === "glass" || (s.theme === "system" && mq.matches);
    root.classList.toggle("dark", dark);
    root.classList.toggle("light", !dark);
    root.classList.toggle("glass", s.theme === "glass");
    root.classList.toggle("reduce-motion", s.reduceMotion);
    const rgb = hexToRgbTriplet(s.accent);
    if (rgb) root.style.setProperty("--accent", rgb);
    root.style.setProperty("--editor-font-size", `${s.editorFontSize}px`);
    root.style.setProperty("--editor-font", s.editorFontFamily.trim() || '"JetBrains Mono", "Cascadia Mono", "Ubuntu Mono", monospace');
  };
  watchEffect(apply);
  mq.addEventListener("change", apply);
  onScopeDispose(() => mq.removeEventListener("change", apply));
}
