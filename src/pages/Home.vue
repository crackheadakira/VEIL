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
      <Dropdown
        :title="'Filter by'"
        :options="['Albums', 'Artists', 'Tracks']"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Dialog, Dropdown, PlaylistCard } from "@/components/";
import { toastBus, usePlayerStore, usePlaylistStore } from "@/composables/";
import { onMounted } from "vue";

const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();

function showToast(type: "success" | "error", description: string) {
  toastBus.addToast(type, description);
}

onMounted(() => {
  playerStore.currentPage = "/";
  playerStore.pageName = "Home";
});
</script>
