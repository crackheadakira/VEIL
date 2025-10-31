import { commands, events, type Tracks } from "@/composables/";
import { StorageSerializers, useStorage } from "@vueuse/core";
import { defineStore } from "pinia";
import { ref, watch } from "vue";

/**
 * The player store composable.
 *
 * Manages the player state, including the current track, queue, player progress, and loop settings.
 *
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

  // Channel is for syncing pause state between main window & widget
  const channel = new BroadcastChannel("player_channel");

  watch(paused, (newValue) => {
    channel.postMessage({ paused: newValue });
  });

  channel.onmessage = (event) => {
    paused.value = event.data.paused;
  };

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
    // Unsure of the purpose of this.
    // await events.playerEvent.emit({ type: "Pause" });
    const duration = await commands.getPlayerDuration();

    if (duration !== 0)
      await events.playerEvent.emit({
        type: "Seek",
        data: { position: playerProgress.value, resume: false },
      });

    await events.playerEvent.emit({
      type: "SetVolume",
      data: { volume: playerVolume.value },
    });
  }

  // LISTENERS

  // will be updated in the future to use its own UIChangeEvent
  const listenNewTrack = events.playerEvent.listen((e) => {
    switch (e.payload.type) {
      case "NewTrack":
        currentTrack.value = e.payload.data.track;
        playerProgress.value = 0;
        paused.value = false;
        break;

      case "Pause":
        break;

      case "Resume":
        break;

      case "Stop":
        break;
    }
  });

  return {
    paused,
    currentTrack,
    playerProgress,
    loop,
    playerVolume,
    isShuffled,
    listenNewTrack,
    loopQueue,
    $reset,
    handleProgress,
    initialLoad,
  };
});
