<script setup lang="ts">
import { onMounted } from "vue";
import { useAppsStore } from "./stores/apps";
import ModeTabs from "./components/ModeTabs.vue";
import AddApp from "./components/AddApp.vue";
import AppList from "./components/AppList.vue";
import Settings from "./components/Settings.vue";

const store = useAppsStore();

onMounted(() => {
  store.loadConfig();
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-50 to-blue-50">
    <!-- 标题栏 -->
    <header class="bg-white shadow-sm border-b border-gray-200 px-6 py-4">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 bg-blue-500 rounded-xl flex items-center justify-center">
          <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </div>
        <div>
          <h1 class="text-xl font-bold text-gray-800">AutoStart Manager</h1>
          <p class="text-sm text-gray-500">管理开机自启动程序 · 多模式切换</p>
        </div>
      </div>
    </header>

    <!-- 主内容 -->
    <main class="max-w-3xl mx-auto px-4 py-6 space-y-5">
      <!-- 模式标签栏 -->
      <ModeTabs />

      <!-- 添加程序 + 一键启动 -->
      <AddApp />

      <!-- 程序列表 -->
      <AppList />

      <!-- 设置 -->
      <Settings />

      <!-- 底部信息 -->
      <footer class="text-center text-sm text-gray-400 py-4">
        AutoStart Manager v1.1 · 多模式开机自启动管理工具
      </footer>
    </main>
  </div>
</template>
