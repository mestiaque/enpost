<script setup lang="ts" generic="T extends string">
import { cn } from "@/utils/cn";

defineProps<{ modelValue: T; options: readonly { value: T; label: string }[] }>();
defineEmits<{ "update:modelValue": [value: T] }>();
</script>

<template>
  <div class="inline-flex rounded-md border border-line bg-panel p-0.5" role="radiogroup">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      role="radio"
      :aria-checked="modelValue === o.value"
      :class="cn('inline-flex h-6 items-center gap-1 rounded px-2 text-xs transition-colors', modelValue === o.value ? 'bg-raised text-fg shadow-sm' : 'text-muted hover:text-fg')"
      @click="$emit('update:modelValue', o.value)"
    >
      {{ o.label }}
    </button>
  </div>
</template>
