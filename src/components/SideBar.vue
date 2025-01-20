<template>
    <div class="flex flex-col items-center gap-8 border-r border-stroke-100 bg-card p-8 text-text">
        <SearchBar />
        <div class="font-main-nonbold flex w-full flex-col gap-4">
            <RouterLink class="flex items-center gap-4 rounded-md p-2 duration-150 hover:bg-stroke-100" to="/">
                <span class="i-fluent-home-16-filled h-8"></span>
                <p>Home</p>
            </RouterLink>
            <div class="flex items-center gap-4 rounded-md p-2 duration-150 hover:bg-stroke-100">
                <span class="i-fluent-heart-16-filled h-8"></span>
                <p>Liked Songs</p>
            </div>
            <RouterLink class="flex items-center gap-4 rounded-md p-2 duration-150 hover:bg-stroke-100"
                to="/all_albums">
                <span class="i-fluent-music-note-2-16-filled h-8"></span>
                <p>Albums</p>
            </RouterLink>
            <div class="flex items-center gap-4 rounded-md p-2 duration-150 hover:bg-stroke-100">
                <span class="i-fluent-settings-16-filled h-8"></span>
                <p>Settings</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import SearchBar from '../components/SearchBar.vue';
</script>