<template>
    <div class="flex aspect-player w-screen items-center justify-center gap-8 border-t border-stroke-100 bg-card p-3 text-text"
        v-if="music.path !== ''">

        <div class="flex w-1/5 items-center gap-5">
            <img class="aspect-square w-20 rounded-md duration-150 group-hover:opacity-90"
                :src="convertFileSrc(music.cover_path)" alt="Album Cover">
            <div class="flex flex-col gap-1 truncate">
                <RouterLink :to="{ name: 'album', params: { artist_id: music.artists_id, album_id: music.albums_id } }">
                    <p class="font-main-nonbold cursor-pointer truncate text-text duration-150 hover:text-placeholder">
                        {{
                            music.name
                        }}
                    </p>
                </RouterLink>
                <p class="font-supporting cursor-pointer truncate text-supporting duration-150 hover:opacity-85">{{
                    music.artist
                }}</p>
            </div>
        </div>

        <div class="flex gap-2">
            <span :class="shuffled ? 'text-primary' : ''"
                class="i-ph-shuffle-bold w-6 cursor-pointer duration-150 hover:opacity-90"
                @click=handleShuffle()></span>
            <span class="i-ph-skip-back-fill w-6 cursor-pointer duration-150 hover:opacity-90"
                @click="skipTrack(false)"></span>
            <div @click="handlePlayAndPause">
                <span v-if="!paused" class="i-ph-pause-fill w-7 cursor-pointer duration-150 hover:opacity-90"></span>
                <span v-else class="i-ph-play-fill w-7 cursor-pointer duration-150 hover:opacity-90"></span>
            </div>
            <span class="i-ph-skip-forward-fill w-6 cursor-pointer duration-150 hover:opacity-90"
                @click="skipTrack(true)"></span>
            <span @click=handleLoop
                :class="(loop === 'queue' ? 'text-primary' : '') || (loop === 'track' ? 'text-primary opacity-75' : '')"
                class="i-ph-repeat-bold w-6 cursor-pointer duration-150 hover:opacity-90"></span>
        </div>

        <div class="font-supporting flex flex-grow select-none items-center gap-4 text-supporting">
            <label for="progress" class=w-10>{{ currentProgress }}</label>
            <input @mousedown="beingHeld = true" @mouseup="selectProgress()" type="range" ref="progressBar"
                name="progress" min="0" value=0 max="100"
                class="h-1.5 w-full rounded-lg bg-stroke-100 accent-placeholder">
            <label for="progress" class=w-10>{{ totalLength }}</label>
        </div>

        <div class="flex items-center gap-4">
            <span class="w-18 i-mingcute-volume-fill cursor-pointer duration-150 hover:text-placeholder"></span>
            <input @input="handleVolume()" type="range" ref="volumeBar" min="-30" max="10" value="1" step="0.5"
                class="h-1.5 w-full rounded-lg bg-stroke-100 accent-placeholder focus:ring-0">
        </div>

    </div>
</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import { commands } from '../bindings';

const progressBar = useTemplateRef<HTMLInputElement>("progressBar");
const volumeBar = useTemplateRef<HTMLInputElement>("volumeBar");
const shuffled = ref(isShuffled());
const loop = ref(getLoop());

const paused = ref(true);
const totalLength = ref('3:33');
const currentProgress = ref('0:00');
const beingHeld = ref(false);

const music = ref(getPlayerTrack());

async function handleProgress() {
    if (!await commands.playerHasTrack()) return;
    if (progressBar) {
        const progress = await commands.getPlayerProgress();
        currentProgress.value = makeReadableTime(progress);

        if (beingHeld.value) return;
        progressBar.value!.value = progress.toString();
        setPlayerProgress(progress);
    }
}

async function selectProgress() {
    if (!await commands.playerHasTrack()) return;
    const progress = progressBar.value!.valueAsNumber;
    const skipTo = await commands.getPlayerState() === "Playing";
    await commands.seekTrack(progress, skipTo);
    beingHeld.value = false;

    handleProgress();
}

async function handleVolume() {
    if (!volumeBar.value) return;
    let volume = volumeBar.value.valueAsNumber;
    if (volume <= -30) volume = -60;
    await commands.setVolume(volume);
    setPlayerVolume(volume);
}

async function handlePlayAndPause() {
    if (!await commands.playerHasTrack()) {
        await commands.playTrack(music.value.id);
        paused.value = false;
        return;
    };

    if (paused.value === true) await commands.resumeTrack();
    else await commands.pauseTrack();

    paused.value = !paused.value;
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

async function handleSongEnd() {
    // to let the song fully end as we emit slightly before the song ends
    while (!(await commands.playerHasEnded())) {
        await new Promise((resolve) => setTimeout(resolve, 100));
    }
    const queue = getQueue();
    const index = getQueueIndex();
    const loop = getLoop();

    if (loop === 'track') {
        // replay the same track
        await setPlayerTrack(music.value);
        await handlePlayAndPause();
        return;
    }

    if (queue.length <= 1 || queue.length === index + 1) {
        if (loop === 'queue') {
            setQueueIndex(0);
            await setPlayerTrack(queue[0]);
        } else {
            paused.value = true;
        }
    } else {
        skipTrack(true);
    }
}

async function initialLoad() {
    await commands.pauseTrack();
    const progress = getPlayerProgress();
    const volume = getPlayerVolume();
    const duration = await commands.getPlayerDuration();

    totalLength.value = makeReadableTime(duration);
    currentProgress.value = makeReadableTime(progress);

    nextTick(() => {
        progressBar.value!.value = progress.toString();
        progressBar.value!.max = duration.toString();
        volumeBar.value!.value = volume.toString();
    });

    if (duration !== 0) await commands.seekTrack(progress, false);
    await commands.setVolume(volume);
}

window.addEventListener('playerTrackChanged', async () => {
    music.value = getPlayerTrack();
    paused.value = true;

    const duration = await commands.getPlayerDuration();
    totalLength.value = makeReadableTime(duration);
    progressBar.value!.max = duration.toString();

    await handlePlayAndPause();
})

window.addEventListener('loopChanged', () => {
    loop.value = getLoop();
})

onMounted(async () => {
    await initialLoad();
})

onUnmounted(async () => {
    await commands.stopPlayer();
})

listen('player-progress', async (_) => {
    await handleProgress();
})

listen('track-end', async (_) => {
    await handleSongEnd();
})
</script>