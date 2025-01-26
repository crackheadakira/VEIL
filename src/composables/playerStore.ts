import { defineStore } from "pinia";
import { useStorage, StorageSerializers } from "@vueuse/core";
import { Albums, commands, Tracks } from "../bindings";

export const usePlayerStore = defineStore('player', () => {
    const currentTrack = useStorage<Tracks>("currentTrack", null, undefined, { serializer: StorageSerializers.object });
    const queue = useStorage<Tracks[]>("queue", []);
    const queueIndex = useStorage("queueIndex", 0);
    const personalQueue = useStorage<Tracks[]>("personalQueue", []);
    const recentlyPlayed = useStorage<Albums[]>("recentlyPlayed", []);
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

    function addToRecentlyPlayed(album: Albums) {
        const index = recentlyPlayed.value.findIndex((a) => a.id === album.id);

        if (index !== -1) recentlyPlayed.value.splice(index, 1);

        recentlyPlayed.value.unshift(album);

        if (recentlyPlayed.value.length > 10) recentlyPlayed.value.pop();
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

    function updateShuffle() {
        if (isShuffled.value) {
            queue.value = queue.value.sort((a, b) => a.id - b.id);
            return isShuffled.value = false;
        }

        const trackIndex = queue.value.findIndex((track) => track.id === currentTrack.value?.id);
        if (trackIndex === -1) return;


        queue.value.splice(trackIndex, 1);
        const shuffledQueue = fisherYatesShuffle(queue.value);
        shuffledQueue.unshift(currentTrack.value);

        queue.value = shuffledQueue;
    }

    return {
        currentTrack,
        setPlayerTrack,
        playerProgress,
        setPlayerProgress,
        queue,
        queueIndex,
        personalQueue,
        recentlyPlayed,
        addToRecentlyPlayed,
        skipTrack,
        loop,
        loopQueue,
        currentPage,
        playerVolume,
        isShuffled,
        updateShuffle
    }
})

function fisherYatesShuffle<T>(array: T[]) {
    const newArray = [];

    while (array.length) {
        const randomIndex = Math.floor(Math.random() * array.length);
        const element = array.splice(randomIndex, 1)[0];
        newArray.push(element);
    }

    return newArray;
}