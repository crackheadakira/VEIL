<template>
    <div class="grid aspect-player w-screen grid-cols-[25%_50%_25%] items-center justify-items-center border-t border-stroke-100 bg-card p-4 text-text"
        v-if="music">

        <div class="flex w-full items-center gap-5">
            <img class="aspect-square w-20 rounded-md group-hover:opacity-90" :src="convertFileSrc(music.cover_path)"
                alt="Album Cover">
            <div class="flex flex-col gap-1 truncate">
                <RouterLink :to="{ name: 'album', params: { artist_id: music.artists_id, album_id: music.albums_id } }">
                    <p class="font-supporting cursor-pointer truncate text-text hover:text-placeholder">
                        {{ music.name }}
                    </p>
                </RouterLink>
                <p class="font-supporting cursor-pointer truncate font-normal text-supporting hover:opacity-85">
                    {{ music.artist }}
                </p>
            </div>
        </div>

        <div class="flex w-full flex-col gap-4 px-8">
            <div class="flex w-full items-center justify-center gap-4">
                <span :class="shuffled ? 'text-primary' : ''"
                    class="i-fluent-arrow-shuffle-20-filled cursor-pointer hover:opacity-90"
                    @click=handleShuffle()></span>
                <span class="i-fluent-previous-20-filled w-6 cursor-pointer hover:opacity-90"
                    @click="skipTrack(false)"></span>
                <span @click="handlePlayAndPause"
                    :class="!paused ? 'i-fluent-pause-24-filled' : 'i-fluent-play-24-filled'"
                    class="i-fluent-pause-20-filled cursor-pointer hover:opacity-90"></span>
                <span class="i-fluent-next-20-filled cursor-pointer hover:opacity-90" @click="skipTrack(true)"></span>
                <span @click=handleLoop
                    :class="(loop === 'queue' ? 'text-primary' : '') || (loop === 'track' ? 'text-primary opacity-75' : '')"
                    class="i-fluent-arrow-repeat-all-20-filled cursor-pointer hover:opacity-90"></span>
            </div>

            <div class="font-supporting flex select-none items-center gap-4 text-supporting">
                <label for="progress" class=w-10>{{ currentProgress }}</label>
                <input @mousedown="beingHeld = true" @mouseup="selectProgress()" type="range" ref="progressBar"
                    name="progress" min="0" value=0 max="100"
                    class="h-1.5 w-full rounded-lg bg-stroke-100 accent-placeholder">
                <label for="progress" class=w-10>{{ totalLength }}</label>
            </div>
        </div>

        <div class="flex items-center gap-4 justify-self-end">
            <span class="i-fluent-speaker-24-filled cursor-pointer hover:text-placeholder"></span>
            <input @input="handleVolume()" type="range" ref="volumeBar" min="-30" max="1.2" value="1" step="0.5"
                class="h-1.5 w-full max-w-36 rounded-lg bg-stroke-100 accent-placeholder focus:ring-0">
        </div>

    </div>
</template>

<script setup lang="ts">
import { Event, listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import { commands, MediaPayload } from '../bindings';

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
    const hasTrack = await commands.playerHasTrack();

    if (!hasTrack && music.value) {
        await commands.playTrack(music.value.id);
        paused.value = false;
        return;
    } else if (!hasTrack) {
        paused.value = true;
        return;
    }

    if (paused.value === true) {
        await commands.resumeTrack()
    } else {
        await commands.pauseTrack();
    }

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
    if (!music.value) return;
    while (!(await commands.playerHasEnded())) {
        await new Promise((resolve) => setTimeout(resolve, 10));
    }

    const loop = getLoop();

    if (loop === 'track') {
        // replay the same track
        await setPlayerTrack(music.value);
        await handlePlayAndPause();
        return;
    }

    const queue = getQueue();
    const index = getQueueIndex();

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

listen('media-control', async (e: Event<MediaPayload>) => {
    const payload = e.payload;
    console.log(payload);

    switch (true) {
        case 'Play' in payload:
            await handlePlayAndPause();
            break;
        case 'Pause' in payload:
            await handlePlayAndPause();
            break;
        case 'Next' in payload:
            skipTrack(true);
            break;
        case 'Previous' in payload:
            skipTrack(false);
            break;
        case 'Seek' in payload:
            console.log(payload.Seek);
            await commands.seekTrack(payload.Seek, true);
            break;
        case 'Volume' in payload:
            // currently 0.0 to 1.0, but needs to be converted -30 to 1.2
            const convertedVolume = payload.Volume * 31.2 - 30;
            console.log(convertedVolume);
            await commands.setVolume(convertedVolume);
            break;
        case 'Position' in payload:
            await commands.seekTrack(payload.Position, false);
            break;
    }
})

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
</script>