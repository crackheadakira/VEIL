<template>
  <div class="flex flex-col justify-between bg-background min-h-screen">
    <div class="flex items-stretch">
      <SideBar class="w-72 fixed h-lvh z-40 top-0 left-0 self-start" />
      <RouterView class="ml-72 p-16 overflow-scroll" />
    </div>
    <Player class="bottom-0 sticky z-50" />
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import Player from './components/Player.vue';
import SideBar from './components/SideBar.vue';

const router = useRouter();

onBeforeMount(() => {
  const page = getCurrentPage();
  router.push(page);
})
</script>
