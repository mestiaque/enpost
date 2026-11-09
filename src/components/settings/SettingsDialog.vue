<script setup lang="ts">
import { Check } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import Button from "@/components/ui/Button.vue";
import Dialog from "@/components/ui/Dialog.vue";
import Kbd from "@/components/ui/Kbd.vue";
import Select from "@/components/ui/Select.vue";
import Switch from "@/components/ui/Switch.vue";
import SettingRow from "./SettingRow.vue";
import appIcon from "@/assets/nepost-icon.svg";
import { SHORTCUTS } from "@/composables/useShortcuts";
import { api, inTauri } from "@/services/ipc";
import { useSettings } from "@/stores/settings";
import { useUi } from "@/stores/ui";
import type { AppInfo, Settings, Theme } from "@/types";

const ui = useUi();
const store = useSettings();
const s = computed(() => store.settings);
const open = computed({ get: () => ui.dialog === "settings", set: (v) => (ui.dialog = v ? "settings" : null) });

const SECTIONS = [
  ["general", "General"],
  ["appearance", "Appearance"],
  ["editor", "Editor"],
  ["requests", "Requests"],
  ["shortcuts", "Shortcuts"],
  ["about", "About"],
] as const;

type BoolKey = { [K in keyof Settings]: Settings[K] extends boolean ? K : never }[keyof Settings];
const bool = (k: BoolKey) => computed({ get: () => s.value[k], set: (v: boolean) => void store.update({ [k]: v }) });
const autosave = bool("autosave");
const checkUpdates = bool("checkUpdates");
const reduceMotion = bool("reduceMotion");
const wordWrap = bool("editorWordWrap");
const followRedirects = bool("followRedirects");

const THEMES: { id: Theme; label: string; bg: string; side: string; bar: string }[] = [
  { id: "dark", label: "Dark", bg: "#131315", side: "#19191c", bar: "#2c2c32" },
  { id: "light", label: "Light", bg: "#f4f4f6", side: "#fcfcfd", bar: "#dcdce2" },
  { id: "glass", label: "Glass", bg: "linear-gradient(135deg,#3b2a7a,#7a2550 55%,#1d4f73)", side: "rgba(255,255,255,.12)", bar: "rgba(255,255,255,.28)" },
  { id: "system", label: "System", bg: "linear-gradient(90deg,#131315 50%,#f4f4f6 50%)", side: "rgba(128,128,128,.3)", bar: "rgba(128,128,128,.5)" },
];
const ACCENTS = ["#3b82f6", "#6366f1", "#8b5cf6", "#ec4899", "#ef4444", "#f97316", "#eab308", "#22c55e", "#14b8a6", "#06b6d4"];
const FONT_SIZES = Array.from({ length: 15 }, (_, i) => ({ value: i + 10, label: `${i + 10} px` }));
const TIMEOUTS = [
  { value: 0, label: "No timeout" },
  { value: 10000, label: "10 seconds" },
  { value: 30000, label: "30 seconds" },
  { value: 60000, label: "1 minute" },
  { value: 300000, label: "5 minutes" },
];
const RETENTION = [
  { value: 7, label: "7 days" },
  { value: 30, label: "30 days" },
  { value: 90, label: "90 days" },
  { value: 365, label: "1 year" },
  { value: 0, label: "Forever" },
];
const REDIRECTS = [5, 10, 20, 50].map((v) => ({ value: v, label: `${v}` }));

// A constant: "{{" inside a template interpolation would end it early.
const FONT_SAMPLE = "GET {{base_url}}/users?page=2 → 200 OK";
const fontFamily = ref(s.value.editorFontFamily);
const saveFont = () => fontFamily.value !== s.value.editorFontFamily && void store.update({ editorFontFamily: fontFamily.value.trim() });

/** Turning certificate checks off is risky: ask first. */
async function setVerifyTls(v: boolean) {
  if (!v) {
    const ok = await ui.confirm({
      title: "Turn off certificate verification?",
      message: "NEPost would accept any TLS certificate, including forged ones, for every request. Only do this for local development servers with self-signed certificates.",
      confirmLabel: "Turn off",
      danger: true,
    });
    if (!ok) return;
  }
  void store.update({ verifyTls: v });
}
const verifyTls = computed({ get: () => s.value.verifyTls, set: (v: boolean) => void setVerifyTls(v) });

async function resetAll() {
  const ok = await ui.confirm({ title: "Reset all settings?", message: "Every setting goes back to its default. Your requests are not affected.", confirmLabel: "Reset", danger: true });
  if (!ok) return;
  await store.reset();
  fontFamily.value = s.value.editorFontFamily;
  ui.toast({ kind: "success", title: "Settings reset" });
}

const info = ref<AppInfo | null>(null);
onMounted(async () => {
  if (inTauri()) info.value = await api.appInfo().catch(() => null);
});
</script>

<template>
  <Dialog v-model:open="open" title="Settings" class="h-[min(620px,88vh)] w-[min(840px,94vw)]">
    <div class="flex min-h-0 flex-1">
      <nav class="w-44 shrink-0 space-y-0.5 border-r border-line p-2" aria-label="Settings sections">
        <button
          v-for="[id, label] in SECTIONS"
          :key="id"
          type="button"
          :class="['flex h-8 w-full items-center rounded-md px-3 text-left text-[13px]', ui.settingsSection === id ? 'bg-accent/15 text-fg' : 'text-muted hover:bg-raised hover:text-fg']"
          @click="ui.settingsSection = id"
        >
          {{ label }}
        </button>
      </nav>
      <div class="min-w-0 flex-1 overflow-y-auto px-6 py-2">
        <template v-if="ui.settingsSection === 'general'">
          <SettingRow label="Save changes automatically" hint="Applies to saved requests (Phase 3)"><Switch v-model="autosave" label="Autosave" /></SettingRow>
          <SettingRow label="Keep request history for" hint="History is recorded from Phase 2; secrets are removed before saving">
            <Select :model-value="s.historyRetentionDays" :options="RETENTION" label="History retention" @update:model-value="(v: number) => store.update({ historyRetentionDays: v })" />
          </SettingRow>
          <SettingRow label="Check for updates automatically" hint="Once a day, from GitHub releases (update installer: Phase 6)"><Switch v-model="checkUpdates" label="Check for updates" /></SettingRow>
          <SettingRow label="Reset all settings" hint="Requests and data are kept"><Button size="sm" variant="danger" @click="resetAll">Reset</Button></SettingRow>
        </template>

        <template v-else-if="ui.settingsSection === 'appearance'">
          <div class="py-3">
            <div class="mb-2.5 text-[13px]">Theme</div>
            <div class="grid grid-cols-4 gap-3">
              <button v-for="t in THEMES" :key="t.id" type="button" class="group text-left" :aria-pressed="s.theme === t.id" @click="store.update({ theme: t.id })">
                <div :class="['relative h-16 overflow-hidden rounded-lg border-2 transition-colors', s.theme === t.id ? 'border-accent' : 'border-line group-hover:border-faint']" :style="{ background: t.bg }">
                  <div class="absolute inset-y-0 left-0 w-1/4" :style="{ background: t.side }" />
                  <div class="absolute left-[32%] right-3 top-3 h-2 rounded" :style="{ background: t.bar }" />
                  <div class="absolute left-[32%] right-8 top-7 h-2 rounded" :style="{ background: t.bar }" />
                </div>
                <div class="mt-1.5 text-xs" :class="s.theme === t.id ? 'text-fg' : 'text-muted'">{{ t.label }}</div>
              </button>
            </div>
          </div>
          <SettingRow label="Accent colour">
            <div class="flex gap-1.5">
              <button
                v-for="c in ACCENTS"
                :key="c"
                type="button"
                class="flex h-6 w-6 items-center justify-center rounded-full"
                :style="{ background: c, boxShadow: s.accent === c ? `0 0 0 2px rgb(var(--panel)), 0 0 0 4px ${c}` : undefined }"
                :aria-label="`Accent ${c}`"
                @click="store.update({ accent: c })"
              >
                <Check v-if="s.accent === c" class="h-3.5 w-3.5 text-white" />
              </button>
            </div>
          </SettingRow>
          <SettingRow label="Reduce motion" hint="Turns off animations and transitions"><Switch v-model="reduceMotion" label="Reduce motion" /></SettingRow>
        </template>

        <template v-else-if="ui.settingsSection === 'editor'">
          <SettingRow label="Font size" hint="URL bar, tables and body editor">
            <Select :model-value="s.editorFontSize" :options="FONT_SIZES" label="Editor font size" @update:model-value="(v: number) => store.update({ editorFontSize: v })" />
          </SettingRow>
          <SettingRow label="Font family" hint="A CSS font list; empty uses JetBrains Mono, Cascadia Mono or Ubuntu Mono">
            <input v-model="fontFamily" class="h-8 w-56 rounded-md border border-line bg-panel px-2.5 font-mono text-[12px] outline-none focus:border-accent" placeholder="Fira Code, monospace" spellcheck="false" @blur="saveFont" @keydown.enter="saveFont" />
          </SettingRow>
          <SettingRow label="Wrap long lines" hint="In the code editors (Phase 2)"><Switch v-model="wordWrap" label="Word wrap" /></SettingRow>
          <div class="mt-3 rounded-lg border border-line bg-raised/50 p-3 font-mono text-[length:var(--editor-font-size)]" :style="{ fontFamily: 'var(--editor-font)' }">
            {{ FONT_SAMPLE }}
          </div>
        </template>

        <template v-else-if="ui.settingsSection === 'requests'">
          <p class="py-3 text-xs text-muted">Defaults for every request. They take effect with the HTTP engine (Phase 2); a request can override them.</p>
          <SettingRow label="Timeout"><Select :model-value="s.requestTimeoutMs" :options="TIMEOUTS" label="Timeout" @update:model-value="(v: number) => store.update({ requestTimeoutMs: v })" /></SettingRow>
          <SettingRow label="Follow redirects"><Switch v-model="followRedirects" label="Follow redirects" /></SettingRow>
          <SettingRow label="Maximum redirects">
            <Select :model-value="s.maxRedirects" :options="REDIRECTS" label="Maximum redirects" @update:model-value="(v: number) => store.update({ maxRedirects: v })" />
          </SettingRow>
          <SettingRow label="Verify TLS certificates" hint="Keep this on. Turning it off asks for confirmation."><Switch v-model="verifyTls" label="Verify TLS certificates" /></SettingRow>
        </template>

        <template v-else-if="ui.settingsSection === 'shortcuts'">
          <div class="divide-y divide-line py-2">
            <div v-for="k in SHORTCUTS" :key="k.keys" class="flex items-center justify-between py-2.5 text-[13px]">
              <span>{{ k.label }}</span><Kbd>{{ k.keys }}</Kbd>
            </div>
          </div>
        </template>

        <template v-else>
          <div class="space-y-2 py-4 text-[13px]">
            <div class="flex items-center gap-3.5 pb-2">
              <img :src="appIcon" alt="" class="h-16 w-16" draggable="false" />
              <div>
                <div class="text-lg font-semibold">NEPost {{ info?.version }}</div>
                <div class="text-muted">API development and testing, offline and local-first.</div>
              </div>
            </div>
            <div v-if="info" class="space-y-1 text-[12px] text-faint">
              <div>{{ info.platform }} · {{ info.arch }} · database schema v{{ info.schemaVersion }}</div>
              <div>Data: <span class="font-mono">{{ info.dataDir }}</span></div>
            </div>
            <div class="pt-3 text-[11px] font-semibold uppercase tracking-wider text-faint">Developer</div>
            <div class="divide-y divide-line rounded-lg bg-raised px-3">
              <div class="flex items-center justify-between py-2.5"><span>Developed by</span><span class="text-muted">M. Estiaque Ahmed Khan</span></div>
              <div class="flex items-center justify-between py-2.5"><span>Contact</span><span class="text-muted">info@mestiaque.com</span></div>
              <div class="flex items-center justify-between py-2.5"><span>Licence</span><span class="text-muted">MIT</span></div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </Dialog>
</template>
