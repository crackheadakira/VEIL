<template>
    <div
        class="relative flex flex-col gap-2 bg-background border-stroke-100 border w-56 h-fit rounded-md text-supporting font-supporting cursor-pointer select-none">
        <div @click="handleShow()" class="flex gap-2 items-center px-3 p-2">
            <span id="dropdown_icon" class="i-ph-caret-down-fill w-6 duration-150"></span>
            <p>{{ selectedValue }}</p>
        </div>
        <Transition enter-from-class="-translate-y-[25%] opacity-0" leave-to-class="-translate-y-[25%] opacity-0"
            enter-active-class="transition duration-150" leave-active-class="transition duration-150">
            <div v-if="showOptions"
                class="flex flex-col w-56 absolute -left-[1px] top-[100%] bg-background border border-stroke-100 rounded-b-md">
                <div v-for="option in options" @click="handleSelect(option), $emit('dropdownSelected', option)"
                    class="duration-150 hover:text-text hover:bg-card p-3">{{ option }}</div>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
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

onMounted(() => {
    window.addEventListener('click', (e) => {
        if (!(e.target as HTMLElement).closest('.relative')) {
            showOptions.value = false;
            document.getElementById('dropdown_icon')?.classList.remove('rotate-180');
        }
    });
});
</script>