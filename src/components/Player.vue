<template>
    <div
        class="flex items-center justify-between absolute bottom-0 w-screen aspect-player bg-card border-stroke-100 border-t p-4">
        <div class="flex items-center gap-5">
            <img class="aspect-square w-20 rounded-md duration-150 group-hover:opacity-90"
                src='/home/akira/.local/share/sodapop-reimagined/covers/DPR IAN - Moodswings In To Order.jpg'
                alt="Album Cover">
            <div class="flex flex-col gap-1">
                <p class="font-main-nonbold text-text">{{ music.title }}</p>
                <p class="font-supporting text-supporting">{{ music.artist }}</p>
            </div>
        </div>
        <div class="flex gap-2">
            <span class="cursor-pointer hover:text-placeholder duration-150 i-ph-shuffle-bold w-6"></span>
            <span class="cursor-pointer hover:text-placeholder duration-150 i-ph-skip-back-fill w-6"></span>
            <span @click="handlePlayAndPause"
                class="cursor-pointer hover:text-placeholder duration-150 i-ph-pause-fill w-7"></span>
            <span class="cursor-pointer hover:text-placeholder duration-150 i-ph-skip-forward-fill w-6"></span>
            <span class="cursor-pointer hover:text-placeholder duration-150 i-ph-repeat-bold w-6"></span>
        </div>
        <div class="flex gap-4 items-center text-supporting font-supporting select-none">
            <audio @loadedmetadata="initialLoad()" @timeupdate="handleProgress()" ref="audioTag"
                :src="music.audio"></audio>
            <label for="progress" class=w-10>{{ currentProgress }}</label>
            <input @input="selectProgress()" type="range" ref="progressBar" name="progress" min="0" max="100" value="0"
                class="w-[50.5rem] h-1.5 bg-stroke-100 rounded-lg accent-placeholder">
            <label for="progress" class=w-10>{{ totalLength }}</label>
        </div>
    </div>
</template>

<script setup lang="ts">
defineProps<{
    music: {
        title: string
        artist: string
        cover: string,
        audio: string
    }
}>()

const audioTag = ref<HTMLAudioElement | null>(null);
const progressBar = ref<HTMLInputElement | null>(null);
const totalLength = ref('3:33');
const currentProgress = ref('0:00');

function handleProgress() {
    const audio = unref(audioTag);
    if (!audio) return;
    if (progressBar) {
        const progress = audio.currentTime;
        progressBar.value!.value = progress.toString();
        currentProgress.value = makeReadableTime(progress);
    }
}

function selectProgress() {
    if (!audioTag.value) return;
    const progress = progressBar.value!.valueAsNumber;
    audioTag.value.fastSeek(progress);
    handleProgress();
}

function handlePlayAndPause() {
    const audio = unref(audioTag);
    if (!audio) return;
    if (audio.paused) audio.play();
    else audio.pause();
}

function initialLoad() {
    totalLength.value = makeReadableTime(audioTag.value!.duration);
    progressBar.value!.max = audioTag.value!.duration.toString();
}

function makeReadableTime(seconds: number) {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}
</script>