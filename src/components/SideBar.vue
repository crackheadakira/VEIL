<template>
    <div class="flex flex-col items-center gap-8 border-r border-stroke-100 bg-card p-8 text-text">
        <SearchBar />
        <div class="flex w-full flex-col gap-2 font-medium *:select-none">
            <div class="flex w-full flex-col gap-4">
                <small class="text-supporting">General</small>
                <hr class="border-t-2 border-stroke-100">
            </div>
            <RouterLink class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text"
                to="/">
                <span class="i-fluent-home-16-filled h-8"></span>
                <small>Home</small>
            </RouterLink>
            <div class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text">
                <span class="i-fluent-heart-16-filled h-8"></span>
                <small>Liked Songs</small>
            </div>
            <RouterLink class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text"
                to="/all_albums">
                <span class="i-fluent-music-note-2-16-filled h-8"></span>
                <small>Albums</small>
            </RouterLink>
            <div class="flex items-center gap-4 rounded-md p-2 text-supporting duration-75 hover:text-text">
                <span class="i-fluent-settings-16-filled h-8"></span>
                <small>Settings</small>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import SearchBar from '../components/SearchBar.vue';
</script>