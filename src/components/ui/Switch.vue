<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from "reka-ui";

const model = defineModel<boolean>({ required: true });
defineProps<{ label?: string }>();
</script>

<template>
  <SwitchRoot v-model="model" :aria-label="label" class="relative h-5 w-9 shrink-0 rounded-full bg-line transition-colors data-[state=checked]:bg-accent">
    <SwitchThumb class="block h-4 w-4 translate-x-0.5 rounded-full bg-white shadow transition-transform data-[state=checked]:translate-x-[18px]" />
  </SwitchRoot>
</template>
