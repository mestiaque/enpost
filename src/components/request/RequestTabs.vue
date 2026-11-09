<script setup lang="ts">
import { ContextMenuContent, ContextMenuItem, ContextMenuPortal, ContextMenuRoot, ContextMenuSeparator, ContextMenuTrigger } from "reka-ui";
import { Plus, X } from "lucide-vue-next";
import { ref } from "vue";
import IconButton from "@/components/ui/IconButton.vue";
import MethodBadge from "./MethodBadge.vue";
import { useTabs } from "@/stores/tabs";

const tabs = useTabs();
const dragFrom = ref<number | null>(null);
const dropAt = ref<number | null>(null);

function onDragStart(e: DragEvent, i: number) {
  dragFrom.value = i;
  e.dataTransfer?.setData("text/plain", String(i));
  if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
}
function onDragOver(e: DragEvent, i: number) {
  if (dragFrom.value === null) return;
  e.preventDefault();
  const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
  dropAt.value = e.clientX < r.left + r.width / 2 ? i : i + 1;
}
function onDrop() {
  if (dragFrom.value !== null && dropAt.value !== null) {
    const to = dropAt.value > dragFrom.value ? dropAt.value - 1 : dropAt.value;
    tabs.move(dragFrom.value, to);
  }
  dragFrom.value = dropAt.value = null;
}
const item = "flex cursor-default items-center rounded px-2 py-1.5 text-[13px] outline-none data-[highlighted]:bg-raised";
</script>

<template>
  <div class="flex h-9 shrink-0 items-end border-b border-line bg-panel" role="tablist" aria-label="Request tabs">
    <div class="flex min-w-0 flex-1 items-end overflow-x-auto" @dragend="dragFrom = dropAt = null">
      <ContextMenuRoot v-for="(t, i) in tabs.tabs" :key="t.id">
        <ContextMenuTrigger as-child>
          <div
            role="tab"
            :aria-selected="t.id === tabs.activeId"
            tabindex="0"
            draggable="true"
            :class="[
              'group relative flex h-9 min-w-[120px] max-w-[220px] shrink-0 cursor-default items-center gap-2 border-r border-line px-3 text-[12px]',
              t.id === tabs.activeId ? 'bg-bg text-fg' : 'text-muted hover:bg-raised/50 hover:text-fg',
              dropAt === i && 'shadow-[inset_2px_0_0_rgb(var(--accent))]',
              dropAt === i + 1 && i === tabs.tabs.length - 1 && 'shadow-[inset_-2px_0_0_rgb(var(--accent))]',
            ]"
            :title="t.draft.url || t.name"
            @mousedown="tabs.activate(t.id)"
            @auxclick.prevent="$event.button === 1 && tabs.requestClose(t.id)"
            @keydown.enter="tabs.activate(t.id)"
            @dragstart="onDragStart($event, i)"
            @dragover="onDragOver($event, i)"
            @drop.prevent="onDrop"
          >
            <span v-if="t.id === tabs.activeId" class="absolute inset-x-0 top-0 h-0.5 bg-accent" />
            <MethodBadge :method="t.draft.method" />
            <span class="min-w-0 flex-1 truncate">{{ t.name }}</span>
            <span v-if="t.dirty" class="h-1.5 w-1.5 shrink-0 rounded-full bg-muted group-hover:hidden" aria-label="Unsaved changes" />
            <button
              type="button"
              :class="['h-4 w-4 shrink-0 items-center justify-center rounded text-faint hover:bg-line hover:text-fg', t.dirty ? 'hidden group-hover:flex' : 'flex opacity-0 group-hover:opacity-100', t.id === tabs.activeId && !t.dirty && 'opacity-100']"
              aria-label="Close tab"
              @mousedown.stop
              @click.stop="tabs.requestClose(t.id)"
            >
              <X class="h-3 w-3" />
            </button>
          </div>
        </ContextMenuTrigger>
        <ContextMenuPortal>
          <ContextMenuContent class="z-[90] min-w-[180px] rounded-lg border border-line bg-panel p-1 shadow-pop data-[state=open]:animate-fade-in">
            <ContextMenuItem :class="item" @select="tabs.duplicate(t.id)">Duplicate</ContextMenuItem>
            <ContextMenuSeparator class="my-1 h-px bg-line" />
            <ContextMenuItem :class="item" @select="tabs.requestClose(t.id)">Close<span class="ml-auto pl-4 text-[11px] text-faint">Ctrl+W</span></ContextMenuItem>
            <ContextMenuItem :class="item" :disabled="tabs.tabs.length < 2" @select="tabs.requestCloseOthers(t.id)">Close other tabs</ContextMenuItem>
          </ContextMenuContent>
        </ContextMenuPortal>
      </ContextMenuRoot>
    </div>
    <IconButton class="mx-1 mb-0.5 h-7 w-7 shrink-0" label="New request" shortcut="Ctrl+T" @click="tabs.open()"><Plus class="h-4 w-4" /></IconButton>
  </div>
</template>
