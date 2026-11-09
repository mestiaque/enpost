<script setup lang="ts" generic="T extends string | number">
import { cn } from "@/utils/cn";

const props = defineProps<{ modelValue: T; options: readonly { value: T; label: string }[]; class?: string; label?: string }>();
const emit = defineEmits<{ "update:modelValue": [value: T] }>();

function onChange(e: Event) {
  const v = (e.target as HTMLSelectElement).value;
  const o = props.options.find((x) => String(x.value) === v);
  if (o) emit("update:modelValue", o.value);
}
</script>

<template>
  <select
    :value="String(props.modelValue)"
    :aria-label="props.label"
    :class="cn('h-8 rounded-md border border-line bg-panel px-2 text-[13px] text-fg outline-none focus:border-accent', props.class)"
    @change="onChange"
  >
    <option v-for="o in props.options" :key="String(o.value)" :value="String(o.value)">{{ o.label }}</option>
  </select>
</template>
