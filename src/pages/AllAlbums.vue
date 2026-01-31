<template>
  <div
    class="bg-bg-primary text-text-primary relative flex h-full w-full flex-col items-center gap-4"
  >
    <h6 class="text-text-secondary w-full select-none">
      {{ totalAlbums }} albums
    </h6>
    <VirtualList
      :items="albums"
      :total="totalAlbums"
      :itemHeight="bigCardHeight"
      :itemWidth="bigCardWidth"
      :gap="gap"
      :fetchMore="fetchMore"
    >
      <template #default="{ items }">
        <div class="grid grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] gap-4">
          <BigCard
            v-for="album in items"
            :key="album.id"
            :data="album"
            class="shrink"
          />
        </div>
      </template>
    </VirtualList>
  </div>
</template>

<script setup lang="ts">
import { BigCard, VirtualList } from "@/components/";
import {
  commands,
  handleBackendError,
  useConfigStore,
  type Albums,
} from "@/composables/";
import { computed, onMounted, ref, useTemplateRef } from "vue";
import { useResizeObserver } from "@vueuse/core";

const rootFontSize = parseFloat(
  window.getComputedStyle(document.documentElement).fontSize,
);

const tailwindSpacing =
  parseFloat(
    window.getComputedStyle(document.body).getPropertyValue("--spacing"),
  ) * rootFontSize;

const bigCardHeight = tailwindSpacing * 70;
const bigCardWidth = tailwindSpacing * 48;
const gap = tailwindSpacing * 4;

const configStore = useConfigStore();

const albums = ref<Albums[]>([]);
const totalAlbums = ref<number>(0);

const container = useTemplateRef<HTMLElement>("container");

const containerWidth = ref(container.value?.clientWidth || 1504);
useResizeObserver(container, (entries) => {
  if (entries[0]) {
    containerWidth.value = entries[0].contentRect.width;
  }
});

const cardsPerRow = computed(() =>
  Math.floor((containerWidth.value + gap) / (bigCardWidth + gap)),
);

const albumsToFetch = computed(() => cardsPerRow.value * 6);

async function fetchMore(offset: number, count: number) {
  const result = await commands.getAlbumsOffset(count, offset);
  if (result.status === "error") return handleBackendError(result.error);
  albums.value.push(...result.data);
}

onMounted(async () => {
  containerWidth.value = container.value?.clientWidth || containerWidth.value;
  configStore.currentPage = "/all_albums";
  configStore.pageName = "All Albums";

  const total = await commands.getTotalAlbums();
  if (total.status === "error") return handleBackendError(total.error);
  totalAlbums.value = total.data;

  const result = await commands.getAlbumsOffset(albumsToFetch.value, 0);
  if (result.status === "error") return handleBackendError(result.error);
  albums.value = result.data;
});
</script>
