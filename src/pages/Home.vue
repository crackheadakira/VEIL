<template>
  <div class="bg-bg-primary text-text-primary flex flex-col gap-4">
    <div class="flex gap-2">
      <Button
        @click="showToast('success', 'This is a success toast')"
        label="Show Success Toast"
      />
      <Button
        @click="showToast('error', 'This is an error toast')"
        label="Show Error Toast"
      />
    </div>
    <RouterLink class="w-fit" to="/color_testing">
      <Button label="Color Testing" />
    </RouterLink>
    <div class="flex gap-2">
      <PlaylistCard />
    </div>
    <Modal>
      <p class="text-text-primary">Hello</p>
    </Modal>
    <Dialog
      title="I am a title."
      description="I am a description."
      placeholder="Placeholder..."
    />
  </div>
</template>

<script setup lang="ts">
import { PlaylistCard, Modal, Dialog, Button } from "@/components/";
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
