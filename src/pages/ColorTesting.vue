<template>
  <div class="bg-bg-primary text-text-primary flex flex-col gap-4"></div>
</template>

<script setup lang="ts">
import { useConfigStore } from "@/composables/";
import { onMounted } from "vue";

const configStore = useConfigStore();
onMounted(() => {
  configStore.currentPage = "/color_testing";
  configStore.pageName = "Color Testing";
});
</script>
