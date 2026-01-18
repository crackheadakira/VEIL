<template>
  <div>
    <div
      v-show="!loadingImage"
      class="group aspect-secondaryCard flex h-70 w-48 cursor-pointer flex-col gap-4 select-none"
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
          @load="loadingImage = false"
        />
      </RouterLink>
      <div class="text-text-secondary flex flex-col gap-1">
        <h6 class="text-text-primary truncate">
          {{ album_data.name }}
        </h6>

        <p class="truncate">{{ album_data.artist_name }}</p>

        <p>{{ album_data.album_type }}</p>
      </div>
    </div>

    <div
      v-if="loadingImage"
      class="group aspect-secondaryCard flex w-48 flex-col gap-4 select-none"
    >
      <div class="relative aspect-square w-full">
        <div class="skeleton-loader bg-border-primary"></div>
      </div>
      <div class="text-text-secondary flex flex-col gap-1">
        <div class="relative h-4">
          <div class="skeleton-loader bg-border-primary"></div>
        </div>

        <div class="relative h-4">
          <div class="skeleton-loader"></div>
        </div>

        <div class="relative h-4 w-16">
          <div class="skeleton-loader"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Albums, AlbumWithTracks } from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

const props = defineProps<{
  data: Albums | AlbumWithTracks;
  loading?: boolean;
}>();

const loadingImage = ref(props.loading ?? true);

const album_data = computed(() => {
  if ("tracks" in props.data) return props.data.album;
  else return props.data;
});
</script>
