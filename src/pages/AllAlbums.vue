<template>
  <div class="bg-background text-text flex flex-col items-center gap-4">
    <div class="flex flex-wrap items-center justify-center gap-4">
      <BigCard v-for="album of albums" :data="album" />
    </div>
  </div>
</template>

<script setup lang="ts">
import BigCard from "../components/BigCard.vue";
import { commands, type Albums } from "../bindings";
import { toastBus } from "../composables/toastBus";

const playerStore = usePlayerStore();
const albums = ref<Albums[]>([]);

onBeforeMount(async () => {
  const result = await commands.getAllAlbums();
  if (result.status === "error")
    return toastBus.addToast(
      "error",
      `[${result.error.type}] ${result.error.data}`,
    );

  const response = result.data;
  albums.value = response;
});

onMounted(() => {
  playerStore.currentPage = "/all_albums";
});
</script>
