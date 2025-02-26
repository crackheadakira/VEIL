<template>
  <div class="flex w-fit items-center justify-center gap-4">
    <span
      v-if="extra"
      :class="playerStore.isShuffled ? 'text-primary' : ''"
      class="i-fluent-arrow-shuffle-20-filled cursor-pointer hover:opacity-90"
      @click="playerStore.shuffleQueue()"
    ></span>
    <span
      class="i-fluent-previous-20-filled w-6 cursor-pointer hover:opacity-90"
      @click="playerStore.skipTrack(false)"
    ></span>
    <span
      @click="playerStore.handlePlayAndPause"
      :class="
        !playerStore.paused
          ? 'i-fluent-pause-24-filled'
          : 'i-fluent-play-24-filled'
      "
      class="i-fluent-pause-20-filled cursor-pointer hover:opacity-90"
    ></span>
    <span
      class="i-fluent-next-20-filled cursor-pointer hover:opacity-90"
      @click="playerStore.skipTrack(true)"
    ></span>
    <span
      v-if="extra"
      @click="playerStore.loopQueue()"
      :class="
        (playerStore.loop === 'queue' ? 'text-primary' : '') ||
        (playerStore.loop === 'track' ? 'text-primary opacity-75' : '')
      "
      class="i-fluent-arrow-repeat-all-20-filled cursor-pointer hover:opacity-90"
    ></span>
  </div>
</template>

<script setup lang="ts">
import { usePlayerStore } from "@/composables/";

const playerStore = usePlayerStore();
defineProps<{
  extra?: boolean;
}>();
</script>
