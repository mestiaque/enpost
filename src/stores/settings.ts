import { defineStore } from "pinia";
import { ref } from "vue";
import { api, inTauri } from "@/services/ipc";
import type { Settings, SettingsPatch } from "@/types";
import { useUi } from "./ui";

/** Shown until the backend answers; the backend owns the real defaults and validation. */
export const DEFAULT_SETTINGS: Settings = {
  startup: "restore",
  alwaysOnTop: false,
  confirmCloseUnsaved: true,
  theme: "dark",
  accent: "#3b82f6",
  reduceMotion: false,
  editorFontSize: 13,
  editorFontFamily: "",
  editorWordWrap: false,
  sidebarCollapsed: false,
  sidebarWidth: 280,
  editorSplit: 0.5,
  requestTimeoutMs: 30000,
  followRedirects: true,
  maxRedirects: 10,
  verifyTls: true,
  proxyUrl: "",
  autosave: true,
  historyRetentionDays: 30,
  checkUpdates: true,
  activeWorkspaceId: "",
};

export const useSettings = defineStore("settings", () => {
  const settings = ref<Settings>({ ...DEFAULT_SETTINGS });
  const loaded = ref(false);

  async function load() {
    if (inTauri()) {
      try {
        settings.value = await api.getSettings();
      } catch (e) {
        useUi().toast({ kind: "error", title: "Could not load settings", detail: String(e) });
      }
    }
    loaded.value = true;
  }

  /** Apply at once (optimistic); the backend validates and returns what it stored. */
  async function update(patch: SettingsPatch) {
    const before = settings.value;
    settings.value = { ...before, ...(patch as Partial<Settings>) };
    if (!inTauri()) return;
    try {
      settings.value = await api.updateSettings(patch);
    } catch (e) {
      settings.value = before;
      useUi().toast({ kind: "error", title: "Setting not saved", detail: e instanceof Error ? e.message : String(e) });
    }
  }

  async function reset() {
    settings.value = inTauri() ? await api.resetSettings() : { ...DEFAULT_SETTINGS };
  }

  return { settings, loaded, load, update, reset };
});
