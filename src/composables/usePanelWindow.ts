import { ref } from "vue";
import { isTauri } from "@tauri-apps/api/core";
import { LogicalSize, PhysicalPosition } from "@tauri-apps/api/dpi";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";

export function usePanelWindow() {
  const desktop = isTauri();
  const pinned = ref(false);
  let compactGeometry: { size: LogicalSize; position: PhysicalPosition } | null = null;

  async function initialize() {
    if (!desktop) return;
    pinned.value = await getCurrentWindow().isAlwaysOnTop();
    const saved = localStorage.getItem("sofsta-panel-pinned") ?? localStorage.getItem("autostart-panel-pinned");
    if (saved !== null) localStorage.setItem("sofsta-panel-pinned", saved);
    if (saved === "true" && !pinned.value) {
      await getCurrentWindow().setAlwaysOnTop(true);
      pinned.value = true;
    }
  }

  async function togglePin() {
    if (!desktop) return;
    const next = !pinned.value;
    await getCurrentWindow().setAlwaysOnTop(next);
    pinned.value = next;
    localStorage.setItem("sofsta-panel-pinned", String(next));
  }

  async function expand() {
    if (!desktop || compactGeometry) return;
    const window = getCurrentWindow();
    const [inner, outer, position, scale, monitor] = await Promise.all([
      window.innerSize(), window.outerSize(), window.outerPosition(), window.scaleFactor(), currentMonitor(),
    ]);
    compactGeometry = { size: inner.toLogical(scale), position };
    const frameWidth = outer.width - inner.width;
    const frameHeight = outer.height - inner.height;
    const width = monitor ? Math.min(660, (monitor.workArea.size.width - frameWidth - 16) / scale) : 660;
    const height = monitor ? Math.min(720, (monitor.workArea.size.height - frameHeight - 16) / scale) : 720;
    try {
      await window.setMinSize(new LogicalSize(Math.min(480, width), Math.min(440, height)));
      await window.setSize(new LogicalSize(width, height));
      if (monitor) {
        const area = monitor.workArea;
        const x = Math.max(area.position.x + 8, Math.min(position.x, area.position.x + area.size.width - width * scale - frameWidth - 8));
        const y = Math.max(area.position.y + 8, Math.min(position.y, area.position.y + area.size.height - height * scale - frameHeight - 8));
        await window.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));
      }
    } catch (error) {
      await restore().catch(() => {});
      throw error;
    }
  }

  async function restore() {
    if (!desktop || !compactGeometry) return;
    const window = getCurrentWindow();
    await window.setMinSize(new LogicalSize(320, 260));
    await window.setPosition(compactGeometry.position);
    await window.setSize(compactGeometry.size);
    compactGeometry = null;
  }

  return { desktop, pinned, initialize, togglePin, expand, restore };
}
