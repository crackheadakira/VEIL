<template>
  <div class="flex w-fit items-center justify-center gap-4">
    <span
      v-if="extra"
      :class="shuffled ? 'text-accent-primary' : ''"
      class="i-fluent-arrow-shuffle-20-filled cursor-pointer hover:opacity-90"
      @click="shuffleQueue"
    ></span>
    <span
      class="i-fluent-previous-20-filled w-6 cursor-pointer hover:opacity-90"
      @click="previousTrack"
    ></span>
    <span
      @click="playerStore.handleResumeAndPause"
      :class="playing ? 'i-fluent-pause-24-filled' : 'i-fluent-play-24-filled'"
      class="i-fluent-pause-20-filled cursor-pointer hover:opacity-90"
    ></span>
    <span
      class="i-fluent-next-20-filled cursor-pointer hover:opacity-90"
      @click="nextTrack"
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
import { events, usePlayerStore } from "@/composables/";
import { onMounted, onUnmounted, ref } from "vue";

const playerStore = usePlayerStore();
const shuffled = ref(false);
const playing = ref(false);

defineProps<{
  extra?: boolean;
}>();

let unlistenUIUpdateEvent: () => void = () => {};

async function previousTrack() {
  await events.playerEvent.emit({ type: "PreviousTrackInQueue" });
}

async function nextTrack() {
  await events.playerEvent.emit({ type: "NextTrackInQueue" });
}

async function shuffleQueue() {
  await events.queueEvent.emit({ type: "ShuffleGlobalQueue" });
  shuffled.value = !shuffled.value;
}

onMounted(async () => {
  unlistenUIUpdateEvent = await events.uiUpdateEvent.listen((event) => {
    if (event.payload.type === "PlayButton") {
      if (event.payload.data.state === "Paused") {
        playing.value = false;
      } else {
        playing.value = true;
      }
    }
  });
});

onUnmounted(() => {
  unlistenUIUpdateEvent();
});
</script>
