<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useAppsStore } from "../stores/apps";

const store = useAppsStore();
withDefaults(defineProps<{ editable?: boolean }>(), { editable: true });
const strip = ref<HTMLElement | null>(null);
watch(() => store.activeModeId, async () => {
  await nextTick();
  strip.value?.querySelector('.is-active')?.scrollIntoView({ block: "nearest", inline: "nearest" });
}, { immediate: true });

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
  <section aria-label="启动模式" :class="{ 'quick-modes': !editable }">
    <div class="mode-label">启动模式 <span>{{ store.modes.length }} 个场景</span></div>
    <div ref="strip" class="glass-panel mode-strip">
      <div
        v-for="mode in store.modes"
        :key="mode.id"
        class="mode-pill"
        :class="{ 'is-active': store.activeModeId === mode.id }"
        :style="{ '--mode-accent': mode.color }"
      >
        <button type="button" class="mode-select" :aria-pressed="store.activeModeId === mode.id" @click="store.setActiveMode(mode.id)">
          <span class="text-base" aria-hidden="true">{{ mode.icon_emoji }}</span>
          <span class="mode-name" :title="mode.name">{{ mode.name }}</span>
          <span v-if="store.runtimeStatus?.modes.some((running) => running.id === mode.id)" class="running-mode-marker">当前</span>
          <span v-if="mode.auto_start" class="auto-indicator" title="此模式已开启开机自启"><span class="sr-only">开机自启</span></span>
        </button>
        <div v-if="editable && store.activeModeId === mode.id" class="mode-actions">
          <button type="button" @click="openEditModal(mode.id)" :disabled="store.operationBusy" class="icon-button" title="编辑模式" aria-label="编辑模式">
            <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" /></svg>
          </button>
          <button type="button" @click="handleDelete(mode.id)" :disabled="store.operationBusy" class="icon-button icon-button--danger" title="删除模式" aria-label="删除模式">
            <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12" /></svg>
          </button>
        </div>
      </div>
      <button v-if="editable" type="button" @click="openAddModal" :disabled="store.operationBusy" class="glass-button mode-new">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="M12 4v16m8-8H4" /></svg>
        新建模式
      </button>
    </div>
  </section>

  <Teleport to="body">
    <div v-if="showModal" class="modal-overlay" @click.self="showModal = false" @keydown.esc="showModal = false">
      <section class="glass-dialog mode-dialog space-y-5" role="dialog" aria-modal="true" aria-labelledby="mode-dialog-title">
        <div class="flex items-center justify-between">
          <h2 id="mode-dialog-title" class="dialog-title">{{ editingModeId ? "编辑模式" : "新建模式" }}</h2>
          <button type="button" @click="showModal = false" class="icon-button" aria-label="关闭模式设置">✕</button>
        </div>
        <div>
          <label for="mode-name" class="field-label">模式名称</label>
          <input id="mode-name" v-model="formName" type="text" placeholder="如：办公模式" class="glass-input w-full" @keydown.enter="handleSave" />
        </div>
        <fieldset>
          <legend class="field-label">选择图标</legend>
          <div class="flex flex-wrap gap-2">
            <button v-for="emoji in EMOJIS" :key="emoji" type="button" @click="formEmoji = emoji" class="emoji-option" :class="{ 'is-selected': formEmoji === emoji }" :aria-pressed="formEmoji === emoji" :aria-label="`选择图标 ${emoji}`">{{ emoji }}</button>
          </div>
        </fieldset>
        <fieldset>
          <legend class="field-label">主题颜色</legend>
          <div class="flex flex-wrap gap-3 px-1 py-1">
            <button v-for="color in COLORS" :key="color" type="button" @click="formColor = color" class="color-option" :class="{ 'is-selected': formColor === color }" :style="{ backgroundColor: color }" :aria-pressed="formColor === color" :aria-label="`选择颜色 ${color}`" />
          </div>
        </fieldset>
        <div class="mode-preview" :style="{ borderColor: formColor + '35' }">
          <span class="text-xl" aria-hidden="true">{{ formEmoji }}</span>
          <span class="text-sm font-medium">{{ formName || "你的新模式" }}</span>
          <span class="ml-auto text-xs text-slate-500">预览</span>
        </div>
        <div class="flex justify-end gap-2 pt-1">
          <button type="button" @click="showModal = false" class="glass-button glass-button--quiet">取消</button>
          <button type="button" @click="handleSave" :disabled="!formName.trim()" class="glass-button glass-button--primary">{{ editingModeId ? "保存" : "创建" }}</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
