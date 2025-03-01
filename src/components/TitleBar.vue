<template>
  <div
    data-tauri-drag-region
    class="bg-background text-supporting border-stroke-100 z-[1000] flex h-fit w-full cursor-default items-center justify-between border-b p-2 px-3 text-sm"
  >
    <small class="pointer-events-none text-sm"
      >Sodapop Reimagined - {{ currentPage }}</small
    >
    <div class="flex items-center gap-2 text-xs *:cursor-pointer *:select-none">
      <RouterLink
        class="i-fluent-window-new-24-filled aspect-square h-fit"
        v-if="playerStore.currentTrack"
        to="/widget"
      >
      </RouterLink>
      <span
        @click="getCurrentWindow().minimize"
        class="i-fluent-subtract-20-filled aspect-square h-fit"
      ></span>
      <span
        @click="getCurrentWindow().toggleMaximize"
        class="i-fluent-maximize-20-filled aspect-square h-fit"
      ></span>
      <span
        @click="getCurrentWindow().close"
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

watchEffect(() => {
  currentPage.value = playerStore.pageName;
});
</script>
