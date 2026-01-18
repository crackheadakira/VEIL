<template>
  <div
    class="bg-bg-primary text-text-primary relative flex h-full w-full flex-col items-center gap-4"
  >
    <div ref="spacer" class="h-full w-full">
      <div
        ref="container"
        class="grid h-full w-full grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] gap-4 overflow-y-scroll"
        @scroll="onScroll"
      >
        <BigCard class="shrink" v-for="album of albums" :data="album" />
      </div>
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
const loadPadding = bigCardHeight;

const totalAlbumsHeight = computed(
  () => totalAlbums.value * (bigCardHeight + gap),
);

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

const albumsToFetch = computed(() => cardsPerRow.value * 5);

const scrollOffset = ref(0);

function onScroll() {
  const el = container.value;
  if (!el) return;

  scrollOffset.value = el.scrollTop;
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
