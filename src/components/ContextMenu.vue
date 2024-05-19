<template>
    <Transition enter-from-class="-translate-y-[25%] opacity-0" leave-to-class="-translate-y-[25%] opacity-0"
        enter-active-class="transition duration-150" leave-active-class="transition duration-150">
        <div v-if="showDropdown" ref="contextMenu" :style="{ top: `${userCoords.y}px`, left: `${userCoords.x}px` }"
            class="absolute z-50 w-fit flex flex-col p-1 bg-card border-stroke-100 border h-fit rounded-md text-supporting font-supporting cursor-pointer select-none">
            <p @click="$emit('add-to-queue', userCoords)"
                class="p-2 rounded-md duration-150 hover:bg-stroke-100 hover:text-text">Add to Queue</p>
        </div>
    </Transition>
</template>

<script setup lang="ts">
defineEmits(['add-to-queue']);

const contextMenu = ref<HTMLDivElement | null>(null);
const userCoords = ref({ x: 0, y: 0 });
const showDropdown = ref(false);

function handleContextEvent(e: MouseEvent) {
    if (e.target instanceof HTMLElement) {
        if (e.target.closest('.contextable')) {
            e.preventDefault();
            showDropdown.value = true;
            userCoords.value = { x: e.pageX, y: e.pageY };
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