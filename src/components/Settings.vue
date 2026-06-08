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
  <div class="bg-white rounded-2xl shadow-sm border border-gray-200 p-5">
    <h2 class="text-lg font-semibold text-gray-700 mb-4 flex items-center gap-2">
      <svg class="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
      设置
    </h2>

    <div class="space-y-4">
      <!-- 模式自启设置 -->
      <div class="flex items-center justify-between">
        <div>
          <span class="font-medium text-gray-700">当前模式开机自启</span>
          <p class="text-sm text-gray-400">
            开机时自动启动「{{ store.activeMode?.name || '-' }}」中的程序
          </p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            :checked="store.activeMode?.auto_start || false"
            @change="store.setModeAutoStart(store.activeModeId, ($event.target as HTMLInputElement).checked)"
            class="sr-only peer"
          />
          <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300
                      rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white
                      after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white
                      after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5
                      after:transition-all peer-checked:bg-blue-500"></div>
        </label>
      </div>

      <!-- 全局开关 -->
      <div class="flex items-center justify-between">
        <div>
          <span class="font-medium text-gray-700">开机自启动（总开关）</span>
          <p class="text-sm text-gray-400">注册 Windows 开机启动项</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            :checked="store.autoStartEnabled"
            @change="store.setAutoStart(($event.target as HTMLInputElement).checked)"
            class="sr-only peer"
          />
          <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300
                      rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white
                      after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white
                      after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5
                      after:transition-all peer-checked:bg-blue-500"></div>
        </label>
      </div>

      <!-- 静默启动 -->
      <div class="flex items-center justify-between">
        <div>
          <span class="font-medium text-gray-700">静默启动</span>
          <p class="text-sm text-gray-400">开机时不显示界面，直接在后台启动程序</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            :checked="store.silentLaunch"
            @change="store.setSilentLaunch(($event.target as HTMLInputElement).checked)"
            class="sr-only peer"
          />
          <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300
                      rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white
                      after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white
                      after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5
                      after:transition-all peer-checked:bg-blue-500"></div>
        </label>
      </div>

      <!-- 自动退出延迟 -->
      <div class="flex items-center justify-between">
        <div>
          <span class="font-medium text-gray-700">自动退出延迟</span>
          <p class="text-sm text-gray-400">所有程序启动后，本应用自动退出的时间</p>
        </div>
        <div class="flex items-center gap-2">
          <input
            type="number"
            :value="store.exitDelayMinutes"
            @change="handleExitDelayChange"
            min="1"
            max="60"
            class="w-16 px-2 py-1.5 text-sm border border-gray-200 rounded-lg
                   focus:outline-none focus:ring-2 focus:ring-blue-300 focus:border-transparent
                   text-center"
          />
          <span class="text-sm text-gray-500">分钟</span>
        </div>
      </div>
    </div>
  </div>
</template>
