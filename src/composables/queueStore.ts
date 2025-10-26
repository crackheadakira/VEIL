import { useStorage } from "@vueuse/core";
import { defineStore } from "pinia";
import { clampRange, commands, handleBackendError, Tracks, usePlayerStore } from "@/composables/";
import { computed } from "vue";

export const useQueueStore = defineStore("queue", () => {
    const globalQueue = useStorage<number[]>("queue", []);
    const _index = useStorage("queueIndex", 0);
    const personalQueue = useStorage<Tracks[]>("personalQueue", []);

    const playerStore = usePlayerStore();

    // prevents from modifying these values outside of the store
    const index = computed(() => _index.value);
    const queueHasTrack = computed(() => {
        const globalQueueLength = globalQueue.value.length;
        const personalQueueLength = personalQueue.value.length;
        const globalQueueReachedEnd = (globalQueueLength - 1) == _index.value;

        if (globalQueueLength != 0) return !globalQueueReachedEnd || personalQueueLength > 0;
        else return personalQueueLength > 0;
    });

    /**
     * Gives you the the next track in the queue depending on the `direction` passed in.
     * 
     * If the user has a track added to their personal queue & `direction` is `next` return from personal queue, otherwise from global queue.
     */
    async function getQueueTrack(direction: "back" | "next"): Promise<Tracks | void> {
        // Can't be undefined as we checked if personalQueue contains items
        if (personalQueue.value.length > 0 && direction === "next") return personalQueue.value.shift()!;

        const directionNumber = direction === "back" ? -1 : 1;
        // Clamp the number to go from 0 to globalQueue.length if we do +1 or -1
        const nextIndex = clampRange(index.value + directionNumber, 0, globalQueue.value.length - 1);

        setQueueIdx(nextIndex);
        return await getTrackAtIdx(nextIndex);
    }

    /**
     * Sets the queue index.
     * 
     * The index input is clamped to the length of the globalQueue
     */
    function setQueueIdx(idx: number): void {
        idx = Math.min(idx, globalQueue.value.length);
        _index.value = idx;
    }

    /**
     * Retrieves the track at the queue index, or if index is passed in from that index.
     */
    async function getTrackAtIdx(idx?: number): Promise<Tracks | void> {
        const trackID = globalQueue.value[idx ?? index.value];

        const result = await commands.trackById(trackID);
        if (result.status === "error") return handleBackendError(result.error);

        return result.data;
    }

    /**
     * Allows for setting the global queue
     */
    function setGlobalQueue(tracks: Tracks[] | number[]) {
        if (tracks.length < 0) return;

        if (!isTrack(tracks)) return globalQueue.value = [...tracks];

        const ids: number[] = [];
        for (const track of tracks) ids.push(track.id);
        globalQueue.value = ids;
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
        // If already shuffled, undo the shuffle & sort it back by id
        if (playerStore.isShuffled) {
            const sorted = globalQueue.value.sort((a, b) => a - b);
            playerStore.isShuffled = false;
            const trackIndex = sorted.findIndex(
                (id) => id === playerStore.currentTrack?.id,
            );

            if (trackIndex === -1) return;

            _index.value = trackIndex;

            return;
        }

        const trackIndex = globalQueue.value.findIndex(
            (id) => id === playerStore.currentTrack?.id,
        );

        if (trackIndex === -1) return;

        globalQueue.value.splice(trackIndex, 1);
        const shuffledQueue = fisherYatesShuffle(globalQueue.value);
        shuffledQueue.unshift(playerStore.currentTrack.id);

        globalQueue.value = shuffledQueue;
        playerStore.isShuffled = true;
    }

    return {
        globalQueue,
        index,
        personalQueue,
        queueHasTrack,
        getQueueTrack,
        shuffleQueue,
        setQueueIdx,
        setGlobalQueue,
        getTrackAtIdx,
    }
})

function isTrack(tracks: Tracks[] | number[] | Tracks | number): tracks is Tracks[] | Tracks {
    if (Array.isArray(tracks)) return typeof tracks[0] === "object" && "id" in tracks[0];
    else return typeof tracks === "object" && "id" in tracks;
}

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