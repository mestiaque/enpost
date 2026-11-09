<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TooltipProvider } from "reka-ui";
import { onMounted, watch } from "vue";
import UpdateDialog from "@/components/update/UpdateDialog.vue";
import { scheduleUpdateCheck } from "@/stores/update";
import CommandPalette from "@/components/common/CommandPalette.vue";
import SaveAsDialog from "@/components/request/SaveAsDialog.vue";
import SettingsDialog from "@/components/settings/SettingsDialog.vue";
import ConfirmHost from "@/components/ui/ConfirmHost.vue";
import PromptHost from "@/components/ui/PromptHost.vue";
import Toaster from "@/components/ui/Toaster.vue";
import { useShortcuts } from "@/composables/useShortcuts";
import { useTheme } from "@/composables/useTheme";
import MainLayout from "@/layouts/MainLayout.vue";
import { inTauri } from "@/services/ipc";
import { useSettings } from "@/stores/settings";
import { useTabs } from "@/stores/tabs";
import { useWorkspace } from "@/stores/workspace";

const settings = useSettings();
useTheme();
useShortcuts();

onMounted(async () => {
  await settings.load();
  await useWorkspace().load().catch((e) => console.error("workspace load failed", e));
  const tabs = useTabs();
  if (settings.settings.startup !== "restore" || !tabs.restoreSession()) tabs.open();
  // Remember open tabs (debounced), for "Reopen the tabs from last time".
  let t: number | undefined;
  watch(
    () => tabs.tabs.map((x) => [x.id, x.name, x.dirty, x.savedId, x.draft]),
    () => {
      window.clearTimeout(t);
      t = window.setTimeout(() => tabs.saveSession(), 500);
    },
    { deep: true },
  );
  watch(() => tabs.activeId, () => tabs.saveSession());
  // Always on top (Settings → General).
  watch(
    () => settings.settings.alwaysOnTop,
    (on) => inTauri() && void getCurrentWindow().setAlwaysOnTop(on).catch(() => {}),
    { immediate: true },
  );
  // Quiet update check about once a day.
  let cancelCheck = scheduleUpdateCheck(settings.settings.checkUpdates);
  watch(() => settings.settings.checkUpdates, (on) => {
    cancelCheck();
    cancelCheck = scheduleUpdateCheck(on);
  });
  // The window starts hidden so it never flashes unstyled; show it once the theme is applied.
  if (inTauri()) await getCurrentWindow().show();
});

// Keep the native context menu out of the app, except where text is edited.
window.addEventListener("contextmenu", (e) => {
  const t = e.target as HTMLElement;
  if (t.tagName !== "INPUT" && t.tagName !== "TEXTAREA") e.preventDefault();
});
</script>

<template>
  <TooltipProvider>
    <MainLayout v-if="settings.loaded"><RouterView /></MainLayout>
    <div v-else class="h-full bg-bg" />
    <SettingsDialog />
    <CommandPalette />
    <SaveAsDialog />
    <UpdateDialog />
    <ConfirmHost />
    <PromptHost />
    <Toaster />
  </TooltipProvider>
</template>
