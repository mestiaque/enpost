<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-vue-next";
import { onBeforeUnmount, onMounted, ref } from "vue";
import { inTauri } from "@/services/ipc";

/** Minimise / maximise / close for NEPost's frameless window (same as the sister apps). */
const maximized = ref(false);
let unlisten: (() => void) | null = null;
const win = () => getCurrentWindow();
const refresh = () => win().isMaximized().then((m) => (maximized.value = m), () => {});

onMounted(async () => {
  if (!inTauri()) return;
  void refresh();
  unlisten = await win().onResized(() => void refresh());
});
onBeforeUnmount(() => unlisten?.());

const btn = "flex h-6 w-6 items-center justify-center rounded-full bg-raised text-muted transition-colors hover:bg-line hover:text-fg";
</script>

<template>
  <div class="ml-1 flex items-center gap-2 pl-1" data-no-drag @dblclick.stop>
    <button :class="btn" aria-label="Minimize" title="Minimize" @click="win().minimize()"><Minus class="h-3.5 w-3.5" /></button>
    <button :class="btn" :aria-label="maximized ? 'Restore' : 'Maximize'" :title="maximized ? 'Restore' : 'Maximize'" @click="win().toggleMaximize()">
      <Copy v-if="maximized" class="h-3 w-3 -scale-x-100" />
      <Square v-else class="h-3 w-3" />
    </button>
    <button :class="[btn, 'hover:!bg-danger hover:!text-white']" aria-label="Close" title="Close" @click="win().close()"><X class="h-3.5 w-3.5" /></button>
  </div>
</template>
