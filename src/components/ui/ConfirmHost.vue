<script setup lang="ts">
import { computed } from "vue";
import Button from "./Button.vue";
import Dialog from "./Dialog.vue";
import { useUi } from "@/stores/ui";

const ui = useUi();
const open = computed({ get: () => ui.confirmReq !== null, set: (v) => !v && ui.answerConfirm(false) });
</script>

<template>
  <Dialog v-model:open="open" :title="ui.confirmReq?.title ?? ''" class="w-[min(420px,92vw)]">
    <div class="px-5 py-4 text-[13px] text-muted">{{ ui.confirmReq?.message }}</div>
    <div class="flex justify-end gap-2 px-5 pb-4">
      <Button @click="ui.answerConfirm(false)">Cancel</Button>
      <Button :variant="ui.confirmReq?.danger ? 'danger' : 'primary'" @click="ui.answerConfirm(true)">{{ ui.confirmReq?.confirmLabel }}</Button>
    </div>
  </Dialog>
</template>
