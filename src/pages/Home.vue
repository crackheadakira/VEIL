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
import { Dialog, Dropdown, PlaylistCard } from "@/components/";
import {
  events,
  commands,
  handleBackendError,
  toastBus,
  usePlayerStore,
  usePlaylistStore,
} from "@/composables/";
import { onMounted, ref } from "vue";

const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();

const persistentToastId = ref<number | null>(null);

async function openDialog() {
  const result = await commands.selectMusicFolder();
  if (result.status === "error") return handleBackendError(result.error);
  else {
    toastBus.addToast("success", "Music added successfully");
  }
}

events.musicDataEvent.once((data) => {
  const id = Date.now();
  persistentToastId.value = id;
  const payload = data.payload;
  // keep modfying the toast description until the data.finished is true
  toastBus.persistentToast(
    id,
    "info",
    `Importing songs (${payload.current} / ${payload.total})`,
  );

  if (payload.finished) {
    setTimeout(() => toastBus.removeToast(id), 2100);
  }
});

events.musicDataEvent.listen((data) => {
  const payload = data.payload;

  if (persistentToastId.value) {
    toastBus.persistentToast(
      persistentToastId.value,
      "info",
      `Importing songs (${payload.current} / ${payload.total})`,
    );
  } else {
    const id = Date.now();
    persistentToastId.value = id;
    toastBus.persistentToast(
      id,
      "info",
      `Importing songs (${payload.current} / ${payload.total})`,
    );
  }

  if (payload.finished && persistentToastId.value) {
    setTimeout(() => {
      toastBus.removeToast(persistentToastId.value!);
      persistentToastId.value = null;
    }, 2100);
  }
});

function showToast(type: "success" | "error", description: string) {
  toastBus.addToast(type, description);
}

onMounted(() => {
  playerStore.currentPage = "/";
});
</script>
