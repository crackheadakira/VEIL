<template>
  <div
    class="aspect-player border-stroke-100 bg-card text-text grid h-28 w-screen grid-cols-[25%_50%_25%] items-center justify-items-center border-t p-4"
    v-if="data"
  >
    <div class="flex w-full items-center gap-5">
      <img
        class="aspect-square w-20 rounded-md group-hover:opacity-90"
        :src="convertFileSrc(data.cover_path)"
        alt="Album Cover"
      />
      <div class="flex flex-col gap-1 truncate">
        <RouterLink
          :to="{
            name: 'album',
            params: { artist_id: data.artists_id, album_id: data.albums_id },
          }"
        >
          <small
            class="text-text hover:text-placeholder cursor-pointer truncate"
          >
            {{ data.name }}
          </small>
        </RouterLink>
        <RouterLink
          :to="{ name: 'artist', params: { artist_id: data.artists_id } }"
        >
          <small
            class="text-supporting cursor-pointer truncate font-normal hover:opacity-85"
          >
            {{ data.artist }}
          </small>
        </RouterLink>
      </div>
    </div>

    <div class="flex w-full flex-col gap-4 px-8">
      <div class="flex w-full items-center justify-center gap-4">
        <span
          :class="shuffled ? 'text-primary' : ''"
          class="i-fluent-arrow-shuffle-20-filled cursor-pointer hover:opacity-90"
          @click="playerStore.shuffleQueue()"
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
          @click="playerStore.loopQueue()"
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

const data = ref(playerStore.currentTrack);

/**
 * If player has a track, update the progress bar. (Called every 100ms and when the user drags the progress bar)
 *
 * If the progress bar is being held, do not update the progress bar.
 *
 * Updates `$playerStore.playerProgress` with the current progress.
 *
 * Updates `$progressBar` with the current progress.
 */
async function handleProgress() {
  if (!(await commands.playerHasTrack())) return;
  if (progressBar.value) {
    const progress = await commands.getPlayerProgress();

    if (beingHeld.value) return;
    progressBar.value!.value = progress.toString();
    playerStore.playerProgress = progress;
  }
}

/**
 * If progress bar is being held, update the progress bar to the selected progress.
 *
 * Gets progress from `$progressBar`, calls {@linkcode commands.playerHasTrack}, {@linkcode commands.getPlayerState}, and {@linkcode commands.seekTrack}.
 *
 * If the player is playing it continues playing from the selected progress. Otherwise it just seeks to the selected progress.
 */
async function selectProgress() {
  if (!(await commands.playerHasTrack())) return;
  const progress = progressBar.value!.valueAsNumber;
  const skipTo = (await commands.getPlayerState()) === "Playing";
  await commands.seekTrack(progress, skipTo);
  beingHeld.value = false;

  handleProgress();
}

/**
 * Updates the volume of the player.
 *
 * Gets volume from `$volumeBar`, updates `$playerStore.playerVolume`, and calls {@linkcode commands.setVolume}.
 *
 * If the volume goes below `-30` it gets clamped to `-60`.
 */
async function handleVolume() {
  if (!volumeBar.value) return;
  let volume = volumeBar.value.valueAsNumber;
  if (volume <= -30) volume = -60;
  await commands.setVolume(volume);
  playerStore.playerVolume = volume;
}

/**
 * Handles the play and pause button.
 *
 * If player is paused and has a track, resume the track.
 *
 * If player is paused and does not have a track, play the current track.
 *
 * If player is playing, pause the track.
 *
 * Updates `$paused` with the current state, and calls {@linkcode commands.playTrack}.
 *
 * @example
 * ```ts
 * // Track is currently paused
 * await handlePlayAndPause(); // Track is now playing
 * await handlePlayAndPause(); // Track is now paused
 */
async function handlePlayAndPause() {
  const hasTrack = await commands.playerHasTrack();

  if (!hasTrack && data.value) {
    const result = await commands.playTrack(data.value.id);
    if (result.status === "error") return handleBackendError(result.error);

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

/**
 * Handles the end of the song.
 *
 * If the player is in track loop, replay the same track.
 *
 * If the player is in queue loop, replay the queue.
 *
 * If the player is not in loop, pause the player.
 */
async function handleSongEnd() {
  if (!data.value) return;
  while (!(await commands.playerHasEnded())) {
    await new Promise((resolve) => setTimeout(resolve, 10));
  }

  if (playerStore.loop === "track") {
    // replay the same track
    await playerStore.setPlayerTrack(data.value);
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

/**
 * Initializes required values for the player.
 *
 * Pauses the player, gets the current progress, volume, and duration of the player.
 *
 * Updates `$progressBar` with the current progress.
 *
 * Updates `$volumeBar` with the current volume.
 *
 * If the duration is not 0, seeks the player to the current progress.
 */
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

playerStore.$onAction(({ name, store, after }) => {
  if (name === "setPlayerTrack") {
    paused.value = true;

    after(async () => {
      data.value = store.currentTrack;
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
      data.value = store.currentTrack;
      paused.value = false;
    });
  } else if (name === "shuffleQueue") {
    after(() => {
      shuffled.value = store.isShuffled;
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
