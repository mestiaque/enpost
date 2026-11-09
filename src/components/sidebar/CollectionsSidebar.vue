<script setup lang="ts">
import { ChevronRight, FolderPlus, FolderTree, MoreHorizontal, Plus, Search, Star, X } from "lucide-vue-next";
import { DropdownMenuContent, DropdownMenuItem, DropdownMenuPortal, DropdownMenuRoot, DropdownMenuSeparator, DropdownMenuTrigger } from "reka-ui";
import { computed, provide, ref } from "vue";
import MethodBadge from "@/components/request/MethodBadge.vue";
import Button from "@/components/ui/Button.vue";
import ContextMenu, { type MenuEntry } from "@/components/ui/ContextMenu.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import IconButton from "@/components/ui/IconButton.vue";
import TreeLevel from "./TreeLevel.vue";
import { TREE_KEY, type TreeApi } from "./treeApi";
import { besideTarget, dragged, dropHint, intoTarget, zone, type Dragged } from "./treeDrag";
import { api as ipc } from "@/services/ipc";
import { newDraft, useTabs } from "@/stores/tabs";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";
import { draftToDefinition } from "@/services/requestMapping";
import type { HttpMethod, MoveTarget, RequestSummary } from "@/types";
import { buildTree, type CollectionView, type FolderView } from "@/utils/tree";

const ws = useWorkspace();
const tabs = useTabs();
const ui = useUi();
const query = ref("");
const view = computed(() => buildTree(ws.tree, query.value));
const favorites = computed(() => ws.tree.requests.filter((r) => r.favorite));

// Expanded nodes: collections start open; everything opens while searching.
const closed = ref(new Set<string>());
const opened = ref(new Set<string>());
const isCollectionOpen = (id: string) => query.value !== "" || !closed.value.has(id);
const isFolderOpen = (id: string) => query.value !== "" || opened.value.has(id);
function toggleSet(set: typeof closed, id: string) {
  const next = new Set(set.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  set.value = next;
}

const toggleCollection = (id: string) => toggleSet(closed, id);

async function newRequest(collectionId: string, folderId: string | null) {
  const created = await ipc.createRequest({ collectionId, folderId, name: "New request", method: "GET", url: "", definition: draftToDefinition(newDraft()) }).catch((e) => {
    ui.toast({ kind: "error", title: "Could not create the request", detail: e instanceof Error ? e.message : String(e) });
    return null;
  });
  if (!created) return;
  if (folderId) opened.value = new Set([...opened.value, folderId]);
  await ws.refresh();
  await tabs.openSaved(created.id);
}
async function newFolder(collectionId: string, parentId: string | null) {
  const name = await ui.prompt({ title: "New folder", confirmLabel: "Create" });
  if (!name) return;
  const id = await ws.createFolder(collectionId, parentId, name);
  if (id && parentId) opened.value = new Set([...opened.value, parentId]);
}
async function newCollection() {
  const name = await ui.prompt({ title: "New collection", confirmLabel: "Create" });
  if (name) await ws.createCollection(name);
}
async function rename(kind: "collection" | "folder" | "request", id: string, current: string) {
  const name = await ui.prompt({ title: `Rename ${kind}`, initial: current, confirmLabel: "Rename" });
  if (name && name !== current) await ws.rename(kind, id, name);
}
async function remove(kind: "collection" | "folder" | "request", id: string, name: string) {
  const what = kind === "request" ? "This request is deleted." : `Everything inside "${name}" is deleted too.`;
  if (await ui.confirm({ title: `Delete ${kind} "${name}"?`, message: what, confirmLabel: "Delete", danger: true })) await ws.remove(kind, id);
}

function collectionMenu(c: CollectionView): MenuEntry[] {
  return [
    { label: "New request", run: () => void newRequest(c.collection.id, null) },
    { label: "New folder", run: () => void newFolder(c.collection.id, null) },
    { separator: true },
    { label: "Rename…", run: () => void rename("collection", c.collection.id, c.collection.name) },
    { label: "Delete…", danger: true, run: () => void remove("collection", c.collection.id, c.collection.name) },
  ];
}

const treeApi: TreeApi = {
  isOpen: isFolderOpen,
  toggle: (id) => toggleSet(opened, id),
  isActive: (id) => tabs.active?.savedId === id,
  openRequest: (id) => void tabs.openSaved(id),
  async move(d: Dragged, target: MoveTarget) {
    await ws.move(d.kind, d.id, target);
    if (target.folderId) opened.value = new Set([...opened.value, target.folderId]);
  },
  folderMenu: (f: FolderView) => [
    { label: "New request", run: () => void newRequest(f.folder.collectionId, f.folder.id) },
    { label: "New folder", run: () => void newFolder(f.folder.collectionId, f.folder.id) },
    { separator: true },
    { label: "Rename…", run: () => void rename("folder", f.folder.id, f.folder.name) },
    { label: "Delete…", danger: true, run: () => void remove("folder", f.folder.id, f.folder.name) },
  ],
  requestMenu: (r: RequestSummary) => [
    { label: "Open", run: () => void tabs.openSaved(r.id) },
    { label: "Duplicate", run: () => void ws.duplicate(r.id) },
    { label: r.favorite ? "Remove from favourites" : "Add to favourites", run: () => void ws.setFavorite(r.id, !r.favorite) },
    { separator: true },
    { label: "Rename…", run: () => void rename("request", r.id, r.name) },
    { label: "Delete…", danger: true, run: () => void remove("request", r.id, r.name) },
  ],
};
provide(TREE_KEY, treeApi);

// Collections: reorder by dragging; folders and requests dropped on one go to its root.
function onCollectionDragOver(e: DragEvent, id: string) {
  const d = dragged.value;
  if (!d || d.id === id) return;
  e.preventDefault();
  dropHint.value = { id, where: d.kind === "collection" ? zone(e, false) : "into" };
}
function onCollectionDrop(c: CollectionView, index: number) {
  const d = dragged.value;
  const h = dropHint.value;
  if (!d || !h) return;
  if (d.kind === "collection") void ws.move("collection", d.id, besideTarget(d, null, null, index, h.where as "before" | "after"));
  else void ws.move(d.kind, d.id, intoTarget(c.collection.id, null));
  dragged.value = dropHint.value = null;
}
const hintClass = (id: string) =>
  dropHint.value?.id !== id ? "" : dropHint.value.where === "into" ? "bg-accent/15 ring-1 ring-inset ring-accent/60" : dropHint.value.where === "before" ? "shadow-[inset_0_2px_0_rgb(var(--accent))]" : "shadow-[inset_0_-2px_0_rgb(var(--accent))]";

async function workspaceAction(a: "new" | "rename" | "delete") {
  const w = ws.active;
  if (a === "new") {
    const name = await ui.prompt({ title: "New workspace", confirmLabel: "Create" });
    if (name) await ws.createWorkspace(name);
  } else if (a === "rename" && w) {
    const name = await ui.prompt({ title: "Rename workspace", initial: w.name, confirmLabel: "Rename" });
    if (name && name !== w.name) await ws.renameWorkspace(w.id, name);
  } else if (a === "delete" && w) {
    if (await ui.confirm({ title: `Delete workspace "${w.name}"?`, message: "All its collections, folders and requests are deleted.", confirmLabel: "Delete", danger: true })) await ws.deleteWorkspace(w.id);
  }
}
const menuItem = "flex cursor-default items-center rounded px-2 py-1.5 text-[13px] outline-none data-[disabled]:opacity-40 data-[highlighted]:bg-raised";
</script>

<template>
  <aside class="flex h-full min-w-0 flex-col bg-panel" aria-label="Collections" @dragend="dragged = dropHint = null">
    <!-- Workspace -->
    <div class="flex h-10 shrink-0 items-center gap-1 pl-2 pr-1.5">
      <select
        :value="ws.activeId"
        class="h-7 min-w-0 flex-1 truncate rounded-md bg-transparent px-1.5 text-[12px] font-semibold uppercase tracking-wider text-faint outline-none hover:bg-raised hover:text-fg"
        aria-label="Workspace"
        @change="ws.switchTo(($event.target as HTMLSelectElement).value)"
      >
        <option v-for="w in ws.workspaces" :key="w.id" :value="w.id" class="normal-case text-fg">{{ w.name }}</option>
      </select>
      <DropdownMenuRoot>
        <DropdownMenuTrigger as-child>
          <button type="button" class="flex h-7 w-7 items-center justify-center rounded-md text-muted hover:bg-raised hover:text-fg" aria-label="Workspace actions"><MoreHorizontal class="h-4 w-4" /></button>
        </DropdownMenuTrigger>
        <DropdownMenuPortal>
          <DropdownMenuContent align="end" :side-offset="4" class="z-[90] min-w-[180px] rounded-lg border border-line bg-panel p-1 shadow-pop data-[state=open]:animate-fade-in">
            <DropdownMenuItem :class="menuItem" @select="workspaceAction('new')">New workspace…</DropdownMenuItem>
            <DropdownMenuItem :class="menuItem" @select="workspaceAction('rename')">Rename workspace…</DropdownMenuItem>
            <DropdownMenuSeparator class="my-1 h-px bg-line" />
            <DropdownMenuItem :class="[menuItem, 'text-danger']" :disabled="ws.workspaces.length < 2" @select="workspaceAction('delete')">Delete workspace…</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenuPortal>
      </DropdownMenuRoot>
      <IconButton label="New collection" @click="newCollection"><Plus class="h-4 w-4" /></IconButton>
    </div>

    <!-- Search -->
    <div class="relative shrink-0 px-2 pb-2">
      <Search class="pointer-events-none absolute left-4 top-2 h-3.5 w-3.5 text-faint" />
      <input v-model="query" class="h-7 w-full rounded-md border border-line bg-bg pl-7 pr-7 text-[12px] outline-none placeholder:text-faint focus:border-accent" placeholder="Search requests" spellcheck="false" aria-label="Search requests" />
      <button v-if="query" type="button" class="absolute right-3.5 top-1.5 rounded p-0.5 text-faint hover:text-fg" aria-label="Clear search" @click="query = ''"><X class="h-3.5 w-3.5" /></button>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto pb-3" role="tree" aria-label="Collections">
      <!-- Favourites -->
      <template v-if="favorites.length && !query">
        <div class="flex items-center gap-1.5 px-3 pb-1 pt-1 text-[10px] font-semibold uppercase tracking-wider text-faint"><Star class="h-3 w-3" />Favourites</div>
        <button
          v-for="r in favorites"
          :key="`fav-${r.id}`"
          type="button"
          :class="['flex h-7 w-full items-center gap-2 pl-3 pr-2 text-left text-[13px] hover:bg-raised/60', tabs.active?.savedId === r.id ? 'bg-accent/15 text-fg' : 'text-muted hover:text-fg']"
          @click="tabs.openSaved(r.id)"
        >
          <span class="w-8 shrink-0 text-right"><MethodBadge :method="(r.method as HttpMethod)" /></span>
          <span class="truncate">{{ r.name }}</span>
        </button>
        <div class="mx-3 my-2 h-px bg-line" />
      </template>

      <EmptyState v-if="!ws.tree.collections.length" title="No collections yet" text="Collections hold saved requests in folders. Save the current request with Ctrl+S, or create a collection.">
        <template #icon><FolderTree class="h-5 w-5" /></template>
        <Button size="sm" class="mt-2" @click="newCollection"><FolderPlus class="h-3.5 w-3.5" />New collection</Button>
      </EmptyState>
      <p v-else-if="!view.length" class="px-4 py-6 text-center text-xs text-faint">Nothing matches “{{ query }}”</p>

      <template v-for="(c, i) in view" :key="c.collection.id">
        <ContextMenu :entries="collectionMenu(c)">
          <div
            role="treeitem"
            :aria-expanded="isCollectionOpen(c.collection.id)"
            tabindex="0"
            draggable="true"
            :class="['group flex h-8 cursor-default items-center gap-1.5 pl-2 pr-1.5 text-[13px] font-medium hover:bg-raised/60', hintClass(c.collection.id)]"
            @click="toggleCollection(c.collection.id)"
            @keydown.enter="toggleCollection(c.collection.id)"
            @dragstart="dragged = { kind: 'collection', id: c.collection.id, collectionId: null, folderId: null, index: i }"
            @dragover="onCollectionDragOver($event, c.collection.id)"
            @dragleave="dropHint?.id === c.collection.id && (dropHint = null)"
            @drop.prevent="onCollectionDrop(c, i)"
          >
            <ChevronRight :class="['h-3.5 w-3.5 shrink-0 text-faint transition-transform', isCollectionOpen(c.collection.id) && 'rotate-90']" />
            <span class="truncate">{{ c.collection.name }}</span>
            <span class="ml-auto hidden shrink-0 gap-0.5 group-hover:flex">
              <button type="button" class="rounded p-1 text-faint hover:bg-line hover:text-fg" title="New request" @click.stop="newRequest(c.collection.id, null)"><Plus class="h-3.5 w-3.5" /></button>
              <button type="button" class="rounded p-1 text-faint hover:bg-line hover:text-fg" title="New folder" @click.stop="newFolder(c.collection.id, null)"><FolderPlus class="h-3.5 w-3.5" /></button>
            </span>
          </div>
        </ContextMenu>
        <div v-if="isCollectionOpen(c.collection.id)">
          <TreeLevel :collection-id="c.collection.id" :folder-id="null" :folders="c.folders" :requests="c.requests" :depth="0" />
          <p v-if="!c.folders.length && !c.requests.length" class="py-1.5 pl-8 text-[11px] text-faint">Empty. Right-click to add a request or folder.</p>
        </div>
      </template>
    </div>
  </aside>
</template>
