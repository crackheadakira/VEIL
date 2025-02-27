<template>
  <div class="w-36">
    <div
      @mouseenter="onMouseEnter"
      @mouseleave="onMouseLeave"
      @mousemove="updateProgress"
      @click="(e) => updateProgress(e, true)"
      ref="container"
      class="bg-stroke-100 relative h-2 w-full cursor-default rounded-lg *:pointer-events-none *:select-none"
    >
      <div
        ref="dotEl"
        :style="{ left: `${sliderDot}px` }"
        :class="dot"
        class="bg-placeholder absolute top-1/2 left-0 h-4 w-4 -translate-y-1/2 rounded-full"
      ></div>
      <div
        :style="{ width: `${sliderDot + 8}px` }"
        :class="accent"
        class="bg-placeholder absolute left-0 h-full rounded-full"
      ></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { templateRef, useResizeObserver } from "@vueuse/core";
import { computed, ref } from "vue";

const progress = defineModel<number>({
  required: false,
  default: 0,
});

const props = defineProps<{
  max: number;
  step?: number;
  dot?: string;
  accent?: string;
}>();

const dragging = ref(false);
const clampedProgress = computed(() =>
  Math.max(0, Math.min(progress.value / props.max, props.max)),
);

const dotEl = templateRef("dotEl");
const container = templateRef("container");
const sliderWidth = ref(container.value?.clientWidth ?? 144);
const dotWidth = ref(dotEl.value?.clientWidth ?? 16);

useResizeObserver(container, (entries) =>
  requestAnimationFrame(
    () => (sliderWidth.value = entries[0].contentRect.width),
  ),
);
useResizeObserver(dotEl, (entries) =>
  requestAnimationFrame(() => (dotWidth.value = entries[0].contentRect.width)),
);

const sliderDot = computed(() => {
  let steppedProgress = clampedProgress.value;
  if (props.step)
    steppedProgress =
      Math.round(clampedProgress.value / props.step) * props.step;

  const dotPosition = sliderWidth.value * steppedProgress;

  return Math.max(0, Math.min(dotPosition, sliderWidth.value - dotWidth.value));
});

function updateProgress(e: MouseEvent, bypass?: boolean) {
  if (!dragging.value && !bypass)
    return document.removeEventListener("mousemove", updateProgress);

  const rect = container.value.getBoundingClientRect();
  const cursor = e.clientX - rect.left;
  let rawProgress = (cursor / sliderWidth.value) * props.max;
  if (props.step)
    rawProgress = Math.round(rawProgress / props.step) * props.step;

  progress.value = Math.max(0, Math.min(rawProgress, props.max));
}

function onMouseUp() {
  dragging.value = false;
  window.removeEventListener("mousemove", updateProgress);
  window.removeEventListener("mouseup", onMouseUp);
}

function onMouseEnter() {
  window.addEventListener(
    "mousedown",
    (e) => ((dragging.value = true), e.preventDefault()),
    {
      once: true,
    },
  );

  window.addEventListener("mousemove", updateProgress);
  window.addEventListener("mouseup", onMouseUp);
}

function onMouseLeave() {
  if (!dragging.value) {
    window.removeEventListener("mousemove", updateProgress);
    window.removeEventListener("mouseup", onMouseUp);
  }
}
</script>
