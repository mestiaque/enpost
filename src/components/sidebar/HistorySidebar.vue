<script setup lang="ts">
import { History, Trash2 } from "lucide-vue-next";
import { computed, onMounted } from "vue";
import MethodBadge from "@/components/request/MethodBadge.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import IconButton from "@/components/ui/IconButton.vue";
import { useHistory } from "@/stores/history";
import { emptyRow, newDraft, useTabs } from "@/stores/tabs";
import { useUi } from "@/stores/ui";
import { HTTP_METHODS, type HistoryEntry, type HttpMethod, type KeyValue } from "@/types";
import { formatBytes, formatMs, statusColor } from "@/utils/format";
import { paramsFromUrl } from "@/utils/url";

const history = useHistory();
const tabs = useTabs();
const ui = useUi();
onMounted(() => void history.refresh());

/** Group by day: Today, Yesterday, then dates. */
const groups = computed(() => {
  const out: { label: string; items: HistoryEntry[] }[] = [];
  const day = (ms: number) => new Date(ms).toDateString();
  const today = day(Date.now());
  const yesterday = day(Date.now() - 86_400_000);
  for (const e of history.entries) {
    const d = day(e.sentAt);
    const label = d === today ? "Today" : d === yesterday ? "Yesterday" : new Date(e.sentAt).toLocaleDateString();
    const g = out[out.length - 1];
    if (g?.label === label) g.items.push(e);
    else out.push({ label, items: [e] });
  }
  return out;
});

/** Reopen in a new tab. Secrets were removed before saving, so they show as [redacted]. */
function reopen(e: HistoryEntry) {
  const draft = newDraft();
  draft.method = (HTTP_METHODS as readonly string[]).includes(e.method) ? (e.method as HttpMethod) : "GET";
  draft.url = e.url;
  draft.params = paramsFromUrl(e.url, []);
  try {
    const snap = JSON.parse(e.snapshot) as { headers?: { name: string; value: string }[] };
    draft.headers = (snap.headers ?? []).map((h): KeyValue => ({ ...emptyRow(), key: h.name, value: h.value }));
  } catch {
    /* an unreadable snapshot only loses the headers */
  }
  tabs.open(draft);
  if (e.url.includes("[redacted]") || draft.headers.some((h) => h.value === "[redacted]")) {
    ui.toast({ kind: "info", title: "Secrets were not saved", detail: "Tokens and passwords show as [redacted]; enter them again before sending." });
  }
}

async function clearAll() {
  if (await ui.confirm({ title: "Clear history?", message: "All sent requests and their stored responses are deleted.", confirmLabel: "Clear", danger: true })) await history.clear();
}
const time = (ms: number) => new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
</script>

<template>
  <aside class="flex h-full min-w-0 flex-col bg-panel" aria-label="History">
    <div class="flex h-10 shrink-0 items-center justify-between pl-3 pr-1.5">
      <span class="text-[11px] font-semibold uppercase tracking-wider text-faint">History</span>
      <IconButton v-if="history.entries.length" label="Clear history" @click="clearAll"><Trash2 class="h-3.5 w-3.5" /></IconButton>
    </div>
    <EmptyState v-if="!history.entries.length" title="No requests yet" text="Every request you send is listed here. Tokens, passwords and cookies are removed before saving.">
      <template #icon><History class="h-5 w-5" /></template>
    </EmptyState>
    <div v-else class="min-h-0 flex-1 overflow-y-auto pb-2">
      <template v-for="g in groups" :key="g.label">
        <div class="px-3 pb-1 pt-2.5 text-[10px] font-semibold uppercase tracking-wider text-faint">{{ g.label }}</div>
        <button v-for="e in g.items" :key="e.id" type="button" class="flex w-full flex-col gap-0.5 px-3 py-1.5 text-left hover:bg-raised/60" :title="e.url" @click="reopen(e)">
          <div class="flex w-full items-center gap-2">
            <span class="w-9 shrink-0"><MethodBadge :method="(e.method as HttpMethod)" /></span>
            <span class="min-w-0 flex-1 truncate font-mono text-[12px]">{{ e.url.replace(/^https?:\/\//, "") }}</span>
          </div>
          <div class="flex items-center gap-2 pl-11 text-[11px] text-faint">
            <span v-if="e.status !== null" :class="statusColor(e.status)">{{ e.status }}</span>
            <span v-else class="text-danger">failed</span>
            <span v-if="e.durationMs !== null">{{ formatMs(e.durationMs) }}</span>
            <span v-if="e.sizeBytes !== null">{{ formatBytes(e.sizeBytes) }}</span>
            <span class="ml-auto">{{ time(e.sentAt) }}</span>
          </div>
        </button>
      </template>
    </div>
  </aside>
</template>
