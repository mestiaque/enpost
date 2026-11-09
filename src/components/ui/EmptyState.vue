<script setup lang="ts">
defineProps<{ title: string; text?: string; badge?: string }>();
</script>

<template>
  <div class="flex h-full flex-col items-center justify-center gap-2 px-6 text-center">
    <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-raised text-muted"><slot name="icon" /></div>
    <div class="flex items-center gap-2 text-[13px] font-medium">
      {{ title }}
      <span v-if="badge" class="rounded-full border border-line px-1.5 py-px text-[10px] font-semibold uppercase tracking-wide text-faint">{{ badge }}</span>
    </div>
    <p v-if="text" class="max-w-[320px] text-xs leading-relaxed text-muted">{{ text }}</p>
    <slot />
  </div>
</template>
