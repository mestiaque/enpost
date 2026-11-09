<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import Button from "@/components/ui/Button.vue";
import Dialog from "@/components/ui/Dialog.vue";
import appIcon from "@/assets/enpost-icon.svg";
import { api, inTauri } from "@/services/ipc";
import { useUpdate } from "@/stores/update";
import { formatBytes } from "@/utils/format";

type Phase = "idle" | "downloading" | "installing" | "installed" | "exiting" | "error";

const upd = useUpdate();
const info = computed(() => upd.available);
const open = computed({ get: () => upd.dialogOpen && info.value !== null, set: (v) => !v && !busy.value && upd.close() });
const version = ref("");
const phase = ref<Phase>("idle");
const progress = ref<{ done: number; total: number } | null>(null);
const error = ref("");
const busy = computed(() => phase.value === "downloading" || phase.value === "installing" || phase.value === "exiting");
const pct = computed(() => (progress.value?.total ? Math.min(100, (progress.value.done / progress.value.total) * 100) : null));

let unlisten: (() => void) | null = null;
onMounted(async () => {
  if (!inTauri()) return;
  version.value = (await api.appInfo().catch(() => null))?.version ?? "";
  unlisten = await listen<{ done: number; total: number }>("update-progress", (e) => {
    progress.value = e.payload;
    if (e.payload.total && e.payload.done >= e.payload.total) phase.value = "installing";
  });
});
onBeforeUnmount(() => unlisten?.());

async function start() {
  const i = info.value;
  if (!i) return;
  if (!i.installable) {
    await api.openReleasePage().catch(() => {});
    upd.close();
    return;
  }
  error.value = "";
  progress.value = null;
  phase.value = "downloading";
  try {
    phase.value = (await api.installUpdate(i)) === "exiting" ? "exiting" : "installed";
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    phase.value = "error";
  }
}
</script>

<template>
  <Dialog v-model:open="open" title="Update available" class="w-[min(460px,94vw)]">
    <div v-if="info" class="space-y-4 p-5">
      <div class="flex items-center gap-3.5">
        <img :src="appIcon" alt="" class="h-14 w-14" draggable="false" />
        <div>
          <div class="text-[16px] font-semibold">ENPost {{ info.version }} is available</div>
          <div class="text-[12px] text-faint">You have version {{ version }}.</div>
        </div>
      </div>
      <div v-if="info.notes" class="max-h-52 select-text overflow-y-auto whitespace-pre-wrap rounded-lg bg-raised p-3 text-[12px] leading-relaxed text-muted">{{ info.notes }}</div>
      <div v-if="phase === 'downloading' || phase === 'installing'" class="space-y-1.5">
        <div class="relative h-1.5 overflow-hidden rounded-full bg-raised">
          <div v-if="pct === null || phase === 'installing'" class="progress-indeterminate" />
          <div v-else class="h-full rounded-full bg-accent transition-[width]" :style="{ width: `${pct}%` }" />
        </div>
        <div class="text-[11px] text-faint">
          <template v-if="phase === 'installing'">Installing… enter your password if Ubuntu asks for it.</template>
          <template v-else-if="progress">Downloading… {{ formatBytes(progress.done) }}<template v-if="progress.total"> of {{ formatBytes(progress.total) }}</template></template>
          <template v-else>Downloading…</template>
        </div>
      </div>
      <div class="text-[12px] text-faint">
        <span v-if="phase === 'error'" class="text-danger">{{ error }}</span>
        <template v-else-if="phase === 'installed'">ENPost {{ info.version }} is installed. Restart to use it.</template>
        <template v-else-if="phase === 'exiting'">Installing… ENPost will restart by itself.</template>
        <template v-else-if="phase === 'idle'">
          {{ info.installable ? "The update downloads, is checked against its published SHA-256, and installs in place. Your collections, history and settings are kept." : "This copy was not installed from a package, so it cannot update itself. Download the new version from the release page." }}
        </template>
      </div>
      <div class="flex justify-end gap-2">
        <template v-if="phase === 'installed'">
          <Button @click="upd.close()">Restart later</Button>
          <Button variant="primary" @click="api.restartApp()">Restart now</Button>
        </template>
        <template v-else>
          <Button :disabled="busy" @click="upd.close()">Later</Button>
          <Button variant="primary" :disabled="busy" @click="start">{{ phase === "error" ? "Try again" : info.installable ? "Update now" : "Open download page" }}</Button>
        </template>
      </div>
    </div>
  </Dialog>
</template>
