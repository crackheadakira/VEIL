<template>
    <div class="flex gap-1">
        <button class="p-2 border font-supporting bg-card border-stroke-100 rounded-md" @click="openDialog">Select
            music folder</button>
    </div>
</template>

<script setup lang="ts">
import { commands } from '../bindings';

const musicLoaded = ref(false);

async function openDialog() {
    await commands.selectMusicFolder();
    musicLoaded.value = true;
}
</script>
