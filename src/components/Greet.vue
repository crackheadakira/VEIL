<template>
    <div class="flex gap-1">
        <button class="font-supporting rounded-md border border-stroke-100 bg-card p-2" @click="openDialog">Select
            music folder</button>
    </div>
</template>

<script setup lang="ts">
import { commands } from '../bindings';

const musicLoaded = ref(false);

async function openDialog() {
    const result = await commands.selectMusicFolder();
    if (result.status === "error") throw new Error(`[${result.error.type}] ${result.error.data}`);
    musicLoaded.value = true;
}
</script>
