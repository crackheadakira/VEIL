<template>
    <div class=p-2>
        <div class="flex gap-1">
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md" @click="openDialog">Select
                music folder</button>
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md"
                @click="debug">Debug</button>
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md" @click="getID">Get album by
                ID</button>
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md" @click="getArtist">Get
                artist by ID</button>
            <button class="p-2 border font-supporting bg-card border-stroke-100 text-text rounded-md"
                @click="setTrack">Set
                player track</button>
        </div>

        {{ musicLoaded ? 'Music loaded' : 'Music not loaded' }}

        <textarea class="p-1 px-2 border font-supporting bg-card border-stroke-100 rounded-md resize-none"
            ref="textArea" cols="1" rows="1"></textarea>
    </div>
</template>

<script setup lang="ts">
import { commands } from '../bindings';

const textArea = ref<HTMLTextAreaElement | null>(null);
const musicLoaded = ref(false);

async function setTrack() {
    const textField = unref(textArea);
    if (!textField) return;
    const res = await commands.trackById(+textField.value);
    setPlayerTrack(res);
}

async function debug() {
    const res = await commands.getSqlite();
    console.log(res);
}

async function getID() {
    const perf = performance.now();
    const res = await commands.getAlbumWithTracks(+textArea.value!.value);
    console.log(res);
    const result = performance.now() - perf;
    console.log(`[Rust] Took ${result.toFixed(2)}ms`);
}

async function getArtist() {
    const perf = performance.now()
    const res = await commands.getArtistWithAlbums(+textArea.value!.value);
    console.log(res);
    const result = performance.now() - perf;
    console.log(`[Rust] Took ${result.toFixed(2)}ms`);
}

async function openDialog() {
    await commands.selectMusicFolder();
    musicLoaded.value = true;
}
</script>
