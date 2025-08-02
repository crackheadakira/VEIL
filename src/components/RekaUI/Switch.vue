<template>
  <SwitchRoot
    v-model="switchState"
    class="border-border-secondary bg-bg-secondary data-[state=checked]:bg-bg-primary data-[state=checked]:border-border-secondary focus-within:inset-ring-border-primary relative flex h-5 w-8 rounded-full border focus-within:inset-ring-1 focus-within:outline-none"
  >
    <SwitchThumb
      class="bg-text-primary my-auto flex h-3.5 w-3.5 translate-x-0.5 items-center justify-center rounded-full text-xs transition-transform will-change-transform data-[state=checked]:translate-x-full"
    />
  </SwitchRoot>
</template>

<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from "reka-ui";
const switchState = defineModel<boolean>({ required: true, default: false });
</script>
