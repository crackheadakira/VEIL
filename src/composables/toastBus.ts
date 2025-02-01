// @ts-nocheck

import { reactive } from "vue";

/**
 * A reactive object that can be used to add toasts to the toast container.
 *
 * ToastManager component listens to this object and adds toasts to the container.
 *
 * The function gets overriden in `ToastManager.vue`, so for the code go there.
 */
export const toastBus = reactive({
  /**
   * Add a toast to the toast container.
   *
   * @param type - The type of the toast. Can be either "success", "error", or "warning".
   * @param description - The description of the toast.
   *
   * @example
   * toastBus.addToast("success", "The operation was successful.");
   */
  addToast: (type: "success" | "error" | "warning", description: string) => {},
});
