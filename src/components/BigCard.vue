<template>
    <div class="flex flex-col gap-4 w-48 aspect-secondaryCard select-none cursor-pointer group">
        <RouterLink :to="{ name: 'album', params: { artist_id: data.artists_id, album_id: data.id } }">
            <img class="aspect-square w-48 rounded-md duration-150 group-hover:opacity-90"
                :src="convertFileSrc(data.cover_path)" alt="Album Cover">
        </RouterLink>
        <div class="flex flex-col gap-1 text-supporting">
            <p class="font-main text-text truncate">{{ data.name }}</p>
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