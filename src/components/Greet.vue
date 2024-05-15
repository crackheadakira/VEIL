<template>
    <button @click="openDialog">Select music folder</button>
    <button @click="debug">Debug</button>
    <button @click="getID">Get album by ID</button>
    <button @click="getArtist">Get artist by ID</button>

    <p v-if="selectedFile">{{ parsedFile }}</p>
    <audio controls v-if="selectedFile" ref="audioTag"></audio>

    <ul>
        <li v-for="file in files" @dblclick="selectFile(file)">
            {{ file.artist }} - {{ file.name }} ({{ file.album }})
        </li>
    </ul>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

type Track = {
    id: number;
    album: string;
    albums_id: number;
    artist: string;
    name: string;
    path: string;
};

const files = ref<Track[]>([]);
const selectedFile = ref<string | null>(null);
const audioTag = ref<HTMLAudioElement>();
const parsedFile = ref<string>("");

async function debug() {
    const res = await invoke<void>("get_sqlite");
    console.log(res);
}

async function getID() {
    const perf = performance.now();
    const res = await invoke<void>("get_album_with_tracks", { id: 6 });
    console.log(res);
    const result = performance.now() - perf;
    console.log(`[Rust] Took ${result.toFixed(2)}ms`);
}

async function getArtist() {
    const perf = performance.now()
    const res = await invoke<void>("get_artist_with_albums", { id: 10 });
    console.log(res);
    const result = performance.now() - perf;
    console.log(`[Rust] Took ${result.toFixed(2)}ms`);
}

async function openDialog() {
    const parsed = await invoke<Track[]>("select_music_folder");
    files.value = parsed;
    console.log(parsed);
}

async function selectFile(file: Track) {
    parsedFile.value = `${file.artist} - ${file.name} (${file.album}})`;
    selectedFile.value = file.path;
    const fileSrc = await invoke<string>("convert_file", {
        path: selectedFile.value,
    });
    audioTag.value!.src = fileSrc;
    audioTag.value!.play();
}
</script>
