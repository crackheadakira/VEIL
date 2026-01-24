<template>
  <div
    class="bg-bg-secondary border-border-primary flex flex-col rounded-md border p-4"
    ref="trackList"
  >
    <div
      class="text-text-tertiary border-border-secondary mb-4 grid items-center gap-4 border-b p-3 px-4 select-none"
      :class="
        playlist_id
          ? 'grid-cols-[auto_2fr_1fr_auto]'
          : 'grid-cols-[auto_1fr_auto]'
      "
    >
      <small>#</small>
      <small class="col-span-1">Title</small>
      <small v-if="playlist_id" class="col-span-1">Album</small>
      <span
        class="i-fluent-clock-12-regular text-text-secondary -end-col-1 size-4"
      ></span>
    </div>

    <VirtualList
      :items="tracks"
      :total="totalTracks"
      :itemHeight="72"
      :gap="0"
      :fetch-more="props.fetchMore"
      mode="list"
      class="h-60vh"
    >
      <template #default="{ items, startIndex }">
        <ContextMenu
          v-for="(track, idx) in items"
          :key="track.id"
          :track="track"
          :playlists="playlistStore.playlists"
          :playlist_id="playlist_id"
          @playlist="handlePlaylist"
          @create-playlist="createNewPlaylist"
        >
          <div
            v-show="!imageLoading[track.id]"
            class="hover:bg-bg-secondary-hovered group grid cursor-pointer items-center gap-4 rounded-md p-3 px-4 select-none"
            :class="
              playlist_id
                ? 'grid-cols-[auto_2fr_1fr_auto]'
                : 'grid-cols-[auto_1fr_auto]'
            "
            @dblclick="emitNewTrack(track, idx + startIndex)"
          >
            <div class="flex shrink-0 items-center gap-4">
              <!-- (10 / 16) rem === 10px if rem = 16px -->
              <p
                :style="{
                  width:
                    (startIndex + idx).toString().length * (10 / 16) + 'rem',
                }"
                class="text-text-tertiary group-hover:text-text-tertiary-hovered text-right"
              >
                {{ startIndex + idx + 1 }}
              </p>
              <div class="relative aspect-square w-10">
                {{ ensureLoading(track.id) }}

                <img
                  :src="convertFileSrc(track.cover_path)"
                  class="aspect-square w-10 rounded-md"
                  @load="imageLoading[track.id] = false"
                  @error="imageLoading[track.id] = false"
                />
              </div>
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
              v-if="playlist_id"
              :to="{
                name: 'album',
                params: { id: track.album_id },
              }"
            >
              <small
                class="text-text-secondary hover:text-text-secondary-hovered truncate hover:underline"
              >
                {{ track.album_name }}
              </small>
            </RouterLink>
            <div
              class="grid grid-cols-[1rem_auto] items-center justify-end gap-5"
            >
              <p
                class="text-text-primary group-hover:text-text-primary-hovered text-right tabular-nums"
              >
                {{ formatTime("mm:ss", track.duration) }}
              </p>
            </div>
          </div>

          <div
            v-if="imageLoading[track.id]"
            class="grid cursor-pointer items-center gap-4 rounded-md p-3 px-4 select-none"
            :class="
              playlist_id
                ? 'grid-cols-[auto_2fr_1fr_auto]'
                : 'grid-cols-[auto_1fr_auto]'
            "
          >
            <div class="relative flex w-10 shrink-0 items-center gap-4">
              <div class="relative aspect-square w-10">
                {{ ensureLoading(track.id) }}
                <div class="skeleton-loader aspect-square w-10 rounded-md" />
              </div>
            </div>

            <div class="grow basis-0">
              <div class="relative mb-1 h-4 w-96">
                <div class="skeleton-loader bg-border-primary"></div>
              </div>
              <div class="relative h-4 w-32">
                <div class="skeleton-loader"></div>
              </div>
            </div>

            <div class="relative h-4 w-32 grow basis-0">
              <div class="skeleton-loader bg-border-primary"></div>
            </div>
          </div>
        </ContextMenu>
      </template>
    </VirtualList>
  </div>
</template>

<script setup lang="ts">
import { ContextMenu, VirtualList } from "@/components/";
import {
  events,
  formatTime,
  type Tracks,
  usePlaylistStore,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ref } from "vue";

const playlistStore = usePlaylistStore();

const props = defineProps<{
  tracks: Tracks[];
  totalTracks: number;
  originId: number;
  playlist_id?: number;
  fetchMore?: (offset: number, count: number) => Promise<void>;
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
        type: props.playlist_id !== undefined ? "Playlist" : "Album",
        data: { id: props.originId },
      },
    },
  });
}

const imageLoading = ref<Record<number, boolean>>({});

function ensureLoading(id: number) {
  if (imageLoading.value[id] === undefined) {
    imageLoading.value[id] = true;
  }
}

async function handlePlaylist(
  type: "add" | "remove",
  playlistId: number,
  trackId: number,
) {
  if (type === "add") await playlistStore.addToPlaylist(playlistId, trackId);
  else await playlistStore.removeFromPlaylist(playlistId, trackId);
}

async function createNewPlaylist(playlist: string, trackId: number) {
  const playlistId = await playlistStore.createPlaylist(playlist);

  if (playlistId) {
    handlePlaylist("add", playlistId, trackId);
  }
}
</script>
