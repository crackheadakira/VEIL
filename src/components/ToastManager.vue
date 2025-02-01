<template>
  <div class="fixed right-4 bottom-4 flex flex-col gap-2">
    <TransitionGroup
      enter-from-class="opacity-0 scale-95"
      leave-to-class="opacity-0 scale-95"
      enter-active-class="transition-all duration-150"
      leave-active-class="transition-all duration-150"
      move-class="transition-all"
    >
      <Toast
        v-for="toast in toasts"
        :key="toast.id"
        :id="toast.id"
        :type="toast.type"
        :description="toast.description"
        :removeToast="removeToast"
      />
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { toastBus } from "../composables/toastBus";
import Toast from "./Toast.vue";

const toasts = ref<
  { id: number; type: "success" | "error"; description: string }[]
>([]);

function addToast(type: "success" | "error", description: string) {
  const id = Date.now();
  toasts.value.push({ id, type, description });

  setTimeout(() => removeToast(id), 2100);
}

function removeToast(id: number) {
  toasts.value = toasts.value.filter((toast) => toast.id !== id);
}

onMounted(() => {
  toastBus.addToast = addToast;
});
</script>
