<template>
  <div
    v-if="showDropdown"
    ref="contextMenu"
    @mouseleave="handleMouseLeave()"
    @mouseenter="reEntered = true"
    :style="{ top: `${userCoords.y - 10}px`, left: `${userCoords.x - 50}px` }"
    class="cardStyle text-supporting absolute z-30 flex h-fit w-fit cursor-pointer flex-col select-none"
  >
    <small
      @click="($emit('add-to-queue', selectedTrack), (showDropdown = false))"
      class="hover:bg-stroke-100 hover:text-text rounded-md p-2 duration-150"
      >Add to Queue</small
    >
    <small
      v-if="'playlist' in props.data"
      @click="handlePlaylistAction(props.data.playlist, 'remove')"
      class="hover:bg-stroke-100 hover:text-text rounded-md p-2 duration-150"
      >Remove from Playlist</small
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
        class="cardStyle absolute"
      >
        <div
          @click="handlePlaylistAction(playlist, 'add')"
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
import type {
  AlbumWithTracks,
  Playlists,
  PlaylistWithTracks,
  Tracks,
} from "@/composables";
import { commands, handleBackendError, usePlaylistStore } from "@/composables";
import { useEventListener } from "@vueuse/core";
import { computed, onMounted, ref } from "vue";

const playlistStore = usePlaylistStore();

defineEmits<{
  (e: "add-to-queue", track: Tracks | null): void;
}>();

const props = defineProps<{
  data: AlbumWithTracks | PlaylistWithTracks;
}>();

const contextMenu = ref<HTMLDivElement | null>(null);
const width = computed(() => contextMenu.value?.clientWidth || 0);
const userCoords = ref({ x: 0, y: 0 });

const showDropdown = ref(false);
const showPlaylists = ref(false);
const reEntered = ref(false);

const selectedTrack = ref<Tracks | null>(null);
const playlists = ref<Playlists[] | null>(null);

/**
 * If the users mouse hovers away from the context menu, spawn a 200ms timeout to check if the user re-entered the context menu.
 * If the user didn't re-enter the context menu, hide the context menu.
 *
 * @param {boolean} onlyPlaylists - Only hide the playlists dropdown. By default, hide the entire context menu.
 */
function handleMouseLeave(onlyPlaylists: boolean = false): void {
  reEntered.value = false;
  setTimeout(() => {
    if (reEntered.value) return;
    if (!onlyPlaylists) showDropdown.value = false;
    showPlaylists.value = false;
  }, 200);
}

/**
 * Handles the context event. Checks if the user clicked on a contextable element and spawns the context menu
 * and positions it at the users cursor.
 *
 * Prevents the default context menu from showing
 *
 * Sets `$selectedTrack` to the track that was clicked on.
 *
 * @param {MouseEvent} e - The mouse event.
 */
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

      selectedTrack.value = props.data.tracks[index];
    } else {
      showDropdown.value = false;
    }
  }
}

/**
 * Handles the outside click event. If the user clicks of the context menu, hide the context menu.
 *
 * Checks if the class of the clicked element is `.absolute` and if it is, hide the context menu.
 *
 * @param {MouseEvent} e - The mouse event.
 */
function handleOutsideClick(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest(".absolute")) {
    showDropdown.value = false;
    showPlaylists.value = false;
  }
}

/**
 * Adds the selected track to the selected playlist.
 *
 * @param playlist
 */
async function handlePlaylistAction(
  playlist: Playlists,
  type: "add" | "remove",
): Promise<void> {
  if (!selectedTrack.value) return;

  if (type === "add")
    playlistStore.addToPlaylist(playlist.id, selectedTrack.value.id);
  else playlistStore.removeFromPlaylist(playlist.id, selectedTrack.value.id);

  showDropdown.value = false;
}

useEventListener(window, "contextmenu", handleContextEvent);
useEventListener(window, "click", handleOutsideClick);

onMounted(async () => {
  const result = await commands.getAllPlaylists();
  if (result.status === "error") return handleBackendError(result.error);

  playlists.value = result.data;
});
</script>
