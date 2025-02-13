import { commands, type Tracks } from "@/composables/";
import { StorageSerializers, useStorage } from "@vueuse/core";
import { defineStore } from "pinia";

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
  const currentPage = useStorage("currentPage", "/home");
  const playerVolume = useStorage("playerVolume", 0.7);
  const isShuffled = useStorage("isShuffled", false);

  /**
   * Sets given track as the current track and plays it.
   *
   * Calls Rust backend (stopPlayer, playTrack), and updates store `currentTrack` & `playerProgress`.
   *
   * @param {Tracks} track - The track to play
   */
  async function setPlayerTrack(track: Tracks): Promise<void> {
    await commands.stopPlayer();

    currentTrack.value = track;
    playerProgress.value = 0;

    await commands.playTrack(track.id);
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
    currentPage.value = "/";
    playerVolume.value = 0.7;
    isShuffled.value = false;
  }

  return {
    currentTrack,
    playerProgress,
    queue,
    queueIndex,
    personalQueue,
    loop,
    currentPage,
    playerVolume,
    isShuffled,
    setPlayerTrack,
    setPlayerProgress,
    skipTrack,
    loopQueue,
    shuffleQueue,
    $reset,
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
