<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { useAppsStore } from "./stores/apps";
import { usePanelWindow } from "./composables/usePanelWindow";
import ModeTabs from "./components/ModeTabs.vue";
import ModeActions from "./components/ModeActions.vue";
import AddApp from "./components/AddApp.vue";
import AppList from "./components/AppList.vue";
import Settings from "./components/Settings.vue";
import OperationFeedback from "./components/OperationFeedback.vue";

const store = useAppsStore();
const { desktop, pinned, initialize, togglePin, expand, restore } = usePanelWindow();
const view = ref<"compact" | "settings">("compact");
const windowBusy = ref(false);
const windowError = ref("");
let runtimeTimer: ReturnType<typeof setInterval> | undefined;

async function changeView(target: "compact" | "settings") {
  if (windowBusy.value || view.value === target) return;
  windowBusy.value = true;
  windowError.value = "";
  try {
    if (target === "settings") await expand();
    else await restore();
    view.value = target;
    await nextTick();
    document.getElementById(target === "settings" ? "back-to-panel" : "open-settings")?.focus();
    window.scrollTo(0, 0);
  } catch (error) {
    windowError.value = `调整窗口失败：${String(error)}`;
  } finally { windowBusy.value = false; }
}

async function pinWindow() {
  if (windowBusy.value) return;
  windowBusy.value = true;
  windowError.value = "";
  try { await togglePin(); }
  catch (error) { windowError.value = `设置置顶失败：${String(error)}`; }
  finally { windowBusy.value = false; }
}

function handleEscape(event: KeyboardEvent) {
  if (event.key === "Escape" && !event.defaultPrevented && view.value === "settings" && !document.querySelector('[role="dialog"]')) {
    event.preventDefault();
    void changeView("compact");
  }
}

onMounted(async () => {
  document.addEventListener("keydown", handleEscape);
  runtimeTimer = setInterval(() => { void store.loadRuntimeStatus(); }, 3000);
  await Promise.all([store.loadConfig(), store.loadRuntimeStatus()]);
  const current = store.runtimeStatus?.modes[0];
  if (current && store.modes.some((mode) => mode.id === current.id)) store.setActiveMode(current.id);
  try { await initialize(); }
  catch (error) { windowError.value = `读取窗口偏好失败：${String(error)}`; }
});
onBeforeUnmount(() => {
  if (runtimeTimer) clearInterval(runtimeTimer);
  document.removeEventListener("keydown", handleEscape);
});
</script>

<template>
  <div class="app-shell" :class="{ 'compact-shell': view === 'compact' }">
    <div :class="view === 'compact' ? 'widget-content' : 'app-content settings-content'">
      <template v-if="view === 'compact'">
        <header class="widget-header">
          <div class="widget-brand">
            <span class="brand-mark widget-logo" aria-hidden="true"><svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg></span>
            <h1>Sofsta <span>模式切换</span></h1>
          </div>
          <div class="widget-tools">
            <button type="button" class="icon-button widget-tool" :class="{ 'is-pinned': pinned }" :disabled="!desktop || windowBusy" :aria-pressed="pinned" :aria-label="pinned ? '取消窗口置顶' : '窗口置顶'" :title="pinned ? '取消窗口置顶' : '窗口置顶'" @click="pinWindow">
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="m16 3 5 5-4 1-4 6-2 1-3-3 1-2 6-4 1-4ZM9 15l-6 6" /></svg>
            </button>
            <button id="open-settings" type="button" class="icon-button widget-tool" :disabled="windowBusy" aria-label="打开设置" title="设置" @click="changeView('settings')">
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="m9.5 4 .5-2h4l.5 2 2 1 2-.5 2 3.5-1.5 1.5v3l1.5 1.5-2 3.5-2-.5-2 1-.5 2h-4l-.5-2-2-1-2 .5-2-3.5L5 13V10L3.5 8.5l2-3.5 2 .5 2-1Z" /><circle cx="12" cy="11.5" r="3" stroke-width="1.6" /></svg>
            </button>
          </div>
        </header>
        <main class="widget-main" aria-label="Sofsta 模式切换面板">
          <div class="current-mode-line">
            <span class="status-dot" :class="{ 'mode-running': store.runtimeStatus?.modes.length }" aria-hidden="true"></span>
            <span class="shrink-0 text-slate-500">当前</span>
            <strong class="min-w-0 truncate font-semibold" :title="store.runningModeLabel">{{ store.runtimeStatus?.modes.length ? store.runningModeLabel : '尚未启动' }}</strong>
          </div>
          <ModeTabs :editable="false" />
          <ModeActions />
        </main>
      </template>
      <template v-else>
        <header class="settings-header">
          <div>
            <h1 class="dialog-title">Sofsta 设置</h1>
            <p class="dialog-description">配置模式、应用与启动偏好</p>
          </div>
          <button id="back-to-panel" type="button" class="glass-button glass-button--small" :disabled="windowBusy" @click="changeView('compact')">
            <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="m14 6-6 6 6 6" /></svg>
            返回小面板
          </button>
        </header>
        <main class="settings-main" aria-label="Sofsta 模式和应用配置">
          <ModeTabs />
          <div class="configuration-actions">
            <p class="min-w-0 truncate text-xs text-slate-500" :title="store.activeMode?.name">配置「{{ store.activeMode?.name || '所选模式' }}」</p>
            <AddApp />
          </div>
          <AppList />
          <Settings />
        </main>
      </template>
      <OperationFeedback />
      <p v-if="windowError" role="alert" class="mt-3 break-words text-xs text-red-700">{{ windowError }}</p>
    </div>
  </div>
</template>
