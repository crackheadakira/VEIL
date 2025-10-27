<template>
  <div class="text-text-primary flex w-full flex-col gap-8" v-if="data">
    <div class="sodapop-card flex items-center gap-8 p-8">
      <img
        class="aspect-square w-64 rounded-md"
        :src="placeholderIfEmpty(data.playlist.cover_path)"
      />

      <div class="flex flex-col gap-4">
        <div class="flex cursor-default flex-col gap-1 select-none">
          <h6 class="text-text-secondary font-medium">Playlist</h6>
          <h4 class="text-text-primary">{{ data.playlist.name }}</h4>
          <p class="text-text-secondary" v-if="data.playlist.description != ''">
            {{ data.playlist.description }}
          </p>
          <small class="text-text-secondary">
            {{ data.tracks.length }}
            {{ data.tracks.length > 1 ? "songs" : "song" }}
          </small>
        </div>

        <div class="flex gap-4 *:cursor-pointer">
          <button
            @click="handlePlayButton(false)"
            class="text aspect-button bg-accent-primary text-bg-primary flex h-12 items-center justify-center gap-2 rounded-md duration-150 hover:opacity-90"
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

    <TrackList :tracks="data.tracks" :playlist="data.playlist" />
  </div>
</template>

<script setup lang="ts">
import { TrackList } from "@/components/";
import {
  events,
  placeholderIfEmpty,
  useConfigStore,
  usePlayerStore,
  usePlaylistStore,
  useQueueStore,
  type PlaylistWithTracks,
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

  queueStore.setGlobalQueue(data.value.tracks);

  if (shuffle) {
    playerStore.isShuffled = false; // To trigger the shuffle no matter current state
    queueStore.shuffleQueue();
  }

  queueStore.setQueueIdx(0);
  const track = await queueStore.getTrackAtIdx(0);
  if (track)
    await events.playerEvent.emit({ type: "NewTrack", data: { track } });
}

async function updateData() {
  const res_data = await playlistStore.getTracksFromPlaylist(
    parseInt(playlist_id.value),
  );
  if (res_data) data.value = res_data;
}

onBeforeMount(async () => {
  await updateData();
  configStore.currentPage = `/playlist/${playlist_id.value}`;
  configStore.pageName = data.value?.playlist.name || "Playlist";
});
</script>
