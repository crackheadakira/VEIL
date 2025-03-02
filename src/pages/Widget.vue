<template>
  <div
    data-tauri-drag-region
    class="bg-background border-stroke-200 flex h-screen flex-col gap-4 rounded-md border-1 p-4 *:select-none"
  >
    <div
      @mouseover="imageHovered = true"
      @mouseleave="imageHovered = false"
      class="group relative h-52 w-52"
    >
      <img
        class="pointer-events-none absolute h-52 w-52 rounded-md transition-opacity duration-150 group-hover:opacity-30"
        :src="convertFileSrc(playerStore.currentTrack.cover_path)"
        alt="Album Cover"
      />
      <Transition
        enter-active-class="animate-zoomIn"
        leave-active-class="animate-zoomOut"
      >
        <div
          class="relative flex h-full items-center justify-center"
          v-show="imageHovered"
        >
          <PlayerControls class="text-text" />
          <VolumeControls
            class="text-text absolute bottom-1 left-1/2 -translate-x-1/2"
          />
        </div>
      </Transition>
    </div>

    <div
      class="pointer-events-none flex w-full items-center justify-between gap-2"
    >
      <div class="flex flex-col gap-2 truncate *:truncate">
        <small class="text-text">{{ playerStore.currentTrack.name }}</small>
        <small class="text-supporting">{{
          playerStore.currentTrack.artist_name
        }}</small>
      </div>
      <RouterLink
        class="i-fluent-window-new-24-filled text-supporting hover:text-stroke-100 pointer-events-auto h-7 w-7 shrink-0 cursor-pointer transition-colors duration-150"
        :to="playerStore.currentPage"
      ></RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  getCurrentWindow,
  LogicalSize,
  PhysicalPosition,
  PhysicalSize,
} from "@tauri-apps/api/window";
import { onBeforeMount, onBeforeUnmount, ref } from "vue";

import { usePlayerStore } from "@/composables/";
import { PlayerControls, VolumeControls } from "@/components/";

const playerStore = usePlayerStore();
const originalWindowSize = ref(new PhysicalSize(1280, 720));
const originalPosition = ref(new PhysicalPosition(1920 / 2, 1080 / 2));

const imageHovered = ref(false);

onBeforeMount(async () => {
  const window = getCurrentWindow();
  originalWindowSize.value = await window.innerSize();
  originalPosition.value = await window.innerPosition();

  await window.unmaximize();
  await window.setSize(new LogicalSize(246, 307));
  await window.setResizable(false);
});

onBeforeUnmount(async () => {
  const window = getCurrentWindow();

  await window.setSize(originalWindowSize.value);
  await window.setPosition(originalPosition.value);
  await window.setResizable(false);
});
</script>
