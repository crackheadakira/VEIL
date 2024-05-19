<template>
    <div
        class="flex items-center justify-center gap-12 2xl:gap-20 w-screen aspect-player bg-card border-stroke-100 border-t p-3 text-text">

        <div class="flex items-center gap-5">
            <img class="aspect-square w-20 rounded-md duration-150 group-hover:opacity-90" src='' alt="Album Cover">
            <div class="flex flex-col gap-1">
                <p class="duration-150 font-main-nonbold text-text hover:text-placeholder cursor-pointer">{{ music.title
                    }}
                </p>
                <p class="duration-150 font-supporting text-supporting hover:opacity-85 cursor-pointer">{{ music.artist
                    }}</p>
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

        <div class="flex gap-4 flex-grow items-center text-supporting font-supporting select-none">
            <audio @loadedmetadata="initialLoad()" @timeupdate="handleProgress()" ref="audioTag"
                :src="music.audio"></audio>
            <label for="progress" class=w-10>{{ currentProgress }}</label>
            <input @input="selectProgress()" type="range" ref="progressBar" name="progress" min="0" max="100" value="0"
                class="w-full h-1.5 bg-stroke-100 rounded-lg accent-placeholder">
            <label for="progress" class=w-10>{{ totalLength }}</label>
        </div>

        <div class="flex gap-4 items-center">
            <span class="cursor-pointer hover:text-placeholder duration-150 i-mingcute-volume-fill w-18"></span>
            <input @input="handleVolume()" type="range" ref="volumeBar" min="0" max="100" value="40"
                class="h-1.5 w-full bg-stroke-100 rounded-lg accent-placeholder focus:ring-0">
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
const volumeBar = ref<HTMLInputElement | null>(null);
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

function handleVolume() {
    if (!audioTag.value || !volumeBar.value) return;
    audioTag.value.volume = volumeBar.value.valueAsNumber / 100;
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
    audioTag.value!.volume = volumeBar.value!.valueAsNumber / 100;
}

function makeReadableTime(seconds: number) {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}
</script>