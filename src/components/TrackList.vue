<template>
    <div class="flex flex-col rounded-md border border-stroke-100 bg-card p-4" ref="trackList">
        <div class="font-supporting contextable flex cursor-pointer select-none items-center gap-8 p-3 px-4 duration-75 hover:bg-background"
            v-for="(track, idx) of data.tracks" @dblclick="$emit('new-track', track.track, idx)">
            <p class="w-9 text-supporting">{{ idx + 1 }}</p>
            <div class="gap flex-grow">
                <p class="mb-1 text-text">{{ track.track.name }}</p>
                <p class="text-supporting">
                    <span v-for="artist in getAsArtists(track)" :key="artist">
                        {{ `${artist} ` }}
                    </span>
                </p>
            </div>
            <p class="text-text">{{ makeReadableTime(track.track.duration) }}</p>
        </div>
    </div>
    <ContextMenu @add-to-queue="handleAddToQueue" />
</template>

<script setup lang="ts">
import type { AlbumWithTracks, Tracks, TrackWithFeatures } from '../bindings';
import ContextMenu from '../components/ContextMenu.vue';

const playerStore = usePlayerStore();

const trackList = ref<HTMLDivElement | null>(null);

const props = defineProps<{
    data: AlbumWithTracks;
}>()

defineEmits<{
    (e: 'new-track', track: Tracks, idx: number): void;
}>();

function getAsArtists(tracks: TrackWithFeatures) {
    const artists = tracks.features.map((artist) => artist.name);
    artists.unshift(tracks.track.artist);
    return artists;
}

async function handleAddToQueue(coords: { x: number, y: number }) {
    if (!props.data) return;
    const trackHeight = trackList.value?.children[0].clientHeight || 76;
    const offsetTop = trackList.value?.offsetTop || 0;
    const index = Math.floor((coords.y - offsetTop) / trackHeight);
    const track = props.data.tracks[index];
    console.log(track.track.name);

    return;
    playerStore.personalQueue.push(track.track);
}
</script>