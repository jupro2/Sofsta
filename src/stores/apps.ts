import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface AppEntry {
  id: string;
  name: string;
  path: string;
  args: string;
  enabled: boolean;
  delay_seconds: number;
}

export interface Mode {
  id: string;
  name: string;
  icon_emoji: string;
  color: string;
  auto_start: boolean;
  apps: AppEntry[];
}

export interface AppConfig {
  modes: Mode[];
  exit_delay_minutes: number;
  auto_start_enabled: boolean;
  silent_launch: boolean;
}

export const useAppsStore = defineStore("apps", () => {
  const modes = ref<Mode[]>([]);
  const activeModeId = ref<string>("");
  const exitDelayMinutes = ref(5);
  const autoStartEnabled = ref(true);
  const silentLaunch = ref(true);

  // 当前选中的模式
  const activeMode = computed(() =>
    modes.value.find((m) => m.id === activeModeId.value) || null
  );

  // 当前模式的程序列表
  const currentApps = computed(() => activeMode.value?.apps ?? []);

  // 当前模式已启用的程序
  const enabledApps = computed(() =>
    currentApps.value.filter((a) => a.enabled)
  );

  // ─────────────────── 加载配置 ───────────────────

  async function loadConfig() {
    try {
      const config = (await invoke("get_config")) as AppConfig;
      modes.value = config.modes;
      exitDelayMinutes.value = config.exit_delay_minutes;
      autoStartEnabled.value = config.auto_start_enabled;
      silentLaunch.value = config.silent_launch;

      // 如果没有选中模式或选中的模式已不存在，选第一个
      if (
        !activeModeId.value ||
        !modes.value.find((m) => m.id === activeModeId.value)
      ) {
        activeModeId.value = modes.value[0]?.id || "";
      }
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  // ─────────────────── 模式管理 ───────────────────

  function setActiveMode(id: string) {
    activeModeId.value = id;
  }

  async function addMode(name: string, emoji: string, color: string) {
    try {
      const mode = (await invoke("add_mode", {
        name,
        iconEmoji: emoji,
        color,
      })) as Mode;
      modes.value.push(mode);
      activeModeId.value = mode.id;
    } catch (e) {
      console.error("Failed to add mode:", e);
      throw e;
    }
  }

  async function updateMode(
    id: string,
    name: string,
    emoji: string,
    color: string
  ) {
    try {
      await invoke("update_mode", { id, name, iconEmoji: emoji, color });
      const mode = modes.value.find((m) => m.id === id);
      if (mode) {
        mode.name = name;
        mode.icon_emoji = emoji;
        mode.color = color;
      }
    } catch (e) {
      console.error("Failed to update mode:", e);
      throw e;
    }
  }

  async function removeMode(id: string) {
    try {
      await invoke("remove_mode", { id });
      modes.value = modes.value.filter((m) => m.id !== id);
      if (activeModeId.value === id) {
        activeModeId.value = modes.value[0]?.id || "";
      }
    } catch (e) {
      console.error("Failed to remove mode:", e);
      throw e;
    }
  }

  async function setModeAutoStart(modeId: string, enabled: boolean) {
    try {
      await invoke("set_mode_auto_start", { modeId, enabled });
      const mode = modes.value.find((m) => m.id === modeId);
      if (mode) mode.auto_start = enabled;
    } catch (e) {
      console.error("Failed to set mode auto start:", e);
    }
  }

  // ─────────────────── 程序管理 ───────────────────

  async function addApp(appPath: string) {
    if (!activeModeId.value) return;
    try {
      const app = (await invoke("add_app_to_mode", {
        modeId: activeModeId.value,
        path: appPath,
      })) as AppEntry;
      const mode = modes.value.find((m) => m.id === activeModeId.value);
      if (mode) mode.apps.push(app);
    } catch (e) {
      console.error("Failed to add app:", e);
      throw e;
    }
  }

  async function removeApp(appId: string) {
    if (!activeModeId.value) return;
    try {
      await invoke("remove_app", { modeId: activeModeId.value, appId });
      const mode = modes.value.find((m) => m.id === activeModeId.value);
      if (mode) mode.apps = mode.apps.filter((a) => a.id !== appId);
    } catch (e) {
      console.error("Failed to remove app:", e);
    }
  }

  async function toggleApp(appId: string, enabled: boolean) {
    if (!activeModeId.value) return;
    try {
      await invoke("toggle_app", {
        modeId: activeModeId.value,
        appId,
        enabled,
      });
      const mode = modes.value.find((m) => m.id === activeModeId.value);
      const app = mode?.apps.find((a) => a.id === appId);
      if (app) app.enabled = enabled;
    } catch (e) {
      console.error("Failed to toggle app:", e);
    }
  }

  async function updateDelay(appId: string, seconds: number) {
    if (!activeModeId.value) return;
    try {
      await invoke("update_delay", {
        modeId: activeModeId.value,
        appId,
        seconds,
      });
      const mode = modes.value.find((m) => m.id === activeModeId.value);
      const app = mode?.apps.find((a) => a.id === appId);
      if (app) app.delay_seconds = seconds;
    } catch (e) {
      console.error("Failed to update delay:", e);
    }
  }

  // ─────────────────── 启动 ───────────────────

  async function launchMode(modeId: string) {
    try {
      await invoke("launch_mode", { modeId });
    } catch (e) {
      console.error("Failed to launch mode:", e);
    }
  }

  async function launchAppNow(path: string) {
    try {
      await invoke("launch_app_now", { path });
    } catch (e) {
      console.error("Failed to launch app:", e);
    }
  }

  async function pickExecutable(): Promise<string | null> {
    try {
      return (await invoke("pick_executable")) as string | null;
    } catch (e) {
      console.error("Failed to pick executable:", e);
      return null;
    }
  }

  // ─────────────────── 设置 ───────────────────

  async function setExitDelay(minutes: number) {
    try {
      await invoke("set_exit_delay", { minutes });
      exitDelayMinutes.value = minutes;
    } catch (e) {
      console.error("Failed to set exit delay:", e);
    }
  }

  async function setAutoStart(enabled: boolean) {
    try {
      await invoke("set_auto_start", { enabled });
      autoStartEnabled.value = enabled;
    } catch (e) {
      console.error("Failed to set auto start:", e);
    }
  }

  async function setSilentLaunch(enabled: boolean) {
    try {
      await invoke("set_silent_launch", { enabled });
      silentLaunch.value = enabled;
    } catch (e) {
      console.error("Failed to set silent launch:", e);
    }
  }

  return {
    // state
    modes,
    activeModeId,
    exitDelayMinutes,
    autoStartEnabled,
    silentLaunch,
    // computed
    activeMode,
    currentApps,
    enabledApps,
    // config
    loadConfig,
    setExitDelay,
    setAutoStart,
    setSilentLaunch,
    // mode
    setActiveMode,
    addMode,
    updateMode,
    removeMode,
    setModeAutoStart,
    // app
    addApp,
    removeApp,
    toggleApp,
    updateDelay,
    // launch
    launchMode,
    launchAppNow,
    pickExecutable,
  };
});
