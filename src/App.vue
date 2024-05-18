<script setup lang="ts">
import Greet from './components/Greet.vue';
import PlaylistCard from './components/PlaylistCard.vue';
import BigCard from './components/BigCard.vue';
import SearchBar from './components/SearchBar.vue';
import Dropdown from './components/Dropdown.vue';
import Player from './components/Player.vue';


const music = {
  title: 'Seraph',
  artist: 'DPR IAN',
  cover: '/home/akira/.local/share/sodapop-reimagined/covers/DPR IAN - Moodswings In To Order.jpg',
  audio: 'http://localhost:16780/home/akira/Music/music/DPR IAN/DPR IAN - Moodswings In To Order (2022)/01. Seraph.flac'
}
</script>

<template>
  <div class="bg-background w-screen h-screen text-text">
    <Greet />
    <div class="flex gap-2">
      <PlaylistCard />
      <BigCard />
      <SearchBar />
      <Dropdown :title="'Filter by'" :options="['Albums', 'Artists', 'Tracks']" />
    </div>
    <Player :music="music" />
  </div>
</template>

<style scoped></style>
