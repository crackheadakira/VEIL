<template>
  <div
    class="border-stroke-100 bg-card flex flex-col rounded-md border p-4"
    ref="trackList"
  >
    <div
      class="contextable hover:bg-background flex cursor-pointer items-center gap-8 rounded-md p-3 px-4 duration-75 select-none"
      v-for="(track, idx) of data.tracks"
      @dblclick="$emit('new-track', track, idx)"
    >
      <small class="text-supporting w-9">{{ idx + 1 }}</small>
      <div class="gap grow">
        <small class="text-text mb-1">{{ track.name }}</small>
        <small class="text-supporting">
          {{ track.artist }}
        </small>
      </div>
      <small class="text-text">{{ makeReadableTime(track.duration) }}</small>
    </div>
  </div>
  <ContextMenu
    :track-list="props.data.tracks"
    @add-to-queue="handleAddToQueue"
  />
</template>

<script setup lang="ts">
import type { AlbumWithTracks, Tracks } from "../bindings";
import ContextMenu from "../components/ContextMenu.vue";

const playerStore = usePlayerStore();

const trackList = ref<HTMLDivElement | null>(null);

const props = defineProps<{
  data: AlbumWithTracks;
}>();

defineEmits<{
  (e: "new-track", track: Tracks, idx: number): void;
}>();

async function handleAddToQueue(track: Tracks | null) {
  if (!props.data || !track) return;
  playerStore.personalQueue.push(track);
}
</script>
