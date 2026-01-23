<template>
  <div class="h-fit w-fit">
    <div @click="model = true">
      <slot name="trigger">
        <Button
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
          v-if="model"
          class="bg-bg-primary/50 fixed inset-0 flex items-center justify-center"
        >
          <slot></slot>
        </div>
      </Transition>
    </teleport>
  </div>
</template>

<script setup lang="ts">
import { Button } from "@/components/";
import { watch } from "vue";

const props = defineProps<{
  triggerLabel?: string;
}>();

const model = defineModel<boolean>({ default: false });

/**
 * Handle the keydown event.
 *
 * If the `Escape` key is pressed, close the modal.
 */
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    model.value = false;
  }
}

watch(model, (value) => {
  if (value) window.addEventListener("keydown", handleKeyDown);
  else window.removeEventListener("keydown", handleKeyDown);
});
</script>
