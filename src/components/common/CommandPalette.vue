<script setup lang="ts">
import { Search } from "lucide-vue-next";
import { computed, nextTick, ref, watch } from "vue";
import Dialog from "@/components/ui/Dialog.vue";
import Kbd from "@/components/ui/Kbd.vue";
import MethodBadge from "@/components/request/MethodBadge.vue";
import { useSettings } from "@/stores/settings";
import { useTabs } from "@/stores/tabs";
import { useUi } from "@/stores/ui";
import type { HttpMethod } from "@/types";

const ui = useUi();
const tabs = useTabs();
const settings = useSettings();
const open = computed({ get: () => ui.dialog === "palette", set: (v) => (ui.dialog = v ? "palette" : null) });
const query = ref("");
const index = ref(0);
const input = ref<HTMLInputElement | null>(null);

interface Entry {
  id: string;
  label: string;
  hint?: string;
  method?: HttpMethod;
  run: () => void;
}

const commands = computed<Entry[]>(() => [
  { id: "new", label: "New request tab", hint: "Ctrl+T", run: () => tabs.open() },
  ...(tabs.activeId
    ? [
        { id: "close", label: "Close tab", hint: "Ctrl+W", run: () => tabs.activeId && tabs.close(tabs.activeId) },
        { id: "dup", label: "Duplicate tab", run: () => tabs.activeId && tabs.duplicate(tabs.activeId) },
      ]
    : []),
  ...(tabs.tabs.length > 1 ? [{ id: "others", label: "Close other tabs", run: () => tabs.activeId && tabs.closeOthers(tabs.activeId) }] : []),
  { id: "goto", label: "Go to an open tab…", hint: "Ctrl+P", run: () => setTimeout(() => ui.openPalette("tabs")) },
  { id: "sidebar", label: settings.settings.sidebarCollapsed ? "Show sidebar" : "Hide sidebar", hint: "Ctrl+B", run: () => void settings.update({ sidebarCollapsed: !settings.settings.sidebarCollapsed }) },
  { id: "t-dark", label: "Theme: Dark", run: () => void settings.update({ theme: "dark" }) },
  { id: "t-light", label: "Theme: Light", run: () => void settings.update({ theme: "light" }) },
  { id: "t-glass", label: "Theme: Glass", run: () => void settings.update({ theme: "glass" }) },
  { id: "t-system", label: "Theme: System", run: () => void settings.update({ theme: "system" }) },
  { id: "settings", label: "Settings", hint: "Ctrl+,", run: () => setTimeout(() => ui.openSettings()) },
  { id: "shortcuts", label: "Keyboard shortcuts", run: () => setTimeout(() => ui.openSettings("shortcuts")) },
]);

const tabEntries = computed<Entry[]>(() => tabs.tabs.map((t) => ({ id: t.id, label: t.name, hint: t.draft.url, method: t.draft.method, run: () => tabs.activate(t.id) })));

/** Case-insensitive match of every word of the query, in any order. */
const results = computed(() => {
  const words = query.value.toLowerCase().split(/\s+/).filter(Boolean);
  const list = ui.paletteMode === "tabs" ? tabEntries.value : commands.value;
  return list.filter((e) => words.every((w) => `${e.label} ${e.hint ?? ""}`.toLowerCase().includes(w)));
});

watch(open, async (o) => {
  if (!o) return;
  query.value = "";
  index.value = 0;
  await nextTick();
  input.value?.focus();
});
watch(query, () => (index.value = 0));

function run(e: Entry | undefined) {
  if (!e) return;
  open.value = false;
  e.run();
}
function onKey(e: KeyboardEvent) {
  const n = results.value.length;
  if (e.key === "ArrowDown" && n) index.value = (index.value + 1) % n;
  else if (e.key === "ArrowUp" && n) index.value = (index.value - 1 + n) % n;
  else if (e.key === "Enter") run(results.value[index.value]);
  else return;
  e.preventDefault();
}
</script>

<template>
  <Dialog v-model:open="open" title="Command palette" hide-header position="top" class="w-[min(560px,92vw)]">
    <div class="flex items-center gap-2 border-b border-line px-3.5">
      <Search class="h-4 w-4 text-faint" />
      <input
        ref="input"
        v-model="query"
        class="h-11 flex-1 bg-transparent text-[14px] text-fg outline-none placeholder:text-faint focus-visible:outline-none"
        :placeholder="ui.paletteMode === 'tabs' ? 'Go to an open tab…' : 'Type a command…'"
        spellcheck="false"
        aria-label="Search"
        @keydown="onKey"
      />
    </div>
    <ul class="max-h-[320px] overflow-y-auto p-1.5" role="listbox">
      <li
        v-for="(e, i) in results"
        :key="e.id"
        role="option"
        :aria-selected="i === index"
        :class="['flex cursor-default items-center gap-2.5 rounded-md px-2.5 py-2 text-[13px]', i === index ? 'bg-raised text-fg' : 'text-muted']"
        @mousemove="index = i"
        @click="run(e)"
      >
        <MethodBadge v-if="e.method" :method="e.method" />
        <span class="truncate">{{ e.label }}</span>
        <span v-if="e.hint" class="ml-auto shrink-0 truncate pl-3 text-[11px] text-faint">
          <Kbd v-if="!e.method">{{ e.hint }}</Kbd><template v-else>{{ e.hint }}</template>
        </span>
      </li>
      <li v-if="!results.length" class="px-2.5 py-6 text-center text-[13px] text-faint">
        {{ ui.paletteMode === "tabs" && !tabs.tabs.length ? "No open tabs" : "Nothing matches" }}
      </li>
    </ul>
  </Dialog>
</template>
