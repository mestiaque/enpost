<script setup lang="ts">
import { Trash2 } from "lucide-vue-next";
import { computed, ref } from "vue";
import { emptyRow } from "@/stores/tabs";
import type { KeyValue } from "@/types";

/**
 * Key/value table (params, headers, later form fields and variables). There is always one
 * empty row at the end; typing into it adds a row. Emits a new array on every change.
 */
const props = defineProps<{ modelValue: KeyValue[]; keyPlaceholder?: string; valuePlaceholder?: string; label: string }>();
const emit = defineEmits<{ "update:modelValue": [rows: KeyValue[]] }>();

// The trailing row becomes the new row under the same id, so the focused input stays
// focused while the user keeps typing; a fresh trailing row takes its place.
const trailing = ref(emptyRow());
const rows = computed(() => [...props.modelValue, trailing.value]);
const isTrailing = (r: KeyValue) => r.id === trailing.value.id;

function change(r: KeyValue, patch: Partial<KeyValue>) {
  if (isTrailing(r)) {
    const added = { ...trailing.value, ...patch };
    trailing.value = emptyRow();
    emit("update:modelValue", [...props.modelValue, added]);
    return;
  }
  emit("update:modelValue", props.modelValue.map((x) => (x.id === r.id ? { ...x, ...patch } : x)));
}
function remove(r: KeyValue) {
  emit("update:modelValue", props.modelValue.filter((x) => x.id !== r.id));
}
const cell = "h-8 w-full bg-transparent px-2.5 font-mono text-[length:var(--editor-font-size)] text-fg outline-none placeholder:text-faint focus:bg-raised/60";
</script>

<template>
  <div class="overflow-hidden rounded-md border border-line" role="table" :aria-label="label">
    <div class="grid grid-cols-[32px_1fr_1fr_32px] border-b border-line bg-raised/50 text-[11px] font-semibold uppercase tracking-wider text-faint" role="row">
      <span role="columnheader" class="sr-only">Enabled</span><span />
      <span role="columnheader" class="px-2.5 py-1.5">Key</span>
      <span role="columnheader" class="border-l border-line px-2.5 py-1.5">Value</span>
      <span />
    </div>
    <div v-for="r in rows" :key="r.id" class="group grid grid-cols-[32px_1fr_1fr_32px] border-b border-line last:border-b-0" role="row">
      <label class="flex items-center justify-center">
        <input
          v-if="!isTrailing(r)"
          type="checkbox"
          class="h-3.5 w-3.5 accent-[rgb(var(--accent))]"
          :checked="r.enabled"
          :aria-label="`Enable ${r.key || 'row'}`"
          @change="change(r, { enabled: ($event.target as HTMLInputElement).checked })"
        />
      </label>
      <input :class="[cell, !r.enabled && 'text-faint line-through']" :value="r.key" :placeholder="keyPlaceholder ?? 'Key'" spellcheck="false" aria-label="Key" @input="change(r, { key: ($event.target as HTMLInputElement).value })" />
      <input :class="[cell, 'border-l border-line', !r.enabled && 'text-faint']" :value="r.value" :placeholder="valuePlaceholder ?? 'Value'" spellcheck="false" aria-label="Value" @input="change(r, { value: ($event.target as HTMLInputElement).value })" />
      <button v-if="!isTrailing(r)" type="button" class="flex items-center justify-center text-faint opacity-0 hover:text-danger focus:opacity-100 group-hover:opacity-100" :aria-label="`Remove ${r.key || 'row'}`" @click="remove(r)">
        <Trash2 class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>
</template>
