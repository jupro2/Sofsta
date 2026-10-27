import { nextTick, onBeforeUnmount, watch, type Ref } from "vue";

export function useDialogFocus(dialog: Ref<HTMLElement | null>, close: () => void) {
  let previousFocus: HTMLElement | null = null;
  function handleKey(event: KeyboardEvent) {
    const element = dialog.value;
    const dialogs = document.querySelectorAll('[role="dialog"]');
    if (!element || dialogs[dialogs.length - 1] !== element) return;
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    } else if (event.key === "Tab") {
      const controls = Array.from(element.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex="0"]'));
      const first = controls[0];
      const last = controls[controls.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
    }
  }
  watch(dialog, async (element) => {
    document.removeEventListener("keydown", handleKey);
    if (element) {
      previousFocus = document.activeElement as HTMLElement | null;
      document.addEventListener("keydown", handleKey);
      await nextTick();
      element.querySelector<HTMLElement>('button:not([disabled]), input:not([disabled])')?.focus();
    } else if (previousFocus?.isConnected) previousFocus.focus();
  });
  onBeforeUnmount(() => document.removeEventListener("keydown", handleKey));
}
