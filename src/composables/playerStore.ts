import { defineStore } from "pinia";
import { useStorage, StorageSerializers } from "@vueuse/core";
import { Albums, commands, Tracks } from "../bindings";

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

  async function setPlayerTrack(track: Tracks) {
    await commands.stopPlayer();

    currentTrack.value = track;
    playerProgress.value = 0;

    await commands.playTrack(track.id);
  }

  async function setPlayerProgress(progress: number) {
    await commands.setPlayerProgress(progress);
    playerProgress.value = progress;
  }

  async function skipTrack(forward: boolean) {
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

  function loopQueue() {
    if (loop.value === "none") loop.value = "queue";
    else if (loop.value === "queue") loop.value = "track";
    else loop.value = "none";
  }

  function shuffleQueue() {
    // If already shuffled, reset the queue
    if (isShuffled.value) {
      queue.value = queue.value.sort((a, b) => a.id - b.id);
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

  return {
    currentTrack,
    setPlayerTrack,
    playerProgress,
    setPlayerProgress,
    queue,
    queueIndex,
    personalQueue,
    skipTrack,
    loop,
    loopQueue,
    currentPage,
    playerVolume,
    isShuffled,
    shuffleQueue,
  };
});

function fisherYatesShuffle<T>(array: T[]) {
  const newArray = [];

  while (array.length) {
    const randomIndex = Math.floor(Math.random() * array.length);
    const element = array.splice(randomIndex, 1)[0];
    newArray.push(element);
  }

  return newArray;
}
