<template>
  <div class="flex items-center gap-4 justify-self-end">
    <span
      class="i-fluent-speaker-24-filled hover:text-accent-secondary cursor-pointer"
    ></span>
    <Slider
      @update:model-value="updateVolume"
      v-model="playerStore.playerVolume"
      :max="1"
      :step="0.01"
    />
  </div>
</template>

<script setup lang="ts">
import { events, usePlayerStore } from "@/composables/";
import { Slider } from "@/components/";
import { nextTick } from "vue";

const playerStore = usePlayerStore();

async function updateVolume(volume: number) {
  nextTick(async () => {
    await events.playerEvent.emit({
      type: "SetVolume",
      data: { volume },
    });
  });
}
</script>
