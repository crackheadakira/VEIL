<template>
  <div class="sodapop-card relative flex h-fit w-96 flex-col gap-3 p-4">
    <div>
      <h6 class="text-text-primary">{{ props.title }}</h6>
      <p v-if="props.description" class="text-text-secondary mt-2">
        {{ props.description }}
      </p>
    </div>

    <InputBar
      v-model="inputValue"
      input-type="text"
      :placeholder="placeholder"
      input-name="dialogInput"
      class="sodapop-card"
    />

    <div class="flex w-full justify-end gap-2">
      <Button label="Cancel" @click="$emit('cancel')" />
      <Button
        label="Submit"
        @click="handleSubmit"
        :disable="inputValue.length === 0"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { InputBar, Button } from "@/components/";
import { ref } from "vue";

const props = defineProps<{
  title: string;
  description?: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: "cancel"): void;
  (e: "submit", inputValue: string): void;
}>();

const inputValue = ref("");

/**
 * Handle the submit button click event.
 *
 * Emits `submitted` event with value from `$inputValue`.
 */
function handleSubmit() {
  if (inputValue.value.length === 0) return;

  emit("submit", inputValue.value);
}
</script>
