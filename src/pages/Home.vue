<template>
  <div class="bg-background text-text flex flex-col">
    <div>
      <Dialog
        class="cardStyle w-fit"
        :title="'New Playlist'"
        placeholder="Nektar's Top Hits"
        @submitted="playlistStore.createPlaylist"
        >Open Dialog</Dialog
      >
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
      <button class="cardStyle cursor-pointer" @click="lastFMToken">
        Get Token
      </button>
      <button class="cardStyle cursor-pointer" @click="lastFMSession">
        Get Session
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
import {
  commands,
  handleBackendError,
  toastBus,
  usePlayerStore,
  usePlaylistStore,
} from "@/composables/";
import { onMounted, ref } from "vue";

const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();

const fmToken = ref("");

function showToast(type: "success" | "error", description: string) {
  toastBus.addToast(type, description);
}

async function lastFMToken() {
  const token = await commands.getToken();
  if (token.status === "error") return handleBackendError(token.error);

  console.log(token.data);

  fmToken.value = token.data[1];
}

async function lastFMSession() {
  if (!fmToken.value) return;
  const session = await commands.getSession(fmToken.value);
  if (session.status === "error") return handleBackendError(session.error);
}

onMounted(() => {
  playerStore.currentPage = "/";
  playerStore.pageName = "Home";
});
</script>
