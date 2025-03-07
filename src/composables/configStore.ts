import { useStorage } from "@vueuse/core";
import { defineStore } from "pinia";
import { SodapopConfig } from "@/composables/";
import { BaseDirectory, readTextFile } from "@tauri-apps/plugin-fs";

export const useConfigStore = defineStore("config", () => {
    const config = useStorage<SodapopConfig>("config", null);
    const currentPage = useStorage("currentPage", "/home");
    const pageName = useStorage("pageName", "Home");

    async function initialize() {
        const file: SodapopConfig = JSON.parse(await readTextFile('config.json', {
            baseDir: BaseDirectory.AppLocalData
        }));
        config.value = file;
    }

    return {
        config,
        initialize,
        currentPage,
        pageName,
    }
})