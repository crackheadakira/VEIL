import { createApp } from "vue";
import { createRouter, createMemoryHistory } from "vue-router";
import { createPinia } from "pinia";
import "./styles.css";
import App from "./App.vue";

import {
  HomeView,
  AlbumView,
  AllAlbumsView,
  ArtistView,
  PlaylistView,
  SettingsView,
} from "@/pages/";

const routes = [
  { path: "/settings", component: SettingsView, meta: { pageName: "Settings" } },
  { path: "/album/:id", component: AlbumView, name: "album", meta: { pageName: "Album" } },
  {
    path: "/playlist/:id",
    component: PlaylistView,
    name: "playlist",
    meta: { pageName: "Playlist" }
  },
  { path: "/artist/:id", component: ArtistView, name: "artist", meta: { pageName: "Artist" } },
  { path: "/all_albums", component: AllAlbumsView, meta: { pageName: "All Albums" } },
  { path: "/", component: HomeView, meta: { pageName: "Home" } },
];
const router = createRouter({
  history: createMemoryHistory(),
  routes,
});

const pinia = createPinia();
const app = createApp(App);

app.use(pinia);
app.use(router);
app.mount("#app");
