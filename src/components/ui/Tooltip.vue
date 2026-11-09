<script setup lang="ts">
import { TooltipArrow, TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger } from "reka-ui";
import Kbd from "./Kbd.vue";

const props = withDefaults(defineProps<{ label: string; shortcut?: string; side?: "top" | "bottom" | "left" | "right" }>(), { side: "bottom" });
</script>

<template>
  <TooltipRoot :delay-duration="450">
    <TooltipTrigger as-child><slot /></TooltipTrigger>
    <TooltipPortal>
      <TooltipContent :side="props.side" :side-offset="6" class="z-[100] flex items-center gap-2 rounded-md bg-raised px-2 py-1 text-xs text-fg shadow-pop data-[state=delayed-open]:animate-fade-in data-[state=instant-open]:animate-fade-in">
        {{ props.label }}
        <Kbd v-if="props.shortcut">{{ props.shortcut }}</Kbd>
        <TooltipArrow class="fill-raised" :width="8" :height="4" />
      </TooltipContent>
    </TooltipPortal>
  </TooltipRoot>
</template>
