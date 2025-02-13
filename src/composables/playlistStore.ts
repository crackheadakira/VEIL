import {
  commands,
  handleBackendError,
  type Playlists,
  toastBus,
} from "@/composables/";
import { defineStore } from "pinia";
import { type Ref, ref } from "vue";

export const usePlaylistStore = defineStore("playlist", () => {
  const playlists: Ref<Playlists[]> = ref([]);

  async function fetchPlaylists(): Promise<void> {
    const result = await commands.getAllPlaylists();
    if (result.status === "error") return handleBackendError(result.error);

    playlists.value = result.data;
  }

  async function createPlaylist(name: string): Promise<void> {
    const result = await commands.newPlaylist(name);
    if (result.status === "error") return handleBackendError(result.error);
    else toastBus.addToast("success", "Playlist created successfully");

    await fetchPlaylists();
  }

  async function addToPlaylist(playlistId: number, trackId: number) {
    const result = await commands.addToPlaylist(playlistId, trackId);
    if (result.status === "error") return handleBackendError(result.error);
  }

  async function removeFromPlaylist(playlistId: number, trackId: number) {
    const result = await commands.removeFromPlaylist(playlistId, trackId);
    if (result.status === "error") return handleBackendError(result.error);
  }

  async function getTracksFromPlaylist(playlistId: number) {
    const result = await commands.getPlaylistTracks(playlistId);
    if (result.status === "error") return handleBackendError(result.error);
    return result.data;
  }

  return {
    playlists,
    createPlaylist,
    fetchPlaylists,
    addToPlaylist,
    removeFromPlaylist,
    getTracksFromPlaylist,
  };
});
