<template>
  <div
    class="border-stroke-100 bg-background text-supporting flex h-fit w-56 flex-col gap-2 rounded-md border select-none"
  >
    <button class="cursor-pointer" @click="showDialog = true">
      Open Dialog
    </button>

    <div
      v-if="showDialog"
      class="bg-background/50 absolute top-0 left-0 z-40 h-screen w-screen"
    ></div>
    <Transition
      enter-from-class="opacity-0 scale-95"
      leave-to-class="opacity-0 scale-95"
      enter-active-class="transition-all duration-150"
      leave-active-class="transition-all duration-150"
    >
      <div
        v-if="showDialog"
        class="absolute inset-0 z-50 flex items-center justify-center"
      >
        <div
          class="bg-card border-stroke-100 relative flex h-fit w-96 flex-col gap-3 rounded-md border p-4"
        >
          <div>
            <p class="text-text">{{ props.title }}</p>
            <small v-if="props.description" class="mt-2">{{
              props.description
            }}</small>
          </div>
          <input
            v-model="playlistName"
            type="text"
            class="text-text placeholder-supporting border-stroke-100 w-full rounded-md border p-2 font-medium focus:outline-hidden"
            placeholder="Nektar's Top Hits"
          />
          <div class="flex w-full justify-end gap-2">
            <button
              @click="showDialog = false"
              class="aspect-button border-stroke-100 bg-background text-supporting w-24 cursor-pointer rounded-md border p-2 hover:opacity-80"
            >
              <small>Cancel</small>
            </button>

            <button
              :class="
                playlistName.length === 0
                  ? 'cursor-not-allowed opacity-80'
                  : 'cursor-pointer'
              "
              @click="handleSubmit"
              class="aspect-button border-stroke-100 bg-background text-supporting w-24 rounded-md border p-2 hover:opacity-80"
            >
              <small>Submit</small>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  title: string;
  description?: string;
}>();

const emit = defineEmits<{
  (e: "submitted", playlistName: string): void;
}>();

const showDialog = ref(false);
const playlistName = ref("");

function handleSubmit() {
  if (playlistName.value.length === 0) return;

  emit("submitted", playlistName.value);
  showDialog.value = false;
}
</script>
