<template>
  <div class="bg-background text-text flex flex-col">
    <div>
      <button
        class="border-stroke-100 bg-card cursor-pointer rounded-md border p-2"
        @click="openDialog"
      >
        <small>Select music folder</small>
      </button>
      <Dialog :title="'New Playlist'" @submitted="newPlaylist" />
      <button
        @click="showToast('success', 'This is a success toast')"
        class="border-stroke-100 bg-card cursor-pointer rounded-md border p-2"
      >
        Show Success Toast
      </button>
      <button
        @click="showToast('error', 'This is an error toast')"
        class="border-stroke-100 bg-card cursor-pointer rounded-md border p-2"
      >
        Show Error Toast
      </button>
    </div>
    <div class="flex gap-2">
      <PlaylistCard />
      <Dropdown
        :title="'Filter by'"
        :options="['Albums', 'Artists', 'Tracks']"
      />
    </div>
    <ToastManager ref="toastManager" />
  </div>
</template>

<script setup lang="ts">
import PlaylistCard from "../components/PlaylistCard.vue";
import Dropdown from "../components/Dropdown.vue";
import Dialog from "../components/Dialog.vue";
import ToastManager from "../components/ToastManager.vue";

import { commands } from "../bindings";

const playerStore = usePlayerStore();
const toastManager = useTemplateRef("toastManager");

async function openDialog() {
  const result = await commands.selectMusicFolder();
  if (result.status === "error") {
    toastManager.value?.addToast(
      "error",
      `[${result.error.type}] ${result.error.data}`,
    );
  } else {
    toastManager.value?.addToast("success", "Music added successfully");
  }
}

async function newPlaylist(playlistName: string) {
  const result = await commands.newPlaylist(playlistName);
  if (result.status === "error") {
    toastManager.value?.addToast(
      "error",
      `[${result.error.type}] ${result.error.data}`,
    );
  } else {
    toastManager.value?.addToast("success", `Created playlist ${playlistName}`);
  }
}

function showToast(type: "success" | "error", description: string) {
  toastManager.value?.addToast(type, description);
}

onMounted(() => {
  playerStore.currentPage = "/";
});
</script>
