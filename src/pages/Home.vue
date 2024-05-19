<script setup lang="ts">
import Greet from '../components/Greet.vue';
import PlaylistCard from '../components/PlaylistCard.vue';
import SearchBar from '../components/SearchBar.vue';
import Dropdown from '../components/Dropdown.vue';
</script>

<template>
    <div class="flex flex-col bg-background text-text">
        <Greet />
        <RouterLink class="text-center p-2 w-32 border font-supporting bg-card border-stroke-100 rounded-md"
            to="/albums">Go to
            Albums
        </RouterLink>
        <div class="flex gap-2">
            <PlaylistCard />
            <SearchBar />
            <Dropdown :title="'Filter by'" :options="['Albums', 'Artists', 'Tracks']" />
        </div>
    </div>
</template>