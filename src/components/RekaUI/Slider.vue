<template>
  <SliderRoot
    class="relative flex h-5 w-36 touch-none items-center select-none"
    v-model="sliderValue"
    :max="props.max"
    :step="props.step"
  >
    <SliderTrack class="bg-stroke-100 relative h-2 grow rounded-full">
      <SliderRange class="bg-placeholder absolute h-full rounded-full" />
    </SliderTrack>
    <SliderThumb
      class="bg-text hover:bg-placeholder focus:inset-ring-text block h-4 w-4 rounded-full transition-colors duration-150 focus:inset-ring-1 focus:outline-none"
    />
  </SliderRoot>
</template>

<script setup lang="ts">
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from "reka-ui";
import { computed } from "vue";

const props = defineProps<{
  max: number;
  step: number;
}>();

const singleValue = defineModel<number>({ required: false, default: 0 });
const sliderValue = computed<number[] | undefined>({
  get: () => [singleValue.value],
  set: (val) => {
    if (val && val.length > 0) singleValue.value = val[0];
  },
});
</script>
