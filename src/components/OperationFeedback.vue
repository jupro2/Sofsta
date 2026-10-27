<script setup lang="ts">
import { computed, ref } from "vue";
import { useAppsStore } from "../stores/apps";
import { useDialogFocus } from "../composables/useDialogFocus";

const store = useAppsStore();
const showDetails = ref(false);
const dialog = ref<HTMLElement | null>(null);
useDialogFocus(dialog, () => { showDetails.value = false; });
const report = computed(() => store.operationReport);
const hasProblem = computed(() => !!store.runtimeError || !!store.operationError || !!report.value?.unclosed.length || !!report.value?.failures.length || !!report.value?.warnings.length);
const summary = computed(() => {
  if (store.operationBusy) return `正在${store.operationKind === "switch" ? "切换" : "启动"}，请稍候…`;
  if (store.operationError) return "操作未完成";
  if (store.runtimeError) return "运行状态读取失败";
  if (report.value?.unclosed.length) return `切换暂停 · ${report.value.unclosed.length} 个程序未退出`;
  if (report.value?.failures.length) return `${report.value.failures.length} 个程序启动失败`;
  if (report.value?.warnings.length) return "操作完成，请查看提示";
  if (report.value) return store.operationKind === "switch" ? `已切换到「${store.operationTargetName}」` : "启动操作完成";
  return "";
});

async function retry(force: boolean) {
  const id = store.blockedSwitchModeId;
  showDetails.value = false;
  await store.switchMode(id, force);
  if (hasProblem.value) showDetails.value = true;
}
</script>

<template>
  <div v-if="summary" class="operation-feedback" :class="{ 'has-problem': hasProblem && !store.operationBusy }">
    <p role="status" class="min-w-0 flex-1 truncate" :title="summary">{{ summary }}</p>
    <button v-if="!store.operationBusy" type="button" @click="showDetails = true" class="feedback-details">{{ hasProblem ? '查看处理' : '详情' }}</button>
  </div>
  <Teleport to="body">
    <div v-if="showDetails" class="modal-overlay" @click.self="showDetails = false">
      <section ref="dialog" class="glass-dialog feedback-dialog" role="dialog" aria-modal="true" aria-labelledby="operation-details-title">
        <div class="dialog-header">
          <h2 id="operation-details-title" class="dialog-title">运行详情</h2>
          <button type="button" class="icon-button" aria-label="关闭运行详情" @click="showDetails = false">✕</button>
        </div>
        <div class="feedback-body space-y-3">
          <p v-if="store.runtimeError" role="alert" class="text-amber-800">{{ store.runtimeError }} <button type="button" class="underline" @click="store.loadRuntimeStatus">重新读取</button></p>
          <p v-if="store.operationError" role="alert" class="text-red-700">{{ store.operationError }}</p>
          <template v-if="report">
            <template v-if="report.unclosed.length">
              <p class="font-medium text-amber-800">切换已暂停，以下程序仍在运行：{{ report.unclosed.join('、') }}</p>
              <p class="text-slate-600">请保存内容并退出这些程序后重试。强制结束可能丢失未保存的内容。</p>
              <div class="flex flex-wrap gap-2">
                <button type="button" class="glass-button glass-button--small" @click="retry(false)">重试切换到「{{ store.operationTargetName }}」</button>
                <button type="button" class="glass-button glass-button--small force-switch" @click="retry(true)">强制结束并切换到「{{ store.operationTargetName }}」</button>
              </div>
            </template>
            <template v-else>
              <p :class="report.failures.length ? 'text-amber-800' : 'text-emerald-800'">
                {{ report.failures.length ? '操作部分完成' : store.operationKind === 'switch' ? `已切换到「${store.operationTargetName}」` : '启动操作完成' }}：
                <span v-if="report.closed">已关闭 {{ report.closed }} 个程序，</span>新启动 {{ report.started }} 个程序。
              </p>
              <p v-if="report.already_running.length" class="text-slate-600">{{ report.already_running.join('、') }} 已在运行，已纳入当前模式管理，未重复启动。</p>
              <p v-for="failure in report.failures" :key="failure.name" class="text-red-700">{{ failure.name }}：{{ failure.message }}</p>
            </template>
            <p v-for="warning in report.warnings" :key="warning" class="text-amber-800">{{ warning }}</p>
          </template>
        </div>
        <div class="dialog-footer justify-end">
          <button type="button" class="glass-button glass-button--primary" @click="showDetails = false">关闭详情</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
