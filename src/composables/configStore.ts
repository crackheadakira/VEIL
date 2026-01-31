import { StorageSerializers, useStorage } from "@vueuse/core";
import { defineStore } from "pinia";
import { commands, VeilConfig } from "@/composables/";

export const useConfigStore = defineStore("config", () => {
    const config = useStorage<VeilConfig>("config", null, undefined, {
        serializer: StorageSerializers.object,
    });
    const currentPage = useStorage("currentPage", "/home");
    const pageName = useStorage("pageName", "Home");

    async function initialize() {
        const configFile = await commands.readConfig();
        if (configFile.status === "ok") {
            config.value = configFile.data;
        } else {
            console.log("Error reading config file");
        }
    }

    return {
        config,
        initialize,
        currentPage,
        pageName,
    }
})