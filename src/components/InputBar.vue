<template>
  <div class="flex flex-col gap-2">
    <label v-if="label" class="text-text-inverse" :for="inputName">{{
      label
    }}</label>
    <input
      :type="inputType"
      :name="inputName"
      :id="inputName"
      :placeholder="placeholder"
      :required="required"
      v-model="inputValue"
      ref="inputRef"
      class="placeholder-text-tertiary text-text-primary bg-bg-primary w-full font-medium focus:outline-hidden"
    />
  </div>
</template>

<script setup lang="ts">
import { useTemplateRef, type InputTypeHTMLAttribute } from "vue";

defineProps<{
  inputName: string;
  inputType: InputTypeHTMLAttribute;
  placeholder?: string;
  label?: string;
  required?: boolean;
}>();

const inputValue = defineModel<string>({ required: true, default: "" });
const inputRef = useTemplateRef<HTMLInputElement>("inputRef");

defineExpose({
  focus: () => inputRef.value?.focus(),
});
</script>
