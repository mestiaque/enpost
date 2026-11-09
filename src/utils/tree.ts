/** Turns the flat lists from `workspace_tree` into nested nodes for the sidebar. */
import type { CollectionNode, FolderNode, RequestSummary, WorkspaceTree } from "@/types";

export interface FolderView {
  kind: "folder";
  folder: FolderNode;
  folders: FolderView[];
  requests: RequestSummary[];
}
export interface CollectionView {
  kind: "collection";
  collection: CollectionNode;
  folders: FolderView[];
  requests: RequestSummary[];
}

/**
 * Nested view in stored order. With a query, only requests whose name or URL contains every
 * word are kept, plus the folders and collections on their path (and collections or
 * folders whose own name matches, with everything in them).
 */
export function buildTree(t: WorkspaceTree, query = ""): CollectionView[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  const matches = (text: string) => words.every((w) => text.toLowerCase().includes(w));
  const foldersOf = (collectionId: string, parentId: string | null) => t.folders.filter((f) => f.collectionId === collectionId && f.parentId === parentId);
  const requestsOf = (collectionId: string, folderId: string | null) => t.requests.filter((r) => r.collectionId === collectionId && r.folderId === folderId);

  const folderView = (f: FolderNode, keepAll: boolean): FolderView | null => {
    const all = keepAll || (words.length > 0 && matches(f.name));
    const folders = foldersOf(f.collectionId, f.id).map((x) => folderView(x, all)).filter((x): x is FolderView => x !== null);
    const requests = requestsOf(f.collectionId, f.id).filter((r) => !words.length || all || matches(`${r.name} ${r.url}`));
    if (words.length && !all && !folders.length && !requests.length) return null;
    return { kind: "folder", folder: f, folders, requests };
  };

  return t.collections
    .map((c): CollectionView | null => {
      const all = words.length > 0 && matches(c.name);
      const folders = foldersOf(c.id, null).map((f) => folderView(f, all)).filter((x): x is FolderView => x !== null);
      const requests = requestsOf(c.id, null).filter((r) => !words.length || all || matches(`${r.name} ${r.url}`));
      if (words.length && !all && !folders.length && !requests.length) return null;
      return { kind: "collection", collection: c, folders, requests };
    })
    .filter((x): x is CollectionView => x !== null);
}

/** "Collection / Folder / Sub" choices for the Save-as dialog, depth-first. */
export function saveTargets(t: WorkspaceTree): { collectionId: string; folderId: string | null; label: string; depth: number }[] {
  const out: { collectionId: string; folderId: string | null; label: string; depth: number }[] = [];
  const walk = (collectionId: string, parentId: string | null, depth: number) => {
    for (const f of t.folders.filter((x) => x.collectionId === collectionId && x.parentId === parentId)) {
      out.push({ collectionId, folderId: f.id, label: f.name, depth });
      walk(collectionId, f.id, depth + 1);
    }
  };
  for (const c of t.collections) {
    out.push({ collectionId: c.id, folderId: null, label: c.name, depth: 0 });
    walk(c.id, null, 1);
  }
  return out;
}
