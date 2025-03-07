<template>
  <div
    class="bg-background text-text flex h-full w-full flex-col items-center gap-4"
  >
    <div
      ref="container"
      class="flex h-full w-full flex-wrap items-center justify-center gap-4 overflow-y-scroll"
    >
      <BigCard v-for="album of albums" :data="album" />
    </div>
    <Pagination
      @update:page="onNewPage"
      :total="totalAlbums"
      :items-per-page="albumsToFetch"
    />
  </div>
</template>

<script setup lang="ts">
import { BigCard, Pagination } from "@/components/";
import {
  commands,
  handleBackendError,
  useConfigStore,
  type Albums,
} from "@/composables/";
import { templateRef } from "@vueuse/core";
import { computed, onMounted, ref } from "vue";

const configStore = useConfigStore();

const albums = ref<Albums[]>([]);
const totalAlbums = ref<number>(0);
const currentPage = ref(1);

const container = templateRef("container");
const containerWidth = ref(container.value?.clientWidth || 1504);

const albumsToFetch = computed(
  () => Math.floor(containerWidth.value / 192) * 5,
);

async function onNewPage(page: number) {
  currentPage.value = page;
  const result = await commands.getAlbumsOffset(
    albumsToFetch.value,
    albumsToFetch.value * (page - 1),
  );
  if (result.status === "error") return handleBackendError(result.error);
  albums.value = result.data;
}

onMounted(async () => {
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
