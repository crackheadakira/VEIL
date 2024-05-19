<template>
    <div>
        Current Album
    </div>
</template>

<script setup lang="ts">
import type { AlbumWithTracks } from '../bindings';
defineProps<{
    album: AlbumWithTracks
}>()
</script>