import { createApp } from "vue";
import { createPinia } from "pinia";
import "./styles.css";
import Widget from "./Widget.vue"

const pinia = createPinia();
const widget = createApp(Widget);

widget.use(pinia);
widget.mount("#widget");