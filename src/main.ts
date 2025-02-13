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
} from "@/pages";

const routes = [
  { path: "/album/:artist_id/:album_id", component: AlbumView, name: "album" },
  {
    path: "/playlist/:playlist_id",
    component: PlaylistView,
    name: "playlist",
  },
  { path: "/artist/:artist_id", component: ArtistView, name: "artist" },
  { path: "/all_albums", component: AllAlbumsView },
  { path: "/", component: HomeView },
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
