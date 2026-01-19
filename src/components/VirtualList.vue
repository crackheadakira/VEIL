<template>
  <div
    ref="container"
    class="relative h-full w-full overflow-y-auto"
    @scroll="onScroll"
  >
    <div :style="{ height: topOffset + 'px' }"></div>
    <slot :items="visibleItems" :startIndex="cardStartIndex"></slot>
    <div :style="{ height: bottomOffset + 'px' }"></div>
  </div>
</template>

<script setup lang="ts" generic="T">
import { ref, computed, watch } from "vue";
import { useResizeObserver } from "@vueuse/core";

const props = withDefaults(
  defineProps<{
    items: T[];
    total: number;
    itemHeight: number;
    itemWidth?: number;
    gap: number;
    mode?: "grid" | "list";
    fetchMore?: (offset: number, count: number) => Promise<void>;
  }>(),
  {
    mode: "grid",
  },
);

const container = ref<HTMLElement | null>(null);
const containerWidth = ref(0);
const scrollOffset = ref(0);

useResizeObserver(container, (entries) => {
  if (entries[0]) containerWidth.value = entries[0].contentRect.width;
});

const cardsPerRow = computed(() => {
  if (props.mode === "list") return 1;

  return Math.max(
    1,
    Math.floor(
      (containerWidth.value + props.gap) / ((props.itemWidth ?? 0) + props.gap),
    ),
  );
});

const itemsToFetch = computed(() =>
  props.mode === "list" ? 40 : cardsPerRow.value * 6,
);

const cardStartIndex = computed(() =>
  Math.max(
    0,
    Math.floor(scrollOffset.value / (props.itemHeight + props.gap)) *
      cardsPerRow.value,
  ),
);

const endIndex = computed(() =>
  Math.min(cardStartIndex.value + itemsToFetch.value, props.total),
);

const visibleItems = computed(() =>
  props.items.slice(cardStartIndex.value, endIndex.value),
);

const topOffset = computed(() => {
  const rowIndex = Math.floor(cardStartIndex.value / cardsPerRow.value);
  return rowIndex * (props.itemHeight + props.gap);
});

const bottomOffset = computed(() => {
  const totalRows = Math.ceil(props.total / cardsPerRow.value);
  const renderedRows = Math.ceil(visibleItems.value.length / cardsPerRow.value);
  return (
    (totalRows -
      (renderedRows + Math.floor(cardStartIndex.value / cardsPerRow.value))) *
    (props.itemHeight + props.gap)
  );
});

watch(cardStartIndex, async (newIndex) => {
  if (!props.fetchMore) return;

  while (
    newIndex + itemsToFetch.value > props.items.length &&
    props.items.length < props.total
  ) {
    const remaining = props.total - props.items.length;
    const count = Math.min(itemsToFetch.value, remaining);
    await props.fetchMore(props.items.length, count);
  }
});

function onScroll() {
  if (!container.value) return;
  scrollOffset.value = container.value.scrollTop;
}
</script>
