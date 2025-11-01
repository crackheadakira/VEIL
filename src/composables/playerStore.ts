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
  const playerProgress = useStorage("playerProgress", 0);
  const playerVolume = useStorage("playerVolume", 0.5);

  const paused = ref(true);

  // Channel is for syncing pause state between main window & widget
  const channel = new BroadcastChannel("player_channel");

  watch(paused, (newValue) => {
    channel.postMessage({ paused: newValue });
  });

  channel.onmessage = (event) => {
    paused.value = event.data.paused;
  };

  function $reset() {
    currentTrack.value = null;
    playerProgress.value = 0;
    playerVolume.value = 0.5;
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
    playerVolume,
    listenNewTrack,
    $reset,
    handleProgress,
    initialLoad,
  };
});
