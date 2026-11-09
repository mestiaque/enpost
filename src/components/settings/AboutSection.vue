<script setup lang="ts">
import { Copy, ExternalLink, FolderOpen } from "lucide-vue-next";
import { onMounted, ref } from "vue";
import Button from "@/components/ui/Button.vue";
import appIcon from "@/assets/enpost-icon.svg";
import { api, inTauri } from "@/services/ipc";
import { useUi } from "@/stores/ui";
import type { AppInfo } from "@/types";

const ui = useUi();
const info = ref<AppInfo | null>(null);
onMounted(async () => {
  if (inTauri()) info.value = await api.appInfo().catch(() => null);
});

const PLATFORM: Record<string, string> = { linux: "Linux", windows: "Windows", macos: "macOS" };
const LIBRARIES = [
  ["Tauri", "desktop shell", "MIT / Apache-2.0"],
  ["Rust · Tokio", "backend runtime", "MIT / Apache-2.0"],
  ["reqwest · rustls", "HTTP and TLS", "MIT / Apache-2.0"],
  ["SQLx · SQLite", "local database", "MIT / Apache-2.0 · public domain"],
  ["Vue · Pinia · Vue Router", "interface", "MIT"],
  ["reka-ui", "accessible components", "MIT"],
  ["CodeMirror", "code editor", "MIT"],
  ["Tailwind CSS", "styling", "MIT"],
  ["Lucide", "icons", "ISC"],
] as const;

async function run(p: Promise<unknown>, what: string) {
  try {
    await p;
  } catch (e) {
    ui.toast({ kind: "error", title: `Could not open ${what}`, detail: e instanceof Error ? e.message : String(e) });
  }
}

/** Everything a bug report needs, without personal data. */
async function copyDiagnostics() {
  const i = info.value;
  if (!i) return;
  const text = [
    `ENPost ${i.version} (${i.build})`,
    `${PLATFORM[i.platform] ?? i.platform} ${i.arch}`,
    `Tauri ${i.tauriVersion} · WebView ${i.webviewVersion || "unknown"}`,
    `Database schema v${i.schemaVersion}`,
  ].join("\n");
  await navigator.clipboard.writeText(text);
  ui.toast({ kind: "success", title: "Diagnostics copied", detail: "Paste them into your bug report" });
}
const row = "flex items-center justify-between gap-4 py-2.5";
</script>

<template>
  <div class="space-y-5 py-4 text-[13px]">
    <div class="flex items-center gap-4">
      <img :src="appIcon" alt="" class="h-16 w-16" draggable="false" />
      <div class="min-w-0">
        <div class="text-lg font-semibold">ENPost <span class="font-normal text-muted">{{ info?.version }}</span></div>
        <div class="text-muted">API development and testing, offline and local-first.</div>
        <div class="mt-1 text-[11px] text-faint">Build, send and test HTTP requests; your data stays on this computer.</div>
      </div>
    </div>

    <section>
      <div class="mb-1.5 text-[11px] font-semibold uppercase tracking-wider text-faint">This copy</div>
      <div class="divide-y divide-line rounded-lg bg-raised px-3">
        <div :class="row"><span>Version</span><span class="font-mono text-[12px] text-muted">{{ info?.version ?? "…" }} <span v-if="info?.build === 'debug'" class="ml-1 rounded bg-warning/20 px-1 text-warning">debug build</span></span></div>
        <div :class="row"><span>System</span><span class="text-muted">{{ info ? `${PLATFORM[info.platform] ?? info.platform} · ${info.arch}` : "…" }}</span></div>
        <div :class="row"><span>Tauri · WebView</span><span class="font-mono text-[12px] text-muted">{{ info ? `${info.tauriVersion} · ${info.webviewVersion || "unknown"}` : "…" }}</span></div>
        <div :class="row"><span>Database</span><span class="text-muted">SQLite, schema v{{ info?.schemaVersion ?? "…" }}</span></div>
        <div :class="row">
          <span class="min-w-0">Data folder<span class="block truncate font-mono text-[11px] text-faint" :title="info?.dataDir">{{ info?.dataDir }}</span></span>
          <Button size="sm" :disabled="!info" @click="run(api.openAppFolder('data'), 'the data folder')"><FolderOpen class="h-3.5 w-3.5" />Open</Button>
        </div>
        <div :class="row">
          <span class="min-w-0">Logs<span class="block truncate font-mono text-[11px] text-faint" :title="info?.logsDir">{{ info?.logsDir }}</span></span>
          <Button size="sm" :disabled="!info" @click="run(api.openAppFolder('logs'), 'the logs folder')"><FolderOpen class="h-3.5 w-3.5" />Open</Button>
        </div>
      </div>
      <div class="mt-2 flex justify-end">
        <Button size="sm" variant="ghost" :disabled="!info" @click="copyDiagnostics"><Copy class="h-3.5 w-3.5" />Copy diagnostics</Button>
      </div>
    </section>

    <section>
      <div class="mb-1.5 text-[11px] font-semibold uppercase tracking-wider text-faint">Developer</div>
      <div class="divide-y divide-line rounded-lg bg-raised px-3">
        <div :class="row"><span>Developed by</span><span class="text-muted">M. Estiaque Ahmed Khan</span></div>
        <div :class="row">
          <span>Contact<span class="block text-[11px] text-faint">Bug reports, ideas and feedback are welcome</span></span>
          <button type="button" class="text-accent hover:underline" @click="run(api.openLink('email'), 'your mail app')">{{ info?.email ?? "info@mestiaque.com" }}</button>
        </div>
        <div :class="row">
          <span>Website</span>
          <button type="button" class="flex items-center gap-1 text-accent hover:underline" @click="run(api.openLink('website'), 'the website')">mestiaque.com<ExternalLink class="h-3 w-3" /></button>
        </div>
        <div :class="row"><span>Also by the developer<span class="block text-[11px] text-faint">Same look and themes</span></span><span class="text-muted">ENViewer · ENSave · ENPlayer</span></div>
        <div :class="row"><span>Licence</span><span class="text-muted">© 2026 M. Estiaque Ahmed Khan · MIT</span></div>
      </div>
    </section>

    <section>
      <div class="mb-1.5 text-[11px] font-semibold uppercase tracking-wider text-faint">Built with</div>
      <div class="divide-y divide-line rounded-lg bg-raised px-3 text-[12px]">
        <div v-for="[name, what, licence] in LIBRARIES" :key="name" class="flex items-center justify-between gap-4 py-2">
          <span>{{ name }} <span class="text-faint">· {{ what }}</span></span><span class="text-faint">{{ licence }}</span>
        </div>
      </div>
    </section>
  </div>
</template>
