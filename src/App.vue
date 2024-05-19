<template>
  <div class="flex flex-col justify-between gap-4 bg-background min-h-screen h-fit">
    <RouterView />
    <Player class="bottom-0 sticky" />
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import Player from './components/Player.vue';

const router = useRouter();

onBeforeMount(() => {
  const page = getCurrentPage();
  router.push(page);
})
</script>
