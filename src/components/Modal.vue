<template>
  <div v-if="!hideTrigger" @click="show = true">
    <slot name="trigger">
      <Button
        wide
        :label="triggerLabel ?? 'Show Modal'"
        icon="i-fluent-layout-row-two-16-filled"
      />
    </slot>
  </div>

  <teleport to="body">
    <Transition
      enter-active-class="animate-zoomIn"
      leave-active-class="animate-zoomOut"
    >
      <div
        id="modal"
        v-if="show"
        class="bg-bg-primary/50 fixed inset-0 z-10 flex items-center justify-center"
        @click="closeOnBackdrop"
      >
        <slot></slot>
      </div>
    </Transition>
  </teleport>
</template>

<script setup lang="ts">
import { Button } from "@/components/";
import { watch } from "vue";

defineProps<{
  triggerLabel?: string;
  hideTrigger?: boolean;
}>();

const show = defineModel<boolean>({ default: false });

/**
 * Handle the keydown event.
 *
 * If the `Escape` key is pressed, close the modal.
 */
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    show.value = false;
  }
}

function closeOnBackdrop(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    show.value = false;
  }
}

watch(show, (value) => {
  if (value) window.addEventListener("keydown", handleKeyDown);
  else window.removeEventListener("keydown", handleKeyDown);
});
</script>
