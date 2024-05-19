<template>
    <div class="flex flex-col gap-4 bg-background text-text items-center">
        <div class="flex flex-wrap gap-4 items-center justify-center">
            <BigCard v-for="album of albums" :data="album" />
        </div>
    </div>
</template>

<script setup lang="ts">
import BigCard from '../components/BigCard.vue';
import { commands, type Albums } from '../bindings';

const albums = ref<Albums[]>([]);

onBeforeMount(async () => {
    const response = await commands.getAllAlbums();
    albums.value = response;
});

onMounted(() => {
    setCurrentPage('/all_albums');
})
</script>