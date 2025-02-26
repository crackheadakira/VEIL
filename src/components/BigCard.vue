<template>
  <div
    class="group aspect-secondaryCard flex w-48 cursor-pointer flex-col gap-4 select-none"
  >
    <RouterLink
      :to="{
        name: 'album',
        params: { id: album_data.id },
      }"
    >
      <img
        class="aspect-square w-48 rounded-md duration-150 group-hover:opacity-90"
        :src="convertFileSrc(album_data.cover_path)"
        alt="Album Cover"
        width="192"
        height="192"
        loading="lazy"
      />
    </RouterLink>
    <div class="text-supporting flex flex-col gap-1">
      <p class="text-text truncate">{{ album_data.name }}</p>
      <small class="truncate">{{ album_data.artist_name }}</small>
      <small>{{ album_data.album_type }}</small>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Albums, AlbumWithTracks } from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed } from "vue";

const album_data = computed(() => {
  if ("tracks" in props.data) return props.data.album;
  else return props.data;
});
const props = defineProps<{
  data: Albums | AlbumWithTracks;
}>();
</script>
