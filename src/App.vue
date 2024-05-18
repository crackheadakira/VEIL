<script setup lang="ts">
import Greet from './components/Greet.vue';
import PlaylistCard from './components/PlaylistCard.vue';
import BigCard from './components/BigCard.vue';

</script>

<template>
  <div class="bg-background min-w-screen min-h-screen text-text">
    <Greet />
    <div class="flex gap-2">
      <PlaylistCard />
      <BigCard />
    </div>
  </div>
</template>

<style scoped></style>
