<template>
  <div class="text-text-primary flex w-full flex-col gap-8" v-if="data">
    <div class="sodapop-card flex items-center gap-8 p-8">
      <img
        class="aspect-square w-64 rounded-md"
        :src="convertFileSrc(data.album.cover_path)"
      />

      <div class="flex flex-col gap-4">
        <div class="flex cursor-default flex-col gap-1 select-none">
          <h6 class="text-text-secondary font-medium">
            {{ data.album.album_type }}
          </h6>
          <h4 class="text-text-primary">{{ data.album.name }}</h4>
          <p class="text-text-secondary">{{ data.album.artist_name }}</p>
          <small class="text-text-secondary">
            {{ formatTime("hh:mm:ss", data.album.duration) }},
            {{ data.album.track_count }}
            {{ data.album.track_count > 1 ? "songs" : "song" }}
          </small>
          <p v-if="data.album.year" class="text-text-secondary">
            {{ data.album.year }}
          </p>
        </div>

        <div class="flex gap-4 *:cursor-pointer">
          <button
            @click="handlePlayButton(false)"
            class="text aspect-button bg-accent-primary text-bg-primary hover:bg-accent-primary-hovered flex h-12 items-center justify-center gap-2 rounded-md duration-150"
          >
            <span class="i-fluent-play-24-filled h-7"></span>
            <p>Play</p>
          </button>
          <button
            @click="handlePlayButton(true)"
            class="text aspect-button border-border-secondary bg-bg-primary text-text-secondary flex h-12 items-center justify-center gap-2 rounded-md border duration-150 hover:opacity-80"
          >
            <span class="i-fluent-arrow-shuffle-20-filled h-7"></span>
            <p>Shuffle</p>
          </button>
        </div>
      </div>
    </div>

    <TrackList :origin-id="data.album.id" :tracks="data.tracks" />
  </div>
</template>

<script setup lang="ts">
import { TrackList } from "@/components/";
import {
  type AlbumWithTracks,
  commands,
  handleBackendError,
  formatTime,
  usePlayerStore,
  useConfigStore,
  useQueueStore,
  events,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { onBeforeMount, ref } from "vue";
import { useRoute } from "vue-router";

const configStore = useConfigStore();
const playerStore = usePlayerStore();
const queueStore = useQueueStore();

const route = useRoute();
const album_id = ref(route.params.id as string);

const data = ref<AlbumWithTracks | null>(null);

/**
 * Handles the big play/shuffle button.
 * @param {boolean} shuffle - Whether the button was play or shuffle
 */
async function handlePlayButton(shuffle: boolean) {
  if (!data.value) return;
  queueStore.setGlobalQueue(data.value.tracks);
  if (shuffle) {
    playerStore.isShuffled = false;
    queueStore.shuffleQueue();
  }

  queueStore.setQueueIdx(0);
  const track = await queueStore.getTrackAtIdx(0);
  if (track)
    await events.playerEvent.emit({ type: "NewTrack", data: { track } });
}

async function updateData() {
  const result = await commands.getAlbumWithTracks(parseInt(album_id.value));

  if (result.status === "error")
    if (result.status === "error") return handleBackendError(result.error);

  data.value = result.data;
}

onBeforeMount(async () => {
  await updateData();
  configStore.currentPage = `/album/${album_id.value}`;
  configStore.pageName = data.value?.album.name || "Album";
});
</script>
