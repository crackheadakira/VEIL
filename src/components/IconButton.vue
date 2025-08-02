<template>
  <div
    @click="$emit('click')"
    class="sodapop-card flex h-12 w-72 cursor-pointer items-center gap-3 overflow-hidden px-0 pr-2 transition duration-150 hover:opacity-90"
  >
    <div
      class="bg-background border-r-stroke-200 flex h-12 w-12 shrink-0 items-center justify-center border-r"
    >
      <span class="text-supporting" :class="icon"></span>
    </div>
    <p class="truncate">{{ placeholder }}</p>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  icon: string;
  placeholder: string;
}>();

defineEmits<{
  (e: "click"): void;
}>();
</script>
