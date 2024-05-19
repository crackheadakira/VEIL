import { createApp } from "vue";
import { createRouter, createMemoryHistory } from "vue-router";
import "./styles.css";
import App from "./App.vue";

import HomeView from "./pages/Home.vue"
import AlbumView from "./pages/Album.vue"
import AllAlbumsView from "./pages/AllAlbums.vue"

const routes = [{ path: "/album/:artist_id/:album_id", component: AlbumView, name: 'album' }, { path: "/all_albums", component: AllAlbumsView }, { path: "/", component: HomeView }];
const router = createRouter({
    history: createMemoryHistory(),
    routes
})
createApp(App).use(router).mount("#app");
