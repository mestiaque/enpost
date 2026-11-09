<script setup lang="ts">
/** Folders, then requests, of one parent (a collection root or a folder). Recursive. */
import { ChevronRight, Folder, FolderOpen, Star } from "lucide-vue-next";
import { inject } from "vue";
import MethodBadge from "@/components/request/MethodBadge.vue";
import ContextMenu from "@/components/ui/ContextMenu.vue";
import { TREE_KEY, type TreeApi } from "./treeApi";
import { besideTarget, dragged, dropHint, intoTarget, zone } from "./treeDrag";
import type { HttpMethod, RequestSummary } from "@/types";
import type { FolderView } from "@/utils/tree";

const props = defineProps<{ collectionId: string; folderId: string | null; folders: FolderView[]; requests: RequestSummary[]; depth: number }>();
const api = inject(TREE_KEY) as TreeApi;
const pad = (extra = 0) => ({ paddingLeft: `${10 + (props.depth + extra) * 14}px` });

function onFolderDragOver(e: DragEvent, f: FolderView, index: number) {
  const d = dragged.value;
  if (!d || d.kind === "collection" || d.id === f.folder.id) return;
  e.preventDefault();
  const where = d.kind === "folder" ? zone(e, true) : "into";
  dropHint.value = { id: f.folder.id, where };
  void index;
}
function onFolderDrop(f: FolderView, index: number) {
  const d = dragged.value;
  const h = dropHint.value;
  if (!d || !h) return;
  if (h.where === "into" || d.kind === "request") void api.move(d, intoTarget(props.collectionId, f.folder.id));
  else void api.move(d, besideTarget(d, props.collectionId, props.folderId, index, h.where));
  dragged.value = dropHint.value = null;
}
function onRequestDragOver(e: DragEvent, r: RequestSummary) {
  const d = dragged.value;
  if (!d || d.kind === "collection" || d.id === r.id) return;
  e.preventDefault();
  dropHint.value = { id: r.id, where: d.kind === "request" ? zone(e, false) : "into" };
}
function onRequestDrop(index: number) {
  const d = dragged.value;
  const h = dropHint.value;
  if (!d || !h) return;
  // A folder dropped on a request goes into that request's parent.
  if (d.kind === "folder" || h.where === "into") void api.move(d, intoTarget(props.collectionId, props.folderId));
  else void api.move(d, besideTarget(d, props.collectionId, props.folderId, index, h.where));
  dragged.value = dropHint.value = null;
}
const hintClass = (id: string) =>
  dropHint.value?.id !== id ? "" : dropHint.value.where === "into" ? "bg-accent/15 ring-1 ring-inset ring-accent/60" : dropHint.value.where === "before" ? "shadow-[inset_0_2px_0_rgb(var(--accent))]" : "shadow-[inset_0_-2px_0_rgb(var(--accent))]";
</script>

<template>
  <template v-for="(f, i) in folders" :key="f.folder.id">
    <ContextMenu :entries="api.folderMenu(f)">
      <div
        role="treeitem"
        :aria-expanded="api.isOpen(f.folder.id)"
        tabindex="0"
        draggable="true"
        :class="['group flex h-7 cursor-default items-center gap-1.5 pr-2 text-[13px] text-muted hover:bg-raised/60 hover:text-fg', hintClass(f.folder.id)]"
        :style="pad()"
        @click="api.toggle(f.folder.id)"
        @keydown.enter="api.toggle(f.folder.id)"
        @dragstart.stop="dragged = { kind: 'folder', id: f.folder.id, collectionId, folderId, index: i }"
        @dragover.stop="onFolderDragOver($event, f, i)"
        @dragleave="dropHint?.id === f.folder.id && (dropHint = null)"
        @drop.stop.prevent="onFolderDrop(f, i)"
      >
        <ChevronRight :class="['h-3.5 w-3.5 shrink-0 text-faint transition-transform', api.isOpen(f.folder.id) && 'rotate-90']" />
        <component :is="api.isOpen(f.folder.id) ? FolderOpen : Folder" class="h-3.5 w-3.5 shrink-0 text-amber-500/80" />
        <span class="truncate">{{ f.folder.name }}</span>
      </div>
    </ContextMenu>
    <TreeLevel v-if="api.isOpen(f.folder.id)" :collection-id="collectionId" :folder-id="f.folder.id" :folders="f.folders" :requests="f.requests" :depth="depth + 1" />
  </template>

  <ContextMenu v-for="(r, i) in requests" :key="r.id" :entries="api.requestMenu(r)">
    <div
      role="treeitem"
      tabindex="0"
      draggable="true"
      :class="[
        'group flex h-7 cursor-default items-center gap-2 pr-2 text-[13px] hover:bg-raised/60',
        api.isActive(r.id) ? 'bg-accent/15 text-fg' : 'text-muted hover:text-fg',
        hintClass(r.id),
      ]"
      :style="pad(1)"
      :title="r.url || r.name"
      @click="api.openRequest(r.id)"
      @keydown.enter="api.openRequest(r.id)"
      @dragstart.stop="dragged = { kind: 'request', id: r.id, collectionId, folderId, index: i }"
      @dragover.stop="onRequestDragOver($event, r)"
      @dragleave="dropHint?.id === r.id && (dropHint = null)"
      @drop.stop.prevent="onRequestDrop(i)"
    >
      <span class="w-8 shrink-0 text-right"><MethodBadge :method="(r.method as HttpMethod)" /></span>
      <span class="truncate">{{ r.name }}</span>
      <Star v-if="r.favorite" class="ml-auto h-3 w-3 shrink-0 fill-amber-400 text-amber-400" />
    </div>
  </ContextMenu>
</template>
