import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { api, inTauri } from "@/services/ipc";
import type { ItemKind, MoveTarget, Workspace, WorkspaceTree } from "@/types";
import { useSettings } from "./settings";
import { useTabs } from "./tabs";
import { useUi } from "./ui";

const EMPTY: WorkspaceTree = { workspaceId: "", collections: [], folders: [], requests: [] };

/** The active workspace and its collection tree (always refreshed from the backend). */
export const useWorkspace = defineStore("workspace", () => {
  const workspaces = ref<Workspace[]>([]);
  const tree = ref<WorkspaceTree>(EMPTY);
  const activeId = computed(() => tree.value.workspaceId);
  const active = computed(() => workspaces.value.find((w) => w.id === activeId.value) ?? null);

  async function load() {
    if (!inTauri()) return;
    workspaces.value = await api.listWorkspaces();
    const wanted = useSettings().settings.activeWorkspaceId;
    const id = workspaces.value.some((w) => w.id === wanted) ? wanted : workspaces.value[0]?.id;
    if (id) tree.value = await api.workspaceTree(id);
  }

  async function refresh() {
    if (inTauri() && activeId.value) tree.value = await api.workspaceTree(activeId.value);
    useTabs().syncWithTree(tree.value.requests);
  }

  async function switchTo(id: string) {
    tree.value = await api.workspaceTree(id);
    void useSettings().update({ activeWorkspaceId: id });
  }

  /** Run a change and refresh the tree; errors become a toast. Returns the change's result. */
  async function change<T>(fn: () => Promise<T>, failTitle: string): Promise<T | undefined> {
    try {
      const r = await fn();
      await refresh();
      return r;
    } catch (e) {
      useUi().toast({ kind: "error", title: failTitle, detail: e instanceof Error ? e.message : String(e) });
      return undefined;
    }
  }

  const createCollection = (name: string) => change(() => api.createCollection(activeId.value, name), "Could not create the collection");
  const createFolder = (collectionId: string, parentId: string | null, name: string) => change(() => api.createFolder(collectionId, parentId, name), "Could not create the folder");
  const rename = (kind: ItemKind, id: string, name: string) => change(() => api.renameItem(kind, id, name), "Could not rename");
  const remove = (kind: ItemKind, id: string) => change(() => api.deleteItem(kind, id), "Could not delete");
  const setFavorite = (id: string, favorite: boolean) => change(() => api.setFavorite(id, favorite), "Could not update favourites");
  const move = (kind: ItemKind, id: string, target: MoveTarget) => change(() => api.moveItem(kind, id, target), "Could not move");
  const duplicate = (id: string) => change(() => api.duplicateRequest(id), "Could not duplicate");

  async function createWorkspace(name: string) {
    const w = await change(() => api.createWorkspace(name), "Could not create the workspace");
    if (!w) return;
    workspaces.value = await api.listWorkspaces();
    await switchTo(w.id);
  }
  async function renameWorkspace(id: string, name: string) {
    await change(() => api.renameWorkspace(id, name), "Could not rename the workspace");
    workspaces.value = await api.listWorkspaces();
  }
  async function deleteWorkspace(id: string) {
    const ok = await change(() => api.deleteWorkspace(id).then(() => true), "Could not delete the workspace");
    if (!ok) return;
    workspaces.value = await api.listWorkspaces();
    const next = workspaces.value[0];
    if (next) await switchTo(next.id);
  }

  return { workspaces, tree, activeId, active, load, refresh, switchTo, createCollection, createFolder, rename, remove, setFavorite, move, duplicate, createWorkspace, renameWorkspace, deleteWorkspace };
});
