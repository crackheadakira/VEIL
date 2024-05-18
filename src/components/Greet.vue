<template>
    <div class=p-2>
        <div class="flex gap-1">
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md" @click="openDialog">Select
                music
                folder</button>
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md"
                @click="debug">Debug</button>
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md" @click="getID">Get album by
                ID</button>
            <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md" @click="getArtist">Get
                artist by
                ID</button>
        </div>

        <textarea class="p-1 px-2 border font-supporting bg-card border-stroke-100 rounded-md resize-none"
            ref="textArea" cols="1" rows="1"></textarea>

        <p v-if="selectedFile">{{ parsedFile }}</p>
        <audio controls v-if="selectedFile" ref="audioTag"></audio>

        <ul v-if="files">
            <li v-for="file in files" @dblclick="selectFile(file)">
                {{ file.artist }} - {{ file.name }} ({{ file.album }})
            </li>
        </ul>
    </div>
</template>

<script setup lang="ts">
import { commands, type Metadata } from '../bindings';

const files = ref<Metadata[]>([]);
const selectedFile = ref<string | null>(null);
const audioTag = ref<HTMLAudioElement | null>(null);
const textArea = ref<HTMLTextAreaElement | null>(null);
const parsedFile = ref<string>("");

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
    const parsed = await commands.selectMusicFolder();
    files.value = parsed;
    console.log(parsed);
}

function selectFile(file: Metadata) {
    parsedFile.value = `${file.artist} - ${file.name} (${file.album})`;
    selectedFile.value = file.path;
    nextTick(() => {
        const audio = unref(audioTag);
        if (!audio) return console.log("Audio tag not found");
        audio.src = `http://localhost:16780${file.path}`;
        audio.play();
    });
}
</script>
