<script setup lang="ts">
import { Database } from "lucide-vue-next";
import { onMounted, ref } from "vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { api, inTauri } from "@/services/ipc";
import { useTabs } from "@/stores/tabs";
import type { AppInfo } from "@/types";

const tabs = useTabs();
const info = ref<AppInfo | null>(null);
const dbError = ref("");
onMounted(async () => {
  if (!inTauri()) return;
  try {
    info.value = await api.appInfo();
  } catch (e) {
    dbError.value = e instanceof Error ? e.message : String(e);
  }
});
</script>

<template>
  <footer class="flex h-6 shrink-0 items-center gap-3 border-t border-line bg-panel px-3 text-[11px] text-faint">
    <Tooltip :label="info ? `Local database, schema v${info.schemaVersion} · ${info.dataDir}` : dbError || 'Starting…'" side="top">
      <span class="flex items-center gap-1.5" :class="dbError && 'text-danger'">
        <Database class="h-3 w-3" />
        {{ dbError ? "Database error" : info ? "Offline · local data" : "…" }}
      </span>
    </Tooltip>
    <span>{{ tabs.tabs.length }} {{ tabs.tabs.length === 1 ? "tab" : "tabs" }}</span>
    <span class="flex-1" />
    <span v-if="info">ENPost {{ info.version }} · {{ info.platform }}</span>
  </footer>
</template>
