<script setup lang="ts">
import { useAppsStore } from "../stores/apps";

const store = useAppsStore();

async function handleAdd() {
  const path = await store.pickExecutable();
  if (path) {
    try {
      await store.addApp(path);
    } catch (e: any) {
      alert("添加失败: " + (e?.message || e));
    }
  }
}
</script>

<template>
  <div class="flex items-center gap-3">
    <button
      @click="handleAdd"
      class="flex items-center gap-2 py-2.5 px-5 rounded-xl
             bg-blue-500 hover:bg-blue-600 active:bg-blue-700
             text-white font-medium transition-colors duration-150
             shadow-sm hover:shadow-md"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      添加程序
    </button>

    <!-- 一键启动按钮 -->
    <button
      v-if="store.activeMode"
      @click="store.launchMode(store.activeModeId)"
      class="flex items-center gap-2 py-2.5 px-5 rounded-xl text-white font-medium
             transition-all duration-150 shadow-sm hover:shadow-md hover:brightness-110"
      :style="{ backgroundColor: store.activeMode.color }"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      一键启动「{{ store.activeMode.name }}」
    </button>
  </div>
</template>
