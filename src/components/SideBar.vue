<template>
  <div
    class="border-stroke-100 bg-card text-text flex flex-col items-center gap-8 border-r p-8"
  >
    <SearchBar />
    <div class="flex w-full flex-col gap-2 font-medium *:select-none">
      <div class="flex w-full flex-col gap-4">
        <small class="text-supporting">General</small>
        <hr class="border-stroke-100 border-t-2" />
      </div>
      <RouterLink
        class="text-supporting hover:text-text flex items-center gap-4 rounded-md p-2 duration-75"
        to="/"
      >
        <span class="i-fluent-home-16-filled h-8"></span>
        <small>Home</small>
      </RouterLink>
      <div
        class="text-supporting hover:text-text flex items-center gap-4 rounded-md p-2 duration-75"
      >
        <span class="i-fluent-heart-16-filled h-8"></span>
        <small>Liked Songs</small>
      </div>
      <RouterLink
        class="text-supporting hover:text-text flex items-center gap-4 rounded-md p-2 duration-75"
        to="/all_albums"
      >
        <span class="i-fluent-music-note-2-16-filled h-8"></span>
        <small>Albums</small>
      </RouterLink>
      <div
        class="text-supporting hover:text-text flex items-center gap-4 rounded-md p-2 duration-75"
      >
        <span class="i-fluent-settings-16-filled h-8"></span>
        <small>Settings</small>
      </div>
      <div
        v-if="allPlaylists && allPlaylists.length > 0"
        class="mb-2 flex w-full flex-col gap-4"
      >
        <small class="text-supporting">Playlists</small>
        <hr class="border-stroke-100 border-t-2" />
      </div>
      <RouterLink
        v-for="playlist of allPlaylists"
        :key="playlist.id"
        :to="`/playlist/${playlist.id}`"
        class="text-supporting hover:text-text flex items-center gap-4 rounded-md duration-75"
      >
        <img :src="playlist.cover_path" class="aspect-square w-16 rounded-sm" />
        <small>{{ playlist.name }}</small>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { commands, Playlists } from "../bindings";
import SearchBar from "../components/SearchBar.vue";
import { RouterLink } from "vue-router";

const allPlaylists = ref<Playlists[] | null>(null);

onMounted(async () => {
  const result = await commands.getAllPlaylists();
  if (result.status === "error") return handleBackendError(result.error);

  allPlaylists.value = result.data;
});
</script>
