<template>
  <div class="sodapop-card flex flex-col p-4" ref="trackList">
    <div
      class="text-text-secondary border-border-secondary mb-4 grid items-center gap-4 rounded-md border-b p-3 px-4 select-none"
      :class="
        playlist ? 'grid-cols-[auto_2fr_1fr_auto]' : 'grid-cols-[auto_1fr_auto]'
      "
    >
      <small>#</small>
      <small class="col-span-1">Title</small>
      <small v-if="playlist" class="col-span-1">Album</small>
      <span
        class="i-fluent-clock-12-regular text-text-secondary -end-col-1 size-4"
      ></span>
    </div>
    <ContextMenu
      :track="track"
      :playlists="playlistStore.playlists"
      :curr_playlist="playlist"
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
        class="hover:bg-bg-hovered group grid cursor-pointer items-center gap-4 rounded-md p-3 px-4 select-none"
        :class="
          playlist
            ? 'grid-cols-[auto_2fr_1fr_auto]'
            : 'grid-cols-[auto_1fr_auto]'
        "
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
          <small
            class="text-text-secondary group-hover:text-text-secondary-hovered"
          >
            {{ track.artist_name }}
          </small>
        </div>
        <RouterLink
          class="grow basis-0 truncate"
          v-if="playlist"
          :to="{
            name: 'album',
            params: { id: track.album_id },
          }"
        >
          <small
            class="text-text-secondary hover:text-accent-secondary truncate hover:underline"
          >
            {{ track.album_name }}
          </small>
        </RouterLink>
        <div class="grid grid-cols-[1rem_auto] items-center justify-end gap-2">
          <p
            class="text-text-primary group-hover:text-text-primary-hovered text-right tabular-nums"
          >
            {{ formatTime("mm:ss", track.duration) }}
          </p>
        </div>
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
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed } from "vue";

const playlistStore = usePlaylistStore();

const idxWidth = computed(() => props.tracks.length.toString().length); // number of digits

const props = defineProps<{
  tracks: Tracks[];
  originId: number;
  playlist?: Playlists;
}>();

// Make the backend set the track and do everything related to it.
async function emitNewTrack(track: Tracks, trackIdx: number) {
  await events.playerEvent.emit({ type: "NewTrack", data: { track } });
  const trackIds = props.tracks.map((track) => track.id);

  await events.queueEvent.emit({
    type: "SetGlobalQueue",
    data: {
      tracks: trackIds,
      queue_idx: trackIdx,
      origin: {
        type: props.playlist !== undefined ? "Playlist" : "Album",
        data: { id: props.originId },
      },
    },
  });
}

async function handlePlaylist(
  type: "add" | "remove",
  playlistId: number,
  trackId: number,
) {
  if (type === "add") await playlistStore.addToPlaylist(playlistId, trackId);
  else await playlistStore.removeFromPlaylist(playlistId, trackId);
}
</script>
