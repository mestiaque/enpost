<script setup lang="ts">
import { Radio } from "lucide-vue-next";
import EmptyState from "@/components/ui/EmptyState.vue";
</script>

<template>
  <section class="flex h-full min-h-0 flex-col" aria-label="Response">
    <div class="flex h-9 shrink-0 items-center border-b border-line px-3">
      <span class="text-[11px] font-semibold uppercase tracking-wider text-faint">Response</span>
    </div>
    <EmptyState title="No response yet" badge="Phase 2" text="Responses appear here once the Rust HTTP engine sends requests: status, time, size, headers, cookies, and pretty, raw and tree views.">
      <template #icon><Radio class="h-5 w-5" /></template>
    </EmptyState>
  </section>
</template>
