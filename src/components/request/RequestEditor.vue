<script setup lang="ts">
import { open as openFile } from "@tauri-apps/plugin-dialog";
import { FileUp, Send, X } from "lucide-vue-next";
import { computed, ref } from "vue";
import Button from "@/components/ui/Button.vue";
import CodeEditor from "@/components/editor/CodeEditor.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import Spinner from "@/components/ui/Spinner.vue";
import FormDataEditor from "./FormDataEditor.vue";
import Segmented from "@/components/ui/Segmented.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import KeyValueEditor from "./KeyValueEditor.vue";
import { METHOD_COLOR } from "./MethodBadge.vue";
import { useSettings } from "@/stores/settings";
import { useTabs } from "@/stores/tabs";
import { BODY_TYPES, HTTP_METHODS, type BodyType, type HttpMethod, type RequestTab } from "@/types";

const props = defineProps<{ tab: RequestTab }>();
const tabs = useTabs();
const settings = useSettings();
const sending = computed(() => props.tab.exec.state === "sending");
const noBodyMethod = computed(() => d.value.method === "GET" || d.value.method === "HEAD");
async function pickBinary() {
  const path = await openFile({ title: "Choose the file to send", multiple: false, directory: false });
  if (typeof path === "string") tabs.edit(props.tab.id, (x) => (x.binaryPath = path));
}
const d = computed(() => props.tab.draft);

type Section = "params" | "headers" | "body" | "auth" | "scripts";
const section = ref<Section>("params");
const SECTIONS: { id: Section; label: string; phase?: number }[] = [
  { id: "params", label: "Params" },
  { id: "headers", label: "Headers" },
  { id: "body", label: "Body" },
  { id: "auth", label: "Auth", phase: 3 },
  { id: "scripts", label: "Scripts", phase: 4 },
];
const count = (id: Section) => (id === "params" ? d.value.params.filter((p) => p.enabled && p.key).length : id === "headers" ? d.value.headers.filter((h) => h.enabled && h.key).length : 0);
const LANG = { json: "json", xml: "xml", text: "text" } as const;

const BODY_LABEL: Record<BodyType, string> = { none: "None", json: "JSON", text: "Text", xml: "XML", "form-data": "Form data", urlencoded: "URL encoded", binary: "Binary" };
const bodyOptions = BODY_TYPES.map((v) => ({ value: v, label: BODY_LABEL[v] }));
const rawBody = computed(() => ["json", "text", "xml"].includes(d.value.bodyType));
</script>

<template>
  <section class="flex h-full min-h-0 flex-col" aria-label="Request">
    <!-- Method, URL, Send -->
    <div class="flex shrink-0 items-center gap-2 p-3">
      <div class="flex h-9 min-w-0 flex-1 items-stretch overflow-hidden rounded-md border border-line bg-panel focus-within:border-accent">
        <select
          :value="d.method"
          aria-label="Method"
          :class="['w-[104px] shrink-0 cursor-pointer border-r border-line bg-transparent pl-3 font-mono text-[12px] font-bold outline-none', METHOD_COLOR[d.method]]"
          @change="tabs.setMethod(tab.id, ($event.target as HTMLSelectElement).value as HttpMethod)"
        >
          <option v-for="m in HTTP_METHODS" :key="m" :value="m" class="text-fg">{{ m }}</option>
        </select>
        <input
          :value="d.url"
          class="min-w-0 flex-1 bg-transparent px-3 font-mono text-[length:var(--editor-font-size)] text-fg outline-none placeholder:text-faint"
          placeholder="https://api.example.com/v1/users or {{base_url}}/users"
          spellcheck="false"
          autocomplete="off"
          aria-label="Request URL"
          @input="tabs.setUrl(tab.id, ($event.target as HTMLInputElement).value)"
          @keydown.enter.exact="tabs.send(tab.id)"
        />
      </div>
      <Button v-if="sending" class="h-9 w-[92px]" @click="tabs.cancel(tab.id)"><Spinner class="h-3.5 w-3.5" />Cancel</Button>
      <Tooltip v-else label="Send" shortcut="Ctrl+Enter">
        <Button variant="primary" class="h-9 w-[92px]" :disabled="!d.url.trim()" @click="tabs.send(tab.id)"><Send class="h-3.5 w-3.5" />Send</Button>
      </Tooltip>
    </div>

    <!-- Section tabs -->
    <div class="flex shrink-0 gap-1 border-b border-line px-3" role="tablist" aria-label="Request sections">
      <button
        v-for="s in SECTIONS"
        :key="s.id"
        type="button"
        role="tab"
        :aria-selected="section === s.id"
        :class="['relative -mb-px flex h-8 items-center gap-1.5 border-b-2 px-2 text-[12px] transition-colors', section === s.id ? 'border-accent text-fg' : 'border-transparent text-muted hover:text-fg']"
        @click="section = s.id"
      >
        {{ s.label }}
        <span v-if="count(s.id)" class="rounded-full bg-raised px-1.5 text-[10px] text-muted">{{ count(s.id) }}</span>
        <span v-if="s.phase" class="text-[10px] text-faint">P{{ s.phase }}</span>
      </button>
    </div>

    <div class="min-h-0 flex-1 overflow-auto p-3">
      <KeyValueEditor v-if="section === 'params'" label="Query parameters" :model-value="d.params" @update:model-value="tabs.setParams(tab.id, $event)" />
      <KeyValueEditor
        v-else-if="section === 'headers'"
        label="Headers"
        key-placeholder="Header"
        :model-value="d.headers"
        @update:model-value="tabs.edit(tab.id, (x) => (x.headers = $event))"
      />
      <div v-else-if="section === 'body'" class="flex h-full min-h-[160px] flex-col gap-3">
        <div class="overflow-x-auto"><Segmented :model-value="d.bodyType" :options="bodyOptions" @update:model-value="(v: BodyType) => tabs.edit(tab.id, (x) => (x.bodyType = v))" /></div>
        <p v-if="noBodyMethod && d.bodyType !== 'none'" class="text-[11px] text-warning">{{ d.method }} requests are sent without a body. Choose another method to send it.</p>
        <p v-if="d.bodyType === 'none'" class="text-xs text-muted">This request has no body.</p>
        <div v-else-if="rawBody" class="min-h-[140px] flex-1 overflow-hidden rounded-md border border-line bg-panel focus-within:border-accent">
          <CodeEditor
            :model-value="d.body"
            :language="LANG[d.bodyType as 'json' | 'xml' | 'text']"
            :wrap="settings.settings.editorWordWrap"
            :placeholder="d.bodyType === 'json' ? '{ &quot;name&quot;: &quot;ENPost&quot; }' : ''"
            label="Request body"
            @update:model-value="(v: string) => tabs.edit(tab.id, (x) => (x.body = v))"
          />
        </div>
        <KeyValueEditor v-else-if="d.bodyType === 'urlencoded'" label="URL-encoded fields" :model-value="d.urlencoded" @update:model-value="tabs.edit(tab.id, (x) => (x.urlencoded = $event))" />
        <FormDataEditor v-else-if="d.bodyType === 'form-data'" :model-value="d.form" @update:model-value="tabs.edit(tab.id, (x) => (x.form = $event))" />
        <div v-else class="flex items-center gap-3">
          <Button size="sm" @click="pickBinary"><FileUp class="h-3.5 w-3.5" />{{ d.binaryPath ? "Change file" : "Choose file" }}</Button>
          <span v-if="d.binaryPath" class="flex min-w-0 items-center gap-1 font-mono text-[12px] text-muted">
            <span class="truncate" :title="d.binaryPath">{{ d.binaryPath }}</span>
            <button type="button" class="shrink-0 rounded p-0.5 text-faint hover:text-danger" aria-label="Remove file" @click="tabs.edit(tab.id, (x) => (x.binaryPath = ''))"><X class="h-3.5 w-3.5" /></button>
          </span>
          <span v-else class="text-xs text-muted">The file is sent as the whole body (Content-Type: application/octet-stream unless you set one).</span>
        </div>
      </div>
      <EmptyState v-else-if="section === 'auth'" title="Authentication" badge="Phase 3" text="Basic, Bearer, API key and OAuth 2.0, with secrets kept in the system credential store." />
      <EmptyState v-else title="Scripts and tests" badge="Phase 4" text="Pre-request and post-response scripts run in an isolated JavaScript runtime with time and memory limits." />
    </div>
  </section>
</template>
