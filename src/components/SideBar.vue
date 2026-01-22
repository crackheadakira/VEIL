<template>
  <nav
    class="border-border-secondary bg-bg-primary text-text-secondary flex min-h-fit w-full flex-col gap-8 border-r p-8"
  >
    <SearchBar />

    <section class="flex flex-col gap-6">
      <small class="text-text-tertiary select-none">General</small>

      <div
        class="*:hover:text-text-secondary-hovered *:active:text-text-secondary-active flex flex-col gap-4 px-2 *:inline-flex *:items-center"
      >
        <RouterLink to="/">
          <p>Home</p>
        </RouterLink>

        <div>
          <p>Liked Songs</p>
        </div>

        <RouterLink to="/all_albums">
          <p>Albums</p>
        </RouterLink>

        <RouterLink to="/settings">
          <p>Settings</p>
        </RouterLink>
      </div>
    </section>

    <section
      v-if="allPlaylists && allPlaylists.length > 0"
      class="flex flex-col gap-6"
    >
      <small class="text-text-tertiary select-none">Playlists</small>
      <div class="flex flex-col gap-4">
        <RouterLink
          v-for="playlist of allPlaylists"
          :key="playlist.id"
          :to="`/playlist/${playlist.id}`"
          class="text-text-secondary hover:text-text-primary flex items-center gap-4 rounded-md"
        >
          <img
            :src="playlist.cover_path"
            class="border-border-primary aspect-square w-16 rounded-sm border"
          />
          <p>{{ playlist.name }}</p>
        </RouterLink>
      </div>
    </section>
  </nav>
</template>

<script setup lang="ts">
import { SearchBar } from "@/components/";
import { usePlaylistStore } from "@/composables/";
import { ref, watchEffect } from "vue";
import { RouterLink } from "vue-router";

const playlistStore = usePlaylistStore();
const allPlaylists = ref(playlistStore.playlists);

watchEffect(() => {
  allPlaylists.value = playlistStore.playlists;
});
</script>
