<script setup lang="ts">
import type { HttpMethod } from "@/types";

defineProps<{ method: HttpMethod }>();
</script>

<script lang="ts">
/** One colour per HTTP method, the same everywhere (tabs, editor, later the sidebar). */
export const METHOD_COLOR: Record<HttpMethod, string> = {
  GET: "text-emerald-500",
  POST: "text-amber-500",
  PUT: "text-sky-500",
  PATCH: "text-violet-500",
  DELETE: "text-rose-500",
  HEAD: "text-teal-500",
  OPTIONS: "text-fuchsia-500",
};
</script>

<template>
  <span :class="['font-mono text-[10px] font-bold tracking-tight', METHOD_COLOR[method]]">{{ method === "DELETE" ? "DEL" : method === "OPTIONS" ? "OPT" : method }}</span>
</template>
