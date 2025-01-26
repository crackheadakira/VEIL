<template>
    <div class="group flex aspect-secondaryCard w-48 cursor-pointer select-none flex-col gap-4">
        <RouterLink :to="{ name: 'album', params: { artist_id: album_data.artists_id, album_id: album_data.id } }">
            <img class="aspect-square w-48 rounded-md duration-150 group-hover:opacity-90"
                :src="convertFileSrc(album_data.cover_path)" alt="Album Cover" width="192" height="192">
        </RouterLink>
        <div class="flex flex-col gap-1 text-supporting">
            <p class="font-main truncate text-text">{{ album_data.name }}</p>
            <small class="truncate">{{ album_data.artist }}</small>
            <small>{{ album_data.album_type }}</small>
        </div>
    </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';
import type { Albums, AlbumWithTracks } from '../bindings';

const album_data = computed(() => {
    if ('tracks' in props.data) return props.data.album;
    else return props.data;
})
const props = defineProps<{
    data: Albums | AlbumWithTracks
}>()
</script>