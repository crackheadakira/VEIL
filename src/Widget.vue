<template>
  <div
    data-tauri-drag-region
    class="bg-bg-primary border-border-secondary flex h-screen flex-col gap-4 overflow-clip border p-4 *:select-none"
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
          <PlayerControls class="text-text-primary" />
          <VolumeControls class="absolute bottom-1 left-1/2 -translate-x-1/2" />
        </div>
      </Transition>
    </div>

    <div
      class="pointer-events-none flex w-full items-center justify-between gap-2"
    >
      <div class="flex flex-col gap-2 truncate *:truncate">
        <p class="text-text-primary">{{ playerStore.currentTrack.name }}</p>
        <p class="text-text-secondary">
          {{ playerStore.currentTrack.artist_name }}
        </p>
      </div>
      <span
        class="i-fluent-window-new-24-filled text-text-tertiary hover:text-text-secondary-hovered pointer-events-auto h-7 w-7 shrink-0 cursor-pointer transition-colors duration-150"
        @click="hideWidget"
      ></span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { getAllWindows, getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, ref, watch } from "vue";

import { commands, useConfigStore, usePlayerStore } from "@/composables/";
import { PlayerControls, VolumeControls } from "@/components/";

const playerStore = usePlayerStore();

const imageHovered = ref(false);
const window = getCurrentWindow();

async function hideWidget() {
  const allWindows = await getAllWindows();
  const mainWindow = allWindows.find((w) => w.label === "sodapop-reimagined");
  if (mainWindow) {
    await window.hide();
    await mainWindow.show();
  }
}

const configStore = useConfigStore();
const theme = configStore.config.ui.theme || "Dark";

watch(
  () => configStore.config.ui.theme,
  (newTheme) => {
    if (newTheme === "Dark") {
      document.documentElement.setAttribute("data-theme", "dark");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
  },
  { immediate: true },
);

onMounted(async () => {
  if (theme === "Dark") {
    document.documentElement.setAttribute("data-theme", "dark");
  } else {
    document.documentElement.setAttribute("data-theme", "light");
  }

  const css = await commands.readCustomStyle();
  let styleElement = document.getElementById("custom-style");

  if (!styleElement) {
    styleElement = document.createElement("style");
    styleElement.id = "custom-style";
    document.head.appendChild(styleElement);
  }

  if (css.status === "ok" && styleElement) {
    styleElement.innerText = css.data;
  }
});
</script>
