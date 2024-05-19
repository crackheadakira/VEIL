<template>
    <div
        class="px-3 flex gap-2 bg-background border-stroke-100 border w-56 h-fit p-2 rounded-md font-supporting duration-150 hover:border-stroke-200">
        <span class="text-supporting w-5 aspect-square i-mingcute-search-line"></span>
        <input @keyup.50="$emit('search', searchQuery)" :value="searchQuery" type="text"
            class="bg-transparent focus:outline-none text-text placeholder-supporting w-full" placeholder="Search..." />
    </div>
</template>

<script setup lang="ts">
defineEmits(['search']);
const searchQuery = ref('');
</script>