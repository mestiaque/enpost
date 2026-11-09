<script setup lang="ts">
import { DialogClose, DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from "reka-ui";
import { X } from "lucide-vue-next";
import { cn } from "@/utils/cn";

const open = defineModel<boolean>("open", { required: true });
// `top` places the dialog high on the screen (command palette); default is centred.
// Positioning uses a flex wrapper, not translate: the open animation sets `transform`
// and would otherwise cancel a translate-based centring.
withDefaults(defineProps<{ title: string; description?: string; class?: string; hideHeader?: boolean; position?: "center" | "top" }>(), { position: "center" });
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-[80] bg-black/50 backdrop-blur-[2px] animate-fade-in" />
      <div :class="['pointer-events-none fixed inset-0 z-[81] flex justify-center p-4', position === 'top' ? 'items-start pt-[14vh]' : 'items-center']">
      <DialogContent
        :class="cn('pointer-events-auto flex max-h-[88vh] flex-col overflow-hidden rounded-xl border border-line bg-panel shadow-pop outline-none animate-scale-in', $props.class)"
        @keydown.stop
      >
        <div v-if="!hideHeader" class="flex items-center justify-between border-b border-line px-5 py-3.5">
          <div>
            <DialogTitle class="text-[15px] font-semibold">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="mt-0.5 text-xs text-muted">{{ description }}</DialogDescription>
          </div>
          <DialogClose class="rounded-md p-1 text-muted hover:bg-raised hover:text-fg" aria-label="Close"><X class="h-4 w-4" /></DialogClose>
        </div>
        <template v-else>
          <DialogTitle class="sr-only">{{ title }}</DialogTitle>
          <DialogDescription class="sr-only">{{ description ?? title }}</DialogDescription>
        </template>
        <slot />
      </DialogContent>
      </div>
    </DialogPortal>
  </DialogRoot>
</template>
