<script setup lang="ts">
import { CircleAlert, CircleCheck, Info, X } from "lucide-vue-next";
import { useUi } from "@/stores/ui";

const ui = useUi();
</script>

<template>
  <div class="pointer-events-none fixed bottom-9 right-4 z-[120] flex w-[340px] flex-col gap-2" aria-live="polite">
    <div v-for="t in ui.toasts" :key="t.id" class="pointer-events-auto flex gap-2.5 rounded-lg border border-line bg-panel p-3 shadow-pop animate-slide-up" role="status">
      <CircleCheck v-if="t.kind === 'success'" class="mt-0.5 h-4 w-4 shrink-0 text-success" />
      <CircleAlert v-else-if="t.kind === 'error'" class="mt-0.5 h-4 w-4 shrink-0 text-danger" />
      <Info v-else class="mt-0.5 h-4 w-4 shrink-0 text-accent" />
      <div class="min-w-0 flex-1">
        <div class="text-[13px] font-medium">{{ t.title }}</div>
        <div v-if="t.detail" class="mt-0.5 break-words text-xs text-muted">{{ t.detail }}</div>
        <button v-if="t.action" type="button" class="mt-1.5 text-xs font-medium text-accent hover:underline" @click="t.action.run(); ui.dismiss(t.id)">{{ t.action.label }}</button>
      </div>
      <button class="h-5 w-5 shrink-0 rounded text-faint hover:text-fg" aria-label="Dismiss" @click="ui.dismiss(t.id)"><X class="h-3.5 w-3.5" /></button>
    </div>
  </div>
</template>
