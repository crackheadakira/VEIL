<template>
  <div class="bg-background text-text flex flex-col">
    <div>
      <button
        class="border-stroke-100 bg-card cursor-pointer rounded-md border p-2"
        @click="openDialog"
      >
        <small>Select music folder</small>
      </button>
      <Dialog :title="'New Playlist'" />
    </div>
    <div class="flex gap-2">
      <PlaylistCard />
      <Dropdown
        :title="'Filter by'"
        :options="['Albums', 'Artists', 'Tracks']"
      />
    </div>
    <div>
      <h6 class="text-text mb-4">Recently listened</h6>
      <div v-if="recentlyPlayed.length > 0" class="flex flex-wrap gap-4">
        <BigCard v-for="album of recentlyPlayed" :data="album" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import PlaylistCard from "../components/PlaylistCard.vue";
import Dropdown from "../components/Dropdown.vue";
import BigCard from "../components/BigCard.vue";
import Dialog from "../components/Dialog.vue";

import { commands } from "../bindings";

const playerStore = usePlayerStore();
const recentlyPlayed = ref(playerStore.recentlyPlayed);

async function openDialog() {
  const result = await commands.selectMusicFolder();
  if (result.status === "error")
    throw new Error(`[${result.error.type}] ${result.error.data}`);
}

onMounted(() => {
  playerStore.currentPage = "/";
});
</script>
