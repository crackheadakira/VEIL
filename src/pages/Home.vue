<template>
    <div class="flex flex-col bg-background text-text">
        <Greet />
        <RouterLink class="text-center p-2 w-32 border font-supporting bg-card border-stroke-100 rounded-md"
            to="/all_albums">All Albums</RouterLink>
        <div class="flex gap-2">
            <PlaylistCard />
            <SearchBar />
            <Dropdown :title="'Filter by'" :options="['Albums', 'Artists', 'Tracks']" />
        </div>
        <div>
            <h6 class="font-h6 text-text mb-4">Recently listened</h6>
            <div class="flex flex-wrap gap-4">
                <BigCard v-for="album of recentlyPlayed" :data="album" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import Greet from '../components/Greet.vue';
import PlaylistCard from '../components/PlaylistCard.vue';
import SearchBar from '../components/SearchBar.vue';
import Dropdown from '../components/Dropdown.vue';
import BigCard from '../components/BigCard.vue';

const recentlyPlayed = ref(getRecentlyPlayed());

onMounted(() => {
    setCurrentPage('/');
})
</script>