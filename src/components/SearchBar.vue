<template>
  <div
    class="border-stroke-100 bg-background hover:border-stroke-200 flex h-fit w-56 items-center gap-2 rounded-md border p-2 px-3 text-base font-medium duration-150"
  >
    <span
      class="i-fluent-search-12-filled text-supporting aspect-square w-5"
    ></span>
    <input
      @keyup.50="$emit('search', searchQuery)"
      :value="searchQuery"
      type="text"
      class="text-text placeholder-supporting w-full bg-transparent focus:outline-hidden"
      placeholder="Search..."
    />
  </div>
</template>

<script setup lang="ts">
defineEmits(["search"]);
const searchQuery = ref("");
</script>
