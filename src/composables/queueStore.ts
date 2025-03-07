import { useStorage } from "@vueuse/core";
import { defineStore } from "pinia";

export const useQueueStore = defineStore("queue", () => {
    const queueIds = useStorage<number[]>("queueIds", null);

    return {
        queueIds
    }
})