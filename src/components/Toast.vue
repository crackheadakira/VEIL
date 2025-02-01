<template>
  <div
    v-if="showToast"
    :class="props.type === 'success' ? 'text-green-500' : 'text-red-500'"
    class="bg-card border-stroke-100 flex h-fit w-96 items-center gap-3 rounded-md border p-4"
  >
    <span
      class="shrink-0"
      :class="
        props.type === 'success'
          ? 'i-fluent-checkmark-20-filled'
          : 'i-fluent-error-circle-20-filled'
      "
    ></span>
    <small>{{ props.description }}</small>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  id: number;
  type: "success" | "error";
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

onMounted(() => {
  callToast();
});
</script>
