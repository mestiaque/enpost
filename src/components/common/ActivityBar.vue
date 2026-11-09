<script setup lang="ts">
import { FlaskConical, FolderTree, History, Layers, Settings as Cog, Wrench } from "lucide-vue-next";
import IconButton from "@/components/ui/IconButton.vue";
import { useUi } from "@/stores/ui";

const ui = useUi();
/** Areas of the app. Only built areas are enabled; the rest say which phase brings them. */
const ITEMS = [
  { id: "collections", label: "Collections", icon: FolderTree, ready: true },
  { id: "environments", label: "Environments (Phase 3)", icon: Layers, ready: false },
  { id: "history", label: "History (Phase 2)", icon: History, ready: false },
  { id: "runner", label: "Collection runner (Phase 4)", icon: FlaskConical, ready: false },
  { id: "utilities", label: "Developer utilities (planned)", icon: Wrench, ready: false },
] as const;
</script>

<template>
  <nav class="flex w-12 shrink-0 flex-col items-center gap-1 border-r border-line bg-panel py-2" aria-label="Activity bar">
    <IconButton v-for="it in ITEMS" :key="it.id" :label="it.label" side="right" :active="it.id === 'collections'" :disabled="!it.ready">
      <component :is="it.icon" class="h-[18px] w-[18px]" />
    </IconButton>
    <div class="flex-1" />
    <IconButton label="Settings" shortcut="Ctrl+," side="right" @click="ui.openSettings()"><Cog class="h-[18px] w-[18px]" /></IconButton>
  </nav>
</template>
