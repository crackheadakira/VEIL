<template>
    <div class="flex flex-col items-center gap-8 border-r border-stroke-100 bg-card p-8 text-text">
        <SearchBar />
        <div class="font-main flex w-full flex-col gap-2 font-medium *:select-none">
            <div class="flex w-full flex-col gap-4">
                <p class="font-supporting text-supporting">General</p>
                <hr class="border-t-2 border-stroke-100">
            </div>
            <RouterLink class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text"
                to="/">
                <span class="i-fluent-home-16-filled h-8"></span>
                <p class="font-supporting">Home</p>
            </RouterLink>
            <div class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text">
                <span class="i-fluent-heart-16-filled h-8"></span>
                <p class="font-supporting">Liked Songs</p>
            </div>
            <RouterLink class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text"
                to="/all_albums">
                <span class="i-fluent-music-note-2-16-filled h-8"></span>
                <p class="font-supporting">Albums</p>
            </RouterLink>
            <div class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text">
                <span class="i-fluent-settings-16-filled h-8"></span>
                <p class="font-supporting">Settings</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import SearchBar from '../components/SearchBar.vue';
</script>