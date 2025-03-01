<template>
  <div class="sodapop-card flex flex-col p-4" ref="trackList">
    <ContextMenu
      :track="track"
      :playlists="playlistStore.playlists"
      :curr_playlist="'playlist' in data ? data.playlist : null"
      @queue="handleAddToQueue"
      @playlist="handlePlaylist"
      v-for="(track, idx) of data.tracks"
    >
      <div
        class="hover:bg-background flex cursor-pointer items-center gap-4 rounded-md p-3 px-4 duration-75 select-none"
        @dblclick="$emit('new-track', track, idx)"
      >
        <div class="flex shrink-0 items-center gap-4">
          <!-- (10 / 16) rem === 10px if rem = 16px -->
          <small
            :style="{ width: idxWidth * (10 / 16) + 'rem' }"
            class="text-supporting text-right"
            >{{ idx + 1 }}</small
          >
          <img
            v-if="'playlist' in data"
            :src="convertFileSrc(track.cover_path)"
            class="aspect-square w-10 rounded-md"
          />
        </div>
        <div class="grow basis-0 truncate *:truncate">
          <small class="text-text mb-1">{{ track.name }}</small>
          <small class="text-supporting">
            {{ track.artist_name }}
          </small>
        </div>
        <RouterLink
          class="grow basis-0 truncate"
          v-if="'playlist' in data"
          :to="{
            name: 'album',
            params: { id: track.album_id },
          }"
        >
          <small class="text-supporting hover:text-text truncate">{{
            track.album_name
          }}</small>
        </RouterLink>
        <small class="text-text text-right">{{
          formatTime("mm:ss", track.duration)
        }}</small>
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
  usePlayerStore,
  usePlaylistStore,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

const playlistStore = usePlaylistStore();
const playerStore = usePlayerStore();

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
  playerStore.personalQueue.push(track);
}
</script>
