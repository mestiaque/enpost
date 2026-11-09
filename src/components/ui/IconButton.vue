<script setup lang="ts">
import Tooltip from "./Tooltip.vue";
import { cn } from "@/utils/cn";

// The root is a Tooltip (no DOM of its own): class and listeners go to the button itself.
defineOptions({ inheritAttrs: false });
const props = withDefaults(defineProps<{ label: string; shortcut?: string; active?: boolean; disabled?: boolean; side?: "top" | "bottom" | "left" | "right" }>(), { side: "bottom" });
</script>

<template>
  <Tooltip :label="props.label" :shortcut="props.shortcut" :side="props.side">
    <button
      v-bind="$attrs"
      type="button"
      :aria-label="props.label"
      :disabled="props.disabled"
      :class="
        cn(
          'inline-flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-raised hover:text-fg disabled:pointer-events-none disabled:opacity-35',
          props.active && 'bg-raised text-fg',
          ($attrs.class as string | undefined),
        )
      "
    >
      <slot />
    </button>
  </Tooltip>
</template>
