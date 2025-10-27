<template>
  <div class="flex w-fit items-center justify-center gap-4">
    <span
      v-if="extra"
      :class="playerStore.isShuffled ? 'text-accent-primary' : ''"
      class="i-fluent-arrow-shuffle-20-filled cursor-pointer hover:opacity-90"
      @click="queueStore.shuffleQueue()"
    ></span>
    <span
      class="i-fluent-previous-20-filled w-6 cursor-pointer hover:opacity-90"
      @click="playerStore.skipTrack('back')"
    ></span>
    <span
      @click="playerStore.handleResumeAndPause"
      :class="
        !playerStore.paused
          ? 'i-fluent-pause-24-filled'
          : 'i-fluent-play-24-filled'
      "
      class="i-fluent-pause-20-filled cursor-pointer hover:opacity-90"
    ></span>
    <span
      class="i-fluent-next-20-filled cursor-pointer hover:opacity-90"
      @click="playerStore.skipTrack('next')"
    ></span>
    <span
      v-if="extra"
      @click="playerStore.loopQueue()"
      :class="
        (playerStore.loop === 'queue' ? 'text-accent-primary' : '') ||
        (playerStore.loop === 'track' ? 'text-accent-primary opacity-75' : '')
      "
      class="i-fluent-arrow-repeat-all-20-filled cursor-pointer hover:opacity-90"
    ></span>
  </div>
</template>

<script setup lang="ts">
import { usePlayerStore, useQueueStore } from "@/composables/";

const playerStore = usePlayerStore();
const queueStore = useQueueStore();

defineProps<{
  extra?: boolean;
}>();
</script>
