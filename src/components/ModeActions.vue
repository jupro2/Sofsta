<script setup lang="ts">
import { useAppsStore } from "../stores/apps";
const store = useAppsStore();
</script>

<template>
  <div class="panel-actions">
    <button v-if="store.activeMode" type="button"
      @click="store.needsModeSwitch ? store.switchMode(store.activeModeId) : store.launchMode(store.activeModeId)"
      :disabled="store.operationBusy || !store.runtimeStatus || !!store.runtimeError || !store.enabledApps.length"
      class="glass-button glass-button--primary panel-switch"
      :title="store.needsModeSwitch ? '关闭当前模式清单中的软件，再启动所选模式。请先保存内容。' : `启动「${store.activeMode.name}」中已启用的软件`">
      <svg class="h-4 w-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path v-if="store.needsModeSwitch" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M4 7h15m-4-4 4 4-4 4M20 17H5m4-4-4 4 4 4" /><path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M9 5l11 7-11 7V5z" /></svg>
      <span v-if="store.operationBusy" class="truncate">正在{{ store.operationKind === 'switch' ? '切换' : '启动' }}「{{ store.operationTargetName }}」…</span>
      <span v-else class="truncate">{{ store.needsModeSwitch ? '切换到' : store.selectedModeIsCurrent ? '补充启动' : '一键启动' }}「{{ store.activeMode.name }}」</span>
    </button>
    <p v-if="store.activeMode && !store.enabledApps.length" class="panel-hint">此模式暂无启用的程序，请在右上角设置中配置。</p>
    <p v-else-if="!store.operationBusy && !store.operationReport && !store.operationError && !store.runtimeError" class="panel-hint">{{ store.needsModeSwitch ? '切换前请保存当前工作内容' : '选择模式，一键开始' }}</p>
  </div>
</template>
