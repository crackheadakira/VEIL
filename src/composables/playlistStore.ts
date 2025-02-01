import { defineStore } from "pinia";
import { commands, Playlists } from "../bindings";

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

  return {
    playlists,
    createPlaylist,
    fetchPlaylists,
  };
});
