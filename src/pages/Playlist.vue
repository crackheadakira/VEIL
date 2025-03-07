<template>
  <div class="text-text flex w-full flex-col gap-8" v-if="data">
    <div class="sodapop-card flex items-center gap-8 p-8">
      <img
        class="aspect-square w-64 rounded-md"
        :src="placeholderIfEmpty(data.playlist.cover_path)"
      />

      <div class="flex flex-col gap-4">
        <div class="flex cursor-default flex-col gap-1 select-none">
          <p class="text-supporting font-medium">Playlist</p>
          <h4 class="text-text">{{ data.playlist.name }}</h4>
          <p class="text-supporting" v-if="data.playlist.description != ''">
            {{ data.playlist.description }}
          </p>
          <small class="text-supporting">
            {{ data.tracks.length }}
            {{ data.tracks.length > 1 ? "songs" : "song" }}
          </small>
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
            class="text aspect-button border-stroke-200 bg-background text-supporting flex h-12 items-center justify-center gap-2 rounded-md border duration-150 hover:opacity-80"
          >
            <span class="i-fluent-arrow-shuffle-20-filled h-7"></span>
            <p>Shuffle</p>
          </button>
        </div>
      </div>
    </div>

    <TrackList :data="data" @new-track="handleNewTrack" />
  </div>
</template>

<script setup lang="ts">
import { TrackList } from "@/components/";
import {
  placeholderIfEmpty,
  useConfigStore,
  usePlayerStore,
  usePlaylistStore,
  useQueueStore,
  type PlaylistWithTracks,
  type Tracks,
} from "@/composables/";
import { onBeforeMount, ref, watch } from "vue";
import { useRoute } from "vue-router";

const configStore = useConfigStore();
const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();
const queueStore = useQueueStore();

const route = useRoute();
const playlist_id = ref(route.params.id as string);

const data = ref<PlaylistWithTracks | null>(null);

watch(
  () => route.params.playlist_id,
  async (newId) => {
    playlist_id.value = newId as string;
    await updateData();

    window.scrollTo(0, 0);
  },
);

playlistStore.$onAction(({ name, args, after }) => {
  if (name === "addToPlaylist") {
    if (args[0] === parseInt(playlist_id.value)) {
      after(async () => {
        await updateData();
      });
    }
  } else if (name === "removeFromPlaylist") {
    if (args[0] === parseInt(playlist_id.value)) {
      after(async () => {
        await updateData();
      });
    }
  }
});

async function handlePlayButton(shuffle: boolean) {
  if (!data.value) return;

  queueStore.globalQueue = [...data.value.tracks];

  if (shuffle) {
    playerStore.isShuffled = false; // To trigger the shuffle no matter current state
    queueStore.shuffleQueue();
  }

  queueStore.index = 0;
  await playerStore.setPlayerTrack(queueStore.getQueueTrack());
}

async function updateData() {
  const res_data = await playlistStore.getTracksFromPlaylist(
    parseInt(playlist_id.value),
  );
  if (res_data) data.value = res_data;
}

async function handleNewTrack(track: Tracks, idx: number) {
  await playerStore.setPlayerTrack(track);

  if (!data.value) return;

  queueStore.globalQueue = [...data.value.tracks];
  queueStore.index = idx;
}

onBeforeMount(async () => {
  await updateData();
  configStore.currentPage = `/playlist/${playlist_id.value}`;
  configStore.pageName = data.value?.playlist.name || "Playlist";
});
</script>
