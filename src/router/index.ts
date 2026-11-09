import { createRouter, createWebHashHistory } from "vue-router";
import WorkspaceView from "@/views/WorkspaceView.vue";

/**
 * Hash history: the app is served from files, so there is no server to rewrite paths.
 * One workspace view for now; environments, history and the runner get routes when built.
 */
export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "workspace", component: WorkspaceView },
    { path: "/:pathMatch(.*)*", redirect: "/" },
  ],
});
