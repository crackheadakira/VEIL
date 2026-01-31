<template>
  <RadioGroupRoot
    v-model="radioSelected"
    class="text-text-secondary flex flex-col items-start gap-4"
    :default-value="radioSelected"
  >
    <div class="flex items-center" v-for="(item, idx) in items">
      <RadioGroupItem
        class="bg-bg-secondary border-border-primary data-[state=checked]:border-border-primary-active hover:border-border-primary-hovered h-5 w-5 rounded-full border transition-colors duration-75"
        :value="item"
      >
        <RadioGroupIndicator
          class="after:bg-text-primary relative flex h-full w-full items-center justify-center after:block after:h-2 after:w-2 after:rounded-[50%] after:content-['']"
          :id="item + idx"
        />
      </RadioGroupItem>
      <label class="pl-[15px] leading-none" :for="item + idx">{{ item }}</label>
    </div>
  </RadioGroupRoot>
</template>

<script setup lang="ts">
import { RadioGroupRoot, RadioGroupItem, RadioGroupIndicator } from "reka-ui";
defineProps<{
  items: string[];
}>();

const radioSelected = defineModel<string>({ required: true });
</script>
