<template>
  <div>
    <button
      @click="$emit('click')"
      class="text aspect-button border-border-secondary bg-bg-primary text-text-secondary hover:border-border-secondary-hovered hover:text-text-secondary-hovered flex h-12 cursor-pointer items-center justify-center gap-2 rounded-md border"
    >
      <span v-if="props.icon" :class="props.icon" class="h-7"></span>
      <p>{{ props.label }}</p>
    </button>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  label: string;
  icon?: string;
}>();

defineEmits<{
  (e: "click"): void;
}>();
</script>
