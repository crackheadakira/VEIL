<template>
  <div class="flex items-center gap-4 justify-self-end">
    <span
      class="i-fluent-speaker-24-filled hover:text-accent-secondary cursor-pointer"
    ></span>
    <Slider
      @update:model-value="playerStore.handleVolume"
      v-model="playerStore.playerVolume"
      :max="1"
      :step="0.01"
    />
  </div>
</template>

<script setup lang="ts">
import { usePlayerStore } from "@/composables/";
import { Slider } from "@/components/";

const playerStore = usePlayerStore();
</script>
