import { defineStore } from "pinia";
import { ref } from "vue";

export interface Toast {
  id: number;
  kind: "info" | "success" | "error";
  title: string;
  detail?: string;
}

export interface ConfirmRequest {
  title: string;
  message: string;
  confirmLabel: string;
  danger: boolean;
  resolve: (ok: boolean) => void;
}

export interface PromptRequest {
  title: string;
  label?: string;
  initial: string;
  confirmLabel: string;
  resolve: (value: string | null) => void;
}

export type DialogKind = null | "settings" | "palette" | "save-as";
export type SidebarPanel = "collections" | "history";

let toastId = 0;

export const useUi = defineStore("ui", () => {
  const toasts = ref<Toast[]>([]);
  const dialog = ref<DialogKind>(null);
  /** Palette mode: commands (Ctrl+K) or open tabs (Ctrl+P). */
  const paletteMode = ref<"commands" | "tabs">("commands");
  const settingsSection = ref("general");
  const confirmReq = ref<ConfirmRequest | null>(null);
  const sidebarPanel = ref<SidebarPanel>("collections");
  const promptReq = ref<PromptRequest | null>(null);
  /** Tab being saved by the "Save as" dialog. */
  const saveAsTabId = ref<string | null>(null);
  function openSaveAs(tabId: string) {
    saveAsTabId.value = tabId;
    dialog.value = "save-as";
  }
  /** Ask for a short text (a name). Resolves to null when cancelled. */
  function prompt(o: { title: string; label?: string; initial?: string; confirmLabel?: string }): Promise<string | null> {
    return new Promise((resolve) => {
      promptReq.value = { initial: "", confirmLabel: "OK", ...o, resolve };
    });
  }
  function answerPrompt(v: string | null) {
    promptReq.value?.resolve(v);
    promptReq.value = null;
  }

  function toast(t: Omit<Toast, "id">, ttl = 4000) {
    const id = ++toastId;
    toasts.value.push({ ...t, id });
    if (ttl > 0) setTimeout(() => dismiss(id), ttl);
    return id;
  }
  function dismiss(id: number) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }
  function openSettings(section = "general") {
    settingsSection.value = section;
    dialog.value = "settings";
  }
  function openPalette(mode: "commands" | "tabs") {
    paletteMode.value = mode;
    dialog.value = "palette";
  }
  function confirm(o: { title: string; message: string; confirmLabel?: string; danger?: boolean }): Promise<boolean> {
    return new Promise((resolve) => {
      confirmReq.value = { confirmLabel: "OK", danger: false, ...o, resolve };
    });
  }
  function answerConfirm(ok: boolean) {
    confirmReq.value?.resolve(ok);
    confirmReq.value = null;
  }
  return { saveAsTabId, openSaveAs, promptReq, prompt, answerPrompt, sidebarPanel, toasts, dialog, paletteMode, settingsSection, confirmReq, toast, dismiss, openSettings, openPalette, confirm, answerConfirm };
});
