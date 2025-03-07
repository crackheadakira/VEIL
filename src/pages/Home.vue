<template>
  <div class="bg-background text-text flex flex-col gap-4">
    <div>
      <Dialog
        class="sodapop-card w-fit"
        :title="'New Playlist'"
        placeholder="Nektar's Top Hits"
        @submitted="playlistStore.createPlaylist"
        >Open Dialog</Dialog
      >
      <button
        @click="showToast('success', 'This is a success toast')"
        class="sodapop-card cursor-pointer"
      >
        Show Success Toast
      </button>
      <button
        @click="showToast('error', 'This is an error toast')"
        class="sodapop-card cursor-pointer"
      >
        Show Error Toast
      </button>
    </div>
    <div class="flex gap-2">
      <PlaylistCard />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Dialog, PlaylistCard } from "@/components/";
import { toastBus, useConfigStore, usePlaylistStore } from "@/composables/";
import { onMounted } from "vue";

const configStore = useConfigStore();
const playlistStore = usePlaylistStore();

function showToast(type: "success" | "error", description: string) {
  toastBus.addToast(type, description);
}

onMounted(() => {
  configStore.currentPage = "/";
  configStore.pageName = "Home";
});
</script>
