import { defineStore } from "pinia";
import { commands, Playlists } from "../bindings";

export const usePlaylistStore = defineStore("playlist", async () => {
  const playlists: Ref<Playlists[]> = ref([]);

  async function createPlaylist(name: string): Promise<void> {
    await commands.newPlaylist(name);
  }

  return {
    playlists,
    createPlaylist,
  };
});
