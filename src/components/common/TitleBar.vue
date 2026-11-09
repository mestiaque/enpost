<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Monitor, Moon, PanelLeft, Search, Settings as Cog, Sparkles, Sun } from "lucide-vue-next";
import { computed } from "vue";
import IconButton from "@/components/ui/IconButton.vue";
import Kbd from "@/components/ui/Kbd.vue";
import WindowControls from "@/components/ui/WindowControls.vue";
import appIcon from "@/assets/enpost-icon.svg";
import { inTauri } from "@/services/ipc";
import { useSettings } from "@/stores/settings";
import { useUi } from "@/stores/ui";
import type { Theme } from "@/types";

const settings = useSettings();
const ui = useUi();
const ORDER: Theme[] = ["dark", "light", "glass", "system"];
const themeIcon = computed(() => ({ dark: Moon, light: Sun, glass: Sparkles, system: Monitor })[settings.settings.theme]);
const nextTheme = () => {
  const i = ORDER.indexOf(settings.settings.theme);
  void settings.update({ theme: ORDER[(i + 1) % ORDER.length] });
};

/** Drag the window from empty bar space; double-click maximises. */
function onMouseDown(e: MouseEvent) {
  if (!inTauri() || e.button !== 0 || (e.target as HTMLElement).closest("button,input,select,textarea,a,[role=menu],[data-no-drag]")) return;
  if (e.detail === 2) void getCurrentWindow().toggleMaximize();
  else void getCurrentWindow().startDragging();
}
</script>

<template>
  <header class="flex h-11 shrink-0 items-center gap-2 border-b border-line bg-panel pl-3 pr-2.5" @mousedown="onMouseDown">
    <img :src="appIcon" alt="" class="h-[22px] w-[22px]" draggable="false" />
    <span class="text-[14px] font-semibold tracking-tight">ENPost</span>
    <IconButton class="ml-2" label="Toggle sidebar" shortcut="Ctrl+B" @click="settings.update({ sidebarCollapsed: !settings.settings.sidebarCollapsed })">
      <PanelLeft class="h-4 w-4" />
    </IconButton>
    <button
      type="button"
      class="mx-auto flex h-8 w-[min(460px,40vw)] items-center gap-2 rounded-md border border-line bg-bg px-2.5 text-[13px] text-faint transition-colors hover:border-faint hover:text-muted"
      data-no-drag
      @click="ui.openPalette('commands')"
    >
      <Search class="h-3.5 w-3.5" />
      <span class="flex-1 truncate text-left">Search commands and tabs</span>
      <Kbd>Ctrl+K</Kbd>
    </button>
    <IconButton :label="`Theme: ${settings.settings.theme}`" @click="nextTheme">
      <component :is="themeIcon" class="h-4 w-4" />
    </IconButton>
    <IconButton label="Settings" shortcut="Ctrl+," @click="ui.openSettings()"><Cog class="h-4 w-4" /></IconButton>
    <WindowControls />
  </header>
</template>
