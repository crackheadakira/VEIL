import { createApp } from "vue";
import { createRouter, createMemoryHistory } from "vue-router";
import "./styles.css";
import App from "./App.vue";

import HomeView from "./pages/Home.vue"
import AlbumView from "./pages/Albums.vue"

const routes = [{ path: "/albums", component: AlbumView }, { path: "/", component: HomeView }];
const router = createRouter({
    history: createMemoryHistory(),
    routes
})
createApp(App).use(router).mount("#app");
