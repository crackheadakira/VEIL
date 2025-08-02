<template>
  <nav
    class="border-border-secondary bg-bg-primary text-text-secondary flex min-h-fit w-full flex-col gap-8 border-r p-8 font-medium"
  >
    <SearchBar />

    <section class="flex flex-col gap-4">
      <h6>General</h6>
      <hr class="border-border-secondary border-t-2" />

      <div
        class="*:hover:text-text-primary flex flex-col gap-4 *:inline-flex *:items-center *:gap-4"
      >
        <RouterLink to="/">
          <span class="i-fluent-home-16-filled h-8"></span>
          <p>Home</p>
        </RouterLink>

        <div>
          <span class="i-fluent-heart-16-filled h-8"></span>
          <p>Liked Songs</p>
        </div>

        <RouterLink to="/all_albums">
          <span class="i-fluent-music-note-2-16-filled h-8"></span>
          <p>Albums</p>
        </RouterLink>

        <RouterLink to="/settings">
          <span class="i-fluent-settings-16-filled h-8"></span>
          <p>Settings</p>
        </RouterLink>
      </div>
    </section>

    <section
      v-if="allPlaylists && allPlaylists.length > 0"
      class="flex flex-col gap-4"
    >
      <h6 class="text-text-secondary">Playlists</h6>
      <hr class="border-border-secondary border-t-2" />
      <RouterLink
        v-for="playlist of allPlaylists"
        :key="playlist.id"
        :to="`/playlist/${playlist.id}`"
        class="text-text-secondary hover:text-text-primary flex items-center gap-4 rounded-md duration-75"
      >
        <img :src="playlist.cover_path" class="aspect-square w-16 rounded-sm" />
        <p>{{ playlist.name }}</p>
      </RouterLink>
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
