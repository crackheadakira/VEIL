import { commands, handleBackendError, MediaPayload, type Tracks } from "@/composables/";
import { listen, Event } from "@tauri-apps/api/event";
import { StorageSerializers, useStorage } from "@vueuse/core";
import { defineStore } from "pinia";
import { nextTick, ref, watch } from "vue";

/**
 * The player store composable.
 *
 * Manages the player state, including the current track, queue, player progress, and loop settings.
 *
 * @example
 * const playerStore = usePlayerStore();
 *
 * // Set the current track
 * playerStore.setPlayerTrack(track);
 *
 * // Set the player progress
 * playerStore.setPlayerProgress(50);
 */
export const usePlayerStore = defineStore("player", () => {
  const currentTrack = useStorage<Tracks>("currentTrack", null, undefined, {
    serializer: StorageSerializers.object,
  });
  const queue = useStorage<Tracks[]>("queue", []);
  const queueIndex = useStorage("queueIndex", 0);
  const personalQueue = useStorage<Tracks[]>("personalQueue", []);
  const loop = useStorage<"none" | "track" | "queue">("loop", "none");
  const playerProgress = useStorage("playerProgress", 0);
  const playerVolume = useStorage("playerVolume", 0.5);
  const isShuffled = useStorage("isShuffled", false);

  const paused = ref(true);

  // Channel is for syncing pause state between main window & widget
  const channel = new BroadcastChannel("player_channel");

  watch(paused, (newValue) => {
    channel.postMessage({ paused: newValue });
  });

  channel.onmessage = (event) => {
    paused.value = event.data.paused;
  };

  /**
   * Sets given track as the current track and plays it.
   *
   * Calls Rust backend (setPlayerProgress, playTrack), and updates store `currentTrack` & `playerProgress`.
   *
   * @param {Tracks} track - The track to play
   */
  async function setPlayerTrack(track: Tracks): Promise<void> {
    await commands.setPlayerProgress(0);

    currentTrack.value = track;
    playerProgress.value = 0;

    await commands.playTrack(track.id);
    paused.value = false;
  }

  /**
   * Sets the player progress to the given value.
   *
   * Calls Rust backend, and updates store `playerProgress` value.
   * @param {number} progress - The progress to set
   */
  async function setPlayerProgress(progress: number): Promise<void> {
    await commands.setPlayerProgress(progress);
    playerProgress.value = progress;
  }

  /**
   * Skips the current track and plays the next/previous track in the queue.
   *
   * Calls {@linkcode setPlayerTrack} to play the track.
   *
   * If `forward` is true, plays the next track in the queue.
   *
   * If `forward` is false, plays the previous track in the queue.
   *
   * Reads the `loop` value to determine the behavior if the track is the last/first in the queue. If `loop` is set to "track", loop is set to "queue".
   *
   * @example
   * // playerStore.queue = [track1, track2, track3]
   * // playerStore.queueIndex = 1 // track2
   * // playerStore.loop = "none"
   * await playerStore.skipTrack(true) // Plays track3
   *
   * @example
   * // playerStore.queue = [track1, track2, track3]
   * // playerStore.queueIndex = 2 // track3
   * // playerStore.loop = "none"
   * await playerStore.skipTrack(true) // Plays track1
   */
  async function skipTrack(forward: boolean): Promise<void> {
    if (loop.value === "track") loop.value = "queue";

    if (personalQueue.value.length > 0) {
      await setPlayerTrack(personalQueue.value[0]);
      personalQueue.value.shift();
      return;
    }

    let desiredIndex;
    if (forward) {
      desiredIndex = queueIndex.value + 1;
    } else {
      desiredIndex = queueIndex.value - 1;
    }

    if (desiredIndex >= queue.value.length) desiredIndex = 0;
    if (desiredIndex < 0) desiredIndex = queue.value.length - 1;

    queueIndex.value = desiredIndex;
    await setPlayerTrack(queue.value[desiredIndex]);
  }

  /**
   * Toggles the loop value between "none", "track", and "queue".
   * @example
   * // playerStore.loop = "none"
   * playerStore.loopQueue() // playerStore.loop = "queue"
   * playerStore.loopQueue() // playerStore.loop = "track"
   * playerStore.loopQueue() // playerStore.loop = "none"
   */
  function loopQueue(): void {
    if (loop.value === "none") loop.value = "queue";
    else if (loop.value === "queue") loop.value = "track";
    else loop.value = "none";
  }

  /**
   * Shuffles the queue using the Fisher-Yates algorithm if `isShuffled` is false, otherwise sorts the queue by track id.
   *
   * Updates the store `queue` and `isShuffled` value.
   *
   * @example
   * // playerStore.queue = [track1, track2, track3]
   * // playerStore.isShuffled = false
   * playerStore.shuffleQueue() // playerStore.queue = [track3, track1, track2]
   * playerStore.shuffleQueue() // playerStore.queue = [track1, track2, track3]
   */
  function shuffleQueue() {
    // If already shuffled, sort the queue by id
    if (isShuffled.value) {
      queue.value.sort((a, b) => a.id - b.id);
      isShuffled.value = false;

      return;
    }

    const trackIndex = queue.value.findIndex(
      (track) => track.id === currentTrack.value?.id,
    );

    if (trackIndex === -1) return;

    queue.value.splice(trackIndex, 1);
    const shuffledQueue = fisherYatesShuffle(queue.value);
    shuffledQueue.unshift(currentTrack.value);

    queue.value = shuffledQueue;
    isShuffled.value = true;
  }

  function $reset() {
    currentTrack.value = null;
    queue.value = [];
    queueIndex.value = 0;
    personalQueue.value = [];
    loop.value = "none";
    playerProgress.value = 0;
    playerVolume.value = 0.5;
    isShuffled.value = false;
  }

  // PLAYER LOGIC

  /**
   * If player has a track, update the progress.
   *
   * If the progress bar is being held, do not update the progress bar.
   *
   * Updates `$playerProgress` with the current progress.
   */
  async function handleProgress(held: boolean, p?: number) {
    const progress = p ? p : await commands.getPlayerProgress();

    if (held) return;
    playerProgress.value = progress;
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

    if (!hasTrack && currentTrack.value) {
      paused.value = false;
      const result = await commands.playTrack(currentTrack.value.id);
      if (result.status === "error") return handleBackendError(result.error);

      return;
    } else if (!hasTrack) {
      paused.value = true;
      return;
    }

    if (paused.value === true) {
      await commands.resumeTrack();
    } else {
      const result = await commands.pauseTrack();
      if (result.status === "error") return handleBackendError(result.error);
    }

    paused.value = !paused.value;
  };

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
    if (!currentTrack.value) return;
    while (!(await commands.playerHasEnded())) {
      await new Promise((resolve) => setTimeout(resolve, 10));
    }

    if (loop.value === "track") {
      // replay the same track
      await setPlayerTrack(currentTrack.value);
      await handlePlayAndPause();
      return;
    }


    if (queue.value.length <= 1 || queue.value.length === queueIndex.value + 1) {
      if (loop.value === "queue") {
        queueIndex.value = 0;
        await setPlayerTrack(queue.value[0]);
      } else {
        await handlePlayAndPause();
      }
    } else {
      skipTrack(true);
    }
  }

  /**
   * Updates the volume of the player.
   */
  async function handleVolume() {
    nextTick(async () => {
      await commands.setVolume(+playerVolume.value);
    });
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
    const duration = await commands.getPlayerDuration();

    if (duration !== 0) await commands.seekTrack(playerProgress.value, false);
    await commands.setVolume(+playerVolume.value);
  }

  // LISTENERS

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
          skipTrack(true);
          break;
        case "Previous" in payload:
          skipTrack(false);
          break;
        case "Seek" in payload:
          await commands.seekTrack(payload.Seek, true);
          break;
        case "Volume" in payload:
          await commands.setVolume(payload.Volume);
          break;
        case "Position" in payload:
          await commands.seekTrack(payload.Position, false);
          break;
      }
    },
  );

  const listenPlayerProgress = listen("player-progress", async (e) => {
    const progress = e.payload as number;
    await handleProgress(false, progress);
  });

  const listenTrackEnd = listen("track-end", async (_) => {
    await handleSongEnd();
  });

  return {
    paused,
    currentTrack,
    playerProgress,
    queue,
    queueIndex,
    personalQueue,
    loop,
    playerVolume,
    isShuffled,
    setPlayerTrack,
    setPlayerProgress,
    skipTrack,
    loopQueue,
    shuffleQueue,
    $reset,
    handleProgress,
    handlePlayAndPause,
    handleVolume,
    initialLoad,
    listenMediaControl,
    listenPlayerProgress,
    listenTrackEnd,
  };
});

/**
 * Shuffles an array using the Fisher-Yates algorithm.
 *
 * @param {T[]} array - The array to shuffle
 * @returns {T[]} The shuffled array
 *
 * @example
 * // Returns [3, 1, 2] or [2, 3, 1] or [1, 2, 3]
 * fisherYatesShuffle([1, 2, 3])
 */
function fisherYatesShuffle<T>(array: T[]): T[] {
  const newArray = [];

  while (array.length) {
    const randomIndex = Math.floor(Math.random() * array.length);
    const element = array.splice(randomIndex, 1)[0];
    newArray.push(element);
  }

  return newArray;
}
