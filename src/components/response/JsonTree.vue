<script setup lang="ts">
/**
 * JSON tree with expand/collapse. Rows are virtualised (fixed row height): only the rows in
 * view are in the DOM, so a 100 000-node document scrolls smoothly.
 */
import { ChevronRight, Copy } from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { expandToDepth, flatten, type JsonValue, type TreeRow } from "@/utils/jsonTree";

const props = defineProps<{ value: JsonValue }>();
const emit = defineEmits<{ copied: [what: string] }>();

const ROW = 22;
const expanded = shallowRef(expandToDepth(props.value, 2));
watch(
  () => props.value,
  (v) => (expanded.value = expandToDepth(v, 2)),
);
const rows = computed(() => flatten(props.value, expanded.value));

const box = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const height = ref(400);
let ro: ResizeObserver | null = null;
onMounted(() => {
  if (!box.value) return;
  height.value = box.value.clientHeight;
  ro = new ResizeObserver(() => box.value && (height.value = box.value.clientHeight));
  ro.observe(box.value);
});
onBeforeUnmount(() => ro?.disconnect());

const first = computed(() => Math.max(0, Math.floor(scrollTop.value / ROW) - 10));
const visible = computed(() => rows.value.slice(first.value, first.value + Math.ceil(height.value / ROW) + 20));

function toggle(r: TreeRow) {
  if (!r.expandable) return;
  const next = new Set(expanded.value);
  if (next.has(r.path)) next.delete(r.path);
  else next.add(r.path);
  expanded.value = next;
}
function expandAll() {
  expanded.value = expandToDepth(props.value, Infinity);
}
function collapseAll() {
  expanded.value = new Set();
}

function preview(r: TreeRow): string {
  if (r.kind === "object") return expanded.value.has(r.path) ? "{" : `{ ${r.size} ${r.size === 1 ? "key" : "keys"} }`;
  if (r.kind === "array") return expanded.value.has(r.path) ? "[" : `[ ${r.size} ${r.size === 1 ? "item" : "items"} ]`;
  if (r.kind === "string") return JSON.stringify(r.value);
  return String(r.value);
}
const COLOR: Record<TreeRow["kind"], string> = { string: "text-[#22c55e]", number: "text-[#f59e0b]", boolean: "text-[#f59e0b]", null: "text-faint", object: "text-muted", array: "text-muted" };

async function copy(text: string, what: string) {
  await navigator.clipboard.writeText(text);
  emit("copied", what);
}
defineExpose({ expandAll, collapseAll });
</script>

<template>
  <div ref="box" class="h-full overflow-auto font-mono text-[length:var(--editor-font-size)]" role="tree" aria-label="JSON tree" @scroll="scrollTop = ($event.target as HTMLElement).scrollTop">
    <div :style="{ height: `${rows.length * ROW}px`, position: 'relative' }">
      <div
        v-for="(r, i) in visible"
        :key="r.path"
        role="treeitem"
        :aria-expanded="r.expandable ? expanded.has(r.path) : undefined"
        :aria-level="r.depth + 1"
        class="group absolute left-0 right-0 flex items-center gap-1 whitespace-nowrap pr-3 hover:bg-raised/60"
        :style="{ top: `${(first + i) * ROW}px`, height: `${ROW}px`, paddingLeft: `${8 + r.depth * 16}px` }"
        @click="toggle(r)"
      >
        <ChevronRight v-if="r.expandable" :class="['h-3.5 w-3.5 shrink-0 text-faint transition-transform', expanded.has(r.path) && 'rotate-90']" />
        <span v-else class="w-3.5 shrink-0" />
        <span v-if="r.label !== null" class="text-[rgb(var(--accent))]">{{ /^\d+$/.test(r.label) ? r.label : JSON.stringify(r.label) }}</span>
        <span v-if="r.label !== null" class="text-muted">:</span>
        <span :class="['truncate', COLOR[r.kind]]">{{ preview(r) }}</span>
        <span class="ml-auto hidden shrink-0 gap-1 group-hover:flex">
          <button type="button" class="flex items-center gap-1 rounded px-1 text-[10px] text-faint hover:bg-line hover:text-fg" title="Copy JSON path" @click.stop="copy(r.path, 'Path')">
            <Copy class="h-3 w-3" />path
          </button>
          <button type="button" class="flex items-center gap-1 rounded px-1 text-[10px] text-faint hover:bg-line hover:text-fg" title="Copy value" @click.stop="copy(typeof r.value === 'string' ? r.value : JSON.stringify(r.value, null, 2), 'Value')">
            <Copy class="h-3 w-3" />value
          </button>
        </span>
      </div>
    </div>
  </div>
</template>
