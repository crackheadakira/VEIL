<template>
  <div class="flex items-center gap-3 justify-self-end">
    <span
      :class="
        volume === 0
          ? 'i-fluent-speaker-mute-16-regular'
          : 'i-fluent-speaker-16-regular'
      "
      class="text-text-secondary size-6"
    ></span>
    <Slider
      @update:model-value="updateVolume"
      v-model="volume"
      :max="1"
      :step="0.01"
    />
  </div>
</template>

<script setup lang="ts">
import { events, usePlayerStore } from "@/composables/";
import { Slider } from "@/components/";
import { nextTick, ref } from "vue";

const playerStore = usePlayerStore();
const volume = ref(playerStore.playerVolume);

async function updateVolume(volume: number) {
  nextTick(async () => {
    await events.playerEvent.emit({
      type: "SetVolume",
      data: { volume },
    });
  });
}
</script>
