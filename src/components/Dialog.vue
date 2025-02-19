<template>
  <div>
    <div class="h-fit w-fit" @click="showDialog = true">
      <slot></slot>
    </div>

    <div
      v-if="showDialog"
      class="bg-background/50 absolute top-0 left-0 z-40 h-screen w-screen"
    ></div>
    <Transition
      enter-from-class="opacity-0 scale-95"
      leave-to-class="opacity-0 scale-95"
      enter-active-class="transition-all duration-150"
      leave-active-class="transition-all duration-150"
    >
      <div
        v-if="showDialog"
        class="absolute inset-0 z-50 flex items-center justify-center"
      >
        <div class="cardStyle relative flex h-fit w-96 flex-col gap-3 p-4">
          <div>
            <p class="text-text">{{ props.title }}</p>
            <small v-if="props.description" class="mt-2">{{
              props.description
            }}</small>
          </div>
          <input
            v-model="inputValue"
            type="text"
            class="text-text placeholder-supporting cardStyle w-full font-medium focus:outline-hidden"
            :placeholder="placeholder"
          />
          <div class="flex w-full justify-end gap-2">
            <button
              @click="showDialog = false"
              class="aspect-button cardStyle text-supporting w-24 cursor-pointer hover:opacity-80"
            >
              <small>Cancel</small>
            </button>

            <button
              :class="
                inputValue.length === 0
                  ? 'cursor-not-allowed opacity-80'
                  : 'cursor-pointer'
              "
              @click="handleSubmit"
              class="aspect-button cardStyle text-supporting w-24 hover:opacity-80"
            >
              <small>Submit</small>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{
  title: string;
  description?: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: "submitted", inputValue: string): void;
}>();

const showDialog = ref(false);
const inputValue = ref("");

/**
 * Handle the submit button click event.
 *
 * Emits `submitted` event with value from `$inputValue`.
 */
function handleSubmit() {
  if (inputValue.value.length === 0) return;

  emit("submitted", inputValue.value);
  showDialog.value = false;
}

/**
 * Handle the keydown event.
 *
 * If the `Escape` key is pressed, close the dialog.
 */
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    showDialog.value = false;
  }
}

watch(showDialog, (value) => {
  if (value) {
    window.addEventListener("keydown", handleKeyDown);
  } else {
    window.removeEventListener("keydown", handleKeyDown);
  }
});
</script>
