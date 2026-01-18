<template>
  <div class="fixed right-4 bottom-4 z-30 flex flex-col gap-2">
    <TransitionGroup
      enter-active-class="animate-zoomIn"
      leave-active-class="animate-zoomOut"
    >
      <Toast
        v-for="toast in toasts"
        :key="toast.id"
        :id="toast.id"
        :type="toast.type"
        :description="toast.description"
        :removeToast="removeToast"
        :persistent="toast.persistent"
      />
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { Toast } from "@/components/";
import { toastBus, ToastType } from "@/composables/";
import { onMounted, ref } from "vue";

const toasts = ref<
  { id: number; type: ToastType; description: string; persistent?: boolean }[]
>([]);

function addToast(type: ToastType, description: string) {
  const id = Date.now();
  toasts.value.push({ id, type, description });

  setTimeout(() => removeToast(id), 2100);
}

function persistentToast(id: number, type: ToastType, description: string) {
  const toast = toasts.value.find((toast) => toast.id === id);
  if (toast) {
    toast.type = type;
    toast.description = description;
    toast.persistent = true;
  } else {
    toasts.value.push({ id, type, description, persistent: true });
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
