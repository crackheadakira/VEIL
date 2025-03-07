import { useStorage } from "@vueuse/core";
import { defineStore } from "pinia";
import { clampRange, Tracks, usePlayerStore } from "@/composables/";
import { computed } from "vue";

export const useQueueStore = defineStore("queue", () => {
    const globalQueue = useStorage<Tracks[]>("queue", []);
    const internalIndex = useStorage("queueIndex", 0);
    const personalQueue = useStorage<Tracks[]>("personalQueue", []);

    const playerStore = usePlayerStore();

    // prevents from modifying index outside of store without using `setQueueIdx()`
    const index = computed(() => internalIndex.value);

    /**
     * Gives you the the next track in the queue depending on the `direction` passed in.
     * 
     * If the user has a track added to their personal queue & `direction` is `next` return from personal queue, otherwise from global queue.
     */
    function getQueueTrack(direction: "back" | "next") {
        // Can't be undefined as we checked if personalQueue contains items
        if (personalQueue.value.length > 0 && direction === "next") return personalQueue.value.shift()!;

        const directionNumber = direction === "back" ? -1 : 1;
        // Clamp the number to go from 0 to globalQueue.length if we do +1 or -1
        const nextIndex = clampRange(index.value + directionNumber, 0, globalQueue.value.length - 1);

        return setQueueIdx(nextIndex)
    }

    /**
     * Sets the queue index, and returns the track at that index.
     * 
     * The index input is clamped to the length of the globalQueue
     */
    function setQueueIdx(idx: number) {
        idx = Math.min(idx, globalQueue.value.length);
        internalIndex.value = idx;
        return globalQueue.value[idx];
    }

    /**
     * Shuffles the queue using the Fisher-Yates algorithm if `isShuffled` is false, otherwise sorts the queue by track id.
     *
     * Updates the store `queue` and `isShuffled` value.
     *
     * @example
     * // queueStore.queue = [track1, track2, track3]
     * // playerStore.isShuffled = false
     * queueStore.shuffleQueue() // playerStore.queue = [track3, track1, track2]
     * queueStore.shuffleQueue() // playerStore.queue = [track1, track2, track3]
     */
    function shuffleQueue() {
        // If already shuffled, sort the queue by id
        if (playerStore.isShuffled) {
            globalQueue.value.sort((a, b) => a.id - b.id);
            playerStore.isShuffled = false;

            return;
        }

        const trackIndex = globalQueue.value.findIndex(
            (track) => track.id === playerStore.currentTrack?.id,
        );

        if (trackIndex === -1) return;

        globalQueue.value.splice(trackIndex, 1);
        const shuffledQueue = fisherYatesShuffle(globalQueue.value);
        shuffledQueue.unshift(playerStore.currentTrack);

        globalQueue.value = shuffledQueue;
        playerStore.isShuffled = true;
    }

    return {
        globalQueue,
        index,
        personalQueue,
        getQueueTrack,
        shuffleQueue,
        setQueueIdx,
    }
})

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
};