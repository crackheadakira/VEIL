<template>
  <div
    data-tauri-drag-region
    class="bg-background text-supporting border-stroke-100 z-[1000] flex h-fit w-full cursor-default items-center justify-between border-b p-2 px-3 text-sm"
  >
    <small class="pointer-events-none text-sm"
      >Sodapop Reimagined - {{ currentPage }}</small
    >
    <div class="flex gap-2 text-xs *:cursor-pointer *:select-none">
      <span
        @click="appWindow.minimize"
        class="i-fluent-subtract-20-filled aspect-square h-fit"
      ></span>
      <span
        @click="appWindow.maximize"
        class="i-fluent-maximize-20-filled aspect-square h-fit"
      ></span>
      <span
        @click="appWindow.close"
        class="i-fluent-dismiss-20-filled aspect-square h-fit"
      ></span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { usePlayerStore } from "@/composables/";
import { getCurrentWindow } from "@tauri-apps/api/window";

const playerStore = usePlayerStore();
const currentPage = ref(playerStore.pageName);
const appWindow = getCurrentWindow();

watchEffect(() => {
  currentPage.value = playerStore.pageName;
});
</script>
