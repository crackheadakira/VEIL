<template>
  <div class="flex gap-3">
    <input
      class="h-4 w-4 focus:ring-2"
      type="radio"
      :id="inputId"
      :value="inputValue"
      :checked="modelValue === inputValue"
      @change="$emit('update:modelValue', inputValue)"
    />
    <label :for="inputId">{{ inputValue }}</label>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  inputId: string;
  inputValue: string;
  modelValue: string;
}>();

defineEmits<{
  (e: "update:modelValue", selected: string): void;
}>();
</script>
