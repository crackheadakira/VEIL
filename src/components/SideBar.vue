<template>
    <div class="flex flex-col items-center text-text bg-card border-stroke-100 border-r p-8 gap-8">
        <SearchBar />
        <div class="flex flex-col w-full gap-4 font-main-nonbold">
            <RouterLink class="flex items-center gap-4 hover:bg-stroke-100 p-2 rounded-md duration-150" to="/">
                <span class="i-ph-house h-8"></span>
                <p>Home</p>
            </RouterLink>
            <div class="flex items-center gap-4 hover:bg-stroke-100 p-2 rounded-md duration-150">
                <span class="i-ph-heart-straight h-8"></span>
                <p>Liked Songs</p>
            </div>
            <RouterLink class="flex items-center gap-4 hover:bg-stroke-100 p-2 rounded-md duration-150"
                to="/all_albums">
                <span class="i-ph-music-notes h-8"></span>
                <p>Albums</p>
            </RouterLink>
            <div class="flex items-center gap-4 hover:bg-stroke-100 p-2 rounded-md duration-150">
                <span class="i-mingcute-settings-3-line h-8"></span>
                <p>Settings</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import SearchBar from '../components/SearchBar.vue';
</script>