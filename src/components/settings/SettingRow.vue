<script setup lang="ts">
defineProps<{ label: string; hint?: string }>();
</script>

<template>
  <div class="flex items-center justify-between gap-6 border-b border-line py-3 last:border-b-0">
    <div class="min-w-0">
      <div class="text-[13px]">{{ label }}</div>
      <div v-if="hint" class="mt-0.5 text-[11px] text-faint">{{ hint }}</div>
    </div>
    <div class="shrink-0"><slot /></div>
  </div>
</template>
