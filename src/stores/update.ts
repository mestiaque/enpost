import { defineStore } from "pinia";
import { ref } from "vue";
import { api, inTauri } from "@/services/ipc";
import type { UpdateInfo } from "@/types";
import { useUi } from "./ui";

const LAST_CHECK_KEY = "enpost.lastUpdateCheck";
// A little under a day, so a computer started at about the same time every day still checks daily.
const DAY = 20 * 3600 * 1000;

function lastCheck(): number {
  try {
    return Number(localStorage.getItem(LAST_CHECK_KEY) ?? 0);
  } catch {
    return 0;
  }
}

/** New releases on GitHub (same behaviour as ENViewer and ENSave). */
export const useUpdate = defineStore("update", () => {
  const available = ref<UpdateInfo | null>(null);
  const dialogOpen = ref(false);
  const checking = ref(false);

  /** Ask GitHub for a newer release. Manual checks report "up to date" and errors too. */
  async function check(manual: boolean): Promise<string> {
    if (checking.value) return "Checking…";
    if (!inTauri()) return "Updates are checked by the desktop app";
    checking.value = true;
    try {
      const info = await api.checkUpdate();
      try {
        localStorage.setItem(LAST_CHECK_KEY, String(Date.now()));
      } catch {
        /* storage unavailable: just check again next start */
      }
      available.value = info;
      if (!info) return "You have the latest version";
      if (manual) dialogOpen.value = true;
      else useUi().toast({ kind: "info", title: `ENPost ${info.version} is available`, action: { label: "Update…", run: () => (dialogOpen.value = true) } }, 30000);
      return `ENPost ${info.version} is available`;
    } catch (e) {
      const msg = "Could not check for updates. Are you online?";
      if (manual) useUi().toast({ kind: "error", title: msg, detail: e instanceof Error ? e.message : String(e) });
      return msg;
    } finally {
      checking.value = false;
    }
  }

  return { available, dialogOpen, checking, check, open: () => (dialogOpen.value = true), close: () => (dialogOpen.value = false) };
});

/** Quiet check a few seconds after start-up, at most about once a day. */
export function scheduleUpdateCheck(enabled: boolean): () => void {
  if (!enabled || Date.now() - lastCheck() < DAY) return () => {};
  const t = window.setTimeout(() => void useUpdate().check(false), 8000);
  return () => window.clearTimeout(t);
}
