<template>
  <div class="flex flex-col justify-between gap-4 bg-background min-h-screen h-fit">
    <RouterView />
    <Player :music="music" class="bottom-0 sticky" />
  </div>
</template>

<script setup lang="ts">
import Player from './components/Player.vue';

const music = {
  title: 'Seraph',
  artist: 'DPR IAN',
  cover: '/home/akira/.local/share/sodapop-reimagined/covers/DPR IAN - Moodswings In To Order.jpg',
  audio: 'http://localhost:16780/home/akira/Music/music/DPR IAN/DPR IAN - Moodswings In To Order (2022)/01. Seraph.flac'
}
</script>
