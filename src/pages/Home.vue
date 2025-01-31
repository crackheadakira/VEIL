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
    </div>
    <div class="flex gap-2">
      <PlaylistCard />
      <Dropdown
        :title="'Filter by'"
        :options="['Albums', 'Artists', 'Tracks']"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import PlaylistCard from "../components/PlaylistCard.vue";
import Dropdown from "../components/Dropdown.vue";
import Dialog from "../components/Dialog.vue";

import { commands } from "../bindings";

const playerStore = usePlayerStore();

async function openDialog() {
  const result = await commands.selectMusicFolder();
  if (result.status === "error")
    throw new Error(`[${result.error.type}] ${result.error.data}`);
}

async function newPlaylist(playlistName: string) {
  const result = await commands.newPlaylist(playlistName);
  if (result.status === "error")
    throw new Error(`[${result.error.type}] ${result.error.data}`);
}

onMounted(() => {
  playerStore.currentPage = "/";
});
</script>
