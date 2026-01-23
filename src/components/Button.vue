<template>
  <div>
    <button
      @click="$emit('click')"
      class="border-border-secondary bg-bg-primary text-text-secondary flex h-12 items-center justify-center gap-2 rounded-md border"
      :class="[
        disable
          ? 'text-text-secondary-disabled border-border-secondary-disabled cursor-not-allowed'
          : 'hover:border-border-secondary-hovered hover:text-text-secondary-hovered cursor-pointer',
        wide ? 'aspect-card' : 'w-24',
      ]"
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
  disable?: boolean;
  wide?: boolean;
}>();

defineEmits<{
  (e: "click"): void;
}>();
</script>
