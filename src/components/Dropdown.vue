<template>
    <div
        class="font-supporting relative flex h-fit w-56 cursor-pointer select-none flex-col gap-2 rounded-md border border-stroke-100 bg-background text-supporting">
        <div @click="handleShow()" class="flex items-center gap-2 p-2 px-3">
            <span id="dropdown_icon" class="i-fluent-caret-down-24-filled w-6 duration-150"></span>
            <p>{{ selectedValue }}</p>
        </div>
        <Transition enter-from-class="-translate-y-[25%] opacity-0" leave-to-class="-translate-y-[25%] opacity-0"
            enter-active-class="transition duration-150" leave-active-class="transition duration-150">
            <div v-if="showOptions"
                class="absolute -left-[1px] top-[100%] z-10 flex w-56 flex-col rounded-b-md border border-stroke-100 bg-background duration-150">
                <div v-for="option in options" @click="handleSelect(option), $emit('dropdownSelected', option)"
                    class="p-3 duration-150 hover:bg-card hover:text-text">{{ option }}</div>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { useEventListener } from '@vueuse/core';

const props = defineProps<{
    title: string,
    options: string[],
}>()

defineEmits(['dropdownSelected']);

const showOptions = ref(false);
const selectedValue = ref(props.title);

function handleShow() {
    showOptions.value = !showOptions.value;
    document.getElementById('dropdown_icon')?.classList.toggle('rotate-180');
}

function handleSelect(option: string) {
    selectedValue.value = option;
    handleShow();
}

useEventListener(window, 'click', (e) => {
    if (!(e.target as HTMLElement).closest('.relative')) {
        showOptions.value = false;
        document.getElementById('dropdown_icon')?.classList.remove('rotate-180');
    }
})
</script>