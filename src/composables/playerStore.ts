import { commands, handleBackendError, MediaPayload, useQueueStore, type Tracks } from "@/composables/";
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
  const loop = useStorage<"none" | "track" | "queue">("loop", "none");
  const playerProgress = useStorage("playerProgress", 0);
  const playerVolume = useStorage("playerVolume", 0.5);
  const isShuffled = useStorage("isShuffled", false);

  const paused = ref(true);

  const queueStore = useQueueStore();

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
  async function skipTrack(direction: "back" | "next"): Promise<void> {
    if (loop.value === "track") loop.value = "queue";

    const nextTrack = await queueStore.getQueueTrack(direction);
    if (nextTrack) await setPlayerTrack(nextTrack);
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

  function $reset() {
    currentTrack.value = null;
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
      const result = await commands.resumeTrack();
      if (result.status === "error") return handleBackendError(result.error);
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
    // while (!(await commands.playerHasEnded())) {
    //   await new Promise((resolve) => setTimeout(resolve, 10));
    // }

    if (loop.value === "track") {
      // replay the same track
      await setPlayerTrack(currentTrack.value);
      await handlePlayAndPause();
      return;
    }

    if (queueStore.queueHasTrack) return await skipTrack('next');
    else return await handlePlayAndPause();
  }

  /**
   * Updates the volume of the player.
   */
  async function handleVolume() {
    nextTick(async () => {
      const result = await commands.setVolume(+playerVolume.value);
      if (result.status === "error") return handleBackendError(result.error);
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

    if (duration !== 0) {
      const result = await commands.seekTrack(playerProgress.value, false);
      if (result.status === "error") return handleBackendError(result.error);
    }

    const result = await commands.setVolume(+playerVolume.value);
    if (result.status === "error") return handleBackendError(result.error);
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
          skipTrack('next');
          break;
        case "Previous" in payload:
          skipTrack('back');
          break;
        case "Seek" in payload: {
          const result = await commands.seekTrack(payload.Seek, true);
          if (result.status === "error") return handleBackendError(result.error);
          break;
        } case "Volume" in payload: {
          const result = await commands.setVolume(+playerVolume.value);
          if (result.status === "error") return handleBackendError(result.error);
          break;
        } case "Position" in payload: {
          const result = await commands.seekTrack(payload.Position, false);
          if (result.status === "error") return handleBackendError(result.error);
          break;
        }
      }
    },
  );

  const listenPlayerProgress = listen("player-progress", async (e) => {
    const progress = e.payload as number;
    await handleProgress(false, progress);
  });

  const listenTrackEnd = listen("track-end", async (_) => {
    console.log("received track-end event");
    await handleSongEnd();
  });

  return {
    paused,
    currentTrack,
    playerProgress,
    loop,
    playerVolume,
    isShuffled,
    setPlayerTrack,
    setPlayerProgress,
    skipTrack,
    loopQueue,
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
