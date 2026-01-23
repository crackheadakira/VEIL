<template>
  <div>
    <div class="h-fit w-fit" @click="showDialog = true">
      <slot></slot>
    </div>
    <Teleport to="body">
      <Transition
        enter-active-class="animate-zoomIn"
        leave-active-class="animate-zoomOut"
      >
        <div
          id="dialog"
          v-if="showDialog"
          class="bg-bg-primary/50 absolute inset-0 z-50 flex items-center justify-center"
        >
          <div class="sodapop-card relative flex h-fit w-96 flex-col gap-3 p-4">
            <div>
              <h6 class="text-text-primary">{{ props.title }}</h6>
              <p v-if="props.description" class="mt-2">
                {{ props.description }}
              </p>
            </div>
            <input
              v-model="inputValue"
              type="text"
              class="text-text-primary placeholder-text-secondary sodapop-card bg-bg-primary w-full focus:outline-hidden"
              :placeholder="placeholder"
            />
            <div class="flex w-full justify-end gap-2">
              <button
                @click="showDialog = false"
                class="aspect-button sodapop-card text-text-secondary hover:border-border-secondary-hovered hover:text-text-secondary-hovered w-24 cursor-pointer"
              >
                <p>Cancel</p>
              </button>

              <button
                :class="
                  inputValue.length === 0
                    ? 'text-text-secondary-disabled border-border-secondary-disabled cursor-not-allowed'
                    : 'hover:border-border-secondary-hovered hover:text-text-secondary-hovered cursor-pointer'
                "
                @click="handleSubmit"
                class="aspect-button sodapop-card text-text-secondary w-24"
              >
                <p>Submit</p>
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";

const props = defineProps<{
  title: string;
  description?: string;
  placeholder?: string;
  modelValue?: boolean;
}>();

const emit = defineEmits<{
  (e: "submitted", inputValue: string): void;
  (e: "update:modelValue", value: boolean): void;
}>();

const showDialog = computed({
  get: () => props.modelValue ?? false,
  set: (value: boolean) => emit("update:modelValue", value),
});

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
