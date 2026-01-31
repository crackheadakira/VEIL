<template>
  <div
    data-tauri-drag-region
    class="bg-bg-primary text-text-secondary border-border-secondary z-1000 flex h-fit w-full cursor-default items-center justify-between border-b p-2 px-3 text-sm"
  >
    <p class="pointer-events-none text-sm">VEIL - {{ currentPage }}</p>
    <div class="flex items-center gap-2 text-xs *:cursor-pointer *:select-none">
      <span
        class="i-fluent-window-new-24-filled aspect-square h-fit"
        v-if="playerStore.currentTrack"
        @click="showWidget"
      >
      </span>
      <span
        @click="window.minimize"
        class="i-fluent-subtract-20-filled aspect-square h-fit"
      ></span>
      <span
        @click="window.toggleMaximize"
        class="i-fluent-maximize-20-filled aspect-square h-fit"
      ></span>
      <span
        @click="close"
        class="i-fluent-dismiss-20-filled aspect-square h-fit"
      ></span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useConfigStore, usePlayerStore } from "@/composables/";
import { getAllWindows, getCurrentWindow } from "@tauri-apps/api/window";

const configStore = useConfigStore();
const playerStore = usePlayerStore();
const currentPage = ref(configStore.pageName);

const window = getCurrentWindow();

async function showWidget() {
  const allWindows = await getAllWindows();
  const widgetWindow = allWindows.find((w) => w.label === "veil-widget");
  if (widgetWindow) {
    await window.hide();
    await widgetWindow.show();
  }
}

async function close() {
  const allWindows = await getAllWindows();
  const widgetWindow = allWindows.find((w) => w.label === "veil-widget");
  await widgetWindow?.close();
  await window.close();
}

watchEffect(() => {
  currentPage.value = configStore.pageName;
});
</script>
