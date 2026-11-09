/**
 * Typed bridge to the Rust backend. Every backend call goes through `call`, which maps the
 * command name to its argument and result types, so a wrong name or payload fails to compile.
 * The frontend never performs API requests itself: those run in Rust (Phase 2).
 */
import { invoke } from "@tauri-apps/api/core";
import type {
  AppErrorPayload,
  AppInfo,
  UpdateInfo,
  HistoryEntry,
  HttpRequest,
  HttpResponse,
  ItemKind,
  MoveTarget,
  RequestDefinition,
  RequestPatch,
  SavedRequest,
  Settings,
  SettingsPatch,
  Workspace,
  WorkspaceTree,
} from "@/types";

interface Commands {
  app_info: { args: Record<string, never>; result: AppInfo };
  open_app_folder: { args: { which: "data" | "logs" }; result: null };
  check_update: { args: Record<string, never>; result: UpdateInfo | null };
  install_update: { args: { info: UpdateInfo }; result: "restart" | "exiting" };
  open_release_page: { args: Record<string, never>; result: null };
  restart_app: { args: Record<string, never>; result: null };
  open_link: { args: { which: "website" | "source" | "issues" | "email" }; result: null };
  get_settings: { args: Record<string, never>; result: Settings };
  update_settings: { args: { patch: SettingsPatch }; result: Settings };
  reset_settings: { args: Record<string, never>; result: Settings };
  send_request: { args: { request: HttpRequest }; result: HttpResponse };
  cancel_request: { args: { requestId: string }; result: boolean };
  save_response_body: { args: { historyId: string; destination: string }; result: null };
  list_history: { args: { limit: number | null }; result: HistoryEntry[] };
  clear_history: { args: Record<string, never>; result: null };
  list_workspaces: { args: Record<string, never>; result: Workspace[] };
  create_workspace: { args: { name: string }; result: Workspace };
  rename_workspace: { args: { id: string; name: string }; result: null };
  delete_workspace: { args: { id: string }; result: null };
  workspace_tree: { args: { workspaceId: string }; result: WorkspaceTree };
  create_collection: { args: { workspaceId: string; name: string }; result: string };
  create_folder: { args: { collectionId: string; parentId: string | null; name: string }; result: string };
  create_request: { args: { collectionId: string; folderId: string | null; name: string; method: string; url: string; definition: RequestDefinition }; result: SavedRequest };
  get_request: { args: { id: string }; result: SavedRequest };
  save_request: { args: { id: string; patch: RequestPatch }; result: SavedRequest };
  duplicate_request: { args: { id: string }; result: SavedRequest };
  rename_item: { args: { kind: ItemKind; id: string; name: string }; result: null };
  delete_item: { args: { kind: ItemKind; id: string }; result: null };
  set_favorite: { args: { id: string; favorite: boolean }; result: null };
  move_item: { args: { kind: ItemKind; id: string; target: MoveTarget }; result: null };
}

export type CommandName = keyof Commands;

/** Thrown by `call`; always has a kind and a readable message. */
export class IpcError extends Error {
  constructor(
    public readonly kind: string,
    message: string,
  ) {
    super(message);
    this.name = "IpcError";
  }
}

export function toIpcError(e: unknown): IpcError {
  if (e instanceof IpcError) return e;
  if (e && typeof e === "object" && "kind" in e && "message" in e) {
    const p = e as AppErrorPayload;
    return new IpcError(String(p.kind), String(p.message));
  }
  return new IpcError("internal", typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
}

/** True inside the desktop app (false in unit tests and a plain browser). */
export const inTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function call<K extends CommandName>(cmd: K, ...args: Commands[K]["args"] extends Record<string, never> ? [] : [Commands[K]["args"]]): Promise<Commands[K]["result"]> {
  try {
    return await invoke<Commands[K]["result"]>(cmd, args[0] ?? {});
  } catch (e) {
    throw toIpcError(e);
  }
}

export const api = {
  appInfo: () => call("app_info"),
  checkUpdate: () => call("check_update"),
  installUpdate: (info: UpdateInfo) => call("install_update", { info }),
  openReleasePage: () => call("open_release_page"),
  restartApp: () => call("restart_app"),
  openAppFolder: (which: "data" | "logs") => call("open_app_folder", { which }),
  openLink: (which: "website" | "source" | "issues" | "email") => call("open_link", { which }),
  getSettings: () => call("get_settings"),
  updateSettings: (patch: SettingsPatch) => call("update_settings", { patch }),
  resetSettings: () => call("reset_settings"),
  sendRequest: (request: HttpRequest) => call("send_request", { request }),
  cancelRequest: (requestId: string) => call("cancel_request", { requestId }),
  saveResponseBody: (historyId: string, destination: string) => call("save_response_body", { historyId, destination }),
  listHistory: (limit: number | null = null) => call("list_history", { limit }),
  clearHistory: () => call("clear_history"),
  listWorkspaces: () => call("list_workspaces"),
  createWorkspace: (name: string) => call("create_workspace", { name }),
  renameWorkspace: (id: string, name: string) => call("rename_workspace", { id, name }),
  deleteWorkspace: (id: string) => call("delete_workspace", { id }),
  workspaceTree: (workspaceId: string) => call("workspace_tree", { workspaceId }),
  createCollection: (workspaceId: string, name: string) => call("create_collection", { workspaceId, name }),
  createFolder: (collectionId: string, parentId: string | null, name: string) => call("create_folder", { collectionId, parentId, name }),
  createRequest: (a: { collectionId: string; folderId: string | null; name: string; method: string; url: string; definition: RequestDefinition }) => call("create_request", a),
  getRequest: (id: string) => call("get_request", { id }),
  saveRequest: (id: string, patch: RequestPatch) => call("save_request", { id, patch }),
  duplicateRequest: (id: string) => call("duplicate_request", { id }),
  renameItem: (kind: ItemKind, id: string, name: string) => call("rename_item", { kind, id, name }),
  deleteItem: (kind: ItemKind, id: string) => call("delete_item", { kind, id }),
  setFavorite: (id: string, favorite: boolean) => call("set_favorite", { id, favorite }),
  moveItem: (kind: ItemKind, id: string, target: MoveTarget) => call("move_item", { kind, id, target }),
};
