<template>
  <Modal v-model="showModal">
    <template #trigger>
      <div class="group relative">
        <span
          class="i-fluent-edit-24-filled text-text-primary pointer-events-none absolute top-1/2 bottom-0 left-1/2 z-2 size-8 -translate-x-1/2 -translate-y-1/2 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
        ></span>
        <div
          class="bg-bg-primary/50 absolute z-1 h-full w-full opacity-0 duration-150 group-hover:opacity-100"
        ></div>
        <img
          class="border-border-secondary group-hover:border-border-secondary-hovered pointer-events-none size-48 cursor-pointer rounded-md border duration-150 select-none"
          :src="placeholderIfEmpty(coverPath)"
        />
      </div>
    </template>

    <template #default>
      <div
        class="bg-bg-primary border-border-primary flex h-64 w-xl gap-4 rounded-sm border p-4"
      >
        <img
          class="border-border-secondary aspect-square h-full cursor-pointer rounded-md border"
        />

        <div class="flex w-full flex-col gap-4">
          <InputBar
            v-model="playlistName"
            input-name="playlistName"
            input-type="text"
            placeholder="nektar's biggest fan..."
            label="Playlist Name"
          />

          <InputBar
            v-model="playlistDescription"
            input-name="playlistDescription"
            input-type="text"
            placeholder="Street's like a jungle..."
            label="Playlist Description"
          />

          <div class="flex w-full justify-end gap-2">
            <Button label="Cancel" @click="showModal = false" />
            <Button
              @click="
                ($emit('update', playlistName, playlistDescription),
                (showModal = false))
              "
              label="Submit"
              :disable="playlistName.length === 0"
            />
          </div>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { placeholderIfEmpty } from "@/composables/";
import { ref, watch } from "vue";
import { InputBar, Modal, Button } from "@/components/";

defineEmits<{
  (e: "update", name: string, description?: string, cover?: string): void;
}>();

const props = defineProps<{
  id: number;
  name: string;
  description?: string;
}>();

const coverPath = ref<string | undefined>(undefined);

const playlistName = ref(props.name);
const playlistDescription = ref(props?.description);
const showModal = ref(false);

watch(showModal, (newVal) => {
  if (!newVal) {
    playlistName.value = props.name;
    playlistDescription.value = props.description;
  }
});

watch(
  () => [props.name, props.description],
  ([newName, newDescription]) => {
    if (newName) playlistName.value = newName;
    playlistDescription.value = newDescription;
  },
);
</script>
