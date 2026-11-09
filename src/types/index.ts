// IPC types generated from Rust (ts-rs; regenerate with `cargo test`). Never edit bindings/.
export type { AppErrorPayload } from "./bindings/AppErrorPayload";
export type { AppInfo } from "./bindings/AppInfo";
export type { Settings } from "./bindings/Settings";
export type { SettingsPatch } from "./bindings/SettingsPatch";
export type { Theme } from "./bindings/Theme";

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

/** A request being edited in a tab (persisted with collections from Phase 3). */
export interface RequestDraft {
  method: HttpMethod;
  url: string;
  params: KeyValue[];
  headers: KeyValue[];
  bodyType: BodyType;
  body: string;
}

export interface RequestTab {
  id: string;
  name: string;
  draft: RequestDraft;
  /** Changed since it was opened or last saved. */
  dirty: boolean;
}
