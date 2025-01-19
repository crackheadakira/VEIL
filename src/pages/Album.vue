<template>
    <div class="flex w-full flex-col gap-8 text-text" v-if="data">
        <div class="flex items-center gap-8 rounded-md border border-stroke-100 bg-card p-8">
            <img class="aspect-square w-64 rounded-md" :src="convertFileSrc(data.album.cover_path)">

            <div class="flex flex-col gap-4">
                <div class="flex cursor-default select-none flex-col gap-1">
                    <p class="font-main-nonbold text-supporting">{{ data.album.album_type }}</p>
                    <h4 class="font-h4 text-text">{{ data.album.name }}</h4>
                    <p class="font-main text-supporting">{{ data.album.artist }}</p>
                    <p class="font-supporting text-supporting">{{ makeTime(data.album.duration) }}, {{
                        data.album.track_count }} {{ data.album.track_count > 1 ?
                            "songs" : "song" }}</p>
                    <p class="font-supporting text-supporting">{{ data.album.year }}</p>
                </div>

                <div class="flex gap-4">
                    <button
                        class="text flex aspect-button h-12 items-center justify-center gap-2 rounded-md bg-primary text-background duration-150 hover:opacity-90">
                        <span class="i-ph-play-fill h-7"></span>
                        <p class="font-main">Play</p>
                    </button>
                    <button
                        class="text flex aspect-button h-12 items-center justify-center gap-2 rounded-md border border-stroke-100 bg-background text-supporting duration-150 hover:opacity-80">
                        <span class="i-ph-shuffle h-7"></span>
                        <p class="font-main">Shuffle</p>
                    </button>
                </div>
            </div>
        </div>

        <div class="flex flex-col rounded-md border border-stroke-100 bg-card" ref="trackList">
            <div class="contextable flex cursor-pointer select-none items-center gap-8 px-8 py-4 hover:opacity-80"
                v-for="(track, idx) of data.tracks" @dblclick="handleNewTrack(track, idx)">
                <p class="font-main w-9 text-supporting">{{ idx + 1 }}</p>
                <div class="flex-grow">
                    <p class="font-main-nonbold text-text">{{ track.name }}</p>
                    <p class="font-supporting text-supporting">{{ track.artist }}</p>
                </div>
                <p class="font-main-nonbold text-text">{{ makeReadableTime(track.duration) }}</p>
            </div>
        </div>

        <div v-if="artist && artist.albums.length">
            <h5 class="font-h5 mb-4 text-text">More from {{ artist.artist.name }}</h5>
            <div class="flex flex-wrap gap-4">
                <BigCard v-for="album of artist.albums" :data="album.album" />
            </div>
        </div>
        <ContextMenu @add-to-queue="handleAddToQueue" />
    </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';
import { ArtistWithAlbums, commands, AlbumWithTracks, Tracks } from '../bindings';
import { useRoute } from 'vue-router';
import BigCard from '../components/BigCard.vue';
import ContextMenu from '../components/ContextMenu.vue';

const trackList = ref<HTMLDivElement | null>(null);

const route = useRoute();
const album_id = ref(route.params.album_id as string);
const artist_id = ref(route.params.artist_id as string);

const artist = ref<ArtistWithAlbums | null>(null);
const data = ref<AlbumWithTracks | null>(null);

watch(() => route.params.album_id, (newId) => {
    album_id.value = newId as string,
        artist_id.value = route.params.artist_id as string;
    updateData();
    window.scrollTo(0, 0);
})

async function handleAddToQueue(coords: { x: number, y: number }) {
    if (!data.value) return;
    const offsetTop = trackList.value?.offsetTop || 0;
    const index = Math.floor((coords.y - offsetTop) / 76);
    const track = data.value.tracks[index];
    addToPersonalQueue(track);
}

async function updateData() {
    const res = await commands.getArtistWithAlbums(+artist_id.value);
    const current_album = res.albums.filter((album) => album.album.id === +album_id.value)[0];
    data.value = current_album;
    res.albums.splice(res.albums.indexOf(current_album), 1);
    artist.value = res;
    setCurrentPage(`/album/${artist_id.value}/${album_id.value}`);
}

async function handleNewTrack(track: Tracks, idx: number) {
    await setPlayerTrack(track);

    if (!data.value) return;
    setRecentlyPlayed(data.value.album);

    setQueue(data.value.tracks);
    setQueueIndex(idx);
}

onBeforeMount(async () => {
    await updateData();

})
</script>