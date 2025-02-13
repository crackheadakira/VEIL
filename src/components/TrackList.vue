<template>
  <div class="cardStyle flex flex-col p-4" ref="trackList">
    <div
      class="contextable hover:bg-background flex cursor-pointer items-center gap-4 rounded-md p-3 px-4 duration-75 select-none"
      v-for="(track, idx) of data.tracks"
      @dblclick="$emit('new-track', track, idx)"
    >
      <div class="flex items-center">
        <small class="text-supporting w-9">{{ idx + 1 }}</small>
        <img
          v-if="'playlist' in data"
          :src="convertFileSrc(track.cover_path)"
          class="aspect-square w-10 rounded-md"
        />
      </div>
      <div class="gap grow">
        <small class="text-text mb-1">{{ track.name }}</small>
        <small class="text-supporting">
          {{ track.artist_name }}
        </small>
      </div>
      <RouterLink
        v-if="'playlist' in data"
        :to="{
          name: 'album',
          params: { album_id: track.album_id, artist_id: track.artist_id },
        }"
      >
        <small class="text-supporting hover:text-text">{{
          track.album_name
        }}</small>
      </RouterLink>
      <small class="text-text">{{ makeReadableTime(track.duration) }}</small>
    </div>
  </div>
  <ContextMenu :data="props.data" @add-to-queue="handleAddToQueue" />
</template>

<script setup lang="ts">
import { ContextMenu } from "@/components/";
import {
  type AlbumWithTracks,
  makeReadableTime,
  type PlaylistWithTracks,
  type Tracks,
  usePlayerStore,
} from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ref } from "vue";

const playerStore = usePlayerStore();

const trackList = ref<HTMLDivElement | null>(null);

const props = defineProps<{
  data: AlbumWithTracks | PlaylistWithTracks;
}>();

defineEmits<{
  (e: "new-track", track: Tracks, idx: number): void;
}>();

async function handleAddToQueue(track: Tracks | null) {
  if (!props.data || !track) return;
  playerStore.personalQueue.push(track);
}
</script>
