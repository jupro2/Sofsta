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
  <div class="bg-white rounded-2xl shadow-sm border border-gray-200 p-5">
    <h2 class="text-lg font-semibold text-gray-700 mb-4 flex items-center gap-2">
      <svg class="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 6h16M4 10h16M4 14h16M4 18h16" />
      </svg>
      {{ store.activeMode?.name || "程序列表" }}
      <span class="text-sm font-normal text-gray-400">
        ({{ store.enabledApps.length }}/{{ store.currentApps.length }} 已启用)
      </span>
    </h2>

    <!-- 空状态 -->
    <div v-if="store.currentApps.length === 0" class="text-center py-10 text-gray-400">
      <svg class="w-14 h-14 mx-auto mb-3 text-gray-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
          d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
      </svg>
      <p class="text-lg">此模式暂无程序</p>
      <p class="text-sm mt-1">点击上方"添加程序"按钮开始</p>
    </div>

    <!-- 程序列表 -->
    <div v-else class="space-y-2">
      <div
        v-for="app in store.currentApps"
        :key="app.id"
        class="group border border-gray-100 rounded-xl p-3.5 hover:border-blue-200 hover:bg-blue-50/30 transition-all duration-150"
      >
        <div class="flex items-center gap-3">
          <!-- 开关 -->
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              :checked="app.enabled"
              @change="handleToggle(app.id, $event)"
              class="sr-only peer"
            />
            <div class="w-10 h-5 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300
                        rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white
                        after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white
                        after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4
                        after:transition-all peer-checked:bg-blue-500"></div>
          </label>

          <!-- 信息 -->
          <div class="flex-1 min-w-0">
            <span class="font-medium text-gray-800" :class="{ 'text-gray-400': !app.enabled }">
              {{ app.name }}
            </span>
            <p class="text-xs text-gray-400 truncate mt-0.5" :title="app.path">
              {{ app.path }}
            </p>
          </div>

          <!-- 延迟 -->
          <div class="flex items-center gap-1.5">
            <span class="text-xs text-gray-400">延迟</span>
            <input
              type="number"
              :value="app.delay_seconds"
              @change="handleDelayChange(app.id, $event)"
              min="0"
              max="300"
              class="w-14 px-1.5 py-1 text-xs border border-gray-200 rounded-lg
                     focus:outline-none focus:ring-2 focus:ring-blue-300 focus:border-transparent
                     text-center"
            />
            <span class="text-xs text-gray-400">秒</span>
          </div>

          <!-- 操作 -->
          <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              @click="store.launchAppNow(app.path)"
              title="立即测试启动"
              class="p-1.5 rounded-lg text-green-500 hover:bg-green-50 transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </button>
            <button
              @click="store.removeApp(app.id)"
              title="删除"
              class="p-1.5 rounded-lg text-red-400 hover:bg-red-50 hover:text-red-600 transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
