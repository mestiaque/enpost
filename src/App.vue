<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TooltipProvider } from "reka-ui";
import { onMounted } from "vue";
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
  useTabs().open();
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
    <ConfirmHost />
    <PromptHost />
    <Toaster />
  </TooltipProvider>
</template>
