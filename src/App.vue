<template>
  <div class="bg-background flex h-screen flex-col">
    <TitleBar class="shrink-0" />
    <div class="flex h-full shrink grow-0 overflow-y-scroll">
      <SideBar class="min-w-72" />
      <RouterView class="overflow-scroll p-16" :key="currentRoute.fullPath" />
    </div>
    <Player class="h-28 grow" />
    <ToastManager ref="toastManager" />
    <DialogSearch />
  </div>
</template>

<script setup lang="ts">
import {
  Player,
  SideBar,
  ToastManager,
  TitleBar,
  DialogSearch,
} from "@/components/";
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
  router.push(page);
});
</script>
