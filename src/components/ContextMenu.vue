<template>
    <Transition enter-from-class="-translate-y-[25%] opacity-0" leave-to-class="-translate-y-[25%] opacity-0"
        enter-active-class="transition duration-150" leave-active-class="transition duration-150">
        <div v-if="showDropdown" ref="contextMenu"
            :style="{ top: `${userCoords.y - 10}px`, left: `${userCoords.x - 50}px` }"
            class="absolute z-50 w-fit flex flex-col p-1 bg-card border-stroke-100 border h-fit rounded-md text-supporting font-supporting cursor-pointer select-none">
            <p @click="$emit('add-to-queue', userCoords), showDropdown = false"
                class="p-2 rounded-md duration-150 hover:bg-stroke-100 hover:text-text">Add to Queue</p>
            <div @click="showDropdown = false" class="rounded-md relative">
                <div @mouseenter="showPlaylists = true"
                    class="flex items-center p-2 duration-150 rounded-md hover:bg-stroke-100 hover:text-text">
                    <p>Add to Playlist</p>
                    <span class="i-ph-caret-right-fill h-5 w-6"></span>
                </div>
                <Transition enter-from-class="-translate-x-[10%] opacity-0"
                    leave-to-class="-translate-x-[10%] opacity-0" enter-active-class="transition duration-150"
                    leave-active-class="transition duration-150">
                    <div @mouseleave="showPlaylists = false" v-if=showPlaylists
                        :style="{ top: 0, left: `${width - 2}px`, width: `${width}px` }"
                        class="absolute bg-card border-stroke-100 border rounded-md p-1">
                        <div v-for="playlist of playlists"
                            class="flex items-center p-2 duration-150 rounded-md hover:bg-stroke-100 hover:text-text">
                            <p>{{ playlist }}</p>
                        </div>
                    </div>
                </Transition>
            </div>
        </div>
    </Transition>
</template>

<script setup lang="ts">
defineEmits(['add-to-queue']);

const contextMenu = ref<HTMLDivElement | null>(null);
const width = computed(() => contextMenu.value?.clientWidth || 0);
const userCoords = ref({ x: 0, y: 0 });
const showDropdown = ref(false);
const showPlaylists = ref(false);

const playlists = ["comfort?", "school", "senpai please notice me UWUWUWUWU"]

function handleContextEvent(e: MouseEvent) {
    if (e.target instanceof HTMLElement) {
        if (e.target.closest('.contextable')) {
            e.preventDefault();
            showDropdown.value = true;
            userCoords.value = { x: e.pageX, y: e.pageY };
        } else {
            showDropdown.value = false;
        }
    }
}

function handleOutsideClick(e: MouseEvent) {
    if (!(e.target as HTMLElement).closest('.absolute')) {
        showDropdown.value = false;
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