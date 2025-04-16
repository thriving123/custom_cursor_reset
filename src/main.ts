import { createApp } from "vue";
import App from "./App.vue";
import 'becomer-ui/es/theme-chalk/dist/index.css'
import BecomerUI from 'becomer-ui'
const app = createApp(App)
app.use(BecomerUI)
app.mount("#app");
