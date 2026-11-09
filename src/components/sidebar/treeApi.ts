import type { InjectionKey } from "vue";
import type { MenuEntry } from "@/components/ui/ContextMenu.vue";
import type { MoveTarget, RequestSummary } from "@/types";
import type { FolderView } from "@/utils/tree";
import type { Dragged } from "./treeDrag";

/** What tree rows need from the sidebar (provided once, used at every depth). */
export interface TreeApi {
  isOpen(id: string): boolean;
  toggle(id: string): void;
  isActive(requestId: string): boolean;
  openRequest(id: string): void;
  move(d: Dragged, target: MoveTarget): Promise<void>;
  folderMenu(f: FolderView): MenuEntry[];
  requestMenu(r: RequestSummary): MenuEntry[];
}

export const TREE_KEY: InjectionKey<TreeApi> = Symbol("tree");
