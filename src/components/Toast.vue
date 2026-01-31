<template>
  <div
    :class="
      matchType([
        'text-green-500',
        'text-red-500',
        'text-yellow-500',
        'text-blue-500',
      ])
    "
    class="veil-card flex h-fit w-96 items-center gap-3 p-4"
  >
    <span
      class="shrink-0"
      :class="
        matchType([
          'i-fluent-checkmark-20-filled',
          'i-fluent-error-circle-20-filled',
          'i-fluent-warning-20-filled',
          'i-fluent-info-20-filled',
        ])
      "
    ></span>
    <p>{{ props.description }}</p>
  </div>
</template>

<script setup lang="ts">
import { ToastType } from "@/composables/";

const props = defineProps<{
  id: number;
  type: ToastType;
  description: string;
}>();

function matchType(conditions: string[]) {
  if (props.type === "success") return conditions[0];
  else if (props.type === "error") return conditions[1];
  else if (props.type === "warning") return conditions[2];
  else return conditions[3];
}
</script>
