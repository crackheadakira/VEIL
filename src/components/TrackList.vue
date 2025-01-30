<template>
  <div
    class="border-stroke-100 bg-card flex flex-col rounded-md border p-4"
    ref="trackList"
  >
    <div
      class="ontextable hover:bg-background flex cursor-pointer items-center gap-8 p-3 px-4 duration-75 select-none"
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
      <p class="text-text">{{ makeReadableTime(track.duration) }}</p>
    </div>
  </div>
  <ContextMenu @add-to-queue="handleAddToQueue" />
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

async function handleAddToQueue(coords: { x: number; y: number }) {
  if (!props.data) return;
  const trackHeight = trackList.value?.children[0].clientHeight || 76;
  const offsetTop = trackList.value?.offsetTop || 0;
  const index = Math.floor((coords.y - offsetTop) / trackHeight);
  const track = props.data.tracks[index];

  playerStore.personalQueue.push(track);
}
</script>
