<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAppsStore, type DiscoveryResult, type InstalledApp } from "../stores/apps";

const emit = defineEmits<{ close: [] }>();
const store = useAppsStore();
const modeId = store.activeModeId;
const mode = computed(() => store.modes.find((item) => item.id === modeId));
const apps = ref<InstalledApp[]>([]);
const query = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const loading = ref(false);
const busy = ref(false);
const scanError = ref("");
const actionError = ref("");
const hasWarnings = ref(false);
const lastAdded = ref("");
const previousFocus = document.activeElement as HTMLElement | null;

const filteredApps = computed(() => {
  const text = query.value.trim().toLocaleLowerCase();
  return apps.value.filter((app) =>
    `${app.name} ${app.path}`.toLocaleLowerCase().includes(text)
  );
});

function isAdded(app: InstalledApp) {
  return mode.value?.apps.some((entry) =>
    entry.path.toLowerCase() === app.path.toLowerCase() && entry.args === app.args
  ) ?? false;
}

function message(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

async function refresh() {
  if (loading.value) return;
  loading.value = true;
  scanError.value = "";
  hasWarnings.value = false;
  try {
    const result = await invoke<DiscoveryResult>("get_installed_apps");
    apps.value = result.apps;
    hasWarnings.value = result.warnings.length > 0;
  } catch (error) {
    scanError.value = message(error);
  } finally {
    loading.value = false;
  }
}

async function add(app: InstalledApp) {
  if (busy.value || isAdded(app)) return;
  busy.value = true;
  actionError.value = "";
  try {
    await store.addApp(app.path, app, modeId);
    lastAdded.value = `已将「${app.name}」添加到「${mode.value?.name ?? "当前模式"}」`;
  } catch (error) {
    actionError.value = message(error);
  } finally {
    busy.value = false;
  }
}

async function browse() {
  if (busy.value) return;
  busy.value = true;
  actionError.value = "";
  try {
    const path = await store.pickExecutable();
    if (path) {
      await store.addApp(path, undefined, modeId);
      lastAdded.value = "程序已添加，可继续选择其他软件";
    }
  } catch (error) {
    actionError.value = message(error);
  } finally {
    busy.value = false;
  }
}

function close() {
  if (!busy.value) emit("close");
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    close();
  }
  if (event.key === "Tab") {
    const elements = Array.from(dialog.value?.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled])'
    ) ?? []);
    const first = elements[0];
    const last = elements[elements.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  }
}

const dialog = ref<HTMLElement | null>(null);
onMounted(async () => {
  document.addEventListener("keydown", handleKeydown);
  await nextTick();
  searchInput.value?.focus();
  void refresh();
});
onBeforeUnmount(() => {
  document.removeEventListener("keydown", handleKeydown);
  previousFocus?.focus();
});
</script>

<template>
  <Teleport to="body">
    <div class="modal-overlay" @click.self="close">
      <section
        ref="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="software-picker-title"
        class="glass-dialog"
      >
        <div class="dialog-header">
          <div>
            <h2 id="software-picker-title" class="dialog-title">选择电脑上的软件</h2>
            <p class="dialog-description">添加到「{{ mode?.name ?? "当前模式" }}」，可连续添加多个软件</p>
          </div>
          <button type="button" :disabled="busy" @click="close" aria-label="关闭软件列表" class="icon-button">✕</button>
        </div>

        <div class="picker-search space-y-3">
          <div class="flex gap-2">
            <input
              ref="searchInput"
              v-model="query"
              type="search"
              aria-label="搜索软件名称或路径"
              placeholder="搜索软件名称或路径…"
              class="glass-input min-w-0 flex-1"
            />
            <button type="button" :disabled="loading || busy" @click="refresh" class="glass-button">刷新</button>
          </div>
          <p v-if="actionError" role="alert" class="text-sm text-red-600">添加失败：{{ actionError }}</p>
          <p v-if="hasWarnings" role="status" class="text-sm text-amber-700">部分软件未能读取，列表可能不完整。找不到的软件可手动添加。</p>
          <p role="status" aria-live="polite" class="text-xs text-emerald-700" :class="{ 'sr-only': !lastAdded }">{{ lastAdded }}</p>
        </div>

        <div class="picker-list" :aria-busy="loading">
          <p v-if="loading" role="status" class="py-12 text-center text-sm text-gray-500">正在读取本机软件，请稍候…</p>
          <div v-else-if="scanError" role="alert" class="space-y-3 py-10 text-center text-sm">
            <p class="text-red-600">{{ scanError }}</p>
            <button type="button" @click="refresh" class="text-blue-600 hover:underline">重新读取</button>
          </div>
          <div v-else-if="filteredApps.length === 0" class="space-y-2 py-12 text-center text-sm text-gray-500">
            <p>{{ query.trim() ? "没有找到匹配的软件" : "暂未发现可添加的软件" }}</p>
            <p class="text-xs text-gray-400">可以通过下方按钮手动选择程序文件</p>
          </div>
          <ul v-else class="space-y-2">
            <li v-for="app in filteredApps" :key="JSON.stringify([app.path, app.args])" class="picker-entry">
              <span class="app-avatar" aria-hidden="true">{{ app.name.slice(0, 1).toUpperCase() }}</span>
              <div class="min-w-0 flex-1">
                <p class="app-name" :title="app.name">{{ app.name }}</p>
                <p class="app-path" :title="app.path">{{ app.path }}</p>
                <p v-if="app.args" class="app-path" :title="app.args">参数：{{ app.args }}</p>
              </div>
              <button
                type="button"
                :disabled="busy || isAdded(app) || !mode"
                @click="add(app)"
                :aria-label="`${isAdded(app) ? '已添加' : '添加'} ${app.name}`"
                class="glass-button glass-button--small picker-add shrink-0"
              >{{ isAdded(app) ? "已添加" : "添加" }}</button>
            </li>
          </ul>
        </div>

        <div class="dialog-footer">
          <div class="text-[11px] text-slate-500">
            <p v-if="!loading && !scanError">显示 {{ filteredApps.length }} / {{ apps.length }} 个软件</p>
            <p class="mt-1">部分商店应用及便携软件可能不会显示</p>
          </div>
          <div class="flex items-center gap-2">
            <button type="button" :disabled="busy || !mode" @click="browse" class="glass-button glass-button--quiet">手动选择文件</button>
            <button type="button" :disabled="busy" @click="close" class="glass-button glass-button--primary">完成</button>
          </div>
        </div>
      </section>
    </div>
  </Teleport>
</template>
