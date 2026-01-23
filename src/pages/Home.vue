<template>
  <div class="bg-bg-primary text-text-primary flex flex-col gap-4">
    <div>
      <button
        @click="showToast('success', 'This is a success toast')"
        class="sodapop-card cursor-pointer"
      >
        Show Success Toast
      </button>
      <button
        @click="showToast('error', 'This is an error toast')"
        class="sodapop-card cursor-pointer"
      >
        Show Error Toast
      </button>
    </div>
    <RouterLink to="/color_testing">Color Testing</RouterLink>
    <div class="flex gap-2">
      <PlaylistCard />
    </div>
    <Modal>
      <p class="text-text-primary">Hello</p>
    </Modal>
    <Dialog
      title="Hello"
      description="I am a description."
      placeholder="Submit..."
    />
  </div>
</template>

<script setup lang="ts">
import { PlaylistCard, Modal, Dialog } from "@/components/";
import { toastBus, useConfigStore } from "@/composables/";
import { onMounted } from "vue";

const configStore = useConfigStore();

function showToast(type: "success" | "error", description: string) {
  toastBus.addToast(type, description);
}

onMounted(() => {
  configStore.currentPage = "/";
  configStore.pageName = "Home";
});
</script>
