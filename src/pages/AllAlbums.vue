<template>
  <div class="bg-background text-text flex flex-col items-center gap-4">
    <div class="flex flex-wrap items-center justify-center gap-4">
      <BigCard v-for="album of albums" :data="album" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { BigCard } from "@/components/";
import {
  commands,
  handleBackendError,
  usePlayerStore,
  type Albums,
} from "@/composables/";
import { onBeforeMount, onMounted, ref } from "vue";

const playerStore = usePlayerStore();
const albums = ref<Albums[]>([]);

onBeforeMount(async () => {
  const result = await commands.getAllAlbums();
  if (result.status === "error") return handleBackendError(result.error);

  const response = result.data;
  albums.value = response;
});

onMounted(() => {
  playerStore.currentPage = "/all_albums";
  playerStore.pageName = "All Albums";
});
</script>
