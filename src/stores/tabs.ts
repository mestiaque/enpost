import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { HttpMethod, KeyValue, RequestDraft, RequestTab } from "@/types";
import { paramsFromUrl, urlWithParams } from "@/utils/url";

export function newId(): string {
  const c = globalThis.crypto;
  if (c && typeof c.randomUUID === "function") return c.randomUUID();
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

export const emptyRow = (): KeyValue => ({ id: newId(), key: "", value: "", enabled: true });

export function newDraft(): RequestDraft {
  return { method: "GET", url: "", params: [], headers: [], bodyType: "none", body: "" };
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
    const t: RequestTab = { id: newId(), name: tabTitle(draft), draft, dirty: false };
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
    const copy = open(structuredClone({ ...t.draft, params: t.draft.params.map((p) => ({ ...p, id: newId() })), headers: t.draft.headers.map((h) => ({ ...h, id: newId() })) }));
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
    t.name = tabTitle(t.draft);
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

  return { tabs, activeId, active, open, activate, close, closeOthers, duplicate, move, cycle, edit, setUrl, setParams, setMethod };
});
