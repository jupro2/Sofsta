<script setup lang="ts">
import { useAppsStore } from "../stores/apps";

const store = useAppsStore();

function handleExitDelayChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const val = parseInt(target.value, 10);
  if (!isNaN(val) && val >= 1 && val <= 60) {
    store.setExitDelay(val);
  }
}
</script>

<template>
  <section class="glass-panel content-panel" aria-labelledby="settings-title">
    <div class="panel-heading mb-1">
      <h2 id="settings-title" class="panel-title">
        <span class="section-icon" aria-hidden="true"><svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 7h9m4 0h3M4 17h3m4 0h9" /><circle cx="15" cy="7" r="2" stroke-width="1.5" /><circle cx="9" cy="17" r="2" stroke-width="1.5" /></svg></span>
        启动偏好
      </h2>
      <span class="text-[11px] text-slate-500">启动与偏好</span>
    </div>
    <div>
      <div class="preference-row">
        <div class="min-w-0">
          <p id="mode-autostart-label" class="preference-title">当前模式开机自启</p>
          <p id="mode-autostart-description" class="preference-description">开机时启动「{{ store.activeMode?.name || '-' }}」中的程序</p>
        </div>
        <label class="glass-switch">
          <input type="checkbox" :checked="store.activeMode?.auto_start || false" :disabled="!store.activeMode" @change="store.setModeAutoStart(store.activeModeId, ($event.target as HTMLInputElement).checked)" aria-labelledby="mode-autostart-label" aria-describedby="mode-autostart-description" class="switch-input" />
          <span class="switch-track" aria-hidden="true"></span>
        </label>
      </div>
      <div class="preference-row">
        <div>
          <p id="autostart-label" class="preference-title">开机自启动</p>
          <p id="autostart-description" class="preference-description">总开关 · 随 Windows 登录自动运行</p>
        </div>
        <label class="glass-switch">
          <input type="checkbox" :checked="store.autoStartEnabled" @change="store.setAutoStart(($event.target as HTMLInputElement).checked)" aria-labelledby="autostart-label" aria-describedby="autostart-description" class="switch-input" />
          <span class="switch-track" aria-hidden="true"></span>
        </label>
      </div>
      <div class="preference-row">
        <div>
          <p id="silent-label" class="preference-title">静默启动</p>
          <p id="silent-description" class="preference-description">开机时在后台启动程序，不显示界面</p>
        </div>
        <label class="glass-switch">
          <input type="checkbox" :checked="store.silentLaunch" @change="store.setSilentLaunch(($event.target as HTMLInputElement).checked)" aria-labelledby="silent-label" aria-describedby="silent-description" class="switch-input" />
          <span class="switch-track" aria-hidden="true"></span>
        </label>
      </div>
      <div class="preference-row pb-0">
        <div>
          <label for="exit-delay" class="preference-title">自动退出延迟</label>
          <p id="exit-delay-description" class="preference-description">程序启动完成后，等待指定时间退出</p>
        </div>
        <div class="flex shrink-0 items-center gap-2 text-xs text-slate-500">
          <input id="exit-delay" type="number" :value="store.exitDelayMinutes" @change="handleExitDelayChange" min="1" max="60" aria-describedby="exit-delay-description" class="glass-input w-14 px-2 py-1.5 text-center text-sm" />
          <span>分钟</span>
        </div>
      </div>
    </div>
  </section>
</template>
