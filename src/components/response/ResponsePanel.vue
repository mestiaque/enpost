<script setup lang="ts">
import { save } from "@tauri-apps/plugin-dialog";
import { CircleAlert, Copy, Download, FoldVertical, Radio, UnfoldVertical } from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import CodeEditor, { type CodeLanguage } from "@/components/editor/CodeEditor.vue";
import Button from "@/components/ui/Button.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import IconButton from "@/components/ui/IconButton.vue";
import Segmented from "@/components/ui/Segmented.vue";
import Spinner from "@/components/ui/Spinner.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import JsonTree from "./JsonTree.vue";
import { api } from "@/services/ipc";
import { useSettings } from "@/stores/settings";
import { useTabs } from "@/stores/tabs";
import { useUi } from "@/stores/ui";
import type { RequestTab } from "@/types";
import { formatBytes, formatMs, languageFor, prettyJson, prettyXml, statusColor } from "@/utils/format";
import type { JsonValue } from "@/utils/jsonTree";

const props = defineProps<{ tab: RequestTab }>();
const tabs = useTabs();
const ui = useUi();
const settings = useSettings();
const exec = computed(() => props.tab.exec);
const res = computed(() => (exec.value.state === "done" ? exec.value.response : null));

type Section = "body" | "headers" | "cookies";
type View = "pretty" | "raw" | "preview" | "tree";
const section = ref<Section>("body");
const view = ref<View>("pretty");

// Elapsed time while sending.
const now = ref(Date.now());
let timer: number | undefined;
onMounted(() => (timer = window.setInterval(() => (now.value = Date.now()), 100)));
onBeforeUnmount(() => window.clearInterval(timer));

const lang = computed<CodeLanguage>(() => languageFor(res.value?.contentType));
const text = computed(() => res.value?.body.text ?? "");
const pretty = computed(() => (lang.value === "json" ? prettyJson(text.value).text : lang.value === "xml" || lang.value === "html" ? prettyXml(text.value) : text.value));
const json = computed<JsonValue | undefined>(() => {
  if (lang.value !== "json" || !text.value || res.value?.body.truncated) return undefined;
  try {
    return JSON.parse(text.value) as JsonValue;
  } catch {
    return undefined;
  }
});
const views = computed(() => {
  const v: { value: View; label: string }[] = [{ value: "pretty", label: "Pretty" }, { value: "raw", label: "Raw" }];
  if (lang.value === "html" || res.value?.body.imageDataUrl) v.push({ value: "preview", label: "Preview" });
  if (json.value !== undefined) v.push({ value: "tree", label: "Tree" });
  return v;
});
// A new response opens on a view it has: images on Preview, others on Pretty.
watch(res, (r) => {
  if (!r) return;
  if (r.body.imageDataUrl) view.value = "preview";
  else if (!views.value.some((v) => v.value === view.value)) view.value = "pretty";
});

const ERROR_TITLE: Record<string, string> = {
  timeout: "The request timed out",
  connect: "Could not connect",
  tls: "Certificate problem",
  redirect: "Too many redirects",
  invalidRequest: "The request is not valid",
  network: "Network error",
};

async function copyBody() {
  await navigator.clipboard.writeText(view.value === "raw" ? text.value : pretty.value);
  ui.toast({ kind: "success", title: "Response copied" });
}
async function saveBody() {
  const r = res.value;
  if (!r) return;
  const ext = lang.value === "text" ? (r.contentType?.split("/")[1]?.split(";")[0] ?? "txt") : lang.value;
  const dest = await save({ title: "Save response", defaultPath: `response.${ext.replace(/[^a-z0-9]/gi, "") || "txt"}` });
  if (!dest) return;
  try {
    await api.saveResponseBody(r.historyId, dest);
    ui.toast({ kind: "success", title: "Response saved", detail: dest });
  } catch (e) {
    ui.toast({ kind: "error", title: "Could not save", detail: e instanceof Error ? e.message : String(e) });
  }
}
const tree = ref<InstanceType<typeof JsonTree> | null>(null);
</script>

<template>
  <section class="flex h-full min-h-0 flex-col" aria-label="Response">
    <!-- Summary bar -->
    <div class="flex h-9 shrink-0 items-center gap-4 border-b border-line px-3 text-[12px]">
      <span class="text-[11px] font-semibold uppercase tracking-wider text-faint">Response</span>
      <template v-if="res">
        <span :class="['font-semibold tabular-nums', statusColor(res.status)]">{{ res.status }} {{ res.statusText }}</span>
        <Tooltip :label="`Waiting ${formatMs(res.timing.ttfbMs)} · download ${formatMs(res.timing.downloadMs)}`">
          <span class="tabular-nums text-muted">{{ formatMs(res.timing.totalMs) }}</span>
        </Tooltip>
        <Tooltip :label="`Body ${formatBytes(res.body.bytes)} · headers ${formatBytes(res.headersBytes)}`">
          <span class="tabular-nums text-muted">{{ formatBytes(res.body.bytes) }}</span>
        </Tooltip>
        <span class="text-faint">{{ res.httpVersion }}</span>
        <span class="flex-1" />
        <div class="flex gap-1">
          <button v-for="s in (['body', 'headers', 'cookies'] as const)" :key="s" type="button" :class="['rounded px-2 py-0.5 capitalize', section === s ? 'bg-raised text-fg' : 'text-muted hover:text-fg']" @click="section = s">
            {{ s }}<span v-if="s === 'headers'" class="ml-1 text-faint">{{ res.headers.length }}</span><span v-if="s === 'cookies' && res.cookies.length" class="ml-1 text-faint">{{ res.cookies.length }}</span>
          </button>
        </div>
      </template>
    </div>

    <!-- Sending -->
    <div v-if="exec.state === 'sending'" class="flex flex-1 flex-col items-center justify-center gap-3 text-[13px] text-muted">
      <Spinner class="h-6 w-6 text-accent" />
      <div class="tabular-nums">
        {{ exec.stage === "receiving" ? "Receiving" : "Waiting for the server" }} · {{ formatMs(now - exec.startedAt) }}
        <template v-if="exec.received"> · {{ formatBytes(exec.received) }}<template v-if="exec.total"> of {{ formatBytes(exec.total) }}</template></template>
      </div>
      <Button size="sm" @click="tabs.cancel(tab.id)">Cancel</Button>
    </div>

    <!-- Failed -->
    <div v-else-if="exec.state === 'error'" class="flex flex-1 flex-col items-center justify-center gap-2 px-8 text-center">
      <CircleAlert class="h-7 w-7 text-danger" />
      <div class="text-[14px] font-medium">{{ ERROR_TITLE[exec.kind] ?? "The request failed" }}</div>
      <p class="max-w-[560px] select-text break-words text-xs text-muted">{{ exec.message }}</p>
      <p v-if="exec.kind === 'tls'" class="max-w-[520px] text-[11px] text-faint">For a local server with a self-signed certificate, turn off certificate verification in Settings → Requests.</p>
    </div>

    <!-- Response -->
    <template v-else-if="res">
      <div v-if="section === 'body'" class="flex min-h-0 flex-1 flex-col">
        <div class="flex h-9 shrink-0 items-center gap-2 px-3">
          <Segmented v-if="res.body.kind !== 'empty'" v-model="view" :options="views" />
          <span v-if="res.body.truncated" class="text-[11px] text-warning">Showing the first {{ formatBytes(res.body.text?.length ?? 0) }} of {{ formatBytes(res.body.bytes) }}; save to get it all</span>
          <span class="flex-1" />
          <template v-if="view === 'tree' && json !== undefined">
            <IconButton label="Expand all" @click="tree?.expandAll()"><UnfoldVertical class="h-3.5 w-3.5" /></IconButton>
            <IconButton label="Collapse all" @click="tree?.collapseAll()"><FoldVertical class="h-3.5 w-3.5" /></IconButton>
          </template>
          <IconButton v-if="res.body.kind === 'text'" label="Copy body" @click="copyBody"><Copy class="h-3.5 w-3.5" /></IconButton>
          <IconButton v-if="res.body.kind !== 'empty'" label="Save to a file" @click="saveBody"><Download class="h-3.5 w-3.5" /></IconButton>
        </div>
        <div class="min-h-0 flex-1 border-t border-line">
          <EmptyState v-if="res.body.kind === 'empty'" title="No body" :text="`The server answered ${res.status} without content.`" />
          <template v-else-if="view === 'preview'">
            <div v-if="res.body.imageDataUrl" class="flex h-full items-center justify-center overflow-auto p-4" style="background: repeating-conic-gradient(rgb(var(--raised)) 0 25%, transparent 0 50%) 0 0 / 16px 16px">
              <img :src="res.body.imageDataUrl" alt="Response image" class="max-h-full max-w-full object-contain" />
            </div>
            <!-- No scripts, no same-origin access: a static look at the HTML. -->
            <iframe v-else :srcdoc="text" sandbox="" referrerpolicy="no-referrer" title="HTML preview" class="h-full w-full bg-white" />
          </template>
          <JsonTree v-else-if="view === 'tree' && json !== undefined" ref="tree" :value="json" @copied="(w: string) => ui.toast({ kind: 'success', title: `${w} copied` })" />
          <EmptyState v-else-if="res.body.kind === 'binary'" title="Binary response" :text="`${res.contentType ?? 'Unknown type'}, ${formatBytes(res.body.bytes)}. Save it to a file to open it.`">
            <Button size="sm" class="mt-2" @click="saveBody"><Download class="h-3.5 w-3.5" />Save to a file</Button>
          </EmptyState>
          <CodeEditor v-else :key="`${res.historyId}-${view}`" :model-value="view === 'raw' ? text : pretty" :language="view === 'raw' ? 'text' : lang" read-only :wrap="settings.settings.editorWordWrap" label="Response body" />
        </div>
      </div>

      <div v-else-if="section === 'headers'" class="min-h-0 flex-1 overflow-auto border-t border-line p-3">
        <table class="w-full select-text font-mono text-[12px]">
          <tbody>
            <tr v-for="(h, i) in res.headers" :key="i" class="border-b border-line last:border-b-0">
              <td class="w-[34%] py-1.5 pr-3 align-top text-muted">{{ h.name }}</td>
              <td class="break-all py-1.5">{{ h.value }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-else class="min-h-0 flex-1 overflow-auto border-t border-line p-3">
        <EmptyState v-if="!res.cookies.length" title="No cookies" text="The response set no cookies." />
        <table v-else class="w-full select-text text-[12px]">
          <thead class="text-left text-[11px] uppercase tracking-wider text-faint">
            <tr><th class="py-1.5 pr-3 font-semibold">Name</th><th class="pr-3 font-semibold">Value</th><th class="pr-3 font-semibold">Domain</th><th class="pr-3 font-semibold">Path</th><th class="pr-3 font-semibold">Expires</th><th class="font-semibold">Flags</th></tr>
          </thead>
          <tbody class="font-mono">
            <tr v-for="c in res.cookies" :key="c.name" class="border-t border-line">
              <td class="py-1.5 pr-3">{{ c.name }}</td>
              <td class="max-w-[260px] truncate pr-3" :title="c.value">{{ c.value }}</td>
              <td class="pr-3 text-muted">{{ c.domain ?? "—" }}</td>
              <td class="pr-3 text-muted">{{ c.path ?? "—" }}</td>
              <td class="pr-3 text-muted">{{ c.maxAge !== null ? `${c.maxAge} s` : (c.expires ?? "Session") }}</td>
              <td class="text-muted">{{ [c.secure && "Secure", c.httpOnly && "HttpOnly", c.sameSite && `SameSite=${c.sameSite}`].filter(Boolean).join(" · ") || "—" }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <EmptyState v-else title="No response yet" text="Press Send or Ctrl+Enter. The request runs in ENPost's Rust engine; the response shows here.">
      <template #icon><Radio class="h-5 w-5" /></template>
    </EmptyState>
  </section>
</template>
