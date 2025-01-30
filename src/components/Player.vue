<template>
  <div
    class="aspect-player border-stroke-100 bg-card text-text grid w-screen grid-cols-[25%_50%_25%] items-center justify-items-center border-t p-4"
    v-if="music"
  >
    <div class="flex w-full items-center gap-5">
      <img
        class="aspect-square w-20 rounded-md group-hover:opacity-90"
        :src="convertFileSrc(music.cover_path)"
        alt="Album Cover"
      />
      <div class="flex flex-col gap-1 truncate">
        <RouterLink
          :to="{
            name: 'album',
            params: { artist_id: music.artists_id, album_id: music.albums_id },
          }"
        >
          <small
            class="text-text hover:text-placeholder cursor-pointer truncate"
          >
            {{ music.name }}
          </small>
        </RouterLink>
        <RouterLink
          :to="{ name: 'artist', params: { artist_id: music.artists_id } }"
        >
          <small
            class="text-supporting cursor-pointer truncate font-normal hover:opacity-85"
          >
            {{ music.artist }}
          </small>
        </RouterLink>
      </div>
    </div>

    <div class="flex w-full flex-col gap-4 px-8">
      <div class="flex w-full items-center justify-center gap-4">
        <span
          :class="shuffled ? 'text-primary' : ''"
          class="i-fluent-arrow-shuffle-20-filled cursor-pointer hover:opacity-90"
          @click="handleShuffle()"
        ></span>
        <span
          class="i-fluent-previous-20-filled w-6 cursor-pointer hover:opacity-90"
          @click="playerStore.skipTrack(false)"
        ></span>
        <span
          @click="handlePlayAndPause"
          :class="
            !paused ? 'i-fluent-pause-24-filled' : 'i-fluent-play-24-filled'
          "
          class="i-fluent-pause-20-filled cursor-pointer hover:opacity-90"
        ></span>
        <span
          class="i-fluent-next-20-filled cursor-pointer hover:opacity-90"
          @click="playerStore.skipTrack(true)"
        ></span>
        <span
          @click="handleLoop"
          :class="
            (loop === 'queue' ? 'text-primary' : '') ||
            (loop === 'track' ? 'text-primary opacity-75' : '')
          "
          class="i-fluent-arrow-repeat-all-20-filled cursor-pointer hover:opacity-90"
        ></span>
      </div>

      <div class="text-supporting flex items-center gap-4 select-none">
        <label for="progress" class="w-10">{{ currentProgress }}</label>
        <input
          @mousedown="beingHeld = true"
          @mouseup="selectProgress()"
          type="range"
          ref="progressBar"
          name="progress"
          min="0"
          value="0"
          max="100"
          class="bg-stroke-100 accent-placeholder h-1.5 w-full rounded-lg"
        />
        <label for="progress" class="w-10">{{ totalLength }}</label>
      </div>
    </div>

    <div class="flex items-center gap-4 justify-self-end">
      <span
        class="i-fluent-speaker-24-filled hover:text-placeholder cursor-pointer"
      ></span>
      <input
        @input="handleVolume()"
        type="range"
        ref="volumeBar"
        min="-30"
        max="1.2"
        value="1"
        step="0.1"
        class="bg-stroke-100 accent-placeholder h-1.5 w-full max-w-36 rounded-lg focus:ring-0"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Event, listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { commands, MediaPayload } from "../bindings";
import { usePlayerStore } from "../composables/playerStore";
import { RouterLink } from "vue-router";

const playerStore = usePlayerStore();

const progressBar = useTemplateRef<HTMLInputElement>("progressBar");
const volumeBar = useTemplateRef<HTMLInputElement>("volumeBar");
const shuffled = ref(playerStore.isShuffled);
const loop = ref(playerStore.loop);

const paused = ref(true);
const beingHeld = ref(false);
const totalLength = computed(() =>
  makeReadableTime(playerStore.currentTrack?.duration || 0),
);
const currentProgress = computed(() =>
  makeReadableTime(playerStore.playerProgress),
);

const music = ref(playerStore.currentTrack);

async function handleProgress() {
  if (!(await commands.playerHasTrack())) return;
  if (progressBar.value) {
    const progress = await commands.getPlayerProgress();

    if (beingHeld.value) return;
    progressBar.value!.value = progress.toString();
    playerStore.playerProgress = progress;
  }
}

async function selectProgress() {
  if (!(await commands.playerHasTrack())) return;
  const progress = progressBar.value!.valueAsNumber;
  const skipTo = (await commands.getPlayerState()) === "Playing";
  await commands.seekTrack(progress, skipTo);
  beingHeld.value = false;

  handleProgress();
}

async function handleVolume() {
  if (!volumeBar.value) return;
  let volume = volumeBar.value.valueAsNumber;
  if (volume <= -30) volume = -60;
  await commands.setVolume(volume);
  playerStore.playerVolume = volume;
}

async function handlePlayAndPause() {
  const hasTrack = await commands.playerHasTrack();

  if (!hasTrack && music.value) {
    const result = await commands.playTrack(music.value.id);
    if (result.status === "error")
      throw new Error(`[${result.error.type}] ${result.error.data}`);
    paused.value = false;
    return;
  } else if (!hasTrack) {
    paused.value = true;
    return;
  }

  if (paused.value === true) {
    await commands.resumeTrack();
  } else {
    await commands.pauseTrack();
  }

  paused.value = !paused.value;
}

function handleShuffle() {
  playerStore.isShuffled = !shuffled.value;
  shuffled.value = !shuffled.value;
}

function handleLoop() {
  playerStore.loopQueue();
}

async function handleSongEnd() {
  if (!music.value) return;
  while (!(await commands.playerHasEnded())) {
    await new Promise((resolve) => setTimeout(resolve, 10));
  }

  if (playerStore.loop === "track") {
    // replay the same track
    await playerStore.setPlayerTrack(music.value);
    await handlePlayAndPause();
    return;
  }

  const queue = playerStore.queue;
  const index = playerStore.queueIndex;

  if (queue.length <= 1 || queue.length === index + 1) {
    if (playerStore.loop === "queue") {
      playerStore.queueIndex = 0;
      await playerStore.setPlayerTrack(queue[0]);
    } else {
      paused.value = true;
    }
  } else {
    playerStore.skipTrack(true);
  }
}

async function initialLoad() {
  await commands.pauseTrack();
  const progress = playerStore.playerProgress;
  const volume = playerStore.playerVolume;
  const duration = await commands.getPlayerDuration();

  if (progressBar.value) {
    progressBar.value!.value = progress.toString();
    progressBar.value!.max = duration.toString();
  }
  if (volumeBar.value) volumeBar.value!.value = volume.toString();

  if (duration !== 0) await commands.seekTrack(progress, false);
  await commands.setVolume(volume);
}

const listenPlayerProgress = listen("player-progress", async (_) => {
  await handleProgress();
});

const listenTrackEnd = listen("track-end", async (_) => {
  await handleSongEnd();
});

const listenMediaControl = listen(
  "media-control",
  async (e: Event<MediaPayload>) => {
    const payload = e.payload;

    switch (true) {
      case "Play" in payload:
        await handlePlayAndPause();
        break;
      case "Pause" in payload:
        await handlePlayAndPause();
        break;
      case "Next" in payload:
        playerStore.skipTrack(true);
        break;
      case "Previous" in payload:
        playerStore.skipTrack(false);
        break;
      case "Seek" in payload:
        await commands.seekTrack(payload.Seek, true);
        break;
      case "Volume" in payload:
        // currently 0.0 to 1.0, but needs to be converted -30 to 1.2
        const convertedVolume = payload.Volume * 31.2 - 30;
        await commands.setVolume(convertedVolume);
        break;
      case "Position" in payload:
        await commands.seekTrack(payload.Position, false);
        break;
    }
  },
);

playerStore.$onAction(({ name, store, args, after }) => {
  if (name === "setPlayerTrack") {
    music.value = args[0];
    paused.value = true;

    after(async () => {
      const duration = await commands.getPlayerDuration();
      progressBar.value!.max = duration.toString();

      await handlePlayAndPause();
    });
  } else if (name === "loopQueue") {
    after(() => {
      loop.value = store.loop;
    });
  } else if (name === "skipTrack") {
    after(() => {
      music.value = store.currentTrack;
      paused.value = false;
    });
  }
});

onMounted(async () => {
  await initialLoad();
});

onUnmounted(async () => {
  await commands.stopPlayer();

  (await listenPlayerProgress)();
  (await listenTrackEnd)();
  (await listenMediaControl)();
});
</script>
