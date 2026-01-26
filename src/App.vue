<template>
  <div class="bg-bg-primary flex h-screen flex-col">
    <TitleBar class="sticky top-0" />

    <div class="flex flex-1 overflow-hidden">
      <div class="h-full w-72 overflow-y-auto">
        <SideBar class="sticky top-16 bottom-28 h-full" />
      </div>

      <RouterView
        class="flex-1 overflow-y-scroll p-8"
        :key="currentRoute.fullPath"
      />
    </div>

    <Player class="sticky bottom-0 h-28" />
    <ToastManager ref="toastManager" />
  </div>
</template>

<script setup lang="ts">
import { Player, SideBar, ToastManager, TitleBar } from "@/components/";
import {
  useConfigStore,
  commands,
  handleBackendError,
  usePlayerStore,
  usePlaylistStore,
  PlayerProgressEvent,
  events,
  toastBus,
  ThemeMode,
} from "@/composables/";
import { Channel } from "@tauri-apps/api/core";
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();
const configStore = useConfigStore();
const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();
const currentRoute = router.currentRoute;

let unlistenConfigEvent: () => void = () => {};

function updateDocumentTheme(theme: ThemeMode) {
  if (theme === "Dark") {
    document.documentElement.setAttribute("data-theme", "dark");
  } else {
    document.documentElement.setAttribute("data-theme", "light");
  }
}

onMounted(async () => {
  unlistenConfigEvent = await events.veilConfigEvent.listen((event) => {
    if (!event.payload.theme) return;

    updateDocumentTheme(event.payload.theme);
  });

  await configStore.initialize();
  const theme = configStore.config?.ui.theme || "Dark";

  updateDocumentTheme(theme);

  const css = await commands.readCustomStyle();
  let styleElement = document.getElementById("custom-style");

  if (!styleElement) {
    styleElement = document.createElement("style");
    styleElement.id = "custom-style";
    document.head.appendChild(styleElement);
  }

  if (css.status === "ok" && styleElement) {
    styleElement.innerText = css.data;
  }

  const result = await commands.getAllAlbums();
  if (result.status === "error") return handleBackendError(result.error);

  const track = playerStore.currentTrack;
  const progress = playerStore.playerProgress;

  if (result.data.length === 0 && track) {
    playerStore.$reset();
  } else if (track) {
    await events.playerEvent.emit({
      type: "Initialize",
      data: { track, progress },
    });
  }

  await playlistStore.fetchPlaylists();

  const page = configStore.currentPage;
  router.push(page);

  const channel = new Channel<PlayerProgressEvent>();
  channel.onmessage = async (msg) => {
    await playerStore.handleProgress(false, msg.data.progress);
  };

  await events.frontendError.listen((e) => {
    toastBus.addToast("error", `${e.payload.type}: ${e.payload.data}`);
  });

  const res = await commands.playerProgressChannel(channel);
  if (res.status === "error") console.error(res.error);
});

onUnmounted(() => {
  unlistenConfigEvent();
});
</script>
