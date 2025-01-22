<template>
    <div class="group flex aspect-secondaryCard w-48 cursor-pointer select-none flex-col gap-4">
        <RouterLink :to="{ name: 'album', params: { artist_id: data.artists_id, album_id: data.id } }">
            <img class="aspect-square w-48 rounded-md duration-150 group-hover:opacity-90"
                :src="convertFileSrc(data.cover_path)" alt="Album Cover" width="192" height="192">
        </RouterLink>
        <div class="flex flex-col gap-1 text-supporting">
            <p class="font-main truncate text-text">{{ data.name }}</p>
            <p class="font-supporting truncate">{{ data.artist }}</p>
            <p class="font-supporting">{{ data.album_type }}</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';
import type { Albums } from '../bindings';
defineProps<{
    data: Albums
}>()
</script>