<template>
  <div v-show="!isWidget" class="bg-background flex h-screen flex-col">
    <TitleBar class="sticky top-0" />

    <div class="flex flex-1 overflow-hidden">
      <div class="h-full w-72 overflow-y-auto">
        <SideBar class="sticky top-16 bottom-28 h-full" />
      </div>

      <RouterView
        class="flex-1 overflow-y-scroll p-8"
        :key="currentRoute.fullPath"
      />
    </div>

    <Player class="sticky bottom-0 h-28" />
    <ToastManager ref="toastManager" />
  </div>
  <RouterView v-if="isWidget" />
</template>

<script setup lang="ts">
import { Player, SideBar, ToastManager, TitleBar } from "@/components/";
import {
  useConfigStore,
  commands,
  handleBackendError,
  usePlayerStore,
  usePlaylistStore,
} from "@/composables/";
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();
const configStore = useConfigStore();
const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();
const currentRoute = router.currentRoute;

const isWidget = computed(() => router.currentRoute.value.name === "widget");

onMounted(async () => {
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
