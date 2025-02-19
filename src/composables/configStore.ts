import { useStorage } from "@vueuse/core";
import { defineStore } from "pinia";
import { SodapopConfig, commands } from "@/composables/";

export const useConfigStore = defineStore("config", () => {
    const config = useStorage<SodapopConfig>("config", null);

    async function initialize() {
        config.value = await commands.getConfig();
    }

    return {
        config,
        initialize
    }
})