<template>
  <div
    v-if="showDropdown"
    ref="contextMenu"
    @mouseleave="handleMouseLeave()"
    @mouseenter="reEntered = true"
    :style="{ top: `${userCoords.y - 10}px`, left: `${userCoords.x - 50}px` }"
    class="border-stroke-100 bg-card text-supporting absolute z-30 flex h-fit w-fit cursor-pointer flex-col rounded-md border p-2 select-none"
  >
    <small
      @click="($emit('add-to-queue', selectedTrack), (showDropdown = false))"
      class="hover:bg-stroke-100 hover:text-text rounded-md p-2 duration-150"
      >Add to Queue</small
    >
    <div v-if="playlists" class="relative rounded-md">
      <div
        @mouseenter="showPlaylists = true"
        class="hover:bg-stroke-100 hover:text-text flex items-center rounded-md p-2 duration-150"
      >
        <small>Add to Playlist</small>
        <span class="i-fluent-caret-right-24-filled h-5 w-6"></span>
      </div>
      <div
        @mouseleave="handleMouseLeave(true)"
        v-if="showPlaylists"
        :style="{ top: 0, left: `${width - 4}px`, width: `${width}px` }"
        class="border-stroke-100 bg-card absolute rounded-md border p-2"
      >
        <div
          @click="addPlaylist(playlist)"
          v-for="playlist of playlists"
          class="hover:bg-stroke-100 hover:text-text flex items-center rounded-md p-2 duration-150"
        >
          <small>{{ playlist.name }}</small>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useEventListener } from "@vueuse/core";
import { commands, Playlists, Tracks } from "../bindings";

defineEmits<{
  (e: "add-to-queue", track: Tracks | null): void;
}>();

const props = defineProps<{
  trackList: Tracks[];
}>();

const contextMenu = ref<HTMLDivElement | null>(null);
const width = computed(() => contextMenu.value?.clientWidth || 0);
const userCoords = ref({ x: 0, y: 0 });

const showDropdown = ref(false);
const showPlaylists = ref(false);
const reEntered = ref(false);

const selectedTrack = ref<Tracks | null>(null);
const playlists = ref<Playlists[] | null>(null);

function handleMouseLeave(onlyPlaylists: boolean = false) {
  reEntered.value = false;
  setTimeout(() => {
    if (reEntered.value) return;
    if (!onlyPlaylists) showDropdown.value = false;
    showPlaylists.value = false;
  }, 200);
}

function handleContextEvent(e: MouseEvent) {
  if (e.target instanceof HTMLElement) {
    if (e.target.closest(".contextable")) {
      e.preventDefault();
      showDropdown.value = true;
      userCoords.value = { x: e.pageX - 5, y: e.pageY - 5 };

      const index =
        Array.from(
          e.target.closest(".contextable")?.parentElement?.children || [],
        )?.indexOf(e.target.closest(".contextable") as Element) ?? -1;

      selectedTrack.value = props.trackList[index];
    } else {
      showDropdown.value = false;
    }
  }
}

function handleOutsideClick(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest(".absolute")) {
    showDropdown.value = false;
    showPlaylists.value = false;
  }
}

async function addPlaylist(playlist: Playlists) {
  if (!selectedTrack.value) return;

  const result = await commands.addToPlaylist(
    playlist.id,
    selectedTrack.value.id,
  );

  showDropdown.value = false;

  if (result.status === "error")
    throw new Error(`[${result.error.type}] ${result.error.data}`);
}

useEventListener(window, "contextmenu", handleContextEvent);
useEventListener(window, "click", handleOutsideClick);

onMounted(async () => {
  const result = await commands.getAllPlaylists();
  if (result.status === "error")
    throw new Error(`[${result.error.type}] ${result.error.data}`);

  playlists.value = result.data;
});
</script>
