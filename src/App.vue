<template>
  <div class="bg-background flex min-h-screen flex-col justify-between">
    <div class="flex items-stretch">
      <SideBar class="fixed top-0 left-0 z-10 h-lvh self-start" />
      <RouterView class="ml-72 overflow-scroll p-16" />
    </div>
    <Player class="sticky bottom-0 z-20" />
    <ToastManager ref="toastManager" />
  </div>
</template>

<script setup lang="ts">
import SideBar from "./components/SideBar.vue";
import Player from "./components/Player.vue";
import ToastManager from "./components/ToastManager.vue";

import { useRouter } from "vue-router";
import { commands } from "./bindings";
import { usePlayerStore } from "./composables/playerStore";

const router = useRouter();
const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();

onBeforeMount(async () => {
  const result = await commands.getAllAlbums();
  if (result.status === "error") return handleBackendError(result.error);

  const track = playerStore.currentTrack;
  const progress = playerStore.playerProgress;

  if (result.data.length === 0 && track) {
    playerStore.$reset();
  } else if (track) {
    await commands.initializePlayer(track.id, progress);
  }

  await playlistStore.fetchPlaylists();

  const page = playerStore.currentPage;
  router.push({ path: page });

  setInterval(async () => {
    await commands.updateProgress();
  }, 100);
});
</script>
