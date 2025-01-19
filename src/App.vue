<template>
  <div class="flex min-h-screen flex-col justify-between bg-background">
    <div class="flex items-stretch">
      <SideBar class="fixed left-0 top-0 z-40 h-lvh w-72 self-start" />
      <RouterView class="ml-72 overflow-scroll p-16" />
    </div>
    <Player class="sticky bottom-0 z-50" />
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import Player from './components/Player.vue';
import SideBar from './components/SideBar.vue';

const router = useRouter();

onBeforeMount(() => {
  // localStorage.clear();
  const page = getCurrentPage();
  router.push(page);
})
</script>
