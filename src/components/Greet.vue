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
    await commands.selectMusicFolder();
    // await commands.asyncMetadata("/run/media/akira/3TB/music");
    musicLoaded.value = true;
}
</script>
