/**
 * Drag and drop in the collection tree. One drag at a time; drop targets read it here.
 * Folders and requests are ordered separately, so "before/after" only applies between
 * items of the same kind; dropping onto a collection or folder moves into it.
 */
import { ref } from "vue";
import type { ItemKind, MoveTarget } from "@/types";

export interface Dragged {
  kind: ItemKind;
  id: string;
  collectionId: string | null;
  folderId: string | null;
  /** Position among its siblings. */
  index: number;
}

export const dragged = ref<Dragged | null>(null);
/** Row currently hovered while dragging, and where the drop would go. */
export const dropHint = ref<{ id: string; where: "before" | "after" | "into" } | null>(null);

export function zone(e: DragEvent, allowInto: boolean): "before" | "after" | "into" {
  const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const y = (e.clientY - r.top) / r.height;
  if (!allowInto) return y < 0.5 ? "before" : "after";
  return y < 0.25 ? "before" : y > 0.75 ? "after" : "into";
}

/**
 * Target for dropping `d` next to a sibling at `index` in the parent (collectionId, folderId).
 * The backend removes the item first, so moving down within the same parent shifts by one.
 */
export function besideTarget(d: Dragged, collectionId: string | null, folderId: string | null, index: number, where: "before" | "after"): MoveTarget {
  let at = where === "before" ? index : index + 1;
  if (d.collectionId === collectionId && d.folderId === folderId && d.index < at) at -= 1;
  return { collectionId, folderId, index: at };
}

export const intoTarget = (collectionId: string | null, folderId: string | null): MoveTarget => ({ collectionId, folderId, index: 1_000_000 });
