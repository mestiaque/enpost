<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import Button from "./Button.vue";
import Dialog from "./Dialog.vue";
import { useUi } from "@/stores/ui";

const ui = useUi();
const open = computed({ get: () => ui.promptReq !== null, set: (v) => !v && ui.answerPrompt(null) });
const value = ref("");
const input = ref<HTMLInputElement | null>(null);
watch(
  () => ui.promptReq,
  async (r) => {
    if (!r) return;
    value.value = r.initial;
    await nextTick();
    input.value?.focus();
    input.value?.select();
  },
);
const submit = () => value.value.trim() && ui.answerPrompt(value.value.trim());
</script>

<template>
  <Dialog v-model:open="open" :title="ui.promptReq?.title ?? ''" class="w-[min(420px,92vw)]">
    <form class="space-y-3 px-5 py-4" @submit.prevent="submit">
      <label class="block text-xs text-muted">
        {{ ui.promptReq?.label ?? "Name" }}
        <input ref="input" v-model="value" class="mt-1.5 h-8 w-full rounded-md border border-line bg-bg px-2.5 text-[13px] text-fg outline-none focus:border-accent" spellcheck="false" maxlength="200" />
      </label>
      <div class="flex justify-end gap-2">
        <Button @click="ui.answerPrompt(null)">Cancel</Button>
        <Button type="submit" variant="primary" :disabled="!value.trim()">{{ ui.promptReq?.confirmLabel }}</Button>
      </div>
    </form>
  </Dialog>
</template>
