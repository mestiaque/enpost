<script setup lang="ts">
import { FilePlus2 } from "lucide-vue-next";
import { ref, watch } from "vue";
import RequestEditor from "@/components/request/RequestEditor.vue";
import RequestTabs from "@/components/request/RequestTabs.vue";
import ResponsePanel from "@/components/response/ResponsePanel.vue";
import Button from "@/components/ui/Button.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import Kbd from "@/components/ui/Kbd.vue";
import ResizeHandle from "@/components/ui/ResizeHandle.vue";
import { useSettings } from "@/stores/settings";
import { useTabs } from "@/stores/tabs";

const tabs = useTabs();
const settings = useSettings();
const box = ref<HTMLElement | null>(null);

// Split while dragging is local; it is saved once when the drag ends.
const split = ref(settings.settings.editorSplit);
watch(() => settings.settings.editorSplit, (v) => (split.value = v));
const clamp = (v: number) => Math.min(0.8, Math.max(0.2, v));
function onDrag(ev: PointerEvent) {
  const r = box.value?.getBoundingClientRect();
  if (r && r.height > 0) split.value = clamp((ev.clientY - r.top) / r.height);
}
const save = () => void settings.update({ editorSplit: Math.round(split.value * 1000) / 1000 });
</script>

<template>
  <div class="flex h-full min-w-0 flex-col">
    <RequestTabs />
    <div v-if="tabs.active" ref="box" class="flex min-h-0 flex-1 flex-col bg-bg">
      <div class="min-h-[120px]" :style="{ height: `${split * 100}%` }">
        <RequestEditor :key="tabs.active.id" :tab="tabs.active" />
      </div>
      <ResizeHandle direction="horizontal" label="Resize request and response" @drag="onDrag" @step="(d) => (split = clamp(split + d * 0.05))" @end="save" />
      <div class="min-h-[120px] flex-1"><ResponsePanel :key="tabs.active.id" :tab="tabs.active" /></div>
    </div>
    <div v-else class="flex-1 bg-bg">
      <EmptyState title="No open requests" text="Open a request tab to build a request. Saved collections come in Phase 3.">
        <template #icon><FilePlus2 class="h-5 w-5" /></template>
        <Button variant="primary" size="sm" class="mt-2" @click="tabs.open()">New request</Button>
        <div class="mt-3 grid grid-cols-[auto_auto] gap-x-3 gap-y-1.5 text-left text-xs text-muted">
          <Kbd>Ctrl+T</Kbd><span>New request tab</span>
          <Kbd>Ctrl+K</Kbd><span>Command palette</span>
          <Kbd>Ctrl+,</Kbd><span>Settings</span>
        </div>
      </EmptyState>
    </div>
  </div>
</template>
