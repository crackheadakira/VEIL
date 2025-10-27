<template>
  <div
    class="aspect-player border-border-secondary bg-bg-secondary text-text-primary z-20 grid h-28 w-screen grid-cols-[25%_50%_25%] items-center justify-items-center border-t p-4"
    v-if="playerStore.currentTrack"
  >
    <div class="flex w-full items-center gap-5">
      <img
        class="aspect-square w-20 rounded-md group-hover:opacity-90"
        :src="convertFileSrc(playerStore.currentTrack.cover_path)"
        alt="Album Cover"
      />
      <div class="flex flex-col gap-1 truncate">
        <ContextMenu
          :track="playerStore.currentTrack"
          :playlists="playlistStore.playlists"
          @queue="handleAddToQueue"
          @playlist="handlePlaylist"
        >
          <RouterLink
            class="max-w-fit"
            :to="{
              name: 'album',
              params: { id: playerStore.currentTrack.album_id },
            }"
          >
            <p
              class="text-text-primary hover:text-text-primary-hovered cursor-pointer truncate"
            >
              {{ playerStore.currentTrack.name }}
            </p>
          </RouterLink>
        </ContextMenu>
        <RouterLink
          class="max-w-fit"
          :to="{
            name: 'artist',
            params: { id: playerStore.currentTrack.artist_id },
          }"
        >
          <p
            class="text-text-secondary hover:text-text-secondary-hovered cursor-pointer truncate font-normal"
          >
            {{ playerStore.currentTrack.artist_name }}
          </p>
        </RouterLink>
      </div>
    </div>

    <div class="flex w-full flex-col items-center gap-4 px-6">
      <PlayerControls :extra="true" />

      <div
        class="text-text-secondary flex w-full items-center gap-4 text-center"
      >
        <label class="w-10 text-sm">{{ currentProgress }}</label>
        <Slider
          class="w-full"
          @pointerdown="beingHeld = true"
          @pointerup="selectProgress"
          v-model="progress"
          :max="playerStore.currentTrack.duration"
          :step="0.1"
        />
        <label class="w-10 text-sm">{{ totalLength }}</label>
      </div>
    </div>

    <VolumeControls class="pr-4" />
  </div>
</template>

<script setup lang="ts">
import {
  commands,
  events,
  formatTime,
  Tracks,
  usePlayerStore,
  usePlaylistStore,
  useQueueStore,
} from "@/composables/";
import {
  VolumeControls,
  PlayerControls,
  Slider,
  ContextMenu,
} from "@/components/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { RouterLink } from "vue-router";

const queueStore = useQueueStore();
const playlistStore = usePlaylistStore();
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
  // TODO: bake this logic into playerStore w/ listeners.
  if (!(await commands.playerHasTrack())) return;
  const skipTo = (await commands.getPlayerState()) === "Playing";

  await events.playerEvent.emit({
    type: "Seek",
    data: { position: progress.value, resume: skipTo },
  });

  beingHeld.value = false;

  playerStore.handleProgress(false, progress.value);
}

async function handlePlaylist(
  type: "add" | "remove",
  playlistId: number,
  trackId: number,
) {
  if (type === "add") await playlistStore.addToPlaylist(playlistId, trackId);
  else await playlistStore.removeFromPlaylist(playlistId, trackId);
}

async function handleAddToQueue(track: Tracks) {
  queueStore.personalQueue.push(track);
}

onMounted(async () => {
  await playerStore.initialLoad();
});

onUnmounted(async () => {
  await events.playerEvent.emit({ type: "Stop" });

  (await playerStore.listenMediaControl)();
});
</script>
