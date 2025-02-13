<template>
  <div class="bg-background text-text flex flex-col">
    <div>
      <button class="cardStyle cursor-pointer" @click="openDialog">
        <small>Select music folder</small>
      </button>
      <Dialog
        :title="'New Playlist'"
        @submitted="playlistStore.createPlaylist"
      />
      <button
        @click="showToast('success', 'This is a success toast')"
        class="cardStyle cursor-pointer"
      >
        Show Success Toast
      </button>
      <button
        @click="showToast('error', 'This is an error toast')"
        class="cardStyle cursor-pointer"
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
import { Dialog, Dropdown, PlaylistCard } from "@/components";
import {
  commands,
  handleBackendError,
  toastBus,
  usePlayerStore,
  usePlaylistStore,
} from "@/composables";
import { onMounted } from "vue";

const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();

async function openDialog() {
  const result = await commands.selectMusicFolder();
  if (result.status === "error") return handleBackendError(result.error);
  else {
    toastBus.addToast("success", "Music added successfully");
  }
}

function showToast(type: "success" | "error", description: string) {
  toastBus.addToast(type, description);
}

onMounted(() => {
  playerStore.currentPage = "/";
});
</script>
