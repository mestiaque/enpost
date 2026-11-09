import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { api, inTauri, toIpcError } from "@/services/ipc";
import { buildHttpRequest, onProgress } from "@/services/http";
import type { HttpMethod, KeyValue, RequestDraft, RequestTab } from "@/types";
import { paramsFromUrl, urlWithParams } from "@/utils/url";
import { draftToDefinition, savedToDraft } from "@/services/requestMapping";
import { useHistory } from "./history";
import { useUi } from "./ui";
import { useWorkspace } from "./workspace";
import { useSettings } from "./settings";

export function newId(): string {
  const c = globalThis.crypto;
  if (c && typeof c.randomUUID === "function") return c.randomUUID();
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

export const emptyRow = (): KeyValue => ({ id: newId(), key: "", value: "", enabled: true });

export function newDraft(): RequestDraft {
  return { method: "GET", url: "", params: [], headers: [], bodyType: "none", body: "", urlencoded: [], form: [], binaryPath: "" };
}

/** Tab title for an unsaved request: the URL's path, or "Untitled request". */
export function tabTitle(d: RequestDraft): string {
  const u = d.url.trim().replace(/^[a-z]+:\/\//i, "").split("?")[0] ?? "";
  return u || "Untitled request";
}

/**
 * Open request tabs. Phase 1 keeps them in memory; from Phase 3 they open saved requests
 * and autosave. Opening, closing, reordering and editing are real.
 */
export const useTabs = defineStore("tabs", () => {
  const tabs = ref<RequestTab[]>([]);
  const activeId = ref<string | null>(null);
  const active = computed(() => tabs.value.find((t) => t.id === activeId.value) ?? null);

  function open(draft: RequestDraft = newDraft()): RequestTab {
    const t: RequestTab = { id: newId(), name: tabTitle(draft), draft, dirty: false, exec: { state: "idle" } };
    const i = tabs.value.findIndex((x) => x.id === activeId.value);
    tabs.value.splice(i < 0 ? tabs.value.length : i + 1, 0, t);
    activeId.value = t.id;
    return t;
  }

  function activate(id: string) {
    if (tabs.value.some((t) => t.id === id)) activeId.value = id;
  }

  /** Close a tab; the neighbour to the right (or left) becomes active. */
  function close(id: string) {
    const closing = tabs.value.find((t) => t.id === id);
    if (closing?.exec.state === "sending") void api.cancelRequest(closing.exec.requestId).catch(() => {});
    const i = tabs.value.findIndex((t) => t.id === id);
    if (i < 0) return;
    tabs.value.splice(i, 1);
    if (activeId.value === id) activeId.value = (tabs.value[i] ?? tabs.value[i - 1])?.id ?? null;
  }

  function closeOthers(id: string) {
    tabs.value = tabs.value.filter((t) => t.id === id);
    activeId.value = id;
  }

  function duplicate(id: string) {
    const t = tabs.value.find((x) => x.id === id);
    if (!t) return;
    activeId.value = id;
    const d = t.draft;
    const fresh = <T extends KeyValue>(rows: T[]): T[] => rows.map((r) => ({ ...r, id: newId() }));
    const copy = open({ ...d, params: fresh(d.params), headers: fresh(d.headers), urlencoded: fresh(d.urlencoded), form: fresh(d.form) });
    copy.dirty = true;
  }

  /** Move tab `from` to index `to` (drag and drop). */
  function move(from: number, to: number) {
    if (from === to || from < 0 || from >= tabs.value.length) return;
    const [t] = tabs.value.splice(from, 1);
    if (t) tabs.value.splice(Math.max(0, Math.min(to, tabs.value.length)), 0, t);
  }

  function cycle(delta: number) {
    if (!tabs.value.length) return;
    const i = tabs.value.findIndex((t) => t.id === activeId.value);
    const next = tabs.value[(i + delta + tabs.value.length) % tabs.value.length];
    if (next) activeId.value = next.id;
  }

  function edit(id: string, fn: (d: RequestDraft) => void) {
    const t = tabs.value.find((x) => x.id === id);
    if (!t) return;
    fn(t.draft);
    t.dirty = true;
    if (!t.savedId) t.name = tabTitle(t.draft);
    else scheduleAutosave(t.id);
  }

  // ---- saved requests ----

  /** Open a saved request (or switch to its tab if it is already open). */
  async function openSaved(savedId: string) {
    const existing = tabs.value.find((t) => t.savedId === savedId);
    if (existing) {
      activeId.value = existing.id;
      return;
    }
    const r = await api.getRequest(savedId);
    const t = open(savedToDraft(r));
    t.savedId = r.id;
    t.name = r.name;
    // Reuse an untouched empty tab instead of piling up blank ones.
    const blank = tabs.value.find((x) => x.id !== t.id && !x.savedId && !x.dirty && !x.draft.url && x.exec.state === "idle");
    if (blank) close(blank.id);
    activeId.value = t.id;
  }

  const autosaveTimers = new Map<string, number>();
  function scheduleAutosave(tabId: string) {
    if (!useSettings().settings.autosave) return;
    window.clearTimeout(autosaveTimers.get(tabId));
    autosaveTimers.set(tabId, window.setTimeout(() => void save(tabId, { quiet: true }), 700));
  }

  /** Ctrl+S: write a saved request; an unsaved one opens "Save as". */
  async function save(tabId: string, opts: { quiet?: boolean } = {}) {
    const t = tabs.value.find((x) => x.id === tabId);
    if (!t) return;
    window.clearTimeout(autosaveTimers.get(tabId));
    if (!t.savedId) {
      if (!opts.quiet) useUi().openSaveAs(tabId);
      return;
    }
    const snapshot = JSON.stringify(t.draft);
    try {
      const r = await api.saveRequest(t.savedId, { method: t.draft.method, url: t.draft.url, definition: draftToDefinition(t.draft) });
      // Edits made while saving keep the tab dirty (and are saved by the next autosave).
      if (JSON.stringify(t.draft) === snapshot) t.dirty = false;
      t.name = r.name;
      void useWorkspace().refresh();
      if (!opts.quiet) useUi().toast({ kind: "success", title: "Saved", detail: r.name });
    } catch (e) {
      const err = toIpcError(e);
      if (err.kind === "notFound") t.savedId = undefined; // deleted meanwhile: now an unsaved tab
      useUi().toast({ kind: "error", title: "Could not save", detail: err.message });
    }
  }

  /** "Save as": create the saved request and bind the tab to it. */
  async function saveAs(tabId: string, collectionId: string, folderId: string | null, name: string) {
    const t = tabs.value.find((x) => x.id === tabId);
    if (!t) return false;
    try {
      const r = await api.createRequest({ collectionId, folderId, name, method: t.draft.method, url: t.draft.url, definition: draftToDefinition(t.draft) });
      t.savedId = r.id;
      t.name = r.name;
      t.dirty = false;
      await useWorkspace().refresh();
      useUi().toast({ kind: "success", title: "Saved", detail: r.name });
      return true;
    } catch (e) {
      useUi().toast({ kind: "error", title: "Could not save", detail: toIpcError(e).message });
      return false;
    }
  }

  /** Keep tabs in step with the tree: renamed requests rename tabs, deleted ones become unsaved. */
  function syncWithTree(requests: { id: string; name: string }[]) {
    const byId = new Map(requests.map((r) => [r.id, r.name]));
    for (const t of tabs.value) {
      if (!t.savedId) continue;
      const name = byId.get(t.savedId);
      if (name === undefined) {
        t.savedId = undefined;
        t.dirty = true;
      } else t.name = name;
    }
  }

  /** URL typed by the user: the Params table follows the query string. */
  function setUrl(id: string, url: string) {
    edit(id, (d) => {
      d.url = url;
      d.params = paramsFromUrl(url, d.params);
    });
  }

  /** Params table edited: the URL's query string follows the enabled rows. */
  function setParams(id: string, params: KeyValue[]) {
    edit(id, (d) => {
      d.params = params;
      d.url = urlWithParams(d.url, params);
    });
  }

  function setMethod(id: string, m: HttpMethod) {
    edit(id, (d) => {
      d.method = m;
    });
  }

  // ---- closing with unsaved changes ----

  /** Changes that would be lost: dirty and not about to be autosaved. */
  function unsaved(t: RequestTab) {
    return t.dirty && (!t.savedId || !useSettings().settings.autosave) && (t.savedId !== undefined || t.draft.url.trim() !== "" || t.draft.body !== "");
  }

  /** Close from the UI: asks first when changes would be lost (Settings → General). */
  async function requestClose(id: string) {
    const t = tabs.value.find((x) => x.id === id);
    if (!t) return;
    if (unsaved(t) && useSettings().settings.confirmCloseUnsaved) {
      const ok = await useUi().confirm({ title: `Close "${t.name}"?`, message: "Its changes are not saved and will be lost.", confirmLabel: "Close without saving", danger: true });
      if (!ok) return;
    }
    close(id);
  }

  async function requestCloseOthers(id: string) {
    const lost = tabs.value.filter((t) => t.id !== id && unsaved(t));
    if (lost.length && useSettings().settings.confirmCloseUnsaved) {
      const ok = await useUi().confirm({ title: `Close ${tabs.value.length - 1} tabs?`, message: `${lost.length} of them ${lost.length === 1 ? "has" : "have"} changes that are not saved.`, confirmLabel: "Close without saving", danger: true });
      if (!ok) return;
    }
    closeOthers(id);
  }

  // ---- session (reopen tabs on the next start) ----

  const SESSION_KEY = "enpost.session";
  /** Open tabs and their drafts, for "Reopen the tabs from last time". Responses are not kept. */
  function saveSession() {
    try {
      const data = { active: tabs.value.findIndex((t) => t.id === activeId.value), tabs: tabs.value.map((t) => ({ name: t.name, draft: t.draft, dirty: t.dirty, savedId: t.savedId })) };
      localStorage.setItem(SESSION_KEY, JSON.stringify(data));
    } catch {
      /* storage unavailable: the session is simply not restored */
    }
  }
  /** Reopen the last session's tabs; returns false when there is nothing to restore. */
  function restoreSession(): boolean {
    try {
      const raw = localStorage.getItem(SESSION_KEY);
      if (!raw) return false;
      const data = JSON.parse(raw) as { active: number; tabs: { name: string; draft: RequestDraft; dirty: boolean; savedId?: string }[] };
      if (!Array.isArray(data.tabs) || !data.tabs.length) return false;
      tabs.value = data.tabs.map((t) => ({ id: newId(), name: String(t.name), draft: { ...newDraft(), ...t.draft }, dirty: Boolean(t.dirty), exec: { state: "idle" as const }, ...(t.savedId ? { savedId: t.savedId } : {}) }));
      activeId.value = tabs.value[Math.max(0, Math.min(data.active, tabs.value.length - 1))]?.id ?? null;
      return true;
    } catch {
      return false;
    }
  }

  /** Send the tab's request through the Rust engine; progress and result land on the tab. */
  async function send(id: string) {
    const t = tabs.value.find((x) => x.id === id);
    if (!t || t.exec.state === "sending") return;
    const requestId = newId();
    const req = buildHttpRequest(t.draft, useSettings().settings, requestId);
    t.exec = { state: "sending", requestId, startedAt: Date.now(), stage: "connecting", received: 0, total: null };
    const stop = onProgress(requestId, (p) => {
      const cur = tabs.value.find((x) => x.id === id);
      if (cur?.exec.state === "sending" && cur.exec.requestId === requestId) cur.exec = { ...cur.exec, stage: p.stage, received: p.receivedBytes, total: p.totalBytes };
    });
    try {
      if (!inTauri()) throw new Error("Requests are sent by the desktop app");
      const response = await api.sendRequest(req);
      setResult(id, requestId, { state: "done", response });
    } catch (e) {
      const err = toIpcError(e);
      setResult(id, requestId, err.kind === "cancelled" ? { state: "idle" } : { state: "error", kind: err.kind, message: err.message, url: req.url });
    } finally {
      stop();
      void useHistory().refresh().catch(() => {});
    }
  }

  /** Only the send that is still current may write its result (a re-send replaces it). */
  function setResult(id: string, requestId: string, exec: RequestTab["exec"]) {
    const t = tabs.value.find((x) => x.id === id);
    if (t && t.exec.state === "sending" && t.exec.requestId === requestId) t.exec = exec;
  }

  function cancel(id: string) {
    const t = tabs.value.find((x) => x.id === id);
    if (t?.exec.state === "sending") void api.cancelRequest(t.exec.requestId).catch(() => {});
  }

  return { tabs, activeId, active, open, requestClose, requestCloseOthers, saveSession, restoreSession, openSaved, save, saveAs, syncWithTree, send, cancel, activate, close, closeOthers, duplicate, move, cycle, edit, setUrl, setParams, setMethod };
});
