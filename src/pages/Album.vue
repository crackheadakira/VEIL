<template>
  <div class="text-text flex w-full flex-col gap-8" v-if="data">
    <div class="cardStyle flex items-center gap-8 p-8">
      <img
        class="aspect-square w-64 rounded-md"
        :src="convertFileSrc(data.album.cover_path)"
      />

      <div class="flex flex-col gap-4">
        <div class="flex cursor-default flex-col gap-1 select-none">
          <p class="text-supporting font-medium">{{ data.album.album_type }}</p>
          <h4 class="text-text">{{ data.album.name }}</h4>
          <p class="text-supporting">{{ data.album.artist_name }}</p>
          <small class="text-supporting">
            {{ makeTime(data.album.duration) }},
            {{ data.album.track_count }}
            {{ data.album.track_count > 1 ? "songs" : "song" }}
          </small>
          <small v-if="data.album.year" class="text-supporting">{{
            data.album.year
          }}</small>
        </div>

        <div class="flex gap-4 *:cursor-pointer">
          <button
            @click="handlePlayButton(false)"
            class="text aspect-button bg-primary text-background flex h-12 items-center justify-center gap-2 rounded-md duration-150 hover:opacity-90"
          >
            <span class="i-fluent-play-24-filled h-7"></span>
            <p>Play</p>
          </button>
          <button
            @click="handlePlayButton(true)"
            class="text aspect-button border-stroke-100 bg-background text-supporting flex h-12 items-center justify-center gap-2 rounded-md border duration-150 hover:opacity-80"
          >
            <span class="i-fluent-arrow-shuffle-20-filled h-7"></span>
            <p>Shuffle</p>
          </button>
        </div>
      </div>
    </div>

    <TrackList :data="data" @new-track="handleNewTrack" />

    <div v-if="artist && artist.albums.length">
      <h5 class="text-text mb-4">More from {{ artist.artist.name }}</h5>
      <div class="flex flex-wrap gap-4">
        <BigCard v-for="album of artist.albums" :data="album.album" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BigCard, TrackList } from "@/components/";
import {
  type AlbumWithTracks,
  type ArtistWithAlbums,
  commands,
  handleBackendError,
  makeTime,
  type Tracks,
  usePlayerStore,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { onBeforeMount, ref } from "vue";
import { useRoute } from "vue-router";

const playerStore = usePlayerStore();

const route = useRoute();
const album_id = ref(route.params.album_id as string);
const artist_id = ref(route.params.artist_id as string);

const artist = ref<ArtistWithAlbums | null>(null);
const data = ref<AlbumWithTracks | null>(null);

/**
 * Handles the big play/shuffle button.
 * @param {boolean} shuffle - Whether the button was play or shuffle
 */
async function handlePlayButton(shuffle: boolean) {
  if (!data.value) return;
  playerStore.queue = [...data.value.tracks];
  if (shuffle) {
    playerStore.isShuffled = false;
    playerStore.shuffleQueue();
  }
  playerStore.queueIndex = 0;
  await playerStore.setPlayerTrack(playerStore.queue[0]);
}

async function updateData() {
  await commands.getAlbumWithTracks(parseInt(album_id.value));
  const result = await commands.getArtistWithAlbums(parseInt(artist_id.value));

  if (result.status === "error")
    if (result.status === "error") return handleBackendError(result.error);

  const res = result.data;

  // Gets only the current album from artist
  const current_album = res.albums.filter(
    (album) => album.album.id === parseInt(album_id.value),
  )[0];
  data.value = current_album;
  res.albums.splice(res.albums.indexOf(current_album), 1); // Removes the current album from the list

  // This is to list more albums from the same artist
  artist.value = res;
}

async function handleNewTrack(track: Tracks, idx: number) {
  await playerStore.setPlayerTrack(track);

  if (!data.value) return;

  playerStore.queue = [...data.value.tracks];
  playerStore.queueIndex = idx;
}

onBeforeMount(async () => {
  await updateData();
  playerStore.currentPage = `/album/${artist_id.value}/${album_id.value}`;
});
</script>
