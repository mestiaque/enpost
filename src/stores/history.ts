import { defineStore } from "pinia";
import { ref } from "vue";
import { api, inTauri } from "@/services/ipc";
import type { HistoryEntry } from "@/types";

export const useHistory = defineStore("history", () => {
  const entries = ref<HistoryEntry[]>([]);
  const loading = ref(false);
  async function refresh() {
    if (!inTauri()) return;
    loading.value = true;
    try {
      entries.value = await api.listHistory(300);
    } finally {
      loading.value = false;
    }
  }
  async function clear() {
    await api.clearHistory();
    entries.value = [];
  }
  return { entries, loading, refresh, clear };
});
