<template>
  <div class="flex w-fit items-center justify-center gap-8">
    <span
      v-if="extra"
      :class="
        shuffled
          ? 'text-accent-primary hover:text-accent-primary-hovered'
          : 'text-text-secondary hover:text-text-primary'
      "
      class="i-fluent-arrow-shuffle-16-filled size-5 cursor-pointer transition-colors duration-150"
      @click="shuffleQueue"
    ></span>
    <span
      class="i-fluent-previous-16-filled text-text-secondary hover:text-text-primary size-5 cursor-pointer transition-colors duration-150"
      @click="previousTrack"
    ></span>
    <span
      @click="updatePlayerState"
      :class="playing ? 'i-fluent-pause-16-filled' : 'i-fluent-play-16-filled'"
      class="i-fluent-pause-16-filled text-text-secondary hover:text-text-primary size-6 cursor-pointer transition-colors duration-150"
    ></span>
    <span
      class="i-fluent-next-16-filled text-text-secondary hover:text-text-primary size-5 cursor-pointer transition-colors duration-150"
      @click="nextTrack"
    ></span>
    <span
      v-if="extra"
      @click="updateRepeatMode"
      :class="
        (repeatMode === 'None'
          ? 'text-text-secondary hover:text-text-primary'
          : '') ||
        (repeatMode === 'Queue'
          ? 'text-accent-primary hover:text-accent-primary-hovered'
          : '') ||
        (repeatMode === 'Track'
          ? 'text-accent-secondary hover:text-accent-secondary-hovered'
          : '')
      "
      class="i-fluent-arrow-repeat-all-16-filled size-5 cursor-pointer transition-colors duration-150"
    ></span>
  </div>
</template>

<script setup lang="ts">
import { events, RepeatMode } from "@/composables/";
import { onMounted, onUnmounted, ref } from "vue";

const shuffled = ref(false);
const playing = ref(false);
const repeatMode = ref<RepeatMode>("None");

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
}

async function updatePlayerState() {
  await events.playerEvent.emit({ type: "UpdatePlayerState" });
}

async function updateRepeatMode() {
  await events.queueEvent.emit({ type: "UpdateRepeatMode" });
}

onMounted(async () => {
  unlistenUIUpdateEvent = await events.uiUpdateEvent.listen((event) => {
    switch (event.payload.type) {
      case "PlayButton":
        if (event.payload.data.state === "Paused") {
          playing.value = false;
        } else {
          playing.value = true;
        }
        break;

      case "LoopButton":
        repeatMode.value = event.payload.data.mode;
        break;

      case "ShuffleButton":
        shuffled.value = event.payload.data.enabled;
        break;
    }
  });
});

onUnmounted(() => {
  unlistenUIUpdateEvent();
});
</script>
