<script setup lang="ts">
import { ref } from "vue";
import { useAppsStore } from "../stores/apps";
import InstalledAppPicker from "./InstalledAppPicker.vue";

const store = useAppsStore();
const showPicker = ref(false);
</script>

<template>
  <button type="button" @click="showPicker = true" :disabled="!store.activeMode" class="glass-button glass-button--primary glass-button--small shrink-0">
    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M12 4v16m8-8H4" /></svg>
    添加程序
  </button>
  <InstalledAppPicker v-if="showPicker" @close="showPicker = false" />
</template>
