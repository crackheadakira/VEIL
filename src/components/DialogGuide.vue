<template>
  <div>
    <div class="h-fit w-fit" @click="showDialog = true">
      <slot></slot>
    </div>
    <Transition
      enter-active-class="animate-zoomIn"
      leave-active-class="animate-zoomOut"
    >
      <div
        v-if="showDialog"
        class="bg-background/50 absolute inset-0 z-50 flex items-center justify-center"
      >
        <TransitionGroup
          enter-from-class="opacity-0 -translate-x-10"
          leave-to-class="opacity-0 translate-x-10"
          enter-active-class="transition-all absolute duration-300"
          leave-active-class="transition-all absolute duration-300"
        >
          <div
            :key="currentPage.title"
            class="sodapop-card text-text flex min-h-36 w-96 flex-col justify-between gap-3 p-4"
          >
            <div class="h-fit">
              <p class="mb-2">{{ currentPage.title }}</p>
              <small class="text-supporting">{{
                currentPage.description
              }}</small>
            </div>
            <div
              :class="
                currentPage.buttons.length === 1
                  ? 'justify-end'
                  : 'justify-between'
              "
              v-if="currentPage.buttons"
              class="flex"
            >
              <button
                v-for="button of currentPage.buttons"
                :disabled="!button.condition"
                :class="
                  button.condition
                    ? 'cursor-pointer'
                    : 'cursor-not-allowed opacity-80'
                "
                @click="(button.click(), closeDialog(button.close))"
                class="aspect-button sodapop-card text-supporting w-24 hover:opacity-80"
              >
                <small>{{ readableCapitalization(button.name) }}</small>
              </button>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { DialogPage, readableCapitalization } from "@/composables/";

const showDialog = ref(false);

defineProps<{
  currentPage: DialogPage;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

/**
 * Handle the keydown event.
 *
 * If the `Escape` key is pressed, close the dialog.
 */
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    closeDialog(true);
  }
}

function closeDialog(shouldClose: boolean | undefined) {
  if (shouldClose) {
    emit("close");
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
