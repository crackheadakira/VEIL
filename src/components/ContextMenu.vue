<template>
    <div v-if="showDropdown" ref="contextMenu" @mouseleave="handleMouseLeave()" @mouseenter="reEntered = true"
        :style="{ top: `${userCoords.y - 10}px`, left: `${userCoords.x - 50}px` }"
        class="font-supporting absolute z-30 flex h-fit w-fit cursor-pointer select-none flex-col rounded-md border border-stroke-100 bg-card p-2 text-supporting">
        <p @click="$emit('add-to-queue', userCoords), showDropdown = false"
            class="rounded-md p-2 duration-150 hover:bg-stroke-100 hover:text-text">Add to Queue</p>
        <div @click="showDropdown = false" class="relative rounded-md">
            <div @mouseenter="showPlaylists = true"
                class="flex items-center rounded-md p-2 duration-150 hover:bg-stroke-100 hover:text-text">
                <p>Add to Playlist</p>
                <span class="i-ph-caret-right-fill h-5 w-6"></span>
            </div>
            <div @mouseleave="handleMouseLeave(true)" v-if=showPlaylists
                :style="{ top: 0, left: `${width - 4}px`, width: `${width}px` }"
                class="absolute rounded-md border border-stroke-100 bg-card p-2">
                <div v-for="playlist of playlists"
                    class="flex items-center rounded-md p-2 duration-150 hover:bg-stroke-100 hover:text-text">
                    <p>{{ playlist }}</p>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
defineEmits(['add-to-queue']);

const contextMenu = ref<HTMLDivElement | null>(null);
const width = computed(() => contextMenu.value?.clientWidth || 0);
const userCoords = ref({ x: 0, y: 0 });

const showDropdown = ref(false);
const showPlaylists = ref(false);
const reEntered = ref(false);

const playlists = ["comfort?", "school", "the beatles"]

function handleMouseLeave(onlyPlaylists: boolean = false) {
    reEntered.value = false;
    setTimeout(() => {
        if (reEntered.value) return;
        if (!onlyPlaylists) showDropdown.value = false;
        showPlaylists.value = false;
    }, 200);
}

function handleContextEvent(e: MouseEvent) {
    if (e.target instanceof HTMLElement) {
        if (e.target.closest('.contextable')) {
            e.preventDefault();
            showDropdown.value = true;
            userCoords.value = { x: e.pageX - 5, y: e.pageY - 5 };
        } else {
            showDropdown.value = false;
        }
    }
}

function handleOutsideClick(e: MouseEvent) {
    if (!(e.target as HTMLElement).closest('.absolute')) {
        showDropdown.value = false;
        showPlaylists.value = false;
    }
}

onMounted(() => {
    window.addEventListener('contextmenu', handleContextEvent);
    window.addEventListener('click', handleOutsideClick);
});

onBeforeUnmount(() => {
    window.removeEventListener('contextmenu', handleContextEvent);
    window.removeEventListener('click', handleOutsideClick);
});

</script>