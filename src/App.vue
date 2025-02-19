<template>
  <div
    class="bg-background grid h-screen grid-cols-[18rem_1fr] grid-rows-[1fr_7rem]"
  >
    <SideBar class="row-span-1" />
    <RouterView class="overflow-scroll p-16" :key="currentRoute.fullPath" />
    <Player class="col-span sticky bottom-0 h-28" />
    <ToastManager ref="toastManager" />
  </div>
</template>

<script setup lang="ts">
import { Player, SideBar, ToastManager } from "@/components/";
import {
  useConfigStore,
  commands,
  handleBackendError,
  usePlayerStore,
  usePlaylistStore,
} from "@/composables/";
import { onBeforeMount } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();
const configStore = useConfigStore();
const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();
const currentRoute = router.currentRoute;

onBeforeMount(async () => {
  await configStore.initialize();

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
