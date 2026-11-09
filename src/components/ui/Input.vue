<script setup lang="ts">
import { cn } from "@/utils/cn";

const model = defineModel<string>({ default: "" });
const props = defineProps<{ class?: string; mono?: boolean }>();
</script>

<template>
  <input
    v-model="model"
    spellcheck="false"
    autocomplete="off"
    :class="cn('h-8 w-full rounded-md border border-line bg-panel px-2.5 text-[13px] text-fg outline-none placeholder:text-faint focus:border-accent', props.mono && 'font-mono text-[12px]', props.class)"
  />
</template>
