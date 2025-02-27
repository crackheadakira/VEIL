<template>
  <div
    class="aspect-player border-stroke-100 bg-card text-text grid h-28 w-screen grid-cols-[25%_50%_25%] items-center justify-items-center border-t p-4"
    v-if="playerStore.currentTrack"
  >
    <div class="flex w-full items-center gap-5">
      <img
        class="aspect-square w-20 rounded-md group-hover:opacity-90"
        :src="convertFileSrc(playerStore.currentTrack.cover_path)"
        alt="Album Cover"
      />
      <div class="flex flex-col gap-1 truncate">
        <RouterLink
          :to="{
            name: 'album',
            params: { id: playerStore.currentTrack.album_id },
          }"
        >
          <small
            class="text-text hover:text-placeholder cursor-pointer truncate"
          >
            {{ playerStore.currentTrack.name }}
          </small>
        </RouterLink>
        <RouterLink
          :to="{
            name: 'artist',
            params: { id: playerStore.currentTrack.artist_id },
          }"
        >
          <small
            class="text-supporting cursor-pointer truncate font-normal hover:opacity-85"
          >
            {{ playerStore.currentTrack.artist_name }}
          </small>
        </RouterLink>
      </div>
    </div>

    <div class="flex w-full flex-col items-center gap-4 px-6">
      <PlayerControls :extra="true" />

      <div
        class="text-supporting flex w-full items-center gap-4 text-center select-none"
      >
        <label class="w-10">{{ currentProgress }}</label>
        <RangeInput
          class="w-full"
          @mousedown="beingHeld = true"
          @mouseup="selectProgress()"
          v-model="progress"
          :max="playerStore.currentTrack.duration"
        />
        <!--<input
          @mousedown="beingHeld = true"
          @mouseup="selectProgress()"
          v-model="progress"
          type="range"
          ref="progressBar"
          name="progress"
          min="0"
          :max="playerStore.currentTrack.duration"
          class="bg-stroke-100 accent-placeholder h-1.5 w-full rounded-lg"
        />-->
        <label class="w-10">{{ totalLength }}</label>
      </div>
    </div>

    <div class="flex items-center gap-4 justify-self-end">
      <span
        class="i-fluent-speaker-24-filled hover:text-placeholder cursor-pointer"
      ></span>
      <RangeInput
        @update:model-value="playerStore.handleVolume"
        v-model="playerStore.playerVolume"
        :max="1"
        :step="0.01"
      />
      <!--<input
        @update:model-value="playerStore.handleVolume"
        v-model="playerStore.playerVolume"
        type="range"
        min="0"
        max="1"
        step="0.01"
        class="bg-stroke-100 accent-placeholder h-1.5 w-full max-w-36 rounded-lg focus:ring-0"
      />-->
    </div>
  </div>
</template>

<script setup lang="ts">
import { commands, formatTime, usePlayerStore } from "@/composables/";
import { PlayerControls, RangeInput } from "@/components/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { RouterLink } from "vue-router";

const playerStore = usePlayerStore();

const beingHeld = ref(false);

const totalLength = computed(() =>
  formatTime("mm:ss", playerStore.currentTrack?.duration || 0),
);

const currentProgress = computed(() =>
  formatTime("mm:ss", playerStore.playerProgress),
);

const progress = ref(playerStore.playerProgress);

watch(
  () => playerStore.playerProgress,
  (newProgress) => {
    if (!beingHeld.value) progress.value = newProgress;
  },
);

/**
 * If progress bar is being held, update the progress bar to the selected progress.
 *
 * Gets progress from `$progressBar`, calls {@linkcode commands.playerHasTrack}, {@linkcode commands.getPlayerState}, and {@linkcode commands.seekTrack}.
 *
 * If the player is playing it continues playing from the selected progress. Otherwise it just seeks to the selected progress.
 */
async function selectProgress() {
  if (!(await commands.playerHasTrack())) return;
  const skipTo = (await commands.getPlayerState()) === "Playing";
  await commands.seekTrack(parseFloat(progress.value.toString()), skipTo);
  beingHeld.value = false;

  playerStore.handleProgress(false, progress.value);
}

onMounted(async () => {
  await playerStore.initialLoad();
});

onUnmounted(async () => {
  await commands.stopPlayer();

  (await playerStore.listenPlayerProgress)();
  (await playerStore.listenTrackEnd)();
  (await playerStore.listenMediaControl)();
});
</script>
