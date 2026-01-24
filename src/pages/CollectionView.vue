<template>
  <div class="text-text-primary flex w-full flex-col gap-8" v-if="data">
    <div
      class="bg-bg-secondary border-border-primary flex items-center gap-8 rounded-md border p-8"
    >
      <img
        v-if="props.type != 'Playlist'"
        class="border-border-secondary size-48 rounded-md border"
        :src="placeholderIfEmpty(data.cover_path)"
      />
      <PlaylistEdit
        :name="data.name"
        :description="data.description"
        :id="data.id"
        @update="updatePlaylist"
        v-else
      />

      <div class="flex flex-col gap-4">
        <div class="flex cursor-default flex-col gap-1 select-none">
          <h6 class="text-text-tertiary">
            {{ data.type }}
          </h6>
          <h2 class="text-text-primary">{{ data.name }}</h2>
          <p class="text-text-secondary" v-if="data.description != ''">
            {{ data.description }}
          </p>
          <p class="text-text-secondary">{{ data.artist_name }}</p>
          <small v-if="data.duration" class="text-text-tertiary">
            {{ formatTime("hh:mm:ss", data.duration) }},
            {{ totalTracks }}
            {{ totalTracks > 1 ? "songs" : "song" }}
          </small>
          <small v-else class="text-text-tertiary">
            {{ totalTracks }}
            {{ totalTracks > 1 ? "songs" : "song" }}
          </small>
          <p v-if="data.year" class="text-text-tertiary">
            {{ data.year }}
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
      :total-tracks="totalTracks"
      :origin-id="collection_id"
      :playlist_id="type === 'Playlist' ? data.id : undefined"
      :fetch-more="type === 'Playlist' ? fetchMore : undefined"
      class="overflow-y-scroll"
    />
  </div>
</template>

<script setup lang="ts">
import { TrackList, Button, PlaylistEdit } from "@/components/";
import {
  commands,
  handleBackendError,
  formatTime,
  useConfigStore,
  events,
  Tracks,
  AlbumType,
  placeholderIfEmpty,
  usePlaylistStore,
} from "@/composables/";
import { onBeforeMount, ref } from "vue";
import { useRoute } from "vue-router";

const props = defineProps<{
  type: "Playlist" | "Album";
}>();

const configStore = useConfigStore();
const playlistStore = usePlaylistStore();

const route = useRoute();
const collection_id = ref<number>(+route.params.id);

type NormalizedCollection = {
  id: number;
  name: string;
  cover_path: string;
  tracks: Tracks[];
  type: AlbumType | "Playlist";
  artist_name?: string;
  year?: number;
  description?: string;
  duration?: number;
};

const data = ref<NormalizedCollection | null>(null);
const totalTracks = ref(0);

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
        type: props.type,
        data: {
          id: data.value.id,
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

async function updatePlaylist(
  name: string,
  description?: string,
  cover?: string,
) {
  const result = await playlistStore.updatePlaylist(
    collection_id.value,
    name,
    description,
    cover,
  );
  if (!result) return;

  if (data.value) {
    data.value.name = name;
    data.value.description = description;
  }
}

async function updateData() {
  if (props.type === "Album") {
    const result = await commands.getAlbumWithTracks(collection_id.value);
    if (result.status === "error") return handleBackendError(result.error);

    data.value = {
      ...result.data.album,
      type: result.data.album.album_type,
      tracks: result.data.tracks,
    };

    totalTracks.value = result.data.tracks.length;
  } else {
    const result = await commands.getPlaylistTracksOffset(
      collection_id.value,
      40,
      0,
    );
    if (result.status === "error") return handleBackendError(result.error);
    data.value = {
      ...result.data.playlist,
      type: "Playlist",
      tracks: result.data.tracks,
    };

    const total = await commands.getTotalTracksInPlaylist(collection_id.value);
    if (total.status === "error") return handleBackendError(total.error);
    totalTracks.value = total.data;
  }
}

async function fetchMore(offset: number, count: number) {
  if (!data.value) return;

  const result = await commands.getPlaylistTracksOffset(
    collection_id.value,
    count,
    offset,
  );
  if (result.status === "error") return handleBackendError(result.error);

  data.value.tracks.push(...result.data.tracks);
}

onBeforeMount(async () => {
  await updateData();
  configStore.currentPage = route.fullPath;
  configStore.pageName = data.value?.name || props.type;
});
</script>
