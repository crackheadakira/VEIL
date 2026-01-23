<template>
  <div class="text-text-primary flex w-full flex-col gap-8" v-if="data">
    <div class="sodapop-card flex items-center gap-8 p-8">
      <img
        class="aspect-square w-64 rounded-md"
        :src="convertFileSrc(data.album.cover_path)"
      />

      <div class="flex flex-col gap-4">
        <div class="flex cursor-default flex-col gap-1 select-none">
          <h6 class="text-text-tertiary">
            {{ data.album.album_type }}
          </h6>
          <h2 class="text-text-primary">{{ data.album.name }}</h2>
          <p class="text-text-secondary">{{ data.album.artist_name }}</p>
          <small class="text-text-tertiary">
            {{ formatTime("hh:mm:ss", data.album.duration) }},
            {{ data.album.track_count }}
            {{ data.album.track_count > 1 ? "songs" : "song" }}
          </small>
          <p v-if="data.album.year" class="text-text-tertiary">
            {{ data.album.year }}
          </p>
        </div>

        <div class="flex gap-4">
          <Button
            @click="handlePlayButton(false)"
            label="Play"
            icon="i-fluent-play-24-filled"
            wide
          />
          <Button
            @click="handlePlayButton(true)"
            label="Shuffle"
            icon="i-fluent-arrow-shuffle-20-filled"
            wide
          />
        </div>
      </div>
    </div>

    <TrackList
      :tracks="data.tracks"
      :total-tracks="data.tracks.length"
      :origin-id="data.album.id"
      class="overflow-y-scroll"
    />
  </div>
</template>

<script setup lang="ts">
import { TrackList, Button } from "@/components/";
import {
  type AlbumWithTracks,
  commands,
  handleBackendError,
  formatTime,
  useConfigStore,
  events,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { onBeforeMount, ref } from "vue";
import { useRoute } from "vue-router";

const configStore = useConfigStore();

const route = useRoute();
const album_id = ref(route.params.id as string);

const data = ref<AlbumWithTracks | null>(null);

/**
 * Handles the big play/shuffle button.
 * @param {boolean} shuffle - Whether the button was play or shuffle
 */
async function handlePlayButton(shuffle: boolean) {
  if (!data.value) return;
  const trackIds = data.value.tracks.map((track) => track.id);

  await events.queueEvent.emit({
    type: "SetGlobalQueue",
    data: {
      tracks: trackIds,
      queue_idx: 0,
      origin: {
        type: "Album",
        data: {
          id: data.value.album.id,
        },
      },
    },
  });

  if (shuffle) {
    await events.queueEvent.emit({
      type: "SetGlobalQueueShuffle",
      data: { shuffle },
    });
  }

  await events.playerEvent.emit({ type: "CurrentTrackInQueue" });
}

async function updateData() {
  const result = await commands.getAlbumWithTracks(parseInt(album_id.value));

  if (result.status === "error") return handleBackendError(result.error);

  data.value = result.data;
}

onBeforeMount(async () => {
  await updateData();
  configStore.currentPage = `/album/${album_id.value}`;
  configStore.pageName = data.value?.album.name || "Album";
});
</script>
