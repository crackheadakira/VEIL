<template>
  <div class="text-text flex w-full flex-col gap-8" v-if="artist_data">
    <div
      class="sodapop-card flex max-h-72 min-h-72 items-center justify-between overflow-hidden"
    >
      <div class="flex h-full flex-col justify-end gap-4 p-8">
        <h4 class="text-text">{{ artist_data.artist.name }}</h4>
        <div class="flex gap-4">
          <button
            class="text aspect-button bg-primary text-background flex h-12 items-center justify-center gap-2 rounded-md duration-150 hover:opacity-90"
          >
            <span class="i-fluent-play-24-filled h-7"></span>
            <p>Play</p>
          </button>
          <button
            class="text aspect-button border-stroke-200 bg-background text-supporting flex h-12 items-center justify-center gap-2 rounded-md border duration-150 hover:opacity-80"
          >
            <span class="i-fluent-arrow-shuffle-20-filled h-7"></span>
            <p>Shuffle</p>
          </button>
        </div>
      </div>
      <img
        class="aspect-square w-2/3 rounded-md object-cover [mask-image:linear-gradient(90deg,rgba(17,17,17,0),rgba(17,17,17,0.4))]"
        :src="convertFileSrc(artist_data.albums[0].album.cover_path)"
      />
    </div>

    <div>
      <p class="text-supporting mb-2">View all</p>
      <div class="flex flex-wrap gap-4">
        <BigCard v-for="album of artist_data.albums" :data="album" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BigCard } from "@/components/";
import {
  type ArtistWithAlbums,
  commands,
  handleBackendError,
  useConfigStore,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { onBeforeMount, ref } from "vue";
import { useRoute } from "vue-router";

const configStore = useConfigStore();

const route = useRoute();
const artist_id = ref(route.params.id as string);
const artist_data = ref<ArtistWithAlbums | null>(null);

async function updateData() {
  if (!artist_id.value) return;
  const response = await commands.getArtistWithAlbums(
    parseInt(artist_id.value),
  );

  if (response.status === "error") return handleBackendError(response.error);

  artist_data.value = response.data;
}

onBeforeMount(async () => {
  await updateData();
  configStore.currentPage = `/artist/${artist_id.value}`;
  configStore.pageName = artist_data.value?.artist.name || "Artist";
});
</script>
