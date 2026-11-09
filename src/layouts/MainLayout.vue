<script setup lang="ts">
import { ref, watch } from "vue";
import ActivityBar from "@/components/common/ActivityBar.vue";
import StatusBar from "@/components/common/StatusBar.vue";
import TitleBar from "@/components/common/TitleBar.vue";
import CollectionsSidebar from "@/components/sidebar/CollectionsSidebar.vue";
import ResizeHandle from "@/components/ui/ResizeHandle.vue";
import { useSettings } from "@/stores/settings";

const settings = useSettings();
const width = ref(settings.settings.sidebarWidth);
watch(() => settings.settings.sidebarWidth, (v) => (width.value = v));
const clamp = (v: number) => Math.round(Math.min(520, Math.max(200, v)));
let left = 0;
function onDrag(ev: PointerEvent) {
  width.value = clamp(ev.clientX - left);
}
function onStart(e: PointerEvent) {
  left = (e.currentTarget as HTMLElement).parentElement?.querySelector("aside")?.getBoundingClientRect().left ?? 48;
}
const save = () => void settings.update({ sidebarWidth: width.value });
</script>

<template>
  <div class="flex h-full flex-col">
    <TitleBar />
    <div class="flex min-h-0 flex-1">
      <ActivityBar />
      <template v-if="!settings.settings.sidebarCollapsed">
        <div class="h-full shrink-0" :style="{ width: `${width}px` }"><CollectionsSidebar /></div>
        <ResizeHandle direction="vertical" label="Resize sidebar" @pointerdown.capture="onStart" @drag="onDrag" @step="(d) => (width = clamp(width + d * 16))" @end="save" />
      </template>
      <main class="min-w-0 flex-1"><slot /></main>
    </div>
    <StatusBar />
  </div>
</template>
