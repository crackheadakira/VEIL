<template>
    <div class="flex items-center justify-center gap-8 w-screen aspect-player bg-card border-stroke-100 border-t p-3 text-text"
        v-if="music.album">

        <div class="flex items-center gap-5 w-1/5">
            <img class="aspect-square w-20 rounded-md duration-150 group-hover:opacity-90"
                :src="convertFileSrc(music.cover_path)" alt="Album Cover">
            <div class="flex flex-col gap-1 truncate">
                <RouterLink :to="{ name: 'album', params: { artist_id: music.artists_id, album_id: music.albums_id } }">
                    <p class="duration-150 font-main-nonbold text-text hover:text-placeholder cursor-pointer truncate">
                        {{
                            music.name
                        }}
                    </p>
                </RouterLink>
                <p class="duration-150 font-supporting text-supporting hover:opacity-85 cursor-pointer truncate">{{
                    music.artist
                }}</p>
            </div>
        </div>

        <div class="flex gap-2">
            <span :class="shuffled ? 'text-primary' : ''"
                class="cursor-pointer hover:opacity-90 duration-150 i-ph-shuffle-bold w-6"
                @click=handleShuffle()></span>
            <span class="cursor-pointer hover:opacity-90 duration-150 i-ph-skip-back-fill w-6"
                @click="skipTrack(false)"></span>
            <div @click="handlePlayAndPause">
                <span v-if="!paused" class="cursor-pointer hover:opacity-90 duration-150 i-ph-pause-fill w-7"></span>
                <span v-else class="cursor-pointer hover:opacity-90 duration-150 i-ph-play-fill w-7"></span>
            </div>
            <span class="cursor-pointer hover:opacity-90 duration-150 i-ph-skip-forward-fill w-6"
                @click="skipTrack(true)"></span>
            <span @click=handleLoop
                :class="loop === 'queue' ? 'text-primary' : '' || loop === 'track' ? 'text-primary opacity-75' : ''"
                class="cursor-pointer hover:opacity-90 duration-150 i-ph-repeat-bold w-6"></span>
        </div>

        <div class="flex gap-4 flex-grow items-center text-supporting font-supporting select-none">
            <audio @loadedmetadata="initialLoad()" @timeupdate="handleProgress()" ref="audioTag"
                :src="`http://localhost:16780${music.path}`" @ended="handleSongEnd"></audio>
            <label for="progress" class=w-10>{{ currentProgress }}</label>
            <input @input="selectProgress()" type="range" ref="progressBar" name="progress" min="0" value=0 max="100"
                class="w-full h-1.5 bg-stroke-100 rounded-lg accent-placeholder">
            <label for="progress" class=w-10>{{ totalLength }}</label>
        </div>

        <div class="flex gap-4 items-center">
            <span class="cursor-pointer hover:text-placeholder duration-150 i-mingcute-volume-fill w-18"></span>
            <input @input="handleVolume()" type="range" ref="volumeBar" min="0" max="1" value="0" step="0.01"
                class="h-1.5 w-full bg-stroke-100 rounded-lg accent-placeholder focus:ring-0">
        </div>

    </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'

const audioTag = ref<HTMLAudioElement | null>(null);
const progressBar = ref<HTMLInputElement | null>(null);
const volumeBar = ref<HTMLInputElement | null>(null);
const shuffled = ref(isShuffled());
const loop = ref(getLoop());

const paused = ref(true);
const totalLength = ref('3:33');
const currentProgress = ref('0:00');

const music = ref(getPlayerTrack());

function handleProgress() {
    const audio = unref(audioTag);
    if (!audio) return;
    if (progressBar) {
        const progress = audio.currentTime;
        progressBar.value!.value = progress.toString();
        currentProgress.value = makeReadableTime(progress);
        setPlayerProgress(progress);
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
    const volume = volumeBar.value.valueAsNumber;
    audioTag.value.volume = volume;
    setPlayerVolume(volume);
}

function handlePlayAndPause() {
    const audio = unref(audioTag);
    if (!audio) return;
    if (audio.paused) audio.play();
    else audio.pause();
    paused.value = audio.paused;
}

function handleShuffle() {
    shuffleQueue();
    shuffled.value = !shuffled.value;
}

function handleLoop() {
    const currentLoop = getLoop();
    if (currentLoop === 'queue') setLoop('track');
    else if (currentLoop === 'track') setLoop('none');
    else setLoop('queue');

    loop.value = getLoop();
}

function handleSongEnd() {
    const queue = getQueue();
    const index = getQueueIndex();
    const loop = getLoop();

    if (loop === 'track') {
        audioTag.value!.currentTime = 0;
        audioTag.value!.play();
        return;
    }

    if (queue.length <= 1 || queue.length === index + 1) {
        if (loop === 'queue') {
            setPlayerTrack(queue[0]);
            setQueueIndex(0);
            audioTag.value!.currentTime = 0;
            audioTag.value!.play();
        } else {
            audioTag.value!.pause();
            paused.value = true;
        }
    } else {
        skipTrack(true);
    }
}

async function initialLoad() {
    const progress = getPlayerProgress();
    const volume = getPlayerVolume();

    totalLength.value = makeReadableTime(audioTag.value!.duration);
    currentProgress.value = makeReadableTime(progress);

    progressBar.value!.value = progress.toString();
    progressBar.value!.max = audioTag.value!.duration.toString();

    volumeBar.value!.value = volume.toString();

    audioTag.value!.currentTime = progress;
    audioTag.value!.volume = volume;
}

window.addEventListener('playerTrackChanged', () => {
    music.value = getPlayerTrack();
    nextTick(() => handlePlayAndPause());
})

window.addEventListener('loopChanged', () => {
    loop.value = getLoop();
})
</script>