<template>
  <div class="bg-background text-text flex flex-col">
    <Greet />
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
import Greet from "../components/Greet.vue";
import PlaylistCard from "../components/PlaylistCard.vue";
import Dropdown from "../components/Dropdown.vue";
import BigCard from "../components/BigCard.vue";

const playerStore = usePlayerStore();
const recentlyPlayed = ref(playerStore.recentlyPlayed);

onMounted(() => {
  playerStore.currentPage = "/";
});
</script>
