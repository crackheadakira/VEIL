<template>
  <div class="flex min-h-screen flex-col justify-between bg-background">
    <div class="flex items-stretch">
      <SideBar class="fixed left-0 top-0 z-40 h-lvh self-start" />
      <RouterView class="ml-72 overflow-scroll p-16" />
    </div>
    <Player class="sticky bottom-0 z-50" />
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import Player from './components/Player.vue';
import SideBar from './components/SideBar.vue';
import { commands } from './bindings';
import { usePlayerStore } from './composables/playerStore';

const router = useRouter();
const playerStore = usePlayerStore();

onBeforeMount(async () => {
  const result = await commands.getAllAlbums();
  if (result.status === 'error') throw new Error(`[${result.error.type}] ${result.error.data}`);

  const page = playerStore.currentPage;
  router.push({ path: page });

  const track = playerStore.currentTrack;
  const progress = playerStore.playerProgress;
  if (track) {
    await commands.initializePlayer(track.id, progress);
  }


  setInterval(async () => {
    await commands.updateProgress();
  }, 100);
})
</script>
