<template>
  <div
    class="bg-background grid h-screen grid-cols-[18rem_1fr] grid-rows-[1fr_7rem]"
  >
    <SideBar class="row-span-1" />
    <RouterView class="overflow-scroll p-16" />
    <Player class="col-span sticky bottom-0 h-28" />
    <ToastManager ref="toastManager" />
  </div>
</template>

<script setup lang="ts">
import { onBeforeMount } from "vue";

import SideBar from "./components/SideBar.vue";
import Player from "./components/Player.vue";
import ToastManager from "./components/ToastManager.vue";

import { usePlaylistStore } from "./composables/playlistStore";
import { usePlayerStore } from "./composables/playerStore";
import { handleBackendError } from "./composables/utils";

import { useRouter } from "vue-router";
import { commands } from "./bindings";

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
});
</script>
