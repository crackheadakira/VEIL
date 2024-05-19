<template>
    <div class="flex flex-col text-text px-12 py-16 gap-8" v-if="data">
        <div class="flex items-center p-8 gap-8 bg-card border-stroke-100 border rounded-md">
            <img class="aspect-square w-64 rounded-md" :src="convertFileSrc(data.album.cover_path)">

            <div class="flex flex-col gap-4">
                <div class="flex flex-col gap-1 select-none cursor-default">
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
                        class="bg-primary rounded-md text flex justify-center items-center gap-2 text-background aspect-button h-12 duration-150 hover:opacity-90">
                        <span class="i-ph-play-fill h-7"></span>
                        <p class="font-main">Play</p>
                    </button>
                    <button
                        class="bg-background border-stroke-100 border rounded-md text flex justify-center items-center gap-2 text-supporting aspect-button h-12 duration-150 hover:opacity-80">
                        <span class="i-ph-shuffle h-7"></span>
                        <p class="font-main">Shuffle</p>
                    </button>
                </div>
            </div>
        </div>

        <div class="flex flex-col bg-card border-stroke-100 border rounded-md">
            <div class="flex items-center gap-8 duration-150 px-8 py-4 cursor-pointer hover:opacity-80"
                v-for="(track, idx) of data.tracks" @dblclick="setPlayerTrack(track)">
                <p class="font-main text-supporting">{{ idx + 1 }}</p>
                <div class="flex-grow">
                    <p class="font-main-nonbold text-text">{{ track.name }}</p>
                    <p class="font-supporting text-supporting">{{ track.artist }}</p>
                </div>
                <p class="font-main-nonbold text-text">{{ makeReadableTime(track.duration) }}</p>
            </div>
        </div>

    </div>
    <RouterLink class="p-2 w-32 border font-supporting bg-card border-stroke-100 rounded-md text-center text-text"
        to="/">Go to Home</RouterLink>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';
import { commands, type AlbumWithTracks } from '../bindings';
import { useRoute } from 'vue-router';

const route = useRoute();
const album_id = route.params.id as string;

const data = ref<AlbumWithTracks | null>(null);

onBeforeMount(async () => {

    data.value = await commands.getAlbumWithTracks(+album_id);
    setCurrentPage(`/album/${album_id}`);
})
</script>