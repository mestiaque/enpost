<script setup lang="ts">
import { startDrag } from "@/composables/useDrag";

/** A thin drag bar between two panes. Emits the pointer position while dragging. */
const props = defineProps<{ direction: "horizontal" | "vertical"; label: string }>();
const emit = defineEmits<{ drag: [ev: PointerEvent]; end: []; step: [delta: number] }>();

function onKey(e: KeyboardEvent) {
  const back = props.direction === "vertical" ? "ArrowLeft" : "ArrowUp";
  const fwd = props.direction === "vertical" ? "ArrowRight" : "ArrowDown";
  if (e.key === back || e.key === fwd) {
    e.preventDefault();
    emit("step", e.key === fwd ? 1 : -1);
    emit("end");
  }
}
</script>

<template>
  <div
    role="separator"
    tabindex="0"
    :aria-label="label"
    :aria-orientation="direction"
    :class="[
      'group relative z-10 shrink-0 outline-none',
      direction === 'vertical' ? 'w-px cursor-col-resize bg-line' : 'h-px cursor-row-resize bg-line',
    ]"
    @pointerdown="startDrag($event, (ev) => emit('drag', ev), () => emit('end'))"
    @keydown="onKey"
  >
    <!-- Wider invisible hit area; highlights on hover and keyboard focus. -->
    <div :class="['absolute transition-colors group-hover:bg-accent/60 group-focus-visible:bg-accent', direction === 'vertical' ? '-left-[3px] top-0 h-full w-[7px]' : '-top-[3px] left-0 h-[7px] w-full']" />
  </div>
</template>
