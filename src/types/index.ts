// IPC types generated from Rust (ts-rs; regenerate with `cargo test`). Never edit bindings/.
export type { AppErrorPayload } from "./bindings/AppErrorPayload";
export type { AppInfo } from "./bindings/AppInfo";
export type { Settings } from "./bindings/Settings";
export type { SettingsPatch } from "./bindings/SettingsPatch";
export type { Theme } from "./bindings/Theme";
export type { BodyKind } from "./bindings/BodyKind";
export type { FieldKind } from "./bindings/FieldKind";
export type { FormField } from "./bindings/FormField";
export type { HeaderPair } from "./bindings/HeaderPair";
export type { HistoryEntry } from "./bindings/HistoryEntry";
export type { HttpProgress } from "./bindings/HttpProgress";
export type { HttpRequest } from "./bindings/HttpRequest";
export type { HttpResponse } from "./bindings/HttpResponse";
export type { RequestBody } from "./bindings/RequestBody";
export type { RequestOptions } from "./bindings/RequestOptions";
export type { ResponseBody } from "./bindings/ResponseBody";
export type { ResponseCookie } from "./bindings/ResponseCookie";
export type { Timing } from "./bindings/Timing";
export type { CollectionNode } from "./bindings/CollectionNode";
export type { FolderNode } from "./bindings/FolderNode";
export type { ItemKind } from "./bindings/ItemKind";
export type { MoveTarget } from "./bindings/MoveTarget";
export type { RequestDefinition } from "./bindings/RequestDefinition";
export type { RequestPatch } from "./bindings/RequestPatch";
export type { RequestSummary } from "./bindings/RequestSummary";
export type { SavedRequest } from "./bindings/SavedRequest";
export type { Workspace } from "./bindings/Workspace";
export type { WorkspaceTree } from "./bindings/WorkspaceTree";
import type { HttpResponse } from "./bindings/HttpResponse";

// Frontend-only types.
export const HTTP_METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"] as const;
export type HttpMethod = (typeof HTTP_METHODS)[number];

export interface KeyValue {
  id: string;
  key: string;
  value: string;
  enabled: boolean;
  description?: string;
}

export const BODY_TYPES = ["none", "json", "text", "xml", "form-data", "urlencoded", "binary"] as const;
export type BodyType = (typeof BODY_TYPES)[number];

/** A form-data row: text, or a file the user picked (`value` is then its path). */
export interface FormRow extends KeyValue {
  kind: "text" | "file";
}

/** A request being edited in a tab (persisted with collections from Phase 3). */
export interface RequestDraft {
  method: HttpMethod;
  url: string;
  params: KeyValue[];
  headers: KeyValue[];
  bodyType: BodyType;
  /** Raw body (JSON, text, XML). */
  body: string;
  /** URL-encoded fields. */
  urlencoded: KeyValue[];
  /** Multipart fields. */
  form: FormRow[];
  /** File sent as a binary body. */
  binaryPath: string;
}

/** What happened to the last send of a tab. */
export type Execution =
  | { state: "idle" }
  | { state: "sending"; requestId: string; startedAt: number; stage: string; received: number; total: number | null }
  | { state: "done"; response: HttpResponse }
  | { state: "error"; kind: string; message: string; url: string };

export interface RequestTab {
  id: string;
  name: string;
  draft: RequestDraft;
  /** Changed since it was opened or last saved. */
  dirty: boolean;
  /** The saved request this tab edits (absent for an unsaved request). */
  savedId?: string;
  exec: Execution;
}
