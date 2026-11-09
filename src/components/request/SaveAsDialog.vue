<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import Button from "@/components/ui/Button.vue";
import Dialog from "@/components/ui/Dialog.vue";
import { useTabs } from "@/stores/tabs";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";
import { saveTargets } from "@/utils/tree";

const ui = useUi();
const tabs = useTabs();
const ws = useWorkspace();
const open = computed({ get: () => ui.dialog === "save-as", set: (v) => (ui.dialog = v ? "save-as" : null) });
const name = ref("");
const target = ref("");
const targets = computed(() => saveTargets(ws.tree));
const nameInput = ref<HTMLInputElement | null>(null);
const busy = ref(false);

watch(open, async (o) => {
  if (!o) return;
  const t = tabs.tabs.find((x) => x.id === ui.saveAsTabId);
  name.value = t?.name && t.name !== "Untitled request" ? t.name : "New request";
  if (!targets.value.some((x) => key(x) === target.value)) target.value = targets.value[0] ? key(targets.value[0]) : "";
  await nextTick();
  nameInput.value?.select();
});
const key = (x: { collectionId: string; folderId: string | null }) => `${x.collectionId}/${x.folderId ?? ""}`;

async function newCollection() {
  const n = await ui.prompt({ title: "New collection", confirmLabel: "Create" });
  if (!n) return;
  ui.dialog = "save-as"; // the prompt replaced this dialog; bring it back
  const id = await ws.createCollection(n);
  if (id) target.value = `${id}/`;
}

async function save() {
  const t = targets.value.find((x) => key(x) === target.value);
  if (!t || !ui.saveAsTabId || !name.value.trim()) return;
  busy.value = true;
  const ok = await tabs.saveAs(ui.saveAsTabId, t.collectionId, t.folderId, name.value.trim());
  busy.value = false;
  if (ok) open.value = false;
}
</script>

<template>
  <Dialog v-model:open="open" title="Save request" class="w-[min(460px,92vw)]">
    <form class="space-y-4 px-5 py-4" @submit.prevent="save">
      <label class="block text-xs text-muted">
        Name
        <input ref="nameInput" v-model="name" class="mt-1.5 h-8 w-full rounded-md border border-line bg-bg px-2.5 text-[13px] text-fg outline-none focus:border-accent" maxlength="200" spellcheck="false" />
      </label>
      <div class="text-xs text-muted">
        <div class="mb-1.5 flex items-center justify-between">
          <span>Save to</span>
          <button type="button" class="text-accent hover:underline" @click="newCollection">New collection…</button>
        </div>
        <div v-if="!targets.length" class="rounded-md border border-dashed border-line px-3 py-4 text-center">No collections yet. Create one to save this request.</div>
        <div v-else class="max-h-56 overflow-y-auto rounded-md border border-line py-1" role="radiogroup" aria-label="Save to">
          <label v-for="x in targets" :key="key(x)" :class="['flex cursor-default items-center gap-2 py-1.5 pr-3 text-[13px]', target === key(x) ? 'bg-accent/15 text-fg' : 'text-muted hover:bg-raised/60']" :style="{ paddingLeft: `${12 + x.depth * 16}px` }">
            <input v-model="target" type="radio" :value="key(x)" class="accent-[rgb(var(--accent))]" />
            <span :class="['truncate', x.depth === 0 && 'font-medium']">{{ x.label }}</span>
          </label>
        </div>
      </div>
      <div class="flex justify-end gap-2">
        <Button @click="open = false">Cancel</Button>
        <Button type="submit" variant="primary" :disabled="busy || !target || !name.trim()">Save</Button>
      </div>
    </form>
  </Dialog>
</template>
