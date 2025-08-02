<template>
  <div class="sodapop-card flex flex-col p-4" ref="trackList">
    <ContextMenu
      :track="track"
      :playlists="playlistStore.playlists"
      :curr_playlist="'playlist' in data ? data.playlist : undefined"
      @queue="handleAddToQueue"
      @playlist="handlePlaylist"
      v-for="(track, idx) of data.tracks"
    >
      <div
        class="hover:bg-stroke-200 flex cursor-pointer items-center gap-4 rounded-md p-3 px-4 select-none"
        @dblclick="$emit('new-track', track, idx)"
      >
        <div class="flex shrink-0 items-center gap-4">
          <!-- (10 / 16) rem === 10px if rem = 16px -->
          <p
            :style="{ width: idxWidth * (10 / 16) + 'rem' }"
            class="text-supporting text-right"
          >
            {{ idx + 1 }}
          </p>
          <img
            v-if="'playlist' in data"
            :src="convertFileSrc(track.cover_path)"
            class="aspect-square w-10 rounded-md"
          />
        </div>
        <div class="grow basis-0 truncate *:truncate">
          <p class="text-text mb-1">{{ track.name }}</p>
          <p class="text-supporting">
            {{ track.artist_name }}
          </p>
        </div>
        <RouterLink
          class="grow basis-0 truncate"
          v-if="'playlist' in data"
          :to="{
            name: 'album',
            params: { id: track.album_id },
          }"
        >
          <p class="text-supporting hover:text-text truncate">
            {{ track.album_name }}
          </p>
        </RouterLink>
        <p class="text-text text-right">
          {{ formatTime("mm:ss", track.duration) }}
        </p>
      </div>
    </ContextMenu>
  </div>
</template>

<script setup lang="ts">
import { ContextMenu } from "@/components/";
import {
  type AlbumWithTracks,
  formatTime,
  Playlists,
  type PlaylistWithTracks,
  type Tracks,
  usePlaylistStore,
  useQueueStore,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

const playlistStore = usePlaylistStore();
const queueStore = useQueueStore();

const trackList = ref<HTMLDivElement | null>(null);
const idxWidth = computed(() => props.data.tracks.length.toString().length); // number of digits

const props = defineProps<{
  data: AlbumWithTracks | PlaylistWithTracks;
}>();

defineEmits<{
  (e: "new-track", track: Tracks, idx: number): void;
}>();

async function handlePlaylist(
  type: "add" | "remove",
  playlist: Playlists,
  track: Tracks,
) {
  if (type === "add") await playlistStore.addToPlaylist(playlist.id, track.id);
  else await playlistStore.removeFromPlaylist(playlist.id, track.id);
}

async function handleAddToQueue(track: Tracks) {
  queueStore.personalQueue.push(track);
}
</script>
