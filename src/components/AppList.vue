<script setup lang="ts">
import { useAppsStore } from "../stores/apps";

const store = useAppsStore();

function handleToggle(appId: string, event: Event) {
  const target = event.target as HTMLInputElement;
  store.toggleApp(appId, target.checked);
}

function handleDelayChange(appId: string, event: Event) {
  const target = event.target as HTMLInputElement;
  const val = parseInt(target.value, 10);
  if (!isNaN(val) && val >= 0) {
    store.updateDelay(appId, val);
  }
}
</script>

<template>
  <section class="glass-panel content-panel" aria-labelledby="app-list-title">
    <div class="panel-heading">
      <h2 id="app-list-title" class="panel-title">
        <span class="section-icon" aria-hidden="true"><svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3.5" y="3.5" width="6" height="6" rx="2" stroke-width="1.5" /><rect x="14.5" y="3.5" width="6" height="6" rx="2" stroke-width="1.5" /><rect x="3.5" y="14.5" width="6" height="6" rx="2" stroke-width="1.5" /><rect x="14.5" y="14.5" width="6" height="6" rx="2" stroke-width="1.5" /></svg></span>
        <span class="truncate">{{ store.activeMode?.name || "程序列表" }}</span>
      </h2>
      <span class="count-badge">{{ store.enabledApps.length }} / {{ store.currentApps.length }} 已启用</span>
    </div>
    <div v-if="store.currentApps.length === 0" class="empty-state">
      <div class="empty-orb" aria-hidden="true"><svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.3" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" /></svg></div>
      <p class="text-sm font-medium text-slate-600">为这个模式添加一些应用</p>
      <p class="mt-2 text-xs">点击上方「添加程序」，安排你的开机清单</p>
    </div>
    <div v-else class="space-y-2">
      <div v-for="app in store.currentApps" :key="app.id" class="group app-row" :class="{ 'is-disabled': !app.enabled }">
        <div class="app-row-layout">
          <div class="app-summary">
            <span class="app-avatar" aria-hidden="true">{{ app.name.slice(0, 1).toUpperCase() }}</span>
            <div class="min-w-0">
              <span class="app-name" :title="app.name">{{ app.name }}</span>
              <p class="app-path" :title="app.path">{{ app.path }}</p>
            </div>
          </div>
          <div class="delay-control">
            <span class="delay-label">延迟</span>
            <input type="number" :value="app.delay_seconds" @change="handleDelayChange(app.id, $event)" min="0" max="300" :aria-label="`${app.name} 启动延迟（秒）`" class="glass-input delay-input" />
            <span>秒</span>
          </div>
          <div class="row-controls">
            <label class="glass-switch">
              <input type="checkbox" :checked="app.enabled" @change="handleToggle(app.id, $event)" :aria-label="`启用 ${app.name}`" class="switch-input" />
              <span class="switch-track" aria-hidden="true"></span>
            </label>
            <div class="row-actions">
              <button type="button" @click="store.launchAppNow(app)" :disabled="store.operationBusy || !store.runtimeStatus || !!store.runtimeError" title="立即测试启动" :aria-label="`立即测试启动 ${app.name}`" class="icon-button">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="M9 5l11 7-11 7V5z" /></svg>
              </button>
              <button type="button" @click="store.removeApp(app.id)" title="删除" :aria-label="`删除 ${app.name}`" class="icon-button icon-button--danger">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
