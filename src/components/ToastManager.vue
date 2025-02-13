<template>
  <div class="fixed right-4 bottom-4 z-30 flex flex-col gap-2">
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
import { Toast } from "@/components/";
import { toastBus, ToastType } from "@/composables/";
import { onMounted, ref } from "vue";

const toasts = ref<{ id: number; type: ToastType; description: string }[]>([]);

function addToast(type: ToastType, description: string) {
  const id = Date.now();
  toasts.value.push({ id, type, description });

  setTimeout(() => removeToast(id), 2100);
}

function persistentToast(id: number, type: ToastType, description: string) {
  const idIdx = toasts.value.findIndex((toast) => toast.id === id);
  if (idIdx !== -1) {
    toasts.value[idIdx] = { id, type, description };
  } else {
    toasts.value.push({ id, type, description });
  }
}

function removeToast(id: number) {
  toasts.value = toasts.value.filter((toast) => toast.id !== id);
}

onMounted(() => {
  toastBus.addToast = addToast;
  toastBus.persistentToast = persistentToast;
  toastBus.removeToast = removeToast;
});
</script>
