<template>
  <div
    class="bg-bg-primary text-text-primary relative flex h-full w-full flex-col items-center gap-4"
  >
    <div
      ref="container"
      class="grid h-full w-full grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] gap-4 overflow-y-scroll"
      @scroll="onScroll"
    >
      <div
        class="col-span-full w-full"
        :style="{ height: topOffset + 'px' }"
      ></div>
      <BigCard class="shrink" v-for="album in visibleAlbums" :data="album" />
      <div :style="{ height: bottomOffset + 'px' }"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BigCard } from "@/components/";
import {
  commands,
  handleBackendError,
  useConfigStore,
  type Albums,
} from "@/composables/";
import { computed, onMounted, ref, useTemplateRef, watch } from "vue";
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

const cardStartIndex = computed(() => {
  const rowHeight = bigCardHeight + gap;
  return Math.max(
    0,
    Math.floor(scrollOffset.value / rowHeight) * cardsPerRow.value,
  );
});

const endIndex = computed(() =>
  Math.min(cardStartIndex.value + albumsToFetch.value, totalAlbums.value),
);

const containerWidth = ref(container.value?.clientWidth || 1504);
useResizeObserver(container, (entries) => {
  if (entries[0]) {
    containerWidth.value = entries[0].contentRect.width;
  }
});

const cardsPerRow = computed(() =>
  Math.floor((containerWidth.value + gap) / (bigCardWidth + gap)),
);

const albumsToFetch = computed(() => cardsPerRow.value * 5);

const scrollOffset = ref(0);

function onScroll() {
  const el = container.value;
  if (!el) return;

  scrollOffset.value = el.scrollTop;

  console.log(
    albumsToFetch.value,
    cardStartIndex.value,
    endIndex.value,
    cardsPerRow.value,
  );
}

const visibleAlbums = computed(() =>
  albums.value.slice(cardStartIndex.value, endIndex.value),
);

const topOffset = computed(() => {
  const rowHeight = bigCardHeight + gap;
  const rowIndex = Math.floor(cardStartIndex.value / cardsPerRow.value);
  return rowIndex * rowHeight;
});

const bottomOffset = computed(() => {
  const totalRows = Math.ceil(totalAlbums.value / cardsPerRow.value);
  const renderedRows = Math.ceil(
    visibleAlbums.value.length / cardsPerRow.value,
  );
  return (
    (totalRows -
      (renderedRows + Math.floor(cardStartIndex.value / cardsPerRow.value))) *
    (bigCardHeight + gap)
  );
});

watch(cardStartIndex, async (newIndex) => {
  if (newIndex + albumsToFetch.value > albums.value.length) {
    const offset = albums.value.length;
    const result = await commands.getAlbumsOffset(albumsToFetch.value, offset);
    if (result.status === "error") return handleBackendError(result.error);
    albums.value.push(...result.data);
  }
});

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
