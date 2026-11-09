<script setup lang="ts">
import { FolderTree, Plus } from "lucide-vue-next";
import Button from "@/components/ui/Button.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import { useTabs } from "@/stores/tabs";

const tabs = useTabs();
</script>

<template>
  <aside class="flex h-full min-w-0 flex-col bg-panel" aria-label="Collections">
    <div class="flex h-10 shrink-0 items-center justify-between px-3">
      <span class="text-[11px] font-semibold uppercase tracking-wider text-faint">Collections</span>
    </div>
    <EmptyState title="No collections yet" badge="Phase 3" text="Collections, folders and saved requests come in Phase 3. Until then, request tabs live for this session.">
      <template #icon><FolderTree class="h-5 w-5" /></template>
      <Button size="sm" class="mt-2" @click="tabs.open()"><Plus class="h-3.5 w-3.5" />New request tab</Button>
    </EmptyState>
  </aside>
</template>
