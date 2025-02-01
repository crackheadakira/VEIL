<template>
  <div
    v-if="showToast"
    :class="matchType(['text-green-500', 'text-red-500', 'text-yellow-500'])"
    class="cardStyle flex h-fit w-96 items-center gap-3 p-4"
  >
    <span
      class="shrink-0"
      :class="
        matchType([
          'i-fluent-checkmark-20-filled',
          'i-fluent-error-circle-20-filled',
          'i-fluent-warning-20-filled',
        ])
      "
    ></span>
    <small>{{ props.description }}</small>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  id: number;
  type: "success" | "error" | "warning";
  description: string;
  removeToast: (id: number) => void;
}>();

const showToast = ref(false);

function callToast() {
  showToast.value = true;
  setTimeout(() => {
    showToast.value = false;
    props.removeToast(props.id);
  }, 2000);
}

function matchType(conditions: string[]) {
  if (props.type === "success") return conditions[0];
  else if (props.type === "error") return conditions[1];
  else return conditions[2];
}

onMounted(() => {
  callToast();
});
</script>
