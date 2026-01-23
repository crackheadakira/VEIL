import { createApp } from "vue";
import { createRouter, createMemoryHistory } from "vue-router";
import { createPinia } from "pinia";
import "./styles.css";
import App from "./App.vue";;

import {
  HomeView,
  AllAlbumsView,
  ArtistView,
  SettingsView,
  ColorTestingView,
  CollectionView,
} from "@/pages/";

const routes = [
  { path: "/settings", component: SettingsView, name: "settings", meta: { pageName: "Settings" } },
  {
    path: "/album/:id",
    component: CollectionView,
    name: "album",
    meta: { pageName: "Album" },
    props: { type: "Album" as const },
  },
  {
    path: "/playlist/:id",
    component: CollectionView,
    name: "playlist",
    meta: { pageName: "Playlist" },
    props: { type: "Playlist" as const },
  },
  { path: "/artist/:id", component: ArtistView, name: "artist", meta: { pageName: "Artist" } },
  { path: "/all_albums", component: AllAlbumsView, name: "all_albums", meta: { pageName: "All Albums" } },
  { path: "/color_testing", component: ColorTestingView, name: "color_Testing", meta: { pageName: "Color Testing" } },
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