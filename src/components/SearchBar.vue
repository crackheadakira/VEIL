<template>
    <div
        class="text-base font-medium flex items-center h-fit w-56 gap-2 rounded-md border border-stroke-100 bg-background p-2 px-3 duration-150 hover:border-stroke-200">
        <span class="i-fluent-search-12-filled aspect-square w-5 text-supporting"></span>
        <input @keyup.50="$emit('search', searchQuery)" :value="searchQuery" type="text"
            class="w-full bg-transparent text-text placeholder-supporting focus:outline-hidden"
            placeholder="Search..." />
    </div>
</template>

<script setup lang="ts">
defineEmits(['search']);
const searchQuery = ref('');
</script>