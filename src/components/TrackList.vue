<template>
  <div class="sodapop-card flex flex-col p-4" ref="trackList">
    <ContextMenu
      :track="track"
      :playlists="playlistStore.playlists"
      :curr_playlist="playlist"
      @queue="handleAddToQueue"
      @playlist="handlePlaylist"
      @create-playlist="
        async (playlist: string, trackId: number) => {
          const playlistId = await playlistStore.createPlaylist(playlist);
          if (playlistId) {
            handlePlaylist('add', playlistId, trackId);
          }
        }
      "
      v-for="(track, idx) of tracks"
    >
      <div
        class="hover:bg-bg-hovered group flex cursor-pointer items-center gap-4 rounded-md p-3 px-4 select-none"
        @dblclick="() => emitNewTrack(track, idx)"
      >
        <div class="flex shrink-0 items-center gap-4">
          <!-- (10 / 16) rem === 10px if rem = 16px -->
          <p
            :style="{ width: idxWidth * (10 / 16) + 'rem' }"
            class="text-text-secondary group-hover:text-text-secondary-hovered text-right"
          >
            {{ idx + 1 }}
          </p>
          <img
            v-if="playlist"
            :src="convertFileSrc(track.cover_path)"
            class="aspect-square w-10 rounded-md"
          />
        </div>
        <div class="grow basis-0 truncate *:truncate">
          <p
            class="text-text-primary group-hover:text-text-primary-hovered mb-1"
          >
            {{ track.name }}
          </p>
          <p
            class="text-text-secondary group-hover:text-text-secondary-hovered"
          >
            {{ track.artist_name }}
          </p>
        </div>
        <RouterLink
          class="grow basis-0 truncate"
          v-if="playlist"
          :to="{
            name: 'album',
            params: { id: track.album_id },
          }"
        >
          <p class="text-text-secondary hover:text-accent-primary truncate">
            {{ track.album_name }}
          </p>
        </RouterLink>
        <p class="text-text-primary text-right">
          {{ formatTime("mm:ss", track.duration) }}
        </p>
      </div>
    </ContextMenu>
  </div>
</template>

<script setup lang="ts">
import { ContextMenu } from "@/components/";
import {
  events,
  formatTime,
  Playlists,
  type Tracks,
  usePlaylistStore,
  useQueueStore,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

const playlistStore = usePlaylistStore();
const queueStore = useQueueStore();

const trackList = ref<HTMLDivElement | null>(null);
const idxWidth = computed(() => props.tracks.length.toString().length); // number of digits

const props = defineProps<{
  tracks: Tracks[];
  playlist?: Playlists;
}>();

// Make the backend set the track and do everything related to it.
async function emitNewTrack(track: Tracks, trackIdx: number) {
  await events.newTrackEvent.emit({ track });

  // Might move queue to backend & instead of persisting
  // queue across runs, make a new queue every time
  // dependent on shuffle state and playlist / album.
  // In backend could simply store a Vec of track IDs.
  // Would allow for pre-fetching the next track in
  // the audio player.
  queueStore.setGlobalQueue(props.tracks);
  queueStore.setQueueIdx(trackIdx);
}

async function handlePlaylist(
  type: "add" | "remove",
  playlistId: number,
  trackId: number,
) {
  if (type === "add") await playlistStore.addToPlaylist(playlistId, trackId);
  else await playlistStore.removeFromPlaylist(playlistId, trackId);
}

async function handleAddToQueue(track: Tracks) {
  queueStore.personalQueue.push(track);
}
</script>
