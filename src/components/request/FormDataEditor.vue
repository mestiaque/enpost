<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { FileUp, Trash2 } from "lucide-vue-next";
import { computed, ref } from "vue";
import { newId } from "@/stores/tabs";
import type { FormRow } from "@/types";

/** Multipart fields: each row is text or a file picked with the system dialog. */
const props = defineProps<{ modelValue: FormRow[] }>();
const emit = defineEmits<{ "update:modelValue": [rows: FormRow[]] }>();

const blank = (): FormRow => ({ id: newId(), key: "", value: "", enabled: true, kind: "text" });
const trailing = ref(blank());
const rows = computed(() => [...props.modelValue, trailing.value]);
const isTrailing = (r: FormRow) => r.id === trailing.value.id;

function change(r: FormRow, patch: Partial<FormRow>) {
  if (isTrailing(r)) {
    const added = { ...trailing.value, ...patch };
    trailing.value = blank();
    emit("update:modelValue", [...props.modelValue, added]);
  } else emit("update:modelValue", props.modelValue.map((x) => (x.id === r.id ? { ...x, ...patch } : x)));
}
async function pick(r: FormRow) {
  const path = await open({ title: "Choose a file to upload", multiple: false, directory: false });
  if (typeof path === "string") change(r, { kind: "file", value: path, key: r.key || (path.split(/[\\/]/).pop() ?? "file") });
}
async function onKind(r: FormRow, e: Event) {
  const el = e.target as HTMLSelectElement;
  if (el.value === "file") {
    await pick(r);
    el.value = rows.value.find((x) => x.id === r.id)?.kind ?? r.kind; // dialog cancelled: show the real type
  } else change(r, { kind: "text", value: "" });
}
const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;
const cell = "h-8 w-full bg-transparent px-2.5 font-mono text-[length:var(--editor-font-size)] text-fg outline-none placeholder:text-faint focus:bg-raised/60";
</script>

<template>
  <div class="overflow-hidden rounded-md border border-line" role="table" aria-label="Form data">
    <div class="grid grid-cols-[32px_1fr_1fr_80px_32px] border-b border-line bg-raised/50 text-[11px] font-semibold uppercase tracking-wider text-faint" role="row">
      <span /><span class="px-2.5 py-1.5">Key</span><span class="border-l border-line px-2.5 py-1.5">Value</span><span class="px-2 py-1.5">Type</span><span />
    </div>
    <div v-for="r in rows" :key="r.id" class="group grid grid-cols-[32px_1fr_1fr_80px_32px] border-b border-line last:border-b-0" role="row">
      <label class="flex items-center justify-center">
        <input type="checkbox" class="h-3.5 w-3.5 accent-[rgb(var(--accent))]" :checked="r.enabled" :disabled="isTrailing(r)" aria-label="Enabled" @change="change(r, { enabled: ($event.target as HTMLInputElement).checked })" />
      </label>
      <input :class="cell" :value="r.key" placeholder="Key" spellcheck="false" aria-label="Key" @input="change(r, { key: ($event.target as HTMLInputElement).value })" />
      <input
        v-if="r.kind === 'text'"
        :class="[cell, 'border-l border-line']"
        :value="r.value"
        placeholder="Value"
        spellcheck="false"
        aria-label="Value"
        @input="change(r, { value: ($event.target as HTMLInputElement).value })"
      />
      <button v-else type="button" class="flex h-8 items-center gap-1.5 truncate border-l border-line px-2.5 text-left text-[12px] text-fg hover:bg-raised/60" :title="r.value" @click="pick(r)">
        <FileUp class="h-3.5 w-3.5 shrink-0 text-accent" /><span class="truncate">{{ r.value ? fileName(r.value) : "Choose file…" }}</span>
      </button>
      <select
        class="h-8 bg-transparent px-2 text-[12px] text-muted outline-none"
        :value="r.kind"
        aria-label="Field type"
        @change="onKind(r, $event)"
      >
        <option value="text">Text</option>
        <option value="file">File</option>
      </select>
      <button v-if="!isTrailing(r)" type="button" class="flex items-center justify-center text-faint opacity-0 hover:text-danger group-hover:opacity-100" aria-label="Remove" @click="emit('update:modelValue', modelValue.filter((x) => x.id !== r.id))">
        <Trash2 class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>
</template>
