/**
 * Typed bridge to the Rust backend. Every backend call goes through `call`, which maps the
 * command name to its argument and result types, so a wrong name or payload fails to compile.
 * The frontend never performs API requests itself: those run in Rust (Phase 2).
 */
import { invoke } from "@tauri-apps/api/core";
import type { AppErrorPayload, AppInfo, Settings, SettingsPatch } from "@/types";

interface Commands {
  app_info: { args: Record<string, never>; result: AppInfo };
  get_settings: { args: Record<string, never>; result: Settings };
  update_settings: { args: { patch: SettingsPatch }; result: Settings };
  reset_settings: { args: Record<string, never>; result: Settings };
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
  getSettings: () => call("get_settings"),
  updateSettings: (patch: SettingsPatch) => call("update_settings", { patch }),
  resetSettings: () => call("reset_settings"),
};
