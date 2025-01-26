<template>
    <div class="flex w-full flex-col gap-8 text-text" v-if="artist_data">
        <div class="flex h-72 items-center justify-between overflow-hidden rounded-md border border-stroke-100 bg-card">
            <div class="flex h-full flex-col justify-end gap-4 p-8">
                <h4 class="font-h4 text-text">{{ artist_data.artist.name }}</h4>
                <div class="flex gap-4">
                    <button
                        class="text flex aspect-button h-12 items-center justify-center gap-2 rounded-md bg-primary text-background duration-150 hover:opacity-90">
                        <span class="i-fluent-play-24-filled h-7"></span>
                        <p class="font-main">Play</p>
                    </button>
                    <button
                        class="text flex aspect-button h-12 items-center justify-center gap-2 rounded-md border border-stroke-100 bg-background text-supporting duration-150 hover:opacity-80">
                        <span class="i-fluent-arrow-shuffle-20-filled h-7"></span>
                        <p class="font-main">Shuffle</p>
                    </button>
                </div>
            </div>
            <img class="aspect-square w-2/3 min-w-fit rounded-md object-cover [mask-image:linear-gradient(90deg,rgba(17,17,17,0),rgba(17,17,17,0.4))]"
                :src="convertFileSrc(artist_data.albums[0].album.cover_path)">
        </div>

        <div>
            <p class="font-supporting mb-2 text-supporting">View all</p>
            <div class="flex flex-wrap gap-4">
                <BigCard v-for="album of artist_data.albums" :data="album" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import BigCard from '../components/BigCard.vue';

import { useRoute } from 'vue-router';
import { ArtistWithAlbums, commands } from '../bindings';
import { convertFileSrc } from '@tauri-apps/api/core';

const playerStore = usePlayerStore();
const route = useRoute();
const artist_id = ref(route.params.artist_id as string);
const artist_data = ref<ArtistWithAlbums | null>(null);

async function updateData() {
    if (!artist_id.value) return;
    const response = await commands.getArtistWithAlbums(parseInt(artist_id.value));
    if (response.status === 'error') throw new Error(`[${response.error.type}] ${response.error.data}`);
    artist_data.value = response.data;
}

watch(() => route.params.artist_id, async (newId) => {
    artist_id.value = newId as string;
    await updateData();
    window.scrollTo(0, 0);
})

onBeforeMount(async () => {
    await updateData();
    playerStore.currentPage = `/artist/${artist_id.value}`;
})
</script>