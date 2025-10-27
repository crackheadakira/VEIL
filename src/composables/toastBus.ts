// @ts-nocheck

import { reactive } from "vue";

export type ToastType = "success" | "error" | "warning" | "info";

/**
 * A reactive object that can be used to add toasts to the toast manager.
 *
 * ToastManager component listens to this object and adds toasts to the manager.
 *
 * The function gets overriden in `ToastManager.vue`, so for the code go there.
 */
export const toastBus = reactive({
  /**
   * Add a toast to the toast manager.
   *
   * @param type - The type of the toast. Can be either "success", "error", or "warning".
   * @param description - The description of the toast.
   *
   * @example
   * toastBus.addToast("success", "The operation was successful.");
   */
  addToast: (type: ToastType, description: string) => { },

  /**
   * Add a persistent toast to the toast manager.
   *
   * Persistent toasts are not removed automatically after a certain time, and can have their data updated.
   * They are useful for showing the progress of an operation.
   *
   * @param id - The id of the toast. Used to update the toast later.
   * @param type - The type of the toast. Can be either "success", "error", or "warning".
   */
  persistentToast: (id: number, type: ToastType, description: string) => { },

  /**
   * Remove an existing toast by `id` from the toast manager.
   *
   * @param id - The id of the toast to remove.
   *
   * @example
   * toastBus.remove(1);
   */
  removeToast: (id: number) => { },
});
