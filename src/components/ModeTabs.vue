<script setup lang="ts">
import { ref } from "vue";
import { useAppsStore } from "../stores/apps";

const store = useAppsStore();

// 预设颜色
const COLORS = [
  "#3B82F6", "#EF4444", "#10B981", "#F59E0B",
  "#8B5CF6", "#EC4899", "#06B6D4", "#F97316",
];

// 预设 emoji
const EMOJIS = [
  "💼", "🎮", "🎨", "📚", "🎵", "🎬", "🛠️", "🏠",
  "🌐", "💻", "📱", "🎯", "🚀", "⭐", "💡", "🔥",
];

// 新建/编辑模式弹窗
const showModal = ref(false);
const editingModeId = ref<string | null>(null);
const formName = ref("");
const formEmoji = ref("💼");
const formColor = ref("#3B82F6");

function openAddModal() {
  editingModeId.value = null;
  formName.value = "";
  formEmoji.value = "💼";
  formColor.value = "#3B82F6";
  showModal.value = true;
}

function openEditModal(modeId: string) {
  const mode = store.modes.find((m) => m.id === modeId);
  if (!mode) return;
  editingModeId.value = modeId;
  formName.value = mode.name;
  formEmoji.value = mode.icon_emoji;
  formColor.value = mode.color;
  showModal.value = true;
}

async function handleSave() {
  if (!formName.value.trim()) return;
  try {
    if (editingModeId.value) {
      await store.updateMode(
        editingModeId.value,
        formName.value.trim(),
        formEmoji.value,
        formColor.value
      );
    } else {
      await store.addMode(
        formName.value.trim(),
        formEmoji.value,
        formColor.value
      );
    }
    showModal.value = false;
  } catch (e: any) {
    alert(e?.message || "操作失败");
  }
}

async function handleDelete(modeId: string) {
  if (store.modes.length <= 1) {
    alert("至少需要保留一个模式");
    return;
  }
  if (confirm("确定要删除此模式及其所有程序吗？")) {
    await store.removeMode(modeId);
  }
}
</script>

<template>
  <!-- 模式标签栏 -->
  <div class="flex items-center gap-2 overflow-x-auto pb-1">
    <button
      v-for="mode in store.modes"
      :key="mode.id"
      @click="store.setActiveMode(mode.id)"
      :class="[
        'group relative flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium transition-all duration-150 whitespace-nowrap',
        store.activeModeId === mode.id
          ? 'text-white shadow-md'
          : 'bg-white text-gray-600 hover:bg-gray-50 border border-gray-200',
      ]"
      :style="
        store.activeModeId === mode.id
          ? { backgroundColor: mode.color }
          : {}
      "
    >
      <span class="text-base">{{ mode.icon_emoji }}</span>
      <span>{{ mode.name }}</span>
      <span
        v-if="mode.auto_start"
        class="text-xs opacity-70"
        title="开机自启"
      >⚡</span>

      <!-- 编辑/删除按钮 -->
      <div
        v-if="store.activeModeId === mode.id"
        class="flex items-center gap-0.5 ml-1"
      >
        <button
          @click.stop="openEditModal(mode.id)"
          class="p-0.5 rounded hover:bg-white/20 transition-colors"
          title="编辑模式"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
        </button>
        <button
          @click.stop="handleDelete(mode.id)"
          class="p-0.5 rounded hover:bg-white/20 transition-colors"
          title="删除模式"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </button>

    <!-- 添加模式按钮 -->
    <button
      @click="openAddModal"
      class="flex items-center gap-1 px-3 py-2 rounded-xl text-sm text-gray-400
             hover:text-blue-500 hover:bg-blue-50 border border-dashed border-gray-300
             hover:border-blue-300 transition-all duration-150 whitespace-nowrap"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      新建模式
    </button>
  </div>

  <!-- 弹窗 -->
  <Teleport to="body">
    <div
      v-if="showModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm"
      @click.self="showModal = false"
    >
      <div class="bg-white rounded-2xl shadow-xl w-[400px] p-6 space-y-5">
        <h3 class="text-lg font-semibold text-gray-800">
          {{ editingModeId ? "编辑模式" : "新建模式" }}
        </h3>

        <!-- 名称 -->
        <div>
          <label class="block text-sm font-medium text-gray-600 mb-1">模式名称</label>
          <input
            v-model="formName"
            type="text"
            placeholder="如：办公模式"
            class="w-full px-3 py-2 border border-gray-200 rounded-xl
                   focus:outline-none focus:ring-2 focus:ring-blue-300 focus:border-transparent"
          />
        </div>

        <!-- Emoji -->
        <div>
          <label class="block text-sm font-medium text-gray-600 mb-1">图标</label>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="emoji in EMOJIS"
              :key="emoji"
              @click="formEmoji = emoji"
              :class="[
                'w-9 h-9 rounded-lg text-lg flex items-center justify-center transition-all',
                formEmoji === emoji
                  ? 'bg-blue-100 ring-2 ring-blue-400 scale-110'
                  : 'hover:bg-gray-100',
              ]"
            >
              {{ emoji }}
            </button>
          </div>
        </div>

        <!-- 颜色 -->
        <div>
          <label class="block text-sm font-medium text-gray-600 mb-1">主题色</label>
          <div class="flex gap-2">
            <button
              v-for="color in COLORS"
              :key="color"
              @click="formColor = color"
              :class="[
                'w-8 h-8 rounded-full transition-all',
                formColor === color ? 'ring-2 ring-offset-2 ring-gray-400 scale-110' : '',
              ]"
              :style="{ backgroundColor: color }"
            />
          </div>
        </div>

        <!-- 预览 -->
        <div class="flex items-center gap-2 p-3 bg-gray-50 rounded-xl">
          <span class="text-xl">{{ formEmoji }}</span>
          <span
            class="px-3 py-1 rounded-lg text-white text-sm font-medium"
            :style="{ backgroundColor: formColor }"
          >
            {{ formName || "预览" }}
          </span>
        </div>

        <!-- 操作按钮 -->
        <div class="flex justify-end gap-2 pt-2">
          <button
            @click="showModal = false"
            class="px-4 py-2 text-sm text-gray-600 rounded-xl hover:bg-gray-100 transition-colors"
          >
            取消
          </button>
          <button
            @click="handleSave"
            class="px-4 py-2 text-sm text-white bg-blue-500 rounded-xl
                   hover:bg-blue-600 transition-colors"
          >
            {{ editingModeId ? "保存" : "创建" }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
