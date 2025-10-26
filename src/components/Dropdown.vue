<template>
  <div
    class="border-border-secondary bg-bg-primary text-text-secondary relative flex h-fit w-56 cursor-pointer flex-col gap-2 rounded-md border select-none"
  >
    <div @click="handleShow()" class="flex items-center gap-2 p-2 px-3">
      <span
        id="dropdown_icon"
        class="i-fluent-caret-down-24-filled w-6 duration-150"
      ></span>
      <p>{{ selectedValue }}</p>
    </div>
    <Transition
      enter-from-class="-translate-y-[10%] opacity-0"
      leave-to-class="-translate-y-[10%] opacity-0"
      enter-active-class="transition duration-150"
      leave-active-class="transition duration-150"
    >
      <div
        v-if="showOptions"
        class="border-border-secondary bg-bg-primary absolute top-1/1 -left-px z-10 flex w-56 flex-col rounded-b-md border duration-150"
      >
        <small
          v-for="option in options"
          @click="(handleSelect(option), $emit('dropdownSelected', option))"
          class="hover:bg-bg-secondary hover:text-text-primary p-3 duration-150"
        >
          {{ option }}
        </small>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { useEventListener } from "@vueuse/core";
import { ref } from "vue";

const props = defineProps<{
  title: string;
  options: string[];
}>();

defineEmits(["dropdownSelected"]);

const showOptions = ref(false);
const selectedValue = ref(props.title);

/**
 * Handle the dropdown show/hide
 */
function handleShow() {
  showOptions.value = !showOptions.value;
  document.getElementById("dropdown_icon")?.classList.toggle("rotate-180");
}

/**
 * Handle the dropdown select.
 *
 * Calls {@linkcode handleShow} to hide the dropdown.
 *
 * @param {string} option - The selected option
 */
function handleSelect(option: string): void {
  selectedValue.value = option;
  handleShow();
}

useEventListener(window, "click", (e) => {
  if (!(e.target as HTMLElement).closest(".relative")) {
    showOptions.value = false;
    document.getElementById("dropdown_icon")?.classList.remove("rotate-180");
  }
});
</script>
