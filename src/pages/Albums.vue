<template>
    <div class="flex flex-col gap-4 bg-background text-text items-center">
        <div class="flex flex-wrap gap-4 items-center w-3/4">
            <BigCard v-for="album of albums" :data="album" />
        </div>
        <RouterLink class="p-2 w-32 border font-supporting bg-card border-stroke-100 rounded-md text-center" to="/">Go
            to Home
        </RouterLink>
    </div>
</template>

<script setup lang="ts">
import BigCard from '../components/BigCard.vue';
import { commands, type Albums } from '../bindings';

const albums = ref<Albums[]>([]);

onMounted(async () => {
    const response = await commands.getAllAlbums();
    albums.value = response;
});
</script>