<script setup lang="ts">
import { ContextMenuContent, ContextMenuItem, ContextMenuPortal, ContextMenuRoot, ContextMenuSeparator, ContextMenuTrigger } from "reka-ui";

export interface MenuEntry {
  label?: string;
  danger?: boolean;
  shortcut?: string;
  disabled?: boolean;
  separator?: boolean;
  run?: () => void;
}
defineProps<{ entries: MenuEntry[] }>();
const item = "flex cursor-default items-center rounded px-2 py-1.5 text-[13px] outline-none data-[disabled]:opacity-40 data-[highlighted]:bg-raised";
</script>

<template>
  <ContextMenuRoot>
    <ContextMenuTrigger as-child><slot /></ContextMenuTrigger>
    <ContextMenuPortal>
      <ContextMenuContent class="z-[90] min-w-[190px] rounded-lg border border-line bg-panel p-1 shadow-pop data-[state=open]:animate-fade-in">
        <template v-for="(e, i) in entries" :key="i">
          <ContextMenuSeparator v-if="e.separator" class="my-1 h-px bg-line" />
          <ContextMenuItem v-else :class="[item, e.danger && 'text-danger']" :disabled="e.disabled" @select="e.run?.()">
            {{ e.label }}<span v-if="e.shortcut" class="ml-auto pl-4 text-[11px] text-faint">{{ e.shortcut }}</span>
          </ContextMenuItem>
        </template>
      </ContextMenuContent>
    </ContextMenuPortal>
  </ContextMenuRoot>
</template>
