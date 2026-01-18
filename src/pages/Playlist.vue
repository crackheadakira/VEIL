<template>
  <div class="text-text-primary flex h-full w-full flex-col gap-8" v-if="data">
    <div class="sodapop-card flex items-center gap-8 p-8">
      <img
        class="aspect-square w-64 rounded-md"
        :src="placeholderIfEmpty(data.playlist.cover_path)"
      />

      <div class="flex flex-col gap-4">
        <div class="flex cursor-default flex-col gap-1 select-none">
          <h6 class="text-text-secondary">Playlist</h6>
          <h2 class="text-text-primary">{{ data.playlist.name }}</h2>
          <p class="text-text-secondary" v-if="data.playlist.description != ''">
            {{ data.playlist.description }}
          </p>
          <small class="text-text-secondary">
            {{ totalTracks }}
            {{ totalTracks > 1 ? "songs" : "song" }}
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

    <TrackList
      :tracks="data.tracks"
      :total-tracks="totalTracks"
      :origin-id="data.playlist.id"
      :playlist="data.playlist"
      :fetch-more="fetchMore"
      class="overflow-y-scroll"
    />
  </div>
</template>

<script setup lang="ts">
import { TrackList } from "@/components/";
import {
  commands,
  events,
  handleBackendError,
  placeholderIfEmpty,
  useConfigStore,
  usePlaylistStore,
  type PlaylistWithTracks,
} from "@/composables/";
import { onBeforeMount, ref, watch } from "vue";
import { useRoute } from "vue-router";

const configStore = useConfigStore();
const playlistStore = usePlaylistStore();

const route = useRoute();
const playlist_id = ref(route.params.id as string);

const data = ref<PlaylistWithTracks | null>(null);
const totalTracks = ref(0);

watch(
  () => route.params.id,
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
        type: "Playlist",
        data: {
          id: data.value.playlist.id,
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
  const result = await commands.getPlaylistTracksOffset(
    parseInt(playlist_id.value),
    40,
    0,
  );
  if (result.status === "error") return handleBackendError(result.error);
  data.value = result.data;

  const total = await commands.getTotalTracksInPlaylist(+playlist_id.value);
  if (total.status === "error") return handleBackendError(total.error);
  totalTracks.value = total.data;
}

async function fetchMore(offset: number, count: number) {
  if (!data.value) return;

  const result = await commands.getPlaylistTracksOffset(
    +playlist_id.value,
    count,
    offset,
  );
  if (result.status === "error") return handleBackendError(result.error);

  data.value.tracks.push(...result.data.tracks);
}

onBeforeMount(async () => {
  await updateData();

  const total = await commands.getTotalTracksInPlaylist(+playlist_id.value);
  if (total.status === "error") return handleBackendError(total.error);
  totalTracks.value = total.data;

  configStore.currentPage = `/playlist/${playlist_id.value}`;
  configStore.pageName = data.value?.playlist.name || "Playlist";
});
</script>
